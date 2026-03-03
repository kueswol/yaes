use yaes::creature::Creature;
use yaes::utils::CreatureEvent;
use std::thread;
use std::time::Duration;

pub const MAX_POPULATION: usize = 50000;

fn main() {
    
    // let test1: u64 = 0b0000_0000_0000_1111;
    // let test2: u64 = 0b0000_0000_1111_0000;
    // let test3: u64 = 0b0000_1111_0000_0000;
    // let test4: u64 = 0b1111_0000_0000_0000;

    // println!("Test1: {:064b} | {} bits set | {} trailing zeros", test1, test1.count_ones(), test1.trailing_zeros());
    // println!("Test2: {:064b} | {} bits set | {} trailing zeros", test2, test2.count_ones(), test2.trailing_zeros());
    // println!("Test3: {:064b} | {} bits set | {} trailing zeros", test3, test3.count_ones(), test3.trailing_zeros());
    // println!("Test4: {:064b} | {} bits set | {} trailing zeros", test4, test4.count_ones(), test4.trailing_zeros());
    // return;
    
    let mut creatures: Vec<Creature> = Vec::new();
    let mut rng = rand::thread_rng();

    println!("--- Initializing Creatures -------------------------------------------------");
    for _ in 0..10 {
        creatures.push(Creature::default());
    }

    // print_creature_states(&creatures, &0);
    
    print!("3... ");
    // thread::sleep(Duration::from_millis(5000));
    print!("2... ");
    // thread::sleep(Duration::from_millis(4000));
    print!("1... ");
    // thread::sleep(Duration::from_millis(3000));
    println!("GO!");
    // thread::sleep(Duration::from_millis(2000));

    let mut counter_newborns = 0;
    let mut counter_deaths = 0;
    println!("--- Starting Simulation -------------------------------------------------");
    for cycle in 0..501 {
        
        if creatures.len() < 10 {
            // for _ in 0..10 {
            //     creatures.push(Creature::default());
            // }
            creatures.push(Creature::default());
        }

        // if cycle < 100 {
        //     thread::sleep(Duration::from_millis(1000 - (cycle as u64 * 10)));
        // }
        thread::sleep(Duration::from_millis(100));
        let mut newborns: Vec<Creature> = Vec::new();
        
        for i in 0..creatures.len() {
            if creatures[i].alive {
                match creatures[i].think_and_act() {
                    CreatureEvent::Reproduce => {
                        let child = Creature::new_from_parent(&creatures[i], &mut rng);
                        newborns.push(child);
                        counter_newborns += 1;
                    }
                    CreatureEvent::Die => {
                        // println!("{} died", creatures[i].name);
                        counter_deaths += 1;
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
        if cycle % 1 == 0 { print_creature_states(&creatures, &cycle, &counter_newborns, &counter_deaths); }
    }
}

fn print_creature_states(creatures: &[Creature], cycle: &usize, counter_newborns: &usize, counter_deaths: &usize) {
    print!("\x1B[2J\x1B[H"); // clear + cursor home
    print!("Population: {:5} (", creatures.len());
    let mut bar: String = String::new();
    for _ in 0..(creatures.len() / MAX_POPULATION * 100) {
        bar+="-";
    };println!("{:<100}) with {} newborns and {} deaths", bar, counter_newborns, counter_deaths);
    println!("Cycle: {:5} / 500", cycle);
    println!("");

    for creature in creatures.iter().take(70) {
        println!("{}", creature.to_string());
    }
    // println!("Cycle: {:5}/5000 | Population: {:5}/{} ({:3}%) | Newborns: {:3} | Deaths: {:7}", cycle, creatures.len(), MAX_POPULATION, creatures.len() * 100 / MAX_POPULATION, counter_newborns, counter_deaths);
}
