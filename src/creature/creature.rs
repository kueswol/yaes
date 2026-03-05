use crate::utils::{Coordinate, CreatureAction, CreatureEvent};
use crate::constants as c;
use super::brain::Brain;
use super::genome::Genome;
use super::dna::Dna;
use rand::Rng;


/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// Creature
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
#[derive(Clone)]
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

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// public methods for Creature
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
impl Creature {
    /******************************************************************************************************************************************/
    pub fn new(rng: &mut impl Rng) -> Creature {
        
        let new_dna: Dna = Dna::random(256, rng);
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
    pub fn think(&mut self) -> CreatureEvent {
        
        self.age += 1;
        self.can_reproduce = c::REPRODUCE_AGE_MIN <= self.age && self.age <= c::REPRODUCE_AGE_MAX && self.energy >= c::ENERGY_COST_REPRODUCE;

        // die, if the energy is depleted
        if self.energy <= 0.0 || self.age >= c::CREATURE_MAX_AGE {
            self.alive = false;
            return CreatureEvent::Die;
        }

        let return_event: CreatureEvent;
        
        let (tick_output, _fired_count) = self.brain.tick(self.sensors_to_bits());
        self.last_output = tick_output;
        let (action, _value1, _value2, _value3) = Self::decode_output(self.last_output);

        // firing neurons do cost energy
        self.energy -= _fired_count as f32 * c::ENERGY_COST_FIRED_NEURON;

        match action {
            
            CreatureAction::Sleep => {
                self.last_action = CreatureAction::Sleep;
                return_event = CreatureEvent::Sleep;
            }
            
            CreatureAction::Move => {
                // _value1: LSB of direction
                // _value2: MSB of direction
                // _value3: speed
                let direction = (_value1 as u16 | (_value2 as u16) << 8) as f32 / (u16::MAX as f32) * 360.0;
                let speed = _value3 as f32 / (u8::MAX as f32) * c::CREATURE_MAX_SPEED;
                
                self.last_action = CreatureAction::Move;
                return_event = CreatureEvent::Move { direction, speed };
            }

            CreatureAction::Eat => {
                self.last_action = CreatureAction::Eat;
                return_event = CreatureEvent::Eat;
            }

            CreatureAction::Reproduce => {
                self.last_action = CreatureAction::Reproduce;
                return_event = CreatureEvent::Reproduce;
            }

            _ => {
                self.last_action = CreatureAction::Idle;
                return_event = CreatureEvent::None;
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
    ///   15 : can_reproduce
    ///   16 : age low
    ///   17 : age mid
    ///   18 : age high
    ///   16 - 24: unused
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
            CreatureAction::Idle      => 0b0 << 9,
            CreatureAction::Sleep     => 0b1 << 10,
            CreatureAction::Move      => 0b1 << 11,
            CreatureAction::Eat       => 0b1 << 12,
            CreatureAction::Reproduce => 0b1 << 13,
        };
        // can reproduce bit
        binary_input |= (self.can_reproduce as u64) << 14;
        // age: 0-33 = low,  34-66 = mid,    67-100 = high
        binary_input |= Self::encode_3bucket(self.age as f32, [30.0, 100.0]) << 15;

        binary_input
    }

    /******************************************************************************************************************************************/
    /// decodes an output value into action and three parameters
    #[inline(always)]
	fn decode_output(output: u64) -> (CreatureAction, u8, u8, u8) {
		// extract the value parameters
		let value1 = ((output & c::OUTPUT_VALUE1_MASK) >> c::OUTPUT_VALUE1_MASK.trailing_zeros()) as u8;
		let value2 = ((output & c::OUTPUT_VALUE2_MASK) >> c::OUTPUT_VALUE2_MASK.trailing_zeros()) as u8;
		let value3 = ((output & c::OUTPUT_VALUE3_MASK) >> c::OUTPUT_VALUE3_MASK.trailing_zeros()) as u8;

		// extract the bits that determine the action
		let action_bits = (output & c::OUTPUT_ACTION_MASK) as u8;

        // if it is zero we define it as Idle
		let action = if action_bits == 0 {
			CreatureAction::Idle
		} else {
			// we're focusing on the highest action bit
			let highest_bit = 7 - action_bits.leading_zeros() as u8;

			match 1 << highest_bit {
				0b00000001 => CreatureAction::Reproduce,
				0b00000010 => CreatureAction::Sleep,
				0b00000100 => CreatureAction::Eat,
				0b00001000 => CreatureAction::Move,
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
            "{}{:<20}, age: {:3}, energy: {:5.1}%, pos:[{:5.01}|{:5.01}], sensor: {:5}, last_output: {:12} ({:<10?}|{}|{}|{}), brain: {}\x1b[0m",
            self.color, self.name, self.age, self.energy, self.pos.x, self.pos.y, self.sensors_to_bits(), self.last_output, action, value1, value2, value3, self.brain.to_string()
        )
    }
}