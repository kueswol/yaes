
/// ---------------------------------------------------------------------------------------------------------
/// GENERAL
/// ---------------------------------------------------------------------------------------------------------

pub const RNG_WORLD_SEED                : u64   = 42;

/// ---------------------------------------------------------------------------------------------------------
/// WORLD
/// ---------------------------------------------------------------------------------------------------------

pub const MAX_POPULATION                : usize = 10_000;
pub const WORLD_WIDTH                   : u32   = 200;
pub const WORLD_HEIGHT                  : u32   = 200;
pub const SPATIAL_HASHMAP_CELL_SIZE     : u32   = 5;   // must be a divisor of WORLD_WIDTH and WORLD_HEIGHT

pub const FOOD_REGROWTH_TICKS           : u64   =  7;  // must be `(2^n) - 1`, so 3, 7, 15, 31, etc
pub const FOOD_REGROWTH_AMOUNT          : u8    = 15;  // primefactors of 255 would be 3, 5, 17   

/// ---------------------------------------------------------------------------------------------------------
/// CREATURE
/// ---------------------------------------------------------------------------------------------------------
pub const OUTPUT_ACTION_MASK               : u64   = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111;
pub const OUTPUT_VALUE1_MASK               : u64   = 0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000;
pub const OUTPUT_VALUE2_MASK               : u64   = 0b00000000_00000000_00000000_00000000_00000000_11111111_00000000_00000000;
pub const OUTPUT_VALUE3_MASK               : u64   = 0b00000000_00000000_00000000_00000000_11111111_00000000_00000000_00000000;
pub const OUTPUT_FIRED_NEURONS_MASK        : u64   = 0b00000000_00000000_11111111_11111111_00000000_00000000_00000000_00000000;

pub const ENERGY_COST_EAT                  : f32   = -15.0;
pub const ENERGY_COST_SLEEP                : f32   =  -1.0;
pub const ENERGY_COST_REPRODUCE            : f32   =  38.0; // with 33 we get a blob only spawning
pub const ENERGY_COST_MOVE                 : f32   =   0.025;
pub const ENERGY_COST_MOVE_SPRINT          : f32   =   0.075;
pub const ENERGY_COST_FIRED_NEURON         : f32   =   0.005;
pub const ENERGY_COST_IDLE                 : f32   =  10.0;

pub const REPRODUCE_AGE_MIN                : u64   =  250;
pub const REPRODUCE_AGE_MAX                : u64   = 5000;

pub const CREATURE_SPEED                   : f32   =    0.06; // Base Speed - will be divided by size
pub const CREATURE_SPEED_SPRINT            : f32   =    0.10; // Base Speed - will be divided by size
pub const CREATURE_MAX_AGE                 : u64   = 5000;

pub const CREATURE_BITFLAG_IS_ALIVE        : u8    = 0b0000_0001;
pub const CREATURE_BITFLAG_CAN_REPRODUCE   : u8    = 0b0000_0010;

pub const BRAIN_INPUTS_BUCKET_NRGY_LOW_MID : f32   =  40.0;
pub const BRAIN_INPUTS_BUCKET_NRGY_MID_HIGH: f32   =  70.0;

pub const BRAIN_INPUTS_BUCKET_AGE_LOW_MID  : u64   =  100;
pub const BRAIN_INPUTS_BUCKET_AGE_MID_HIGH : u64   =  500;

pub const BRAIN_INPUTS_BUCKET_POSX_L_C     : f32   =  30.0;
pub const BRAIN_INPUTS_BUCKET_POSX_C_R     : f32   =  70.0;
pub const BRAIN_INPUTS_BUCKET_POSY_T_C     : f32   =  30.0;
pub const BRAIN_INPUTS_BUCKET_POSY_C_B     : f32   =  70.0;

pub const BRAIN_OUTPUT_ACTION_REPRODUCE    : u8    = 0b0000_0001;
pub const BRAIN_OUTPUT_ACTION_SLEEP        : u8    = 0b0000_0010;
pub const BRAIN_OUTPUT_ACTION_EAT          : u8    = 0b0000_0100;
pub const BRAIN_OUTPUT_ACTION_MOVE         : u8    = 0b0000_1000;
//  const BRAIN_OUTPUT_ACTION_UNUSED       : u8    = 0b0001_0000;
//  const BRAIN_OUTPUT_ACTION_UNUSED       : u8    = 0b0010_0000;
//  const BRAIN_OUTPUT_ACTION_UNUSED       : u8    = 0b0100_0000;
//  const BRAIN_OUTPUT_ACTION_UNUSED       : u8    = 0b1000_0000;

pub const BRAIN_OUTPUT_VALUE1_TURN_LEFT    : u8    = 0b0000_0001;
pub const BRAIN_OUTPUT_VALUE1_TURN_RIGHT   : u8    = 0b0000_0010;
pub const BRAIN_OUTPUT_VALUE1_SPRINT       : u8    = 0b0000_0100;
//  const BRAIN_OUTPUT_VALUE1_UNUSED       : u8    = 0b0000_1000;
//  const BRAIN_OUTPUT_VALUE1_UNUSED       : u8    = 0b0001_0000;
//  const BRAIN_OUTPUT_VALUE1_UNUSED       : u8    = 0b0010_0000;
//  const BRAIN_OUTPUT_VALUE1_UNUSED       : u8    = 0b0100_0000;
//  const BRAIN_OUTPUT_VALUE1_UNUSED       : u8    = 0b1000_0000;

//  const BRAIN_OUTPUT_VALUE2_UNUSED       : u8    = 0b0000_0001;
//  const BRAIN_OUTPUT_VALUE2_UNUSED       : u8    = 0b0000_0010;
//  const BRAIN_OUTPUT_VALUE2_UNUSED       : u8    = 0b0000_0100;
//  const BRAIN_OUTPUT_VALUE2_UNUSED       : u8    = 0b0000_1000;
//  const BRAIN_OUTPUT_VALUE2_UNUSED       : u8    = 0b0001_0000;
//  const BRAIN_OUTPUT_VALUE2_UNUSED       : u8    = 0b0010_0000;
//  const BRAIN_OUTPUT_VALUE2_UNUSED       : u8    = 0b0100_0000;
//  const BRAIN_OUTPUT_VALUE2_UNUSED       : u8    = 0b1000_0000;

//  const BRAIN_OUTPUT_VALUE3_UNUSED       : u8    = 0b0000_0001;
//  const BRAIN_OUTPUT_VALUE3_UNUSED       : u8    = 0b0000_0010;
//  const BRAIN_OUTPUT_VALUE3_UNUSED       : u8    = 0b0000_0100;
//  const BRAIN_OUTPUT_VALUE3_UNUSED       : u8    = 0b0000_1000;
//  const BRAIN_OUTPUT_VALUE3_UNUSED       : u8    = 0b0001_0000;
//  const BRAIN_OUTPUT_VALUE3_UNUSED       : u8    = 0b0010_0000;
//  const BRAIN_OUTPUT_VALUE3_UNUSED       : u8    = 0b0100_0000;
//  const BRAIN_OUTPUT_VALUE3_UNUSED       : u8    = 0b1000_0000;


/// ---------------------------------------------------------------------------------------------------------
/// MUTATION
/// ---------------------------------------------------------------------------------------------------------

pub const MUTATE_CHANCE_BIT_FLIP_MASK      : f64   = 0.33;
pub const MUTATE_CHANCE_CHANGE_THRESHOLD   : f64   = 0.75;
pub const MUTATE_CHANCE_CHANGE_TARGET_BIT  : f64   = 0.001;
pub const MUTATE_CHANCE_GAINING_NEW_NEURON : f64   = 0.10;
pub const MUTATE_CHANCE_LOOSING_NEW_NEURON : f64   = 0.01;

/// ---------------------------------------------------------------------------------------------------------
/// BRAIN
/// ---------------------------------------------------------------------------------------------------------

pub const NEURON_HIDDEN1_MASK_SCOPE        : u64   = 0b00000000_00000000_00000000_00000000_00000000_11111111_11111111_11111111;
pub const NEURON_HIDDEN1_MASK_OFFSET       : u8    = 0;
pub const NEURON_HIDDEN1_TARGET_OFFSET     : u8    = 24;

pub const NEURON_HIDDEN2_MASK_SCOPE        : u64   = 0b00000000_00000000_11111111_11111111_11111111_00000000_00000000_00000000;
pub const NEURON_HIDDEN2_MASK_OFFSET       : u8    = 24;
pub const NEURON_HIDDEN2_TARGET_OFFSET     : u8    = 48;

pub const NEURON_OUTPUT_MASK_SCOPE         : u64   = 0b11111111_11111111_11111111_11111111_11111111_00000000_00000000_00000000;
pub const NEURON_OUTPUT_MASK_OFFSET        : u8    = 24;
pub const NEURON_OUTPUT_TARGET_OFFSET      : u8    = 0;
