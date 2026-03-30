pub mod constants;
pub mod ecs;
pub mod main_simulation;
pub mod main_webserver;
pub mod utils;
pub mod web;

use crate::ecs::World;
use crate::utils::ChannelWeb2SimMessage;
use rayon::ThreadPoolBuilder;
use std::sync::{Arc, Mutex};
use std::thread;

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

    // prepare some channels for communication between the simulation and the webserver
    let (channel_web2sim_tx, channel_web2sim_rx) =
        std::sync::mpsc::channel::<ChannelWeb2SimMessage>();

    // THE WORLD
    let world = Arc::new(Mutex::new(initialize_world()));
    println!("[MAIN ]: starting simulation thread");
    start_simulation(world.clone(),channel_web2sim_rx);

    // THE WEB
    println!("[MAIN ]: starting webserver");
    start_webserver(world.clone(),channel_web2sim_tx).await;
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// main's helper functions
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

/******************************************************************************************************************************************/
/// Initializes the world and spawns some initial creatures.
fn initialize_world() -> World {
    // let world = World::new(c::RNG_WORLD_SEED);
    // let world = World::new(3263687907895456594);
    let world = World::default();

    // println!("[MAIN ]: initializing world");
    // world.spawn_random_creatures(2000);

    world
}

/******************************************************************************************************************************************/
/// Starts the simulation in a separate thread.
fn start_simulation(world: Arc<Mutex<World>>, channel_web2sim_rx: std::sync::mpsc::Receiver<ChannelWeb2SimMessage>) {
    thread::spawn(move || {
        main_simulation::run_simulation(world, channel_web2sim_rx);
    });
}

/******************************************************************************************************************************************/
/// Starts the simulation in a separate thread.
async fn start_webserver(world: Arc<Mutex<World>>, channel_web2sim_tx: std::sync::mpsc::Sender<ChannelWeb2SimMessage>) {
    main_webserver::start_webserver(world, channel_web2sim_tx).await;
}

#[test]
fn test() {
    // let mut world = World::default();
    // // world.spawn_creature(None, None);
    // world.tick();


    let total_food: u64 = 3999999;
    let cycle_count = 4 - (total_food / 1_000_000);
    println!("{}", cycle_count);
}