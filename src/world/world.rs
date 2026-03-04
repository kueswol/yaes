use crate::constants as c;
use crate::utils::CreatureEvent;
use crate::creature::Creature;

use rand::{rngs::StdRng, Rng, SeedableRng};


/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// The World
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
pub struct World {
    pub rng               :  rand::rngs::StdRng,
    pub cycle             :  u64,
    pub creatures         :  Vec<Creature>,
    pub counter_newborns  :  u64,
    pub counter_deaths    :  u64,
    pub dimension         :  (u32, u32),
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// The World's public functions
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
impl World {
    /******************************************************************************************************************************************/
    pub fn new(rng_seed:u64) -> Self {
        Self {
            rng               :  StdRng::seed_from_u64(rng_seed),
            cycle             :  0,
            creatures         :  Vec::with_capacity(c::MAX_POPULATION as usize * 2),
            counter_newborns  :  0,
            counter_deaths    :  0,
            dimension         :  (c::WORLD_WIDTH, c::WORLD_HEIGHT),
        }
    }
    
    /******************************************************************************************************************************************/
    pub fn tick(&mut self) {
        self.cycle += 1;
        self.let_creatures_think_and_act();
        self.remove_dead_creatures();
    }
    
    /******************************************************************************************************************************************/
    pub fn print_to_terminal(&mut self,world_stats: bool,creature_details: bool) {
        if world_stats {
            println!("Cycle: {:9}, Pop: {:9} ({:9} *, {:9} †)", self.cycle, self.creatures.len(), self.counter_newborns, self.counter_deaths);
        }
        if creature_details {
            for creature in &self.creatures {
                println!("{}", creature.to_string());
            }
        }
    }

    /******************************************************************************************************************************************/
    pub fn spawn_random_creature(&mut self, mut count: usize) {
        if count + self.creatures.len() > c::MAX_POPULATION {
            count = c::MAX_POPULATION - self.creatures.len();
        }
        if count < 1 {
            return;
        }
        for _ in 0..count {
            let creature = Creature::new(&mut self.rng);
            self.creatures.push(creature);
        }
    }
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// The World's private functions
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
impl World {
    /******************************************************************************************************************************************/
    fn let_creatures_think_and_act(&mut self) {
        let mut newborns: Vec<Creature> = Vec::new();
        self.creatures
          .iter_mut()
          .filter(|c|c.alive)
          .for_each(|creature| {
            
            match creature.think_and_act() {
            
                CreatureEvent::Reproduce => {
                    let child = Creature::new_from_parent(creature, &mut self.rng);
                    newborns.push(child);
                    self.counter_newborns += 1;
                }
            
                CreatureEvent::Die => {
                    self.counter_deaths += 1;
                }
            
                _ => {}
            }
        });
        
        if self.creatures.len() + newborns.len() > c::MAX_POPULATION {
            self.creatures.sort_by_key(|c| c.age);
            let overflow = self.creatures.len() + newborns.len() - c::MAX_POPULATION;
            self.creatures.drain(0..overflow);
        }
        self.creatures.extend(newborns);

    }
    /******************************************************************************************************************************************/
    fn remove_dead_creatures(&mut self) {
        self.creatures.retain(|c| c.alive);
    }
    
    
}
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// traits
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
impl Default for World {
    fn default() -> Self {
        Self::new(rand::thread_rng().r#gen())
    }
}
