use crate::utils::{Coordinate, CreatureAction, CreatureEvent};
use super::brain::{Brain, Genome};
use rand::Rng;

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// DNA
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct Dna {
    pub bytes: Vec<u8>,
}

impl Dna {
    pub fn random(len: usize, rng: &mut impl Rng) -> Self {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&[
            0b00000001, // mask low byte
            0,
            0,
            0,
            0,
            4,          // Output
            0,          // threshold source
            0,          // target source
        ]);

        let mut random_bytes = vec![0u8; len];
        rng.fill(&mut random_bytes[..]);
        bytes.extend_from_slice(&random_bytes);

        Self { bytes }
    }

    pub fn mutate(&mut self, rng: &mut impl Rng) {
        // some bit wise mutations
        for byte in &mut self.bytes {
            if rng.gen_bool(0.05) {
                let bit = 1 << rng.gen_range(0..8);
                *byte ^= bit;
            }
        }

        // Insert mutation (new gene)
        let max_genes = 64;
        if (self.bytes.len() / 8) < max_genes && rng.gen_bool(0.02) {
            let mut new_gene = [0u8; 8];
            rng.fill(&mut new_gene);
            self.bytes.extend_from_slice(&new_gene);
        }

        // Delete mutation (loose a gene)
        if self.bytes.len() >= 8 && rng.gen_bool(0.02) {
            let gene_index = rng.gen_range(0..(self.bytes.len() / 8));
            let start = gene_index * 8;
            self.bytes.drain(start..start + 8);
        }
    }
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// Creature
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

pub struct Creature {
    pub dna   : Dna,
    pub name  : String,
    pub alive : bool,
    pub can_reproduce: bool,
    pub age   : u32,
    pub energy: f32,
    pub pos   : Coordinate,
    pub last_action: CreatureAction,
    pub last_output: u64,
    pub color : String,
    #[allow(dead_code)]
    genome    : Genome,
    brain     : Brain,
}

pub const OUTPUT_ACTION_MASK  : u64 = 0b00000000_00000000_00000000_11111111;
pub const OUTPUT_VALUE1_MASK  : u64 = 0b00000000_00000000_11111111_00000000;
pub const OUTPUT_VALUE2_MASK  : u64 = 0b00000000_11111111_00000000_00000000;
pub const OUTPUT_VALUE3_MASK  : u64 = 0b11111111_00000000_00000000_00000000;
// pub const OUTPUT_VALUE1_OFFSET: u8  =  4;
// pub const OUTPUT_VALUE2_OFFSET: u8  =  8;
// pub const OUTPUT_VALUE3_OFFSET: u8  = 12;

pub const ENERGY_COST_SLEEP: f32 = 5.1;
pub const ENERGY_COST_REPRODUCE: f32 = 50.0;
pub const ENERGY_COST_MOVE: f32 = 1.5;
pub const ENERGY_COST_FIRED_NEURON: f32 = 0.005;

pub const REPRODUCE_AGE_MIN: u32 = 20;
pub const REPRODUCE_AGE_MAX: u32 = 80;

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// public methods for Creature
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
impl Creature {
    /******************************************************************************************************************************************/
    pub fn new(rng: &mut impl Rng) -> Creature {
        
        // let mut genome = Genome::new();
        // // add the neurons to the genome
        // for i in 0..24 {
        //     genome.add_neuron(rng.gen_range(0..=0xFF_FFFF), rand::thread_rng().gen_range(0..=5), super::brain::NeuronKind::Hidden1, i);
        // }
        // for i in 0..16 {
        //     genome.add_neuron(rng.gen_range(0..=0xFF_FFFF), rand::thread_rng().gen_range(5..=12), super::brain::NeuronKind::Hidden2, i);
        // }
        // for i in 0..16 {
            
        //     genome.add_neuron(rng.gen_range(0..=0xFF_FFFF_FFFF), rand::thread_rng().gen_range(5..=12), super::brain::NeuronKind::Output, i);
        // }

        
        let new_dna: Dna = Dna::random(1024, rng);
        let new_genome: Genome = Genome::from_dna(&new_dna);
        let new_brain: Brain = Brain::recompile(&new_genome);

        Creature {
            dna: new_dna,
            name: format!("Creature{:03}", rand::random::<u8>()),
            alive: true,
            can_reproduce: false,
            age : 0,
            energy: 50.0,
            pos : Coordinate { x: 50.0, y: 50.0 },
            last_action: CreatureAction::Sleep,
            last_output: 0,
            genome: new_genome,
            brain: new_brain,
            color: format!("\x1b[{}m", (rng.gen_range(31..=37) as u8).to_string()),
        }
    }
    
    /******************************************************************************************************************************************/
    /// Constructor for when we have a newborn
    pub fn new_from_parent(parent: &Creature, rng: &mut impl Rng) -> Self {

        let mut child_dna = parent.dna.clone();
        child_dna.mutate(rng);
        let new_genome = Genome::from_dna(&child_dna);
        let new_brain = Brain::recompile(&new_genome);

        let child = Self {
            dna: child_dna,
            name: parent.name.clone() + "+",
            alive: true,
            can_reproduce: false,
            age : 0,
            energy: 50.0,
            pos: parent.pos,
            last_action: CreatureAction::Idle,
            last_output: 0,
            brain: new_brain,
            genome: new_genome,
            color: parent.color.clone(),
        };

        child
    }
    
    /******************************************************************************************************************************************/
    /// thought process and act accordingly
    pub fn think_and_act(&mut self) -> CreatureEvent {
        
        self.age += 1;
        self.can_reproduce = REPRODUCE_AGE_MIN <= self.age && self.age <= REPRODUCE_AGE_MAX && self.energy >= ENERGY_COST_REPRODUCE;

        // die, if the energy is depleted
        if self.energy <= 0.0 || self.age >= 100 {
            self.alive = false;
            return CreatureEvent::Die;
        }

        let mut return_event = CreatureEvent::None;
        
        let (tick_output, _fired_count) = self.brain.tick(self.sensors_to_bits());
        self.last_output = tick_output;
        let (action, _value1, _value2, _value3) = Self::decode_output(tick_output);

        // firing neurons do cost energy
        self.energy -= _fired_count as f32 * ENERGY_COST_FIRED_NEURON;

        match action {
            CreatureAction::Sleep => {
                self.last_action = CreatureAction::Sleep;
                if self.energy > 0.1 { self.energy -= ENERGY_COST_SLEEP; }
            }
            CreatureAction::Move => {
                // _value1: LSB of direction
                // _value2: MSB of direction
                // _value3: speed
                self.last_action = CreatureAction::Move;
                if self.energy > 0.2 {
                    let direction = (_value1 as u16 | (_value2 as u16) << 8) as f32 / 255.0 * 360.0;
                    let speed = _value3 as f32 / 255.0 * 10.0;
                    let mut new_pos = Coordinate {
                        x: self.pos.x + speed * direction.cos(),
                        y: self.pos.y + speed * direction.sin(),
                    };
                    // do the move if the new position is within bounds
                    if new_pos.x < 0.0   { new_pos.x = 0.0;   }
                    if new_pos.x > 100.0 { new_pos.x = 100.0; }
                    if new_pos.y < 0.0   { new_pos.y = 0.0;   }
                    if new_pos.y > 100.0 { new_pos.y = 100.0; }
                    
                    if new_pos.x >= 0.0 && new_pos.x <= 100.0 {
                        self.pos.x = new_pos.x;
                        self.energy -= ENERGY_COST_MOVE / 2.0;
                    }   
                    if new_pos.y >= 0.0 && new_pos.y <= 100.0 {
                        self.pos.y = new_pos.y;
                        self.energy -= ENERGY_COST_MOVE / 2.0;
                    }   
                }
            }
            CreatureAction::Eat => {
                self.last_action = CreatureAction::Eat;
                // eat if there is food at the current position (for simplicity: if pos.x and pos.y are both outside of 33 and 66)
                if self.energy < 90.0 && (self.pos.x <= 33.0 || self.pos.x >= 66.0 || self.pos.y <= 33.0 || self.pos.y >= 66.0) {
                    self.energy += 20.0;
                }
            }
            CreatureAction::Reproduce => {
                self.last_action = CreatureAction::Reproduce;
                if self.can_reproduce && self.energy >= ENERGY_COST_REPRODUCE {
                    self.can_reproduce = false;
                    self.energy -= ENERGY_COST_REPRODUCE;
                    return_event = CreatureEvent::Reproduce;
                }
            }
            _ => {
                self.last_action = CreatureAction::Idle;
                self.energy -= 10.0;
            }
        }

        return_event
    }
    
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// private methods for Creature
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
impl Creature {
    /******************************************************************************************************************************************/
    /// takes a value and encodes it into 3 bits according to the provided buckets
    #[inline(always)]
    fn encode_3bucket(val: f32, buckets: [f32; 2]) -> u64 {
        let low  = (val <= buckets[0]) as u64;
        let mid  = ((val > buckets[0]) && (val <= buckets[1])) as u64;
        let high = (val > buckets[1]) as u64;

        (low << 2) | (mid << 1) | high
    }
    
    /******************************************************************************************************************************************/
    /// creates an u64 with the input bits:
    ///    1 : Energy low
    ///    2 : Energy mid
    ///    3 : Energy high
    ///    4 : PosX left
    ///    5 : PosX center
    ///    6 : PosX right
    ///    7 : PosY top
    ///    8 : PosY center
    ///    9 : PosY bottom
    ///   10 : LastAction Idle
    ///   11 : LastAction Sleep
    ///   12 : LastAction Move
    ///   13 : LastAction Eat
    ///   14 : LastAction Reproduce
    ///   15 - 24: unused
    #[inline(always)]
    fn sensors_to_bits(&self) -> u64 {
        let mut binary_input: u64 = 0;
        // Energy: 0-33 = low,  34-66 = mid,    67-100 = high
        binary_input |= Self::encode_3bucket(self.energy, [33.0, 66.0]);
        // PosX:   0-33 = left, 34-66 = center, 67-100 = right
        binary_input |= Self::encode_3bucket(self.pos.x, [33.0, 66.0]) << 3;
        // PosY:   0-33 = top,  34-66 = center, 67-100 = bottom
        binary_input |= Self::encode_3bucket(self.pos.y, [33.0, 66.0]) << 6;
        // dedicated bit per action
        binary_input |= match self.last_action {
            CreatureAction::Idle      => 0b00000000_00000000_00000000,
            CreatureAction::Sleep     => 0b00000000_00000010_00000000,
            CreatureAction::Move      => 0b00000000_00000100_00000000,
            CreatureAction::Eat       => 0b00000000_00001000_00000000,
            CreatureAction::Reproduce => 0b00000000_00010000_00000000,
        };

        binary_input
    }

    /******************************************************************************************************************************************/
    /// decodes an output value into action and three parameters
    #[inline(always)]
	fn decode_output(output: u64) -> (CreatureAction, u8, u8, u8) {
		// ---- Parameter extrahieren ----
		let value1 = ((output & OUTPUT_VALUE1_MASK) >> OUTPUT_VALUE1_MASK.trailing_zeros()) as u8;
		let value2 = ((output & OUTPUT_VALUE2_MASK) >> OUTPUT_VALUE2_MASK.trailing_zeros()) as u8;
		let value3 = ((output & OUTPUT_VALUE3_MASK) >> OUTPUT_VALUE3_MASK.trailing_zeros()) as u8;

		// ---- Aktionsbits extrahieren ----
		let action_bits = (output & OUTPUT_ACTION_MASK) as u8;

		let action = if action_bits == 0 {
			CreatureAction::Idle
		} else {
			// höchstes gesetztes Bit gewinnt
			let highest_bit = 7 - action_bits.leading_zeros() as u8;

			match 1 << highest_bit {
				0b00000001 => CreatureAction::Move,
				0b00000010 => CreatureAction::Sleep,
				0b00000100 => CreatureAction::Eat,
				0b00001000 => CreatureAction::Reproduce,
				0b00010000 => CreatureAction::Idle,
				0b00100000 => CreatureAction::Idle,
				0b01000000 => CreatureAction::Idle,
				0b10000000 => CreatureAction::Idle,
				_ => CreatureAction::Idle,
			}
		};

		(action, value1, value2, value3)
	}

    /******************************************************************************************************************************************/
    /// debug output of the brain's thought process
    #[allow(dead_code)]
    #[inline(always)]
    fn print_input(&self) {
        println!("---- ---- ---- ---- ---- ---- ---- ---- ---- ---- ---- ---- ---- -A- -Y- -X- -E-");
        // println!("---- ---- ---- ---- ---- ---- ---- ---- ---- ---- ---- ---- ---- MES TCB LCR LMH");
        let b = format!("{:064b}", self.sensors_to_bits());

        // 2. Den String in den gewünschten Gruppen ausgeben (4-3-3-3-3)
        println!(
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            &b[0..4],  &b[4..8],  &b[8..12], &b[12..16],
            &b[16..20],&b[20..24],&b[24..28],&b[28..32],
            &b[32..36],&b[36..40],&b[40..44],&b[44..48],
            &b[48..52],&b[52..55],&b[55..58],&b[58..61],&b[61..64]
        );
    }
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// traits for Creature
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
impl Default for Creature {
    fn default() -> Self {
        Creature::new(&mut rand::thread_rng())
    }
}

impl ToString for Creature {
    fn to_string(&self) -> String {
        let (action, value1, value2, value3) = Self::decode_output(self.last_output);
        format!(
            "{}{:<20}, age: {:3}, energy: {:5.1}%, pos:[{:5.01}|{:5.01}], sensor: {:5}, last_output: {:5} ({:<9?}|{}|{}|{})\x1b[0m",
            self.color, self.name, self.age, self.energy, self.pos.x, self.pos.y, self.sensors_to_bits(), self.last_output, action, value1, value2, value3
        )
    }
}