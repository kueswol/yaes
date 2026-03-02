use crate::utils::{Coordinate, CreatureAction, CreatureEvent};
use super::brain::{Brain, Genome};
use rand::Rng;

pub struct Creature {
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

pub const OUTPUT_ACTION_MASK  : u64 = 0b0000_0000_0000_1111;
pub const OUTPUT_VALUE1_MASK  : u64 = 0b0000_0000_1111_0000;
pub const OUTPUT_VALUE2_MASK  : u64 = 0b0000_1111_0000_0000;
pub const OUTPUT_VALUE3_MASK  : u64 = 0b1111_0000_0000_0000;
pub const OUTPUT_VALUE1_OFFSET: u8  =  4;
pub const OUTPUT_VALUE2_OFFSET: u8  =  8;
pub const OUTPUT_VALUE3_OFFSET: u8  = 12;

pub const ENERGY_COST_SLEEP: f32 = 5.1;
pub const ENERGY_COST_REPRODUCE: f32 = 50.0;
pub const ENERGY_COST_MOVE: f32 = 1.5;

pub const REPRODUCE_AGE_MIN: u32 = 20;
pub const REPRODUCE_AGE_MAX: u32 = 60;

/// ---------------------------------------------------------------------------------------------------------
/// public methods for Creature
/// ---------------------------------------------------------------------------------------------------------
impl Creature {
    
    pub fn new() -> Creature {
        let mut genome = Genome::new();
        // add the neurons to the genome
        for i in 0..16 {
            let rnd: u16 = rand::random();
            genome.add_neuron(rnd as u64, rand::thread_rng().gen_range(0..=3), super::brain::NeuronKind::Hidden1, i);
        }
        for i in 0..16 {
            let rnd: u16 = rand::random();
            genome.add_neuron((rnd as u64) << 16, rand::thread_rng().gen_range(5..=10), super::brain::NeuronKind::Hidden2, i);
        }
        for i in 0..16 {
            let rnd: u32 = rand::random();
            genome.add_neuron((rnd as u64) << 16, rand::thread_rng().gen_range(5..=11), super::brain::NeuronKind::Output, i);
        }

        let brain = Brain::recompile(&genome);

        Creature {
            name: format!("Creature{:03}", rand::random::<u8>()),
            alive: true,
            can_reproduce: false,
            age : 0,
            energy: 50.0,
            pos : Coordinate { x: 50.0, y: 50.0 },
            last_action: CreatureAction::Sleep,
            last_output: 0,
            genome: genome,
            brain: brain,
            color: format!("\x1b[{}m", (rand::thread_rng().gen_range(31..=37) as u8).to_string()),
        }
    }
    
    /// Constructor for when we have a newborn
    pub fn new_from_parent(parent: &Creature) -> Self {
        let mut rng = rand::thread_rng();
        let mut genome = parent.genome.clone();
        genome.mutate(&mut rng);

        let child = Self {
            name: parent.name.clone() + "+",
            alive: true,
            can_reproduce: false,
            age : 0,
            energy: 50.0,
            pos: parent.pos,
            last_action: CreatureAction::Idle,
            last_output: 0,
            brain: Brain::recompile(&genome),
            genome: genome,
            color: parent.color.clone(),
        };

        child
    }
    
    /// Einen Gedankenzyklus durchführen und danach handeln
    pub fn think_and_act(&mut self) -> CreatureEvent {
        
        self.age += 1;
        self.can_reproduce = REPRODUCE_AGE_MIN <= self.age && self.age <= REPRODUCE_AGE_MAX && self.energy >= ENERGY_COST_REPRODUCE;

        // die, if the energy is depleted
        if self.energy <= 0.0 || self.age >= 100 {
            self.alive = false;
            return CreatureEvent::Die;
        }

        let mut return_event = CreatureEvent::None;
        
        self.last_output = self.brain.tick(self.sensors_to_bits());        
        let (action, _value1, _value2, _value3) = Self::decode_output(self.last_output);

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
                    let direction = (_value1 | _value2 << 4) as f32 / 15.0 * 360.0;
                    let speed = _value3 as f32 / 15.0 * 10.0;
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

/// ---------------------------------------------------------------------------------------------------------
/// private methods for Creature
/// ---------------------------------------------------------------------------------------------------------
impl Creature {
    /// Den übergebenen Wert in 3 Buckets (low/mid/high) einteilen und als Bits zurückgeben
    #[inline(always)]
    fn encode_3bucket(val: f32, buckets: [f32; 2]) -> u64 {
        let low  = (val <= buckets[0]) as u64;
        let mid  = ((val > buckets[0]) && (val <= buckets[1])) as u64;
        let high = (val > buckets[1]) as u64;

        (low << 2) | (mid << 1) | high
    }
    
    /// Setzt die Input-Werte in Form von Bits:
    ///    1 -  3: Energy (low|mid|high)
    ///    4 -  6: PosX (left|center|right)
    ///    7 -  9: PosY (top|center|bottom)
    ///   10 - 13: LastAction index binary encoded
    ///   14 - 16: unused
    #[inline(always)]
    fn sensors_to_bits(&self) -> u64 {
        let mut binary_input: u64 = 0;
        // Energy: 0-33 = low,  34-66 = mid,    67-100 = high
        // PosX:   0-33 = left, 34-66 = center, 67-100 = right
        // PosY:   0-33 = top,  34-66 = center, 67-100 = bottom
        // LastAction: 4 bits binary encoded index of the last action (0-15)
        binary_input |= Self::encode_3bucket(self.energy, [33.0, 66.0]);
        binary_input |= Self::encode_3bucket(self.pos.x, [33.0, 66.0]) << 3;
        binary_input |= Self::encode_3bucket(self.pos.y, [33.0, 66.0]) << 6;
        binary_input |= (self.last_action.index() as u64) << 9;

        binary_input
    }

    // #[inline(always)]
	fn decode_output(output: u64) -> (CreatureAction, u8, u8, u8) {
		// ---- Parameter extrahieren ----
		let value1 = ((output & OUTPUT_VALUE1_MASK) >> OUTPUT_VALUE1_OFFSET) as u8;
		let value2 = ((output & OUTPUT_VALUE2_MASK) >> OUTPUT_VALUE2_OFFSET) as u8;
		let value3 = ((output & OUTPUT_VALUE3_MASK) >> OUTPUT_VALUE3_OFFSET) as u8;

		// ---- Aktionsbits extrahieren ----
		let action_bits = (output & OUTPUT_ACTION_MASK) as u8;

		let action = if action_bits == 0 {
			CreatureAction::Idle
		} else {
			// höchstes gesetztes Bit gewinnt
			let highest_bit = 7 - action_bits.leading_zeros() as u8;

			match 1 << highest_bit {
				0b0001 => CreatureAction::Reproduce,
				0b0010 => CreatureAction::Sleep,
				0b0100 => CreatureAction::Eat,
				0b1000 => CreatureAction::Move,
				_ => CreatureAction::Idle,
			}
		};

		(action, value1, value2, value3)
	}

    /// Helper zum Ausgeben des binären Input
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

/// ---------------------------------------------------------------------------------------------------------
/// traits for Creature
/// ---------------------------------------------------------------------------------------------------------
impl Default for Creature {
    fn default() -> Self {
        Creature::new()
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