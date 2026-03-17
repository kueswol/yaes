use serde::Serialize;

#[derive(Serialize)]
pub struct WorldView {
    pub tick: u64,
    pub population: usize,
}

#[derive(Serialize)]
pub struct CreatureView {
    pub x: u32,
    pub y: u32,
}