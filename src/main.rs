use yaes::creature::Creature;
use yaes::utils::CreatureEvent;
use std::thread;
use std::time::Duration;

pub const MAX_POPULATION: usize = 100;

fn main() {
    let mut creatures: Vec<Creature> = Vec::new();

    println!("--- Starting Simulation -------------------------------------------------");
    for _ in 0..2000 {
        
        if creatures.len() < 50 {
            creatures.push(Creature::default());
        }

        thread::sleep(Duration::from_millis(10));
        let mut newborns: Vec<Creature> = Vec::new();
        
        for i in 0..creatures.len() {
            if creatures[i].alive {
                match creatures[i].think_and_act() {
                    CreatureEvent::Reproduce => {
                        let child = Creature::new_from_parent(&creatures[i]);
                        newborns.push(child);
                    }
                    CreatureEvent::Die => {
                        print!("--- {} is dead -------------------------------------------------------------------", creatures[i].name);
                    }
                    _ => {}
                }
            }
        }
        
        // remove dead ones, add newborns, limit population
        creatures.retain(|c| c.alive);
        creatures.extend(newborns);
        if creatures.len() > MAX_POPULATION {
            creatures.truncate(MAX_POPULATION);
        }

        // give overview
        print_creature_states(&creatures);
    }
}

fn print_creature_states(creatures: &[Creature]) {
    print!("\x1B[2J\x1B[H"); // clear + cursor home
    print!("Population: {} (", creatures.len());
    let mut bar: String = String::new();
    for _ in 0..creatures.len() {
        bar+="-";
    };println!("{:<100})", bar);

    for creature in creatures.iter().take(70) {
        println!("{}", creature.to_string());
    }
}
