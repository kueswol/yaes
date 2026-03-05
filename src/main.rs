pub mod constants;
pub mod utils;
pub mod creature;
pub mod world;

use crate::world::World;
use std::thread;
use std::time::Duration;

fn main() {
    
    // let mut world = World::new(c::RNG_WORLD_SEED);
    let mut world = World::default();

    println!("Creating creatures...");
    world.spawn_random_creature(10000);
    
    // // uncomment to print the DNA of the first creature and exit immediately (for testing)
    // println!("{}",world.creatures[0].dna.to_compact_string());
    // return;

    println!("Starting simulation...");
    for _ in 0..50_000 {
        world.tick();
        world.print_to_terminal(true,false);
        thread::sleep(Duration::from_millis(5));
        if world.creatures.len() < 1 {
            // println!("The world is empty. Spawning new creature...");
            // world.spawn_random_creature(100);
            break;
        }
    }
    println!("Simulation ended.");
    println!("We had a total of {} newborns and {} deaths during {} cycles.", world.counter_newborns, world.counter_deaths, world.cycle);
}

