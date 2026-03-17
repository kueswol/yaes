
/// ---------------------------------------------------------------------------------------------------------
/// GENERAL
/// ---------------------------------------------------------------------------------------------------------

pub const RNG_WORLD_SEED                : u64   = 42;

/// ---------------------------------------------------------------------------------------------------------
/// WORLD
/// ---------------------------------------------------------------------------------------------------------

pub const MAX_POPULATION                : usize = 10_000;
pub const WORLD_WIDTH                   : u32   = 100;
pub const WORLD_HEIGHT                  : u32   = 100;

/// ---------------------------------------------------------------------------------------------------------
/// CREATURE
/// ---------------------------------------------------------------------------------------------------------
pub const OUTPUT_ACTION_MASK            : u64   = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111;
pub const OUTPUT_VALUE1_MASK            : u64   = 0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000;
pub const OUTPUT_VALUE2_MASK            : u64   = 0b00000000_00000000_00000000_00000000_00000000_11111111_00000000_00000000;
pub const OUTPUT_VALUE3_MASK            : u64   = 0b00000000_00000000_00000000_00000000_11111111_00000000_00000000_00000000;
pub const OUTPUT_FIRED_NEURONS_MASK     : u64   = 0b00000000_00000000_11111111_11111111_00000000_00000000_00000000_00000000;

pub const ENERGY_COST_SLEEP             : f32   =  -1.0;
pub const ENERGY_COST_REPRODUCE         : f32   =  30.0;
pub const ENERGY_COST_MOVE              : f32   =   1.0;
pub const ENERGY_COST_FIRED_NEURON      : f32   =   0.01;
pub const ENERGY_COST_IDLE              : f32   =  10.0;

pub const REPRODUCE_AGE_MIN             : u32   =   20;
pub const REPRODUCE_AGE_MAX             : u32   = 2000;

pub const CREATURE_MAX_SPEED            : f32   =    2.5;
pub const CREATURE_MAX_AGE              : u32   = 2000;

pub const CREATURE_BITFLAG_IS_ALIVE     : u8    = 0b0000_0001;
pub const CREATURE_BITFLAG_CAN_REPRODUCE: u8    = 0b0000_0010;

pub const BRAIN_INPUTS_ENERGY_LOW       : f32   =  30.0;
pub const BRAIN_INPUTS_ENERGY_MID       : f32   =  70.0;
pub const BRAIN_INPUTS_ENERGY_HIGH      : f32   = 100.0;

pub const BRAIN_INPUTS_AGE_LOW          : u32   =  100;
pub const BRAIN_INPUTS_AGE_MID          : u32   =  500;
pub const BRAIN_INPUTS_AGE_HIGH         : u32   = 1000;

pub const BRAIN_INPUTS_POSX_LEFT        : f32   =  30.0;
pub const BRAIN_INPUTS_POSX_CENTER      : f32   =  70.0;
pub const BRAIN_INPUTS_POSX_RIGHT       : f32   = 100.0;

pub const BRAIN_INPUTS_POSY_TOP         : f32   =  30.0;
pub const BRAIN_INPUTS_POSY_CENTER      : f32   =  70.0;
pub const BRAIN_INPUTS_POSY_BOTTOM      : f32   = 100.0;

/// ---------------------------------------------------------------------------------------------------------
/// BRAIN
/// ---------------------------------------------------------------------------------------------------------

pub const NEURON_HIDDEN1_MASK_SCOPE     : u64   = 0b00000000_00000000_00000000_00000000_00000000_11111111_11111111_11111111;
pub const NEURON_HIDDEN1_MASK_OFFSET    : u8    = 0;
pub const NEURON_HIDDEN1_TARGET_OFFSET  : u8    = 24;

pub const NEURON_HIDDEN2_MASK_SCOPE     : u64   = 0b00000000_00000000_11111111_11111111_11111111_00000000_00000000_00000000;
pub const NEURON_HIDDEN2_MASK_OFFSET    : u8    = 24;
pub const NEURON_HIDDEN2_TARGET_OFFSET  : u8    = 48;

pub const NEURON_OUTPUT_MASK_SCOPE      : u64   = 0b11111111_11111111_11111111_11111111_11111111_00000000_00000000_00000000;
pub const NEURON_OUTPUT_MASK_OFFSET     : u8    = 24;
pub const NEURON_OUTPUT_TARGET_OFFSET   : u8    = 0;
