use serde::Serialize;

#[derive(Serialize)]
pub struct WorldView {
    pub tick: u64,
    pub population: usize,
}

#[derive(Serialize)]
pub struct CreatureView {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub color: [u8; 3],
    pub orientation: f32,
}

#[derive(Serialize)]
pub struct DeadCreatureView {
    pub x: f32,
    pub y: f32,
    pub orientation: f32,
    pub size: f32,
}

#[derive(Serialize)]
pub struct CreatureDetailView {
    pub id: usize,
    pub energy: f32,
}