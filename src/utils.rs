use serde::Serialize;


/****************************************************************************************************************/
#[derive(Clone, Copy)]
pub struct Coordinate {
    pub x: f32,
    pub y: f32,
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
    Move { sprint: bool },
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
    pub births: u64,
    pub deaths: u64,
    pub eat_success: u64,
    pub eat_failed: u64,
    pub reproduce_success: u64,
    pub reproduce_failed_age: u64,
    pub reproduce_failed_energy: u64,
    pub reproduce_failed_cooldown: u64,
}
