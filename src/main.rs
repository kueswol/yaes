pub mod constants;
pub mod utils;
pub mod creature;
pub mod world;

use crate::world::World;
// use crate::constants as c;

use std::thread;
use std::time::Duration;

fn main() {
    
    // let mut world = World::new(c::RNG_WORLD_SEED);
    let mut world = World::default();

    println!("Creating creatures...");
    world.spawn_random_creature(500);
    
    
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

// give overview
// if cycle % 100 == 0 { print_creature_states(&creatures, &cycle, &counter_newborns, &counter_deaths); }
// fn print_creature_states(creatures: &[Creature], cycle: &usize, counter_newborns: &usize, counter_deaths: &usize) {
//     print!("\x1B[2J\x1B[H"); // clear + cursor home
//     print!("Population: {:5} (", creatures.len());
//     let mut bar: String = String::new();
//     for _ in 0..(creatures.len() / c::MAX_POPULATION * 100) {
//         bar+="-";
//     };println!("{:<100}) with {} newborns and {} deaths", bar, counter_newborns, counter_deaths);
//     println!("Cycle: {:5} / 500", cycle);
//     println!("");

//     // for creature in creatures.iter().take(70) {
//     //     println!("{}", creature.to_string());
//     // }
//     // println!("Cycle: {:5}/5000 | Population: {:5}/{} ({:3}%) | Newborns: {:3} | Deaths: {:7}", cycle, creatures.len(), MAX_POPULATION, creatures.len() * 100 / MAX_POPULATION, counter_newborns, counter_deaths);
// }
