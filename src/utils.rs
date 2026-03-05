
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
    Move { direction: f32, speed: f32 },
    Eat,
    Reproduce,
    Die,
}