#[allow(dead_code)]
use crate::ecs::World;
use crate::utils::WorldStats;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

pub fn run_simulation(world: Arc<Mutex<World>>) {
    let target_tps: f64 = 20.0;
    let target_tick_duration: Duration = Duration::from_secs_f64(1.0 / target_tps);
    println!("[SIM  ]: simulation loop ready - sleeping 5 sec");
    thread::sleep(std::time::Duration::from_secs(5));
    println!("[SIM  ]: Starting simulation loop");
    loop {
        let stats: WorldStats;
        let tick_duration_start = Instant::now();
        {
            let mut world = world.lock().unwrap();
            world.tick();
            stats = world.get_stats();
        }
        let tick_duration: Duration = tick_duration_start.elapsed();

        if stats.tick % 1000 == 0 {
            println!(
                "[SIM  ]: Tick {:5}: {:6} creatures, *{}, †{}, AvgE:{:0.2}, AvgAge:{:0.2}, Eat:{}✓|{}✕, Repro:{}✓|{}✕(Age)|{}✕(E)",
                stats.tick,
                stats.population,
                stats.births,
                stats.deaths,
                stats.avg_energy,
                stats.avg_age,
                stats.eat_success,
                stats.eat_failed,
                stats.reproduce_success,
                stats.reproduce_failed_age,
                stats.reproduce_failed_energy
            );
        }
        thread::sleep(target_tick_duration.saturating_sub(tick_duration));
    }
}
