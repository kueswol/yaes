use serde::Serialize;


/****************************************************************************************************************/
#[derive(Clone, Copy)]
pub struct Coordinate {
    pub x: f32,
    pub y: f32,
}

/****************************************************************************************************************/
#[derive(Clone, Copy, Serialize, serde::Deserialize)]
pub struct SimParams {
    pub world: SimParamWorld,
    pub energy: SimParamEnergy,
    pub mutation: SimParamMutation,
    pub target_tps: f64,
    pub paused: bool,
}

/****************************************************************************************************************/
#[derive(Clone, Copy, Serialize, serde::Deserialize)]
pub struct SimParamWorld {
    pub max_population: usize,
    pub min_population: usize,
    pub food_regrowth_amount: u8,
    pub food_regrowth_ticks: u64,
}

/****************************************************************************************************************/
#[derive(Clone, Copy, Serialize, serde::Deserialize)]
pub struct SimParamEnergy {
    pub cost_eat: f32,
    pub cost_sleep: f32,
    pub cost_reproduce: f32,
    pub cost_move_slow: f32,
    pub cost_move_norm: f32,
    pub cost_move_fast: f32,
}

/****************************************************************************************************************/
#[derive(Clone, Copy, Serialize, serde::Deserialize)]
pub struct SimParamMutation {
    pub chance_bit_flip_mask      : f64,
    pub chance_change_threshold   : f64,
    pub chance_change_target_bit  : f64,
    pub chance_gaining_new_neuron : f64,
    pub chance_loosing_new_neuron : f64,
    pub chance_mutate_looks       : f64,
}

/****************************************************************************************************************/
/// actions a creature can take
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CreatureAction{
    Idle,      // 0b0000
    Sleep,     // 0b0001
    Move,      // 0b0010
    Eat,       // 0b0100
    Reproduce, // 0b1000
}
impl CreatureAction {
    pub const fn index(self) -> usize {
        self as usize
    }
}

/****************************************************************************************************************/
/// events that can happen to a creature after thinking
pub enum CreatureEvent {
    None,
    Sleep,
    Move { sprint: bool, creep: bool },
    Eat,
    Reproduce,
    Die,
}

/****************************************************************************************************************/
/// a struct to transport the world's statistics
#[derive(Serialize)]
pub struct WorldStats {
    pub tick: u64,
    pub population: u64,
    pub avg_energy: f32,
    pub avg_age: f32,
    pub total_food: u64,
    pub births: u64,
    pub deaths: u64,
    pub eat_success: u64,
    pub eat_failed: u64,
    pub reproduce_success: u64,
    pub reproduce_failed_age: u64,
    pub reproduce_failed_energy: u64,
    pub reproduce_failed_cooldown: u64,
}

/****************************************************************************************************************/
/// events that can happen to a creature after thinking
#[derive(Serialize,serde::Deserialize)]
pub enum ChannelWeb2SimMessage {
    PauseSim,
    ResumeSim,
    SetTargetTPS(f64),
    UpdateSimParams(SimParams),
}
