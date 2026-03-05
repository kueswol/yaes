use crate::{constants as c, utils::Coordinate};
use crate::utils::CreatureEvent;
use crate::creature::Creature;

use rand::{rngs::StdRng, Rng, SeedableRng};


/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// The World
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
pub struct World {
    pub rng                      :  rand::rngs::StdRng,
    pub cycle                    :  u64,
    pub creatures                :  Vec<Creature>,
    pub counter_newborns         :  u64,
    pub counter_deaths           :  u64,
    pub dimension                :  (u32,u32),
    pub pending_creature_events  :  Vec<(Option<usize>, CreatureEvent)>,
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// The World's public functions
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
impl World {
    /******************************************************************************************************************************************/
    pub fn new(rng_seed:u64) -> Self {
        Self {
            rng                      :  StdRng::seed_from_u64(rng_seed),
            cycle                    :  0,
            creatures                :  Vec::with_capacity(c::MAX_POPULATION as usize * 2),
            counter_newborns         :  0,
            counter_deaths           :  0,
            dimension                :  (c::WORLD_WIDTH, c::WORLD_HEIGHT),
            pending_creature_events  :  Vec::new(),
        }
    }
    
    /******************************************************************************************************************************************/
    pub fn tick(&mut self) {
        self.cycle += 1;
        self.let_creatures_think();
        self.let_creatures_act();
        // self.remove_dead_creatures();
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
    // trigger each creature's brain and collect their intended actions
    fn let_creatures_think(&mut self) {
        
        for (id, creature) in &mut self.creatures.iter_mut().enumerate() {
            // let the creature think and collect its intended action
            self.pending_creature_events.push((Some(id), creature.think()));
        };

    }
    
    /******************************************************************************************************************************************/
    /// process the collected events (reproduction and death) after all creatures have thought
    fn let_creatures_act(&mut self) {
        
        let mut creatures_to_kill: Vec<usize> = Vec::new();

        for (id, event) in self.pending_creature_events.drain(..) {
            match event {
                
                CreatureEvent::Move { direction, speed } => {
                    if let Some(id) = id {
                        if self.creatures[id].energy < c::ENERGY_COST_MOVE { continue; }
                        
                        self.creatures[id].pos = World::get_new_coordinate(self.creatures[id].pos, direction, speed);
                        self.creatures[id].energy = (self.creatures[id].energy - c::ENERGY_COST_MOVE).clamp(0.0, 100.0);
                    }
                },

                CreatureEvent::Sleep => {
                    if let Some(id) = id {
                        self.creatures[id].energy = (self.creatures[id].energy - c::ENERGY_COST_SLEEP).clamp(0.0, 100.0);
                    }
                }

                CreatureEvent::Eat => {
                    if let Some(id) = id {
                        // eat if there is food at the current position (for simplicity: if pos.x and pos.y are both outside of 33 and 66)
                        if self.creatures[id].energy < 90.0 && (self.creatures[id].pos.x <= 33.0 || self.creatures[id].pos.x >= 66.0 || self.creatures[id].pos.y <= 33.0 || self.creatures[id].pos.y >= 66.0) {
                            self.creatures[id].energy = (self.creatures[id].energy + 20.0).clamp(0.0, 100.0);
                        }
                    }
                }

                CreatureEvent::Reproduce => {
                    if let Some(id) = id {
                        if self.creatures[id].energy < c::ENERGY_COST_REPRODUCE ||
                          !self.creatures[id].can_reproduce { continue; }
                        
                        self.creatures[id].can_reproduce = false;
                        self.creatures[id].energy = (self.creatures[id].energy - c::ENERGY_COST_REPRODUCE).clamp(0.0, 100.0);
                        
                        let parent = &self.creatures[id];
                        let child = Creature::new_from_parent(parent, &mut self.rng);
                        
                        self.creatures.push(child);
                        self.counter_newborns += 1;
                    }
                }
                
                CreatureEvent::Die => {
                    if let Some(id) = id {
                        creatures_to_kill.push(id);
                    }
                }
                
                _ => { continue; }
            }

        }
        
        // kill the creatures with pending death events
        self.kill_creatures_by_id(creatures_to_kill);

        // if we are overpopulated, kill some random creatures to maintain a manageable population size
        if self.creatures.len() > c::MAX_POPULATION {
            self.kill_creatures_by_random(self.creatures.len() - c::MAX_POPULATION);
        }

    }

    /******************************************************************************************************************************************/
    /// helper function to calculate the new coordinate based on the original coordinate, direction and speed (with world boundary checks)
    #[inline(always)]
    fn get_new_coordinate(original: Coordinate, direction: f32, speed: f32) -> Coordinate {
        let radians = direction.to_radians();
        
        let new_coordinate = Coordinate {
            x: (original.x + radians.cos() * speed).clamp(0.0, c::WORLD_WIDTH as f32),
            y: (original.y + radians.sin() * speed).clamp(0.0, c::WORLD_HEIGHT as f32),
        };
        
        // we might add collision checks here
        
        new_coordinate
    }

    /******************************************************************************************************************************************/
    /// kill the creatures with the given ids
    #[inline(always)]
    fn kill_creatures_by_id(&mut self, ids: Vec<usize>) {
        for id in ids {
            if id < self.creatures.len() {
                self.creatures.swap_remove(id);
                self.counter_deaths += 1;
            }
        }
    }

    /******************************************************************************************************************************************/
    /// kill a random selection of creatures to maintain population control
    #[inline(always)]
    fn kill_creatures_by_random(&mut self, count: usize) {
        for _ in 0..count {
            self.creatures.swap_remove(self.rng.gen_range(0..self.creatures.len()));
            self.counter_deaths += 1;
        }
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
