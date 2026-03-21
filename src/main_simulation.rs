#[allow(dead_code)]
use crate::ecs::World;
use crate::utils::WorldStats;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

pub fn run_simulation(world: Arc<Mutex<World>>) {
    let target_tps: f64 = 30.0;
    let target_tick_duration: Duration = Duration::from_secs_f64(1.0 / target_tps);
    let mut tick_durations: Vec<Duration> = vec![Duration::new(0, 0);32];
    let mut stats: WorldStats;
    
    println!("[SIM  ]: simulation loop ready - sleeping 5 sec");
    thread::sleep(std::time::Duration::from_secs(5));
    println!("[SIM  ]: Starting simulation loop");
    
    loop {
        let tick_duration_start = Instant::now();
        {
            let mut world = world.lock().unwrap();
            world.tick();
            stats = world.get_stats();
        }
        let tick_duration: Duration = tick_duration_start.elapsed();

        let index: usize = (stats.tick & 31_u64) as usize;
        tick_durations[index] = tick_duration;
        if index == 0 {
            let avg_tick_duration: Duration = tick_durations.iter().sum::<Duration>() / (tick_durations.len() as u32);
            println!(
                "[SIM  ]: Tick {:8}: {:5} creatures | tick: {:5}µs | avg: {:5}µs (≙ {:5.1}tps)",
                stats.tick,
                stats.population,
                tick_duration.as_micros(),
                avg_tick_duration.as_micros(),
                1.0 / avg_tick_duration.as_secs_f64()
            );
        }
        // if stats.tick % 1000 == 0 {
        //     println!(
        //         "[SIM  ]: Tick {:5}: {:6} creatures, *{}, †{}, AvgE:{:0.2}, AvgAge:{:0.2}, Eat:{}✓|{}✕, Repro:{}✓|{}✕(Age)|{}✕(E)",
        //         stats.tick,
        //         stats.population,
        //         stats.births,
        //         stats.deaths,
        //         stats.avg_energy,
        //         stats.avg_age,
        //         stats.eat_success,
        //         stats.eat_failed,
        //         stats.reproduce_success,
        //         stats.reproduce_failed_age,
        //         stats.reproduce_failed_energy
        //     );
        // }
        thread::sleep(target_tick_duration.saturating_sub(tick_duration));
    }
}
