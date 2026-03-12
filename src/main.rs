pub mod constants;
pub mod creature;
pub mod ecs;
pub mod utils;
pub mod world;

// use crate::constants as c;
use crate::ecs::World;

use rayon::ThreadPoolBuilder;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    // // 10,50,100,150,200,300,400,500
    // // 750,1000,1250,1500,2000,2500
    // // 3_000,3_500,4_000,5_000,6_000,8_000,10_000
    // // 12_500,15_000,20_000,25_000,30_000,40_000,50_000
    // for i in [10,50,100,150,200,300,400,500,750,1000,1250,1500,2000,2500,3_000,3_500,4_000,5_000,6_000,8_000,10_000,12_500,15_000,20_000,25_000,30_000,40_000,50_000] {
    //     // println!("Spawning {} creatures...", i);
    //     let mut world = World::new(c::RNG_WORLD_SEED);
    //     world.spawn_random_creatures(i);
    //     let tick_duration_start = Instant::now();
    //     world.tick();
    //     let tick_duration: Duration = tick_duration_start.elapsed();
    //     println!("Tick duration for {:5} creatures: {:15}µs", world.get_creature_count(), tick_duration.as_micros());
    // }
    // return;

    ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .expect("Failed to build Rayon thread pool");

    // let mut world = World::new(c::RNG_WORLD_SEED);
    let mut world = World::default();

    // world.spawn_creature(None, Some(Coordinate{x:51.0,y:30.0}));
    // world.spawn_creature(None, Some(Coordinate{x:52.0,y:52.0}));
    // world.spawn_creature(None, Some(Coordinate{x:53.0,y:70.0}));
    for _ in 0..1_000 {
        world.spawn_random_creatures(50_000);

        thread::sleep(Duration::from_millis(100));
        let ticks: u64 = 10_000;
        let mut tick_durations: Vec<f64> = Vec::with_capacity(ticks as usize);
        println!("LOOP START");
        for _ in 0..ticks {
            let tick_duration_start = Instant::now();
            world.tick();
            let tick_duration: Duration = tick_duration_start.elapsed();
            println!(
                "Tick duration for {:7} creatures: {:15}µs --> {:7.2} tps",
                world.get_creature_count(),
                tick_duration.as_micros(),
                1.0 / tick_duration.as_secs_f64()
            );
            tick_durations.push(tick_duration.as_secs_f64());
            if world.get_creature_count() < 1 {
                println!("The world is empty.");
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        println!("LOOP END");
        if world.successfully_reproducing_dna.len() > 0 {
            println!(
                "Average Tick Duration: {:15.0}µs --> {:7.2} tps",
                (tick_durations.iter().sum::<f64>() / tick_durations.len() as f64) * 1_000_000.0,
                1.0 / (tick_durations.iter().sum::<f64>() / tick_durations.len() as f64)
            );
            println!(
                "Creatures:\n  * {:10}\n  † {:10}\nWorld-Seed:\n  {}",
                world.births, world.deaths, world.seed
            );
            println!(
                "Successfully Reproducing DNA:\n  {}",
                world.successfully_reproducing_dna.join("\n  ")
            );
            break;
        }
        else {
            println!("No successful reproduction. Restarting...");
        }
    }

    /********************************************************************************************************/
    /* OOP World:
    println!("Creating creatures...");
    world.spawn_random_creatures(10000);

    // // uncomment to print the DNA of the first creature and exit immediately (for testing)
    // println!("{}",world.creatures[0].dna.to_compact_string());
    // return;

    println!("Starting simulation...");
    for _ in 0..50_000 {
        world.tick();
        world.print_to_terminal(true,false, true);
        thread::sleep(Duration::from_millis(5));
        if world.creatures.len() < 1 {
            // println!("The world is empty. Spawning new creature...");
            // world.spawn_random_creatures(100);
            break;
        }
    }
    println!("Simulation ended.");
    println!("We had a total of {} newborns and {} deaths during {} cycles.", world.counter_newborns, world.counter_deaths, world.cycle);
    */
}
