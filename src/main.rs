pub mod constants;
pub mod ecs;
pub mod utils;
pub mod main_simulation;
pub mod main_webserver;
pub mod web;

use crate::ecs::World;
use rayon::ThreadPoolBuilder;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// MAIN
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

#[tokio::main]
async fn main() {

    // Initialize Rayon thread pool
    ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .expect("Failed to build Rayon thread pool");

    // THE WORLD
    let world = Arc::new(Mutex::new(initialize_world()));    
    println!("[MAIN ]: starting simulation thread");
    start_simulation(world.clone());
    
    // THE WEB
    println!("[MAIN ]: starting webserver");
    start_webserver(world.clone()).await;
    
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// main's helper functions
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

/******************************************************************************************************************************************/
/// Initializes the world and spawns some initial creatures.
fn initialize_world() -> World {
    
    // let world = World::new(c::RNG_WORLD_SEED);
    // let world = World::new(3263687907895456594);
    let mut world = World::default();

    println!("[MAIN ]: initializing world");
    world.spawn_random_creatures(5000);

    world
}

/******************************************************************************************************************************************/
/// Starts the simulation in a separate thread.
fn start_simulation(world: Arc<Mutex<World>>) {
    thread::spawn(move || {
        main_simulation::run_simulation(world);
    });
}

/******************************************************************************************************************************************/
/// Starts the simulation in a separate thread.
async fn start_webserver(world: Arc<Mutex<World>>) {
    main_webserver::start_webserver(world).await;
}

/******************************************************************************************************************************************/
/// Keeps the main thread alive indefinitely.
#[allow(dead_code)]
fn wait_forever() {
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}


#[test]
fn test() {

    let cost: f32 = -15.0;
    let cost_u8: u8 = cost.abs() as u8;
    let old_value: u8 = 100;
    let new_value: u8 = old_value.saturating_sub(cost.abs() as u8);
    println!("=== TESTING ===\n");
    println!("cost      : {}", cost);
    println!("cost_u8   : {}", cost_u8);
    println!("old_value : {}", old_value);
    println!("new_value : {}", new_value);
    println!("\n===============");
}