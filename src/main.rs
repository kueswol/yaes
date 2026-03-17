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
    world.spawn_random_creatures(1000);

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

/*
     // let mut world = World::new(c::RNG_WORLD_SEED);
     // let mut world = World::new(3263687907895456594);
     let mut world = World::default();
 
     let dna_string: String = "kTUeQxUECwD+APq9ygQJATKQO9dyBAYC6/UY3ncEBwOfJhjkw4q5zHyeMfJKOML4IlGSC7RSiKht8dk7b7uG/1xCtHiPZvypt4jofbil2twSes/Cr9dTG7mcU8T8XuT1NYsXedGNb+Ie2EtnNI/7pMbczhQkIe7nx+f1/gFbZzkjxHTqyQkfcf+1KJ8Zje5h/M1Rlu6T8kYiI4CYBDBUCExW5BQhPS7tAZpXPUVBCOy/qKcYrTvm4wp9nX12f6rFjHLbaIvyl6RnUfXc/KEqzsUKh5qaAwsHx7r3xTCxUuhLZOwVAapDAJ70N2oXvpS98RpNj88+h7nePVCiC5l9pcAHin8J1Mi+cMaagQSjuY7ka6bOSZNoHHwUd6uKCiiVt+OkkqgQNkTPqMfXBtqSQC6o0UB1fW1Y6KDsz7Y2QEQSO/Cj2zGItmMlkshoOkZ+mnasR9HsbUbmF0n0Kat3nQzaghBBAYT8QaMlokdmocdf1pCtWXoPsCXJnxMt6hrb".to_string();
 
     for _ in 0..1_000 {
         // world.spawn_creature(None, Some(Coordinate{x:51.0,y:30.0}));
         // world.spawn_creature(None, Some(Coordinate{x:52.0,y:52.0}));
         // world.spawn_creature(None, Some(Coordinate{x:53.0,y:70.0}));
         world.spawn_creature(Some(Dna::from_compact_string(&dna_string)), Some(Coordinate{x:51.0,y:51.0}));
         world.spawn_random_creatures(999);
 
         thread::sleep(Duration::from_millis(100));
         let ticks: u64 = 10_000;
         let mut tick_durations: Vec<f64> = Vec::with_capacity(ticks as usize);
         println!("------------------------------------------------------------");
         println!("LOOP START");
         for _tick in 0..ticks {
             let tick_duration_start = Instant::now();
             world.tick();
             let tick_duration: Duration = tick_duration_start.elapsed();
             
             if _tick % 100 == 0 {
                 // println!(
                 //     "Tick duration for {:7} creatures: {:15}µs --> {:7.2} tps",
                 //     world.get_creature_count(),
                 //     tick_duration.as_micros(),
                 //     1.0 / tick_duration.as_secs_f64()
                 // );
                 println!("Tick {:5}: {:7} creatures, * {:10},  † {:10}, AvgE: {:6.2}, Eat {:10}✓, {:10}✕, Repro {:10}✓, {:10}✕ Age, {:10}✕ Energy",
                     _tick,
                     world.get_creature_count(),
                     world.births, world.deaths, world.avg_energy, world.eat_success, world.eat_failed, world.reproduce_success, world.reproduce_failed_age, world.reproduce_failed_energy
                 );
                 world.eat_success             = 0;
                 world.eat_failed              = 0;
                 world.reproduce_success       = 0;
                 world.reproduce_failed_age    = 0;
                 world.reproduce_failed_energy = 0;
             }
             tick_durations.push(tick_duration.as_secs_f64());
             if world.get_creature_count() < 1 {
                 println!("The world is empty.");
                 break;
             }
             thread::sleep(Duration::from_millis(10));
         }
         println!("LOOP END");
         println!(
             "Average Tick Duration for {} ticks: {:15.0}µs --> {:7.2} tps",
             tick_durations.len(),
             (tick_durations.iter().sum::<f64>() / tick_durations.len() as f64) * 1_000_000.0,
             1.0 / (tick_durations.iter().sum::<f64>() / tick_durations.len() as f64)
         );
         println!(
             "Creatures:  * {:10},  † {:10}",
             world.births, world.deaths
         );
         if world.successfully_reproducing_dna.len() > 0 {
             println!(
                 "World-Seed:\n  {}\nSuccessfully Reproducing DNA:\n  {}",
                 world.seed,
                 world.successfully_reproducing_dna[0]
             );
             break;
         }
         else {
             println!("No successful reproduction. Restarting...");
             thread::sleep(Duration::from_millis(500));
         }
     }
*/

// #[test]
// fn test() {

// }