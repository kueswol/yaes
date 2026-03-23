#[allow(dead_code)]
use crate::ecs::World;
use crate::utils::*;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

/******************************************************************************************************************************************/
/// the main simulation loop
pub fn run_simulation(
    world: Arc<Mutex<World>>,
    channel_web2sim_rx: std::sync::mpsc::Receiver<ChannelWeb2SimMessage>,
) {
    let target_tps: f64 = 25.0;
    let mut target_tick_duration: Duration = Duration::from_secs_f64(1.0 / target_tps);
    let mut tick_durations: Vec<Duration> = vec![Duration::new(0, 0); 32];
    let mut stats: WorldStats;
    let mut paused: bool = true;

    println!("[SIM  ]: Starting simulation loop");
    println!("[SIM  ]: - it will be paused until the webserver sends a resume command");

    loop {
        process_web2sim_messages(&mut paused, &mut target_tick_duration, &channel_web2sim_rx);

        while paused {
            thread::sleep(Duration::from_millis(100));
            process_web2sim_messages(&mut paused, &mut target_tick_duration, &channel_web2sim_rx);
        }

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
            let avg_tick_duration: Duration =
                tick_durations.iter().sum::<Duration>() / (tick_durations.len() as u32);
            println!(
                "[SIM  ]: Tick {:8}: {:5} creatures | tick: {:5}µs | avg: {:5}µs (≙ {:5.1}tps)",
                stats.tick,
                stats.population,
                tick_duration.as_micros(),
                avg_tick_duration.as_micros(),
                1.0 / avg_tick_duration.as_secs_f64()
            );
        }

        thread::sleep(target_tick_duration.saturating_sub(tick_duration));
    }
}

/******************************************************************************************************************************************/
/// processes incoming messages from the webserver to control the simulation
#[inline(always)]
fn process_web2sim_messages(
    paused: &mut bool,
    target_tick_duration: &mut Duration,
    channel_web2sim_rx: &std::sync::mpsc::Receiver<ChannelWeb2SimMessage>,
) {
    while let Ok(message) = channel_web2sim_rx.try_recv() {
        match message {
            ChannelWeb2SimMessage::PauseSim => {
                *paused = true;
                println!("[SIM  ]: Simulation paused");
            },
            ChannelWeb2SimMessage::ResumeSim => {
                *paused = false;
                println!("[SIM  ]: Simulation resumed");
            },
            ChannelWeb2SimMessage::SetTargetTPS(tps) => {
                *target_tick_duration = Duration::from_secs_f64(1.0 / tps);
                println!("[SIM  ]: Target TPS set to {}", tps);
            },
        }
    }
}
