use crate::{
    constants as c,
    ecs::components::*,
    utils::*,
    web::views::*,
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use rayon::prelude::*;

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// The World
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

pub struct World {
    // the world wide root of all "randomness"
    rng: rand::rngs::StdRng,
    spatial_map: SpatialHashmap,
    pub foodmap: Vec<u8>,

    // statistics
    pub tick_counter: u64,
    pub seed: u64,
    pub deaths: u64,
    pub births: u64,
    pub eat_success: u64,
    pub eat_failed: u64,
    pub reproduce_success: u64,
    pub reproduce_failed_age: u64,
    pub reproduce_failed_energy: u64,
    pub reproduce_failed_cooldown: u64,
    pub avg_energy: f32,
    pub avg_age: f32,
    
    // entity management
    next_creature_id: usize,
    /// bitmap:
    ///    0b0001 = exists/alive
    ///    0b0010 = can_reproduce
    creatures: Vec<u8>,
    
    // light components
    positions: Vec<Coordinate>,
    orientations: Vec<f32>,
    energies: Vec<f32>,
    ages: Vec<u64>,
    sizes: Vec<f32>,
    brain_inputs: Vec<u64>,
    brain_outputs: Vec<u64>,
    reproduce_cooldown: Vec<u64>,

    // heavy components
    brains: Vec<Brain>,
    dnas: Vec<Dna>,

    // pending actions/events
    pending_move: Vec<(usize, CreatureEvent)>,
    pending_eat: Vec<(usize, CreatureEvent)>,
    pending_sleep: Vec<(usize, CreatureEvent)>,
    pending_reproduce: Vec<(usize, CreatureEvent)>,
    pending_energy_costs: Vec<(usize, f32)>,
    pending_deaths: Vec<usize>,
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// The World's public functions
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
impl World {
    /******************************************************************************************************************************************/
    pub fn new(rng_seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(rng_seed);

        let mut foodmap: Vec<u8> = vec![0;(c::WORLD_WIDTH * c::WORLD_HEIGHT) as usize];
        for i in 0..foodmap.len() {
            // foodmap[i] = if rng.gen_bool(0.30) { 255_u8 } else { 0_u8 };
            foodmap[i] = rng.gen_range(0..=255);
        }
                
        Self {
            rng,
            foodmap,
            tick_counter: 0,
            seed: rng_seed,
            deaths: 0,
            births: 0,
            eat_success: 0,
            eat_failed: 0,
            reproduce_success: 0,
            reproduce_failed_age: 0,
            reproduce_failed_energy: 0,
            reproduce_failed_cooldown: 0,
            avg_energy: 0.0,
            avg_age: 0.0,
            next_creature_id: 0,
            spatial_map: SpatialHashmap::new(),
            creatures: Vec::with_capacity(c::MAX_POPULATION),
            positions: Vec::with_capacity(c::MAX_POPULATION),
            orientations: Vec::with_capacity(c::MAX_POPULATION),
            energies: Vec::with_capacity(c::MAX_POPULATION),
            ages: Vec::with_capacity(c::MAX_POPULATION),
            sizes: Vec::with_capacity(c::MAX_POPULATION),
            brain_inputs: Vec::with_capacity(c::MAX_POPULATION),
            brain_outputs: Vec::with_capacity(c::MAX_POPULATION),
            brains: Vec::with_capacity(c::MAX_POPULATION),
            dnas: Vec::with_capacity(c::MAX_POPULATION),
            reproduce_cooldown: Vec::with_capacity(c::MAX_POPULATION),
            pending_move: Vec::with_capacity(c::MAX_POPULATION),
            pending_eat: Vec::with_capacity(c::MAX_POPULATION),
            pending_sleep: Vec::with_capacity(c::MAX_POPULATION),
            pending_reproduce: Vec::with_capacity(c::MAX_POPULATION),
            pending_energy_costs: Vec::with_capacity(c::MAX_POPULATION),
            pending_deaths: Vec::with_capacity(c::MAX_POPULATION),
        }
    }

    /******************************************************************************************************************************************/
    /// let the world tick
    pub fn tick(&mut self) {
        
        self.update_brain_inputs();
        self.update_brain_outputs();
        self.handle_turning();
        self.schedule_actions();
        self.handle_action_eat();
        self.handle_action_move();
        self.handle_action_sleep();
        self.handle_action_reproduce();
        self.handle_energy_costs();
        self.handle_age_events();
        self.handle_deaths();
        self.update_spatial_map();
        // self.handle_separation();

        if self.tick_counter & c::FOOD_REGROWTH_TICKS == 0 {
            self.grow_food();
        }

        self.update_stats();
        self.tick_counter += 1;
    }

    /******************************************************************************************************************************************/
    /// spawn a new creature
    pub fn spawn_creature(&mut self, dna: Option<Dna>, position: Option<Coordinate>) -> bool {
        // abort here, if we don't have capacity for more creatures
        if self.next_creature_id >= c::MAX_POPULATION {
            return false;
        }
        self.next_creature_id += 1;

        // prepare some values:
        let new_creature_position: Coordinate = position.unwrap_or_else(|| Coordinate {
                x: self.rng.gen_range((c::WORLD_WIDTH  as f32 * 0.05)..(c::WORLD_WIDTH  as f32 * 0.95)),
                y: self.rng.gen_range((c::WORLD_HEIGHT as f32 * 0.05)..(c::WORLD_HEIGHT as f32 * 0.95))
            });
        let new_orientation: f32 = self.rng.gen_range(0.0..std::f32::consts::TAU);
        let new_creature_energy: f32 = 100.0;
        let new_creature_birthtick: u64 = self.tick_counter;
        let new_creature_brain_input: u64 = 0;
        let new_creature_brain_output: u64 = 0;

        let new_creature_dna: Dna = dna.unwrap_or_else(|| Dna::random(128, &mut self.rng)); // we had with 384 bytes
        let new_creature_genome: Genome = Genome::from_dna(&new_creature_dna);
        let new_creature_brain: Brain = Brain::recompile(&new_creature_genome);
        let new_size: f32 = 0.2;
        let new_creature_reproduce_cooldown: u64 = self.tick_counter + c::REPRODUCE_AGE_MIN - 10 + (self.rng.gen_range(0..20));

        // we trust, that all vectors are aligned, so new creatures and its components will just be pushed at the end of the vectors
        self.creatures.push(0b0000_0001);
        self.positions.push(new_creature_position);
        self.orientations.push(new_orientation);
        self.energies.push(new_creature_energy);
        self.ages.push(new_creature_birthtick);
        self.sizes.push(new_size);
        self.brain_inputs.push(new_creature_brain_input);
        self.brain_outputs.push(new_creature_brain_output);
        self.brains.push(new_creature_brain);
        self.dnas.push(new_creature_dna);
        self.reproduce_cooldown.push(new_creature_reproduce_cooldown);

        true // return successfully spawned
    }

    /******************************************************************************************************************************************/
    /// Delete a creature
    pub fn delete_creature(&mut self, id: usize) {
        self.creatures.swap_remove(id);
        self.positions.swap_remove(id);
        self.orientations.swap_remove(id);
        self.energies.swap_remove(id);
        self.ages.swap_remove(id);
        self.sizes.swap_remove(id);
        self.brain_inputs.swap_remove(id);
        self.brain_outputs.swap_remove(id);
        self.brains.swap_remove(id);
        self.dnas.swap_remove(id);
        self.reproduce_cooldown.swap_remove(id);
        
        self.next_creature_id -= 1;
        self.deaths += 1;
    }

    /******************************************************************************************************************************************/
    pub fn spawn_random_creatures(&mut self, mut count: usize) {
        if count + self.creatures.len() > c::MAX_POPULATION {
            count = c::MAX_POPULATION - self.creatures.len();
        }
        if count < 1 {
            return;
        }
        for _ in 0..count {
            self.spawn_creature(None, None);
        }
    }

    /******************************************************************************************************************************************/
    pub fn get_creature_count(&self) -> usize {
        self.creatures.len()
    }

    /******************************************************************************************************************************************/
    /// get statics about the world
    pub fn get_stats(&self) -> WorldStats {
        WorldStats {
            tick: self.tick_counter,
            population: self.creatures.len() as u64,
            avg_energy: self.avg_energy,
            avg_age: self.avg_age,
            births: self.births,
            deaths: self.deaths,
            eat_success: self.eat_success,
            eat_failed: self.eat_failed,
            reproduce_success: self.reproduce_success,
            reproduce_failed_age: self.reproduce_failed_age,
            reproduce_failed_energy: self.reproduce_failed_energy,
            reproduce_failed_cooldown: self.reproduce_failed_cooldown,
        }
    }
    
    /******************************************************************************************************************************************/
    /// export a view of the world for the webserver
    pub fn to_view(&self) -> WorldView {
        WorldView {
            tick: self.tick_counter,
            population: self.creatures.len(),
        }
    }
    
    /******************************************************************************************************************************************/
    /// export a view of the creatures for the webserver
    pub fn creatures_view(&self) -> Vec<CreatureView> {

        let mut result = Vec::with_capacity(self.creatures.len());
        for i in 0..self.creatures.len() {
            if self.creatures[i] & c::CREATURE_BITFLAG_IS_ALIVE == 0 {
                continue;
            }

            let pos = self.positions[i];

            result.push(CreatureView {
                x: pos.x as f32,
                y: pos.y as f32,
            });
        }

        result
    }
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// The ECS' systems
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
impl World {

    /******************************************************************************************************************************************/
    /// update each creature's sensoric brain inputs
    ///   [ 1] Energy low,              [ 2] Energy mid,              [ 3] Energy high,
    ///   [ 4] PosX left,               [ 5] PosX center,             [ 6] PosX right,
    ///   [ 7] PosY top,                [ 8] PosY center,             [ 9] PosY bottom,
    ///   [10] LastAction Idle,         [11] LastAction Sleep,        [12] LastAction Move,
    ///   [13] LastAction Eat,          [14] LastAction Reproduce,    [15] can_reproduce,
    ///   [16] age low,                 [17] age mid,                 [18] age high,
    ///   [19] can_eat,                 [20] unused,                  [21] unused,
    ///   [22] unused,                  [23] unused,                  [24] unused
    fn update_brain_inputs(&mut self) {
        // for each creature, gather sensory data and update brain_inputs
        let energies = &self.energies;
        let positions = &self.positions;
        let creatures = &self.creatures;
        let brain_outputs = &self.brain_outputs;
        let ages = &self.ages;
        let tick_counter = &self.tick_counter;
        let foodmap = &self.foodmap;

        self.brain_inputs
            .par_iter_mut()
            .with_min_len(100)
            .enumerate()
            .for_each(move |(entity_id, input)| {
                // energie low|mid|high
                *input |= Self::encode_3bucket(
                    energies[entity_id],
                    [c::BRAIN_INPUTS_BUCKET_NRGY_LOW_MID, c::BRAIN_INPUTS_BUCKET_NRGY_MID_HIGH],
                ) << 0;

                // pos.x left|center|right
                let pos = &positions[entity_id];
                *input |= Self::encode_3bucket(
                    pos.x,
                    [c::BRAIN_INPUTS_BUCKET_POSX_L_C, c::BRAIN_INPUTS_BUCKET_POSX_C_R],
                ) << 3;

                // pos.y top|center|bottom
                *input |= Self::encode_3bucket(
                    pos.y,
                    [c::BRAIN_INPUTS_BUCKET_POSY_T_C, c::BRAIN_INPUTS_BUCKET_POSY_C_B],
                ) << 6;

                // last action
                let last_action = Self::decode_output(&brain_outputs[entity_id]).0;
                *input |= ((last_action == CreatureAction::Idle) as u64) << 9;
                *input |= ((last_action == CreatureAction::Sleep) as u64) << 10;
                *input |= ((last_action == CreatureAction::Move) as u64) << 11;
                *input |= ((last_action == CreatureAction::Eat) as u64) << 12;
                *input |= ((last_action == CreatureAction::Reproduce) as u64) << 13;

                // can reproduce
                *input |= (((creatures[entity_id] & c::CREATURE_BITFLAG_CAN_REPRODUCE) != 0) as u64) << 14;

                // age low|mid|high
                *input |= Self::encode_3bucket(
                    ((*tick_counter) - ages[entity_id]) as f32,
                    [
                        c::BRAIN_INPUTS_BUCKET_AGE_LOW_MID as f32,
                        c::BRAIN_INPUTS_BUCKET_AGE_MID_HIGH as f32,
                    ],
                ) << 15;

                // can eat (is there food at the current position?)
                *input |= ((foodmap[(pos.y as usize).clamp(0,99) * (c::WORLD_WIDTH as usize) + (pos.x as usize).clamp(0,99)] > 0) as u64) << 18;
            });

        // *inputs |= something else << 19;
        // *inputs |= something else << 20;
        // *inputs |= something else << 21;
        // *inputs |= something else << 22;
        // *inputs |= something else << 23;
    }

    /******************************************************************************************************************************************/
    /// THINK!
    /// fetch each creature's brain outputs
    fn update_brain_outputs(&mut self) {
        let brain_inputs = &self.brain_inputs;
        let brains = &self.brains;

        self.brain_outputs
            .par_iter_mut()
            .with_min_len(100)
            .enumerate()
            .for_each(move |(entity_id, brain_output)| {
                // let brain = &self.brains[entity_id];
                let input = brain_inputs[entity_id];
                *brain_output = brains[entity_id].tick(input);
            });
    }

    /******************************************************************************************************************************************/
    /// schedule the creatures' actions based on their brain outputs
    fn schedule_actions(&mut self) {
        let brain_outputs = &self.brain_outputs;

        // we're collecting actions in separate buffers for each thread to avoid contention, and then we'll merge them into the main queues
        let (buffer_move, buffer_eat, buffer_sleep, buffer_reproduce, buffer_energy_costs): (
            Vec<(usize, CreatureEvent)>,
            Vec<(usize, CreatureEvent)>,
            Vec<(usize, CreatureEvent)>,
            Vec<(usize, CreatureEvent)>,
            Vec<(usize, f32)>,
        ) = brain_outputs
            .par_iter()
            .with_min_len(100)
            .enumerate()
            .fold(
                || (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()), // local buffer per thread
                |(mut moves, mut eats, mut sleeps, mut reproduces, mut energy_costs),
                 (entity_id, &brain_output)| {
                    let (action, _value1, _value2, _value3, fired_neurons_count) =
                        Self::decode_output(&brain_output);
                    let mut energy_cost = (fired_neurons_count as f32) * c::ENERGY_COST_FIRED_NEURON;
                    match action {
                        CreatureAction::Move => {
                            let sprint: bool = (_value1 & c::BRAIN_OUTPUT_VALUE1_SPRINT) != 0;
                            moves.push((entity_id, CreatureEvent::Move{ sprint }));
                        }
                        CreatureAction::Eat => {
                            eats.push((entity_id, CreatureEvent::Eat));
                        }
                        CreatureAction::Sleep => {
                            sleeps.push((entity_id, CreatureEvent::Sleep));
                        }
                        CreatureAction::Reproduce => {
                            // if self.energies[entity_id] >= c::ENERGY_COST_REPRODUCE {
                                reproduces.push((entity_id, CreatureEvent::Reproduce));
                            // } else {
                                // moves.push((entity_id, CreatureEvent::Move{ sprint: false })); // if they want to reproduce but don't have enough energy, we make them move instead, to hopefully find food
                                // energy_cost += c::ENERGY_COST_REPRODUCE; // we still apply the energy cost, even if the creature can't reproduce, to encourage them to only reproduce when they have enough energy
                            // }
                        }
                        CreatureAction::Idle => {
                            energy_cost += c::ENERGY_COST_IDLE;
                        }
                    }
                    energy_costs.push((entity_id, energy_cost));
                    (moves, eats, sleeps, reproduces, energy_costs)
                },
            )
            .reduce(
                || (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()), // Neutrales Element
                |(mut a_moves, mut a_eats, mut a_sleeps, mut a_reproduces, mut a_energy_costs),
                 (b_moves, b_eats, b_sleeps, b_reproduces, b_energy_costs)| {
                    a_moves.extend(b_moves);
                    a_eats.extend(b_eats);
                    a_sleeps.extend(b_sleeps);
                    a_reproduces.extend(b_reproduces);
                    a_energy_costs.extend(b_energy_costs);
                    (a_moves, a_eats, a_sleeps, a_reproduces, a_energy_costs)
                },
            );

        // move the buffers to the global queues
        self.pending_move.extend(buffer_move);
        self.pending_eat.extend(buffer_eat);
        self.pending_sleep.extend(buffer_sleep);
        self.pending_reproduce.extend(buffer_reproduce);
        self.pending_energy_costs.extend(buffer_energy_costs);
    }

    /******************************************************************************************************************************************/
    /// turning is always possible
    fn handle_turning(&mut self) {
        let orientations = &mut self.orientations;
        let brain_outputs = &self.brain_outputs;

        orientations
            .par_iter_mut()
            .with_min_len(100)
            .enumerate()
            .for_each(move |(entity_id, orientation)| {
                let brain_output = brain_outputs[entity_id];
                let value1 = Self::decode_output(&brain_output).1;
                let turn_left : bool = (value1 & c::BRAIN_OUTPUT_VALUE1_TURN_LEFT ) != 0;
                let turn_right: bool = (value1 & c::BRAIN_OUTPUT_VALUE1_TURN_RIGHT) != 0;                
                let mut delta: f32 = 0.0;
                if turn_left  { delta += -5.0 };
                if turn_right { delta +=  5.0 };
                *orientation += delta;
                // clamping to rad
                if *orientation < std::f32::consts::TAU {
                    *orientation += std::f32::consts::TAU;
                }
                else if *orientation >= std::f32::consts::TAU {
                    *orientation -= std::f32::consts::TAU;
                }
            });
    }

    /******************************************************************************************************************************************/
    /// schedule the creatures' actions based on their brain outputs
    fn handle_action_eat(&mut self) {
        let pending_eat =
            std::mem::replace(&mut self.pending_eat, Vec::with_capacity(c::MAX_POPULATION));
        for (entity_id, _action) in pending_eat {
            let pos = &self.positions[entity_id];
            let (has_food, index) = self.has_food(pos);
            if has_food {
                self.foodmap[index] = self.foodmap[index].saturating_sub(c::ENERGY_COST_EAT as u8); // we subtract (add the negative) energy cost
                self.pending_energy_costs.push((entity_id, c::ENERGY_COST_EAT)); // negative cost = energy gain
                self.eat_success += 1;
                return;
            }
            self.eat_failed += 1;
        }
    }

    /******************************************************************************************************************************************/
    /// schedule the creatures' actions based on their brain outputs
    fn handle_action_move(&mut self) {
        for (entity_id, action) in &self.pending_move {
            match action {
                CreatureEvent::Move { sprint } => {
                    let speed = if *sprint { c::CREATURE_SPEED_SPRINT } else { c::CREATURE_SPEED };
                    let dx = self.orientations[*entity_id].cos() * speed;
                    let dy = self.orientations[*entity_id].sin() * speed;

                    self.positions[*entity_id].x =
                        (self.positions[*entity_id].x + dx).clamp(0.0, c::WORLD_WIDTH as f32);
                    self.positions[*entity_id].y =
                        (self.positions[*entity_id].y + dy).clamp(0.0, c::WORLD_HEIGHT as f32);
                    
                    let mut factor: f32 = 1.0;
                    if *sprint { factor = 1.5; }

                    self.pending_energy_costs
                        .push((*entity_id, c::ENERGY_COST_MOVE * factor));
                }
                _ => {}
            }
        }
        self.pending_move.clear();
    }

    /******************************************************************************************************************************************/
    /// schedule the creatures' actions based on their brain outputs
    fn handle_action_sleep(&mut self) {
        // self.pending_sleep.par_iter().for_each(|(entity_id, action)| {
        //    // TODO: implement sleeping
        // } );
        self.pending_sleep.clear();
    }

    /******************************************************************************************************************************************/
    /// schedule the creatures' actions based on their brain outputs
    fn handle_action_reproduce(&mut self) {
        let mut reproductions = std::mem::replace(
            &mut self.pending_reproduce,
            Vec::with_capacity(c::MAX_POPULATION),
        );

        if self.creatures.len() + reproductions.len() > c::MAX_POPULATION {
            // if we have more reproductions than capacity, we randomly select which ones will reproduce
            let overpopulation = (self.creatures.len() + reproductions.len()) - c::MAX_POPULATION;
            for _ in 0..overpopulation {
                reproductions.swap_remove(self.rng.gen_range(0..reproductions.len()));
            }
        }

        for (entity_id, _action) in reproductions {
            let age = (self.tick_counter) - self.ages[entity_id];
            if age < c::REPRODUCE_AGE_MIN || age > c::REPRODUCE_AGE_MAX {
                self.reproduce_failed_age += 1;
                continue;
            }
            if self.energies[entity_id] < c::ENERGY_COST_REPRODUCE {
                self.reproduce_failed_energy += 1;
                self.pending_energy_costs.push((entity_id, c::ENERGY_COST_REPRODUCE / 2.0));
                continue;
            }
            if self.reproduce_cooldown[entity_id] > self.tick_counter {
                self.reproduce_failed_cooldown += 1;
                continue;
            }
            
            let mut new_dna = self.dnas[entity_id].clone();
            self.reproduce_success += 1;
            self.creatures[entity_id] &= !c::CREATURE_BITFLAG_CAN_REPRODUCE; // reset reproduce ability until next age check
            self.reproduce_cooldown[entity_id] = self.tick_counter + (c::REPRODUCE_AGE_MIN / 2) - 10 + (self.rng.gen_range(0..20)); // set reproduce cooldown
            
            new_dna.mutate(&mut self.rng);
            let new_position = Coordinate {
                x: (self.positions[entity_id].x + self.rng.gen_range(-0.5..0.5)).clamp(0.0, c::WORLD_WIDTH as f32),
                y: (self.positions[entity_id].y + self.rng.gen_range(-0.5..0.5)).clamp(0.0, c::WORLD_HEIGHT as f32),
            };

            self.spawn_creature(Some(new_dna), Some(new_position));

            self.pending_energy_costs
                .push((entity_id, c::ENERGY_COST_REPRODUCE));
            self.births += 1;
        }
    }

    /******************************************************************************************************************************************/
    /// schedule the creatures' actions based on their brain outputs
    fn handle_energy_costs(&mut self) {
        let mut pending_energy_costs = std::mem::replace(
            &mut self.pending_energy_costs,
            Vec::with_capacity(c::MAX_POPULATION * 2),
        );
        pending_energy_costs.sort_unstable_by_key(|&(id, _)| id);

        let mut iter = pending_energy_costs.into_iter().peekable();
        while let Some((id, mut total)) = iter.next() {
            while iter.peek().map_or(false, |&(next_id, _)| next_id == id) {
                total += iter.next().unwrap().1;
            }

            let energy = &mut self.energies[id];
            *energy = (*energy - total).clamp(-1.0, 100.0);
            if *energy <= 0.0 {
                self.pending_deaths.push(id);
            }
        }
    }

    /******************************************************************************************************************************************/
    /// handle the deaths of creatures
    fn handle_age_events(&mut self) {
        let creatures = &mut self.creatures;
        let ages = &self.ages;
        let tick_counter = self.tick_counter;
        
        creatures
            .par_iter_mut()
            .with_min_len(100)
            .enumerate()
            .filter_map(|(entity_id, creature)| {
                let age = tick_counter - ages[entity_id] as u64;
                if age > c::REPRODUCE_AGE_MIN as u64 
                && age < c::REPRODUCE_AGE_MAX as u64
                && self.reproduce_cooldown[entity_id] == 0 {
                    *creature |= c::CREATURE_BITFLAG_CAN_REPRODUCE;
                } else {
                    *creature &= !c::CREATURE_BITFLAG_CAN_REPRODUCE;
                }
                if age > c::CREATURE_MAX_AGE as u64 {
                    return Some(entity_id);
                }
                None
            })
            .collect::<Vec<usize>>()
            .into_iter()
            .for_each(|entity_id| {
                self.pending_deaths.push(entity_id);
            });
    }

    /******************************************************************************************************************************************/
    /// handle the deaths of creatures
    fn handle_deaths(&mut self) {
        let mut deaths = std::mem::replace(
            &mut self.pending_deaths,
            Vec::with_capacity(c::MAX_POPULATION),
        );
        deaths.sort_unstable_by(|a, b| b.cmp(a));
        deaths.dedup();
        for entity_id in deaths {
            self.delete_creature(entity_id);
        }
    }
    
    /******************************************************************************************************************************************/
    /// schedule the creatures' actions based on their brain outputs
    fn update_spatial_map(&mut self) {
        let mut spatial_map = SpatialHashmap::new();
        let positions = &self.positions;
        
        positions.iter().enumerate().for_each(|(entity_id, &position)| {
                spatial_map.insert(entity_id, position);
        });
        self.spatial_map = spatial_map;
    }
    
    /******************************************************************************************************************************************/
    /// schedule the creatures' actions based on their brain outputs
    #[allow(dead_code)]
    fn handle_separation(&mut self) {
        // collect adjustments locally as we shouldn't apply them immediately in the parallel loop
        let mut position_adjustments: Vec<(usize, f32, f32)> = vec![(0, 0.0, 0.0); self.creatures.len()];
        
        position_adjustments.par_iter_mut().with_min_len(100).enumerate().for_each(|(id, adjustment)| {
            let pos = self.positions[id];
            let size = self.sizes[id];
            let neighbors = self.spatial_map.get_creatures_in_cell(pos);
            
            for &other_id in &neighbors {
                if id >= other_id { continue; } // prevent double checking pairs and self-checking
                
                let other_pos = self.positions[other_id];
                let other_size = self.sizes[other_id];
                let dx = pos.x - other_pos.x;
                let dy = pos.y - other_pos.y;
                let distance_sqr = dx * dx + dy * dy;
                let min_distance = (size + other_size) as f32;
                let min_distance_sqr = min_distance * min_distance;
                
                if distance_sqr >= min_distance_sqr || distance_sqr == 0.0 { continue; }
                
                let distance = distance_sqr.sqrt();
                let overlap = min_distance - distance;
                let nx = dx / distance;
                let ny = dy / distance;
                let half_overlap = overlap / 2.0;
                
                adjustment.0 = id;
                adjustment.1 += nx * half_overlap;
                adjustment.2 += ny * half_overlap;
                
            }
        });
        
        // apply the collected adjustments
        for (id, dx, dy) in position_adjustments {
            if id != 0 || dx != 0.0 || dy != 0.0 {
                self.positions[id].x = (self.positions[id].x + dx).clamp(0.0, c::WORLD_WIDTH as f32);
                self.positions[id].y = (self.positions[id].y + dy).clamp(0.0, c::WORLD_HEIGHT as f32);
            }
        } 
    }

    /******************************************************************************************************************************************/
    /// let the food spread and regrow
    fn grow_food(&mut self) {
        let foodmap = &mut self.foodmap;
        let x = self.rng.gen_range(1..(c::WORLD_WIDTH - 1));
        let y = self.rng.gen_range(1..(c::WORLD_HEIGHT - 1));
        let index: usize = (y * c::WORLD_WIDTH + x) as usize;
        let left : usize = index - 1;
        let right: usize = index + 1;
        let up   : usize = index - (c::WORLD_WIDTH as usize);
        let down : usize = index + (c::WORLD_WIDTH as usize);

        // spread to neighbors if the cell is full, otherwise regrow in the cell
        if foodmap[index] == 255 {
            foodmap[left]  = foodmap[left] .saturating_add(c::FOOD_REGROWTH_AMOUNT);
            foodmap[right] = foodmap[right].saturating_add(c::FOOD_REGROWTH_AMOUNT);
            foodmap[up]    = foodmap[up]   .saturating_add(c::FOOD_REGROWTH_AMOUNT);   
            foodmap[down]  = foodmap[down] .saturating_add(c::FOOD_REGROWTH_AMOUNT);
        }
        else if foodmap[index] > 0 {
            foodmap[index] = foodmap[index].saturating_add(c::FOOD_REGROWTH_AMOUNT);
        }
    }
    /******************************************************************************************************************************************/
    /// update internal world statistics
    fn update_stats(&mut self) {
        let total_energy: f32 = self.energies.iter().sum();
        self.avg_energy = total_energy / self.creatures.len() as f32;
        let total_age: u64 = self.ages.iter().sum();
        self.avg_age = self.tick_counter as f32 - (total_age as f32 / self.creatures.len() as f32);
    }
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// Helper functions
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
impl World {
    /******************************************************************************************************************************************/
    /// encodes a value into 3 buckets and returns a 3-bit representation as u64
    #[inline(always)]
    fn encode_3bucket(val: f32, buckets: [f32; 2]) -> u64 {
        let low = (val <= buckets[0]) as u64;
        let mid = ((val > buckets[0]) && (val <= buckets[1])) as u64;
        let high = (val > buckets[1]) as u64;

        (low << 2) | (mid << 1) | high
    }

    /******************************************************************************************************************************************/
    /// decodes an output value into action and three parameters
    #[inline(always)]
    fn decode_output(output: &u64) -> (CreatureAction, u8, u8, u8, u16) {
        // extract the value parameters
        let action_bits = (output & c::OUTPUT_ACTION_MASK) as u8;
        let value1 = ((output & c::OUTPUT_VALUE1_MASK) >> c::OUTPUT_VALUE1_MASK.trailing_zeros()) as u8;
        let value2 = ((output & c::OUTPUT_VALUE2_MASK) >> c::OUTPUT_VALUE2_MASK.trailing_zeros()) as u8;
        let value3 = ((output & c::OUTPUT_VALUE3_MASK) >> c::OUTPUT_VALUE3_MASK.trailing_zeros()) as u8;
        let fired_neurons_count: u16 = ((output & c::OUTPUT_FIRED_NEURONS_MASK) >> c::OUTPUT_FIRED_NEURONS_MASK.trailing_zeros()) as u16;

        // map the action - first match gets priority
        let action = 
                 if action_bits & c::BRAIN_OUTPUT_ACTION_MOVE      == c::BRAIN_OUTPUT_ACTION_MOVE      { CreatureAction::Move      }
            else if action_bits & c::BRAIN_OUTPUT_ACTION_EAT       == c::BRAIN_OUTPUT_ACTION_EAT       { CreatureAction::Eat       }
            else if action_bits & c::BRAIN_OUTPUT_ACTION_REPRODUCE == c::BRAIN_OUTPUT_ACTION_REPRODUCE { CreatureAction::Reproduce }
            else if action_bits & c::BRAIN_OUTPUT_ACTION_SLEEP     == c::BRAIN_OUTPUT_ACTION_SLEEP     { CreatureAction::Sleep     }
            else { CreatureAction::Idle };
        

        (action, value1, value2, value3, fired_neurons_count)
    }

    /******************************************************************************************************************************************/
    /// calculates the tick-age of a creature
    #[inline(always)]
    #[allow(dead_code)]
    fn get_age(&mut self, creature_id: usize) -> u64 {
        self.tick_counter - self.ages[creature_id]
    }

    /******************************************************************************************************************************************/
    /// checks if there's food at the current position
    #[inline(always)]
    #[allow(dead_code)]
    fn has_food(&self, pos: &Coordinate) -> (bool, usize) {
        let x = (pos.x as usize).clamp(0,99);
        let y = (pos.y as usize).clamp(0,99);
        let index = y * (c::WORLD_WIDTH as usize) + x;
        (self.foodmap[index] as f32 + c::ENERGY_COST_EAT >= 0.0, index)
    }

}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// The World's traits
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

impl Default for World {
    fn default() -> Self {
        let random_seed = rand::thread_rng().r#gen();
        Self::new(random_seed)
    }
}
