use std::collections::VecDeque;

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
    params: SimParams,
    rng: rand::rngs::StdRng,        // the world wide root of all "randomness"
    spatial_map: SpatialHashmap,
    carrion_map: SpatialHashmap,
    pub terrain_map: Vec<u8>,
    fertility_map: Vec<u8>,
    pub food_map: Vec<u8>,

    // statistics
    pub tick_counter: u64,
    pub seed: u64,
    pub deaths: u64,
    pub births: u64,
    pub herbivore_eat_success: u64,
    pub herbivore_eat_failed: u64,
    pub carnivore_eat_success: u64,
    pub carnivore_eat_failed: u64,
    pub reproduce_success: u64,
    pub reproduce_failed_age: u64,
    pub reproduce_failed_energy: u64,
    pub reproduce_failed_cooldown: u64,
    pub avg_energy: f32,
    pub avg_age: f32,
    pub total_food: u64,
    pub population_herbivore: u64,
    pub population_carnivore: u64,

    // entity management
    next_creature_id: usize,
    /// bitmap:
    ///    0b0001 = exists/alive
    ///    0b0010 = can_reproduce
    ///    0b0100 = herbivore
    ///    0b1000 = carnivore
    creatures: Vec<u8>,
    /// pos,orientation,size
    dead_creatures: VecDeque<(Coordinate,f32,f32)>,

    // light components
    positions: Vec<Coordinate>,
    orientations: Vec<f32>,
    energies: Vec<f32>,
    ages: Vec<u64>,
    sizes: Vec<f32>,
    colors: Vec<[u8; 3]>,
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

        let mut terrain_map: Vec<u8> = vec![0;(c::WORLD_WIDTH * c::WORLD_HEIGHT) as usize];
        let mut fertility_map: Vec<u8> = vec![0;(c::WORLD_WIDTH * c::WORLD_HEIGHT) as usize];
        
        Self::create_map(&mut terrain_map, &mut fertility_map,&mut rng);
        
        let food_map = fertility_map.clone();


        Self {
            params: SimParams {
                target_tps: 25.0,
                paused: true,
                world: SimParamWorld {
                    max_population      : c::MAX_POPULATION,
                    min_population_herb : c::MIN_POPULATION_HERBIVORE,
                    min_population_carn : c::MIN_POPULATION_CARNIVORE,
                    food_regrowth_amount: c::FOOD_REGROWTH_AMOUNT,
                    food_regrowth_ticks : c::FOOD_REGROWTH_TICKS,
                },
                energy: SimParamEnergy {
                    cost_eat      : c::ENERGY_COST_EAT,
                    cost_sleep    : c::ENERGY_COST_SLEEP,
                    cost_reproduce: c::ENERGY_COST_REPRODUCE,
                    cost_move_slow: c::ENERGY_COST_MOVE_SLOWLY,
                    cost_move_norm: c::ENERGY_COST_MOVE_NORMAL,
                    cost_move_fast: c::ENERGY_COST_MOVE_SPRINT,
                },
                mutation: SimParamMutation {
                    chance_bit_flip_mask     : c::MUTATE_CHANCE_BIT_FLIP_MASK,
                    chance_change_threshold  : c::MUTATE_CHANCE_CHANGE_THRESHOLD,
                    chance_change_target_bit : c::MUTATE_CHANCE_CHANGE_TARGET_BIT,
                    chance_gaining_new_neuron: c::MUTATE_CHANCE_GAINING_NEW_NEURON,
                    chance_loosing_new_neuron: c::MUTATE_CHANCE_LOOSING_NEW_NEURON,
                    chance_mutate_looks      : c::MUTATE_CHANCE_MUTATE_LOOKS,
                },
                creature: SimParamCreature {
                    max_age: c::CREATURE_MAX_AGE,
                    reproduce_age_min: c::REPRODUCE_AGE_MIN,
                    reproduce_age_max: c::REPRODUCE_AGE_MAX,
                    speed: c::CREATURE_SPEED,
                    speed_sprint: c::CREATURE_SPEED_SPRINT,
                    speed_creep: c::CREATURE_SPEED_CREEP,
                },
            },
            rng,
            terrain_map,
            fertility_map,
            food_map,
            tick_counter: 0,
            seed: rng_seed,
            deaths: 0,
            births: 0,
            herbivore_eat_success: 0,
            herbivore_eat_failed: 0,
            carnivore_eat_success: 0,
            carnivore_eat_failed: 0,
            reproduce_success: 0,
            reproduce_failed_age: 0,
            reproduce_failed_energy: 0,
            reproduce_failed_cooldown: 0,
            avg_energy: 0.0,
            avg_age: 0.0,
            total_food: 0,
            population_herbivore: 0,
            population_carnivore: 0,
            next_creature_id: 0,
            spatial_map: SpatialHashmap::new(),
            carrion_map: SpatialHashmap::new(),
            creatures: Vec::with_capacity(c::MAX_POPULATION),
            dead_creatures: VecDeque::with_capacity(c::MAX_POPULATION),
            positions: Vec::with_capacity(c::MAX_POPULATION),
            orientations: Vec::with_capacity(c::MAX_POPULATION),
            energies: Vec::with_capacity(c::MAX_POPULATION),
            ages: Vec::with_capacity(c::MAX_POPULATION),
            sizes: Vec::with_capacity(c::MAX_POPULATION),
            colors: Vec::with_capacity(c::MAX_POPULATION),
            brain_inputs: Vec::with_capacity(c::MAX_POPULATION),
            brain_outputs: Vec::with_capacity(c::MAX_POPULATION),
            brains: Vec::with_capacity(c::MAX_POPULATION),
            dnas: Vec::with_capacity(c::MAX_POPULATION),
            reproduce_cooldown: Vec::with_capacity(c::MAX_POPULATION),
            pending_move: Vec::with_capacity(c::MAX_POPULATION),
            pending_eat: Vec::with_capacity(c::MAX_POPULATION),
            pending_sleep: Vec::with_capacity(c::MAX_POPULATION),
            pending_reproduce: Vec::with_capacity(c::MAX_POPULATION),
            pending_energy_costs: Vec::with_capacity(c::MAX_POPULATION * 2),
            pending_deaths: Vec::with_capacity(c::MAX_POPULATION),
        }
    }

    /******************************************************************************************************************************************/
    /// let the world tick
    pub fn tick(&mut self) {
        
        if (self.population_herbivore as usize) < self.params.world.min_population_herb {
            // let center_x: f32 = c::WORLD_WIDTH as f32 / 2.0;
            // let center_y: f32 = c::WORLD_HEIGHT as f32 / 2.0;
            // let rng_x = self.rng.gen_range((center_x - 10.0)..(center_x + 10.0));
            // let rng_y = self.rng.gen_range((center_y - 10.0)..(center_y + 10.0));
            let rng_x = self.rng.gen_range(100.0..200.0);
            let rng_y = self.rng.gen_range(100.0..200.0);
            self.spawn_creature_herbivore(Some(Coordinate { x: rng_x, y: rng_y }));
            // self.spawn_creature(None, Some(Coordinate { x: 30.0, y: 30.0 }));
        }
        if (self.population_carnivore as usize) < self.params.world.min_population_carn {
            // let center_x: f32 = c::WORLD_WIDTH as f32 / 2.0;
            // let center_y: f32 = c::WORLD_HEIGHT as f32 / 2.0;
            // let rng_x = self.rng.gen_range((center_x - 10.0)..(center_x + 10.0));
            // let rng_y = self.rng.gen_range((center_y - 10.0)..(center_y + 10.0));
            let rng_x = self.rng.gen_range(100.0..200.0);
            let rng_y = self.rng.gen_range(100.0..200.0);
            self.spawn_creature_carnivore(Some(Coordinate { x: rng_x, y: rng_y }));
            // self.spawn_creature(None, Some(Coordinate { x: 30.0, y: 30.0 }));
        }

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

        if self.tick_counter & self.params.world.food_regrowth_ticks == 0 {
            self.grow_food();
        }

        self.update_stats();
        self.tick_counter += 1;
    }

    /******************************************************************************************************************************************/
    /// spawn a new creature
    pub fn spawn_creature(&mut self, dna: Option<Dna>, position: Option<Coordinate>) -> bool {
        // abort here, if we don't have capacity for more creatures
        if self.next_creature_id >= self.params.world.max_population {
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

        let new_creature_dna: Dna = dna.unwrap_or_else(|| Dna::random(512, &mut self.rng)); // we had 256 bytes which work out good
        let new_creature_genome: Genome = Genome::from_dna(&new_creature_dna);
        let new_creature_brain: Brain = Brain::recompile(&new_creature_genome);
        let new_size: f32 = 0.1 + (0.9 * (new_creature_dna.bytes[0] as f32 / 255.0)); // size between 0.1 and 1.0 based on a dna byte
        
        
        let gene_values_kind: Vec<f64> = new_creature_dna.bytes.chunks_exact(8).skip(9)
            .map(|chunk| chunk[5] as f64).collect();
        let gene_values_threshold: Vec<f64> = new_creature_dna.bytes.chunks_exact(8).skip(9)
            .map(|chunk| chunk[6] as f64).collect();
        let gene_values_targetbit: Vec<f64> = new_creature_dna.bytes.chunks_exact(8).skip(9)
            .map(|chunk| chunk[7] as f64).collect();

        let creature_value: u8;
        let new_creature_color: [u8; 3];
        if new_creature_dna.bytes[4] <= 127 {
            // herbivore
            creature_value = c::CREATURE_BITFLAG_IS_ALIVE | c::CREATURE_BITFLAG_IS_HERBIVORE;
            new_creature_color = [
                ((gene_values_kind.iter().sum::<f64>() % 256.0) as u8).clamp(0,150),
                (gene_values_threshold.iter().sum::<f64>() * 2.0 % 256.0) as u8,
                ((gene_values_targetbit.iter().sum::<f64>() * 3.5 % 256.0) as u8).clamp(150,255)
            ];
        } else {
            // carnivore
            creature_value = c::CREATURE_BITFLAG_IS_ALIVE | c::CREATURE_BITFLAG_IS_CARNIVORE;
            new_creature_color = [
                ((gene_values_kind.iter().sum::<f64>() % 256.0) as u8).clamp(150,255),
                (gene_values_threshold.iter().sum::<f64>() * 2.0 % 256.0) as u8,
                ((gene_values_targetbit.iter().sum::<f64>() * 3.5 % 256.0) as u8).clamp(0,150)
            ];
        }
        // let new_creature_color: [u8; 3] = [
        //     new_creature_dna.bytes[1],
        //     new_creature_dna.bytes[2],
        //     new_creature_dna.bytes[3]
        // ];
        let new_creature_reproduce_cooldown: u64 = self.tick_counter + self.params.creature.reproduce_age_min - 10 + (self.rng.gen_range(0..20));

        // we trust, that all vectors are aligned, so new creatures and its components will just be pushed at the end of the vectors
        self.creatures.push(creature_value);
        self.positions.push(new_creature_position);
        self.orientations.push(new_orientation);
        self.energies.push(new_creature_energy);
        self.ages.push(new_creature_birthtick);
        self.sizes.push(new_size);
        self.colors.push(new_creature_color);
        self.brain_inputs.push(new_creature_brain_input);
        self.brain_outputs.push(new_creature_brain_output);
        self.brains.push(new_creature_brain);
        self.dnas.push(new_creature_dna);
        self.reproduce_cooldown.push(new_creature_reproduce_cooldown);

        true // return successfully spawned
    }

    /******************************************************************************************************************************************/
    /// Spawn a herbivore creature
    pub fn spawn_creature_herbivore(&mut self, position: Option<Coordinate>) {
        let mut new_creature_dna: Dna = Dna::random(512, &mut self.rng);
        new_creature_dna.bytes[4] = 90; // ensure herbivore bit is set
        self.spawn_creature(Some(new_creature_dna), position);
    }
    
    /******************************************************************************************************************************************/
    /// Spawn a carnivore creature
    pub fn spawn_creature_carnivore(&mut self, position: Option<Coordinate>) {
        let mut new_creature_dna: Dna = Dna::random(512, &mut self.rng);
        new_creature_dna.bytes[4] = 150; // ensure carnivore bit is set
        self.spawn_creature(Some(new_creature_dna), position);
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
        self.colors.swap_remove(id);
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
        if count + self.creatures.len() > self.params.world.max_population {
            count = self.params.world.max_population - self.creatures.len();
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
            total_food: self.total_food,
            births: self.births,
            deaths: self.deaths,
            herbivore_eat_success: self.herbivore_eat_success,
            herbivore_eat_failed: self.herbivore_eat_failed,
            carnivore_eat_success: self.carnivore_eat_success,
            carnivore_eat_failed: self.carnivore_eat_failed,
            reproduce_success: self.reproduce_success,
            reproduce_failed_age: self.reproduce_failed_age,
            reproduce_failed_energy: self.reproduce_failed_energy,
            reproduce_failed_cooldown: self.reproduce_failed_cooldown,
            population_herbivore: self.population_herbivore,
            population_carnivore: self.population_carnivore,
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
    pub fn get_creatures_view(&self) -> Vec<CreatureView> {

        let mut result = Vec::with_capacity(self.creatures.len());
        for i in 0..self.creatures.len() {
            if self.creatures[i] & c::CREATURE_BITFLAG_IS_ALIVE == 0 {
                continue;
            }

            let pos = self.positions[i];
            let size = self.sizes[i];
            let color = self.colors[i];
            let orientation = self.orientations[i];

            result.push(CreatureView {
                x: pos.x as f32,
                y: pos.y as f32,
                size,
                color,
                orientation,
            });
        }

        result
    }
    
    /******************************************************************************************************************************************/
    /// export a view of the dead creatures for the webserver
    pub fn get_dead_creatures_view(&self) -> Vec<DeadCreatureView> {

        let mut result = Vec::with_capacity(self.dead_creatures.len());
        for (pos,orientation,size) in self.dead_creatures.iter() {
            result.push(DeadCreatureView {
                x: pos.x as f32,
                y: pos.y as f32,
                orientation: *orientation,
                size: *size,
            });
        }

        result
    }
    
    /******************************************************************************************************************************************/
    /// export a view of the creatures for the webserver
    pub fn get_creature_detail_view(&self, id: usize) -> CreatureDetailView {
        if id >= self.creatures.len() {
            return CreatureDetailView {
                id,
                energy: -1.0, // indicate non-existence with negative energy
            };
        }
        CreatureDetailView {
             id: id,
             energy: self.energies[id],
        }
    }

    /******************************************************************************************************************************************/
    /// apply new settings to the world
    pub fn set_sim_params(&mut self, params: SimParams) {
        self.params = params;
    }

    /******************************************************************************************************************************************/
    /// get current settings of the world
    pub fn get_sim_params(&self) -> SimParams {
        self.params
    }

}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// The ECS' systems
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
impl World {

    /******************************************************************************************************************************************/
    /// update each creature's sensoric brain inputs
    ///   [ 1] Energy low,                [ 2] Energy high,               [ 3] PosX left,
    ///   [ 4] PosX right,                [ 5] PosY top,                  [ 6] PosY bottom,
    ///   [ 7] heavy_terrain_ahead_left,  [ 8] heavy_terrain_ahead_right, [ 9] unused,
    ///   [10] unused,                    [11] unused,                    [12] LastAction Move,
    ///   [13] LastAction Eat,            [14] LastAction Reproduce,      [15] can_reproduce,
    ///   [16] age mid,                   [17] age high,                  [18] unused,
    ///   [19] can_eat,                   [20] border_ahead,              [21] more_food_ahead_left,
    ///   [22] more_food_ahead_right,     [23] more food far ahead,       [24] is_full (E >= 90%)
    fn update_brain_inputs(&mut self) {
        // for each creature, gather sensory data and update brain_inputs
        let energies = &self.energies;
        let positions = &self.positions;
        let orientations = &self.orientations;
        let creatures = &self.creatures;
        let dead_creatures = &self.dead_creatures;
        let sizes = &self.sizes;
        let brain_outputs = &self.brain_outputs;
        let ages = &self.ages;
        let tick_counter = &self.tick_counter;
        let food_map = &self.food_map;
        let terrain_map = &self.terrain_map;
        let spatial_map = &self.spatial_map;
        let carrion_map = &self.carrion_map;
        let max_speed = self.params.creature.speed_sprint;

        self.brain_inputs
            .par_iter_mut()
            .with_min_len(100)
            .enumerate()
            .for_each(move |(entity_id, input)| {
                // reset input
                *input = 0;
                
                // #1 energie low
                *input |= ((energies[entity_id] < c::BRAIN_INPUTS_BUCKET_NRGY_LOW_MID) as u64) << 0;
                // #2 energie high
                *input |= ((energies[entity_id] >= c::BRAIN_INPUTS_BUCKET_NRGY_MID_HIGH) as u64) << 1;

                let pos = &positions[entity_id];
                // #3 pos.x left
                *input |= ((pos.x < (c::BRAIN_INPUTS_BUCKET_POSX_L_C as f32)) as u64) << 2;
                // #4 pos.x right
                *input |= ((pos.x > (c::BRAIN_INPUTS_BUCKET_POSX_C_R as f32)) as u64) << 3;
                // #5 pos.y top
                *input |= ((pos.y < (c::BRAIN_INPUTS_BUCKET_POSY_T_C as f32)) as u64) << 4;
                // #6 pos.y bottom
                *input |= ((pos.y > (c::BRAIN_INPUTS_BUCKET_POSY_C_B as f32)) as u64) << 5;

                let (heavy_terrain_ahead_left, heavy_terrain_ahead_right) = Self::check_heavy_terrain_ahead(pos, orientations[entity_id], terrain_map, max_speed);
                // #07 heavy_terrain_ahead_left
                *input |= (heavy_terrain_ahead_left as u64) << 6;
                // #08 heavy_terrain_ahead_right
                *input |= (heavy_terrain_ahead_right as u64) << 7;
                
                // #09 unused
                // *input |= something else << 8;
                // #10 unused
                // *input |= something else << 9;
                // #11 unused
                // *input |= something else << 10;

                // last action
                let last_action = Self::decode_output(&brain_outputs[entity_id]).0;
                // #12 last action move
                *input |= ((last_action == CreatureAction::Move) as u64) << 11;
                // #13 last action eat
                *input |= ((last_action == CreatureAction::Eat) as u64) << 12;
                // #14 last action reproduce
                *input |= ((last_action == CreatureAction::Reproduce) as u64) << 13;

                // #15 can reproduce
                *input |= (((creatures[entity_id] & c::CREATURE_BITFLAG_CAN_REPRODUCE) != 0) as u64) << 14;
                // #16 age mid
                *input |= ((c::BRAIN_INPUTS_BUCKET_AGE_LOW_MID < (*tick_counter - ages[entity_id]) && (*tick_counter - ages[entity_id]) < c::BRAIN_INPUTS_BUCKET_AGE_MID_HIGH) as u64) << 15;
                // #17 age high
                *input |= (((*tick_counter - ages[entity_id]) >= c::BRAIN_INPUTS_BUCKET_AGE_MID_HIGH) as u64) << 16;
                
                // #18 others nearby (are there other creatures in the 8 surrounding tiles?)
                *input |= ((spatial_map.get_creatures_in_cell_with_neighbors(*pos).len() > 0) as u64) << 17;

                // #20 are we facing the border; #19 comes below, as we're handling herbivore and canivor specific below
                *input |= (Self::check_border_ahead(pos, orientations[entity_id], max_speed) as u64) << 19;
                
                let max_energy = 90.0 + (sizes[entity_id] * 100.0);
                // #24 90% of max_energy reached
                *input |= ((energies[entity_id] >= max_energy * 0.9) as u64) << 23;

                let can_eat: bool;
                let food_ahead_left: bool;
                let food_ahead_right: bool;
                let food_ahead_center: bool;

                if Self::is_herbivore(&creatures[entity_id]) {
                    can_eat = food_map[(pos.y as usize).clamp(0,c::WORLD_HEIGHT as usize - 1) * (c::WORLD_WIDTH as usize) + (pos.x as usize).clamp(0,c::WORLD_WIDTH as usize - 1)] > 0;
                    (food_ahead_left, food_ahead_center, food_ahead_right) = Self::check_food_ahead(pos, orientations[entity_id], food_map, max_speed);
                } else {
                    (can_eat, _) = Self::can_eat_carrion(carrion_map, pos, dead_creatures);
                    (food_ahead_left, food_ahead_center, food_ahead_right) = Self::check_carrion_nearby(pos, orientations[entity_id], carrion_map, dead_creatures, max_speed);
                    
                }
                
                // #19 can eat (is there food at the current position?)
                *input |= (can_eat as u64) << 18;
                // #21 more food ahead left
                *input |= (food_ahead_left  as u64) << 20;
                // #22 more food ahead right
                *input |= (food_ahead_right as u64) << 21;
                // #23 more food far ahead (in the center)
                *input |= (food_ahead_center as u64) << 22;

        });

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
        let sizes = &self.sizes;

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
                    
                    let (action, _value1, _value2, _value3, fired_neurons_count) = Self::decode_output(&brain_output);
                    let mut energy_cost = (fired_neurons_count as f32) * c::ENERGY_COST_FIRED_NEURON;

                    // basal metabolism cost, based on size
                    let basal_metabolism = sizes[entity_id] * 0.1;  // Anpassbar, z.B. 0.1 pro Größeneinheit
                    energy_cost += basal_metabolism;

                    match action {
                        CreatureAction::Move => {
                            let sprint: bool = (_value1 & c::BRAIN_OUTPUT_VALUE1_MOVE_FAST) != 0;
                            let creep: bool = (_value1 & c::BRAIN_OUTPUT_VALUE1_MOVE_SLOW) != 0;
                            moves.push((entity_id, CreatureEvent::Move{ sprint, creep }));
                        }
                        CreatureAction::Eat => {
                            eats.push((entity_id, CreatureEvent::Eat));
                        }
                        CreatureAction::Sleep => {
                            sleeps.push((entity_id, CreatureEvent::Sleep));
                        }
                        CreatureAction::Reproduce => {
                            reproduces.push((entity_id, CreatureEvent::Reproduce));
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
                
                // order matters - normal turning > fast turning > slow turning,
                // so we overwrite the turn_left and turn_right values accordingly
                let mut turn_left: f32 = 0.0;
                turn_left = if (value1 & c::BRAIN_OUTPUT_VALUE1_TURN_L_SLOW) != 0 { 0.017 } else { turn_left };
                turn_left = if (value1 & c::BRAIN_OUTPUT_VALUE1_TURN_L_FAST) != 0 { 0.175 } else { turn_left };
                turn_left = if (value1 & c::BRAIN_OUTPUT_VALUE1_TURN_L_NORM) != 0 { 0.087 } else { turn_left };
                let mut turn_right: f32 = 0.0;
                turn_right = if (value1 & c::BRAIN_OUTPUT_VALUE1_TURN_R_SLOW) != 0 { 0.017 } else { turn_right };
                turn_right = if (value1 & c::BRAIN_OUTPUT_VALUE1_TURN_R_FAST) != 0 { 0.175 } else { turn_right };
                turn_right = if (value1 & c::BRAIN_OUTPUT_VALUE1_TURN_R_NORM) != 0 { 0.087 } else { turn_right };
                
                // both values are applied to the delta, so if both turn left and turn right are activated,
                // they will cancel each other out to some extent
                let mut delta: f32 = 0.0;
                delta -= turn_left;
                delta += turn_right;
                *orientation += delta;
                
                // clamping to rad
                *orientation = orientation.rem_euclid(std::f32::consts::TAU);
                
            });
    }

    /******************************************************************************************************************************************/
    /// schedule the creatures' actions based on their brain outputs
    fn handle_action_eat(&mut self) {
        let pending_eat =
            std::mem::replace(&mut self.pending_eat, Vec::with_capacity(self.params.world.max_population));
        for (entity_id, _action) in pending_eat {
            let pos = &self.positions[entity_id];

            if Self::is_carnivore(&self.creatures[entity_id]) {
                let (can_eat, carrion_index) = Self::can_eat_carrion(&self.carrion_map, pos, &self.dead_creatures);
                if can_eat {
                    let carrion_nutrition = self.dead_creatures[carrion_index].2 * 200.0;
                    self.pending_energy_costs.push((entity_id, -carrion_nutrition)); // negative cost => energy gain
                    self.carrion_map.remove(carrion_index,self.dead_creatures[carrion_index].0);
                    self.dead_creatures[carrion_index].2 = 0.0; // mark as eaten - we can only remove it later
                    self.carnivore_eat_success += 1;
                    continue;
                } else {
                    self.carnivore_eat_failed += 1;
                    continue;
                }
            }

            let (has_food, index, food_amount) = Self::has_food(&self.food_map, pos);
            if has_food {
                let food_eaten = (self.params.energy.cost_eat.abs() as u8).min(food_amount); // we can only eat as much food as there is, and we want to eat at most the absolute value of the energy cost
                self.food_map[index] = self.food_map[index].saturating_sub(food_eaten);
                let mut nutrition_value = (food_eaten as f32) * -1.0 * self.sizes[entity_id] * self.sizes[entity_id]; // bigger creatures get more nutrition out of the same amount of food
                
                if self.terrain_map[index] == 1 { // it's a swamp
                    nutrition_value *= 3.0;
                }
                nutrition_value *= 1.0 + (self.spatial_map.get_creatures_in_cell_with_neighbors(*pos).len() as f32 * 0.1);
                self.pending_energy_costs.push((entity_id, nutrition_value)); // negative cost => energy gain
                self.herbivore_eat_success += 1;
                continue;
            }
            self.herbivore_eat_failed += 1;
        }

        // remove eaten ones
        self.dead_creatures.retain(|&(_, _, size)| size > 0.0);
    }

    /******************************************************************************************************************************************/
    /// schedule the creatures' actions based on their brain outputs
    fn handle_action_move(&mut self) {
        for (entity_id, action) in &self.pending_move {
            match action {
                CreatureEvent::Move { sprint, creep } => {
                    let speed = if *sprint {
                        self.params.creature.speed_sprint / self.sizes[*entity_id]
                    } else if *creep {
                        self.params.creature.speed_creep / self.sizes[*entity_id]
                    } else {
                        self.params.creature.speed / self.sizes[*entity_id]
                    };
                    let mut energy = if *sprint {
                        self.params.energy.cost_move_fast
                    } else if *creep {
                        self.params.energy.cost_move_slow
                    } else {
                        self.params.energy.cost_move_norm
                    };
                    
                    let dx = self.orientations[*entity_id].cos() * speed;
                    let dy = self.orientations[*entity_id].sin() * speed;

                    let new_x = (self.positions[*entity_id].x + dx).clamp(0.0, c::WORLD_WIDTH as f32);
                    let new_y = (self.positions[*entity_id].y + dy).clamp(0.0, c::WORLD_HEIGHT as f32);
                    
                    self.positions[*entity_id].x = new_x;
                    self.positions[*entity_id].y = new_y;
                    
                    
                    if Self::get_biome(&self.terrain_map, &self.positions[*entity_id]) == 1 { // it's a swamp
                        energy *= 3.0;
                    }

                    // if Self::is_carnivore(&self.creatures[*entity_id]) {
                    //     energy *= 0.1; // carnivores get a movement bonus
                    // }
                    self.pending_energy_costs.push((*entity_id, energy));
                    
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
            Vec::with_capacity(self.params.world.max_population),
        );

        if self.creatures.len() + reproductions.len() > self.params.world.max_population {
            // if we have more reproductions than capacity, we randomly select which ones will reproduce
            let overpopulation = (self.creatures.len() + reproductions.len()) - self.params.world.max_population;
            for _ in 0..overpopulation {
                reproductions.swap_remove(self.rng.gen_range(0..reproductions.len()));
            }
        }

        for (entity_id, _action) in reproductions {
            let age = (self.tick_counter) - self.ages[entity_id];
            if age < self.params.creature.reproduce_age_min || age > self.params.creature.reproduce_age_max {
                self.reproduce_failed_age += 1;
                continue;
            }
            if self.energies[entity_id] < self.params.energy.cost_reproduce {
                self.reproduce_failed_energy += 1;
                self.pending_energy_costs.push((entity_id, self.params.energy.cost_reproduce / 2.0));
                continue;
            }
            if self.reproduce_cooldown[entity_id] > self.tick_counter {
                self.reproduce_failed_cooldown += 1;
                continue;
            }
            
            let mut new_dna = self.dnas[entity_id].clone();
            self.reproduce_success += 1;
            self.creatures[entity_id] &= !c::CREATURE_BITFLAG_CAN_REPRODUCE; // reset reproduce ability until next age check
            self.reproduce_cooldown[entity_id] = self.tick_counter + (self.params.creature.reproduce_age_min / 4) - 10 + (self.rng.gen_range(0..20)); // set reproduce cooldown
            
            new_dna.mutate(&mut self.rng,&self.params.mutation);
            let new_position = Coordinate {
                x: (self.positions[entity_id].x + self.rng.gen_range(-0.5..0.5)).clamp(0.0, c::WORLD_WIDTH as f32),
                y: (self.positions[entity_id].y + self.rng.gen_range(-0.5..0.5)).clamp(0.0, c::WORLD_HEIGHT as f32),
            };

            self.spawn_creature(Some(new_dna), Some(new_position));

            self.pending_energy_costs
                .push((entity_id, self.params.energy.cost_reproduce));
            self.births += 1;
        }
    }

    /******************************************************************************************************************************************/
    /// schedule the creatures' actions based on their brain outputs
    fn handle_energy_costs(&mut self) {
        let mut pending_energy_costs = std::mem::replace(
            &mut self.pending_energy_costs,
            Vec::with_capacity(self.params.world.max_population * 2),
        );
        pending_energy_costs.sort_unstable_by_key(|&(id, _)| id);

        let mut iter = pending_energy_costs.into_iter().peekable();
        while let Some((id, mut total)) = iter.next() {
            while iter.peek().map_or(false, |&(next_id, _)| next_id == id) {
                total += iter.next().unwrap().1;
            }

            let energy = &mut self.energies[id];
            let mut max_energy = 90.0 + (self.sizes[id] * 100.0); // max energy based on size
            
            if Self::is_carnivore(&self.creatures[id]) {
                total *= 0.1;
                max_energy *= 2.0;
            }
            
            *energy = (*energy - total).clamp(-1.0, max_energy);
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
                if age > self.params.creature.reproduce_age_min as u64 
                && age < self.params.creature.reproduce_age_max as u64
                && self.reproduce_cooldown[entity_id] <= self.tick_counter {
                    *creature |= c::CREATURE_BITFLAG_CAN_REPRODUCE;
                } else {
                    *creature &= !c::CREATURE_BITFLAG_CAN_REPRODUCE;
                }
                if age > self.params.creature.max_age as u64 {
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
            Vec::with_capacity(self.params.world.max_population),
        );
        deaths.sort_unstable_by(|a, b| b.cmp(a));
        deaths.dedup();
        for entity_id in deaths {
            
            if self.dead_creatures.len() > self.params.world.max_population - 10 {
                for _ in 0..10 { self.dead_creatures.pop_front(); }
            }
            let pos = self.positions[entity_id];
            let orientation = self.orientations[entity_id];
            let size = self.sizes[entity_id];
            
            self.dead_creatures.push_back((pos, orientation, size));

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

        let mut carrion_map = SpatialHashmap::new();
        let dead_creatures = &self.dead_creatures;

        dead_creatures.iter().enumerate().for_each(|(index, &(position, _, _))| {
                carrion_map.insert(index, position);
        });
        self.carrion_map = carrion_map;
    }
    
    /******************************************************************************************************************************************/
    /// schedule the creatures' actions based on their brain outputs
    #[allow(dead_code)]
    fn handle_separation(&mut self) {
        // collect adjustments locally as we shouldn't apply them immediately in the parallel loop
        let position_adjustments: Vec<(usize, f32, f32)> = (0..self.creatures.len())
            .into_par_iter()
            .with_min_len(100)
            .filter_map(|id| {
                let mut dx = 0.0;
                let mut dy = 0.0;
                
                let pos = self.positions[id];
                let size = self.sizes[id];
                let neighbors = self.spatial_map.get_creatures_in_cell_with_neighbors(pos);
                
                for &other_id in &neighbors {
                    if id >= other_id { continue; }
                    
                    let other_pos = self.positions[other_id];
                    let other_size = self.sizes[other_id];
                    let dx_raw = pos.x - other_pos.x;
                    let dy_raw = pos.y - other_pos.y;
                    let distance_sqr = dx_raw * dx_raw + dy_raw * dy_raw;
                    let min_distance = (size + other_size) as f32;
                    let min_distance_sqr = min_distance * min_distance;
                    
                    if distance_sqr >= min_distance_sqr || distance_sqr == 0.0 { continue; }
                    
                    let distance = distance_sqr.sqrt();
                    let overlap = min_distance - distance;
                    let nx = dx_raw / distance;
                    let ny = dy_raw / distance;
                    
                    dx += nx * overlap / 2.0;
                    dy += ny * overlap / 2.0;
                }
                
                if dx != 0.0 || dy != 0.0 {
                    Some((id, dx, dy))
                } else {
                    None
                }
            }
        )
        .collect();
        
        // apply the collected adjustments
        for (id, dx, dy) in position_adjustments {
            if dx != 0.0 || dy != 0.0 {
                self.positions[id].x = (self.positions[id].x + dx).clamp(0.0, c::WORLD_WIDTH as f32);
                self.positions[id].y = (self.positions[id].y + dy).clamp(0.0, c::WORLD_HEIGHT as f32);
            }
        } 
    }

    /******************************************************************************************************************************************/
    /// let the food spread and regrow
    fn grow_food(&mut self) {
        let mut new_food_map = self.food_map.clone();
        let fertility_map = &self.fertility_map;
        let terrain_map = &self.terrain_map;

        new_food_map.par_iter_mut().with_min_len(100).enumerate().for_each(|(index, cell)| {
            let fertility = fertility_map[index];
            if fertility == 0 { return; } // no food can grow here, so we skip
            
            let is_swamp = terrain_map[index] == 1;
            let biome_factor = if is_swamp { 2 } else { 1 };

            let max_food = fertility.saturating_add(self.params.world.food_regrowth_amount * 2);
            let regrowth = ((self.params.world.food_regrowth_amount as f32 * (fertility as f32 / 255.0)) as u8).max(1);
            let new_val = cell.saturating_add(regrowth * biome_factor);
            
            *cell = new_val.min(max_food);

            // if rng.gen_bool(fertility as f64 / 255.0) {
            //     *cell = cell.saturating_add(self.params.world.food_regrowth_amount);
            // }
        });
        self.food_map = new_food_map;
    }
    /******************************************************************************************************************************************/
    /// update internal world statistics
    fn update_stats(&mut self) {
        let total_energy: f32 = self.energies.iter().sum();
        self.avg_energy = total_energy / self.creatures.len().max(1) as f32;
        
        let total_age: u64 = self.ages.iter().sum();
        self.avg_age = self.tick_counter as f32 - (total_age as f32 / self.creatures.len().max(1) as f32);

        self.total_food = self.food_map.par_iter().map(|&b| b as u64).sum();

        (self.population_carnivore, self.population_herbivore) = self.creatures.par_iter().fold(
            || (0u64, 0u64),
            |(mut carnivores, mut herbivores), &creature| {
                if Self::is_carnivore(&creature) {
                    carnivores += 1;
                }
                if Self::is_herbivore(&creature) {
                    herbivores += 1;
                }
                (carnivores, herbivores)
            },
        ).reduce(
            || (0u64, 0u64),
            |(c1, h1), (c2, h2)| (c1 + c2, h1 + h2)
        );
    }
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// Helper functions
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
impl World {
    /******************************************************************************************************************************************/
    /// encodes a value into 3 buckets and returns a 3-bit representation as u64
    #[inline(always)]
    #[allow(dead_code)]
    fn encode_3bucket(val: f32, buckets: [f32; 2]) -> u64 {
        let low = (val <= buckets[0]) as u64;
        let mid = ((val > buckets[0]) && (val <= buckets[1])) as u64;
        let high = (val > buckets[1]) as u64;

        (high << 2) | (mid << 1) | low
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
        // "sleep" & "idle" are "move" for now, as we haven't implemented sleeping yet, and idle would just be doing nothing
        let action = 
                 if action_bits & c::BRAIN_OUTPUT_ACTION_EAT       == c::BRAIN_OUTPUT_ACTION_EAT       { CreatureAction::Eat       }
            else if action_bits & c::BRAIN_OUTPUT_ACTION_MOVE      == c::BRAIN_OUTPUT_ACTION_MOVE      { CreatureAction::Move      }
            else if action_bits & c::BRAIN_OUTPUT_ACTION_REPRODUCE == c::BRAIN_OUTPUT_ACTION_REPRODUCE { CreatureAction::Reproduce }
            else if action_bits & c::BRAIN_OUTPUT_ACTION_SLEEP     == c::BRAIN_OUTPUT_ACTION_SLEEP     { CreatureAction::Move      }
            else { CreatureAction::Move };
        

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
    // #[allow(dead_code)]
    fn has_food(food_map: &Vec<u8>, pos: &Coordinate) -> (bool, usize, u8) { // return whether there's food, the index in the food_map and the amount of food
        let x = (pos.x as usize).clamp(0,(c::WORLD_WIDTH  - 1) as usize);
        let y = (pos.y as usize).clamp(0,(c::WORLD_HEIGHT - 1) as usize);
        let index = y * (c::WORLD_WIDTH as usize) + x;
        let has_food: bool = food_map[index] > 0;
        (has_food, index, food_map[index])
    }

    /******************************************************************************************************************************************/
    /// checks if it's a herbivore
    #[inline(always)]
    fn is_herbivore(creature: &u8) -> bool {
        (*creature & c::CREATURE_BITFLAG_IS_HERBIVORE) != 0
    }

    /******************************************************************************************************************************************/
    /// checks if it's a carnivore
    #[allow(dead_code)]
    #[inline(always)]
    fn is_carnivore(creature: &u8) -> bool {
        (*creature & c::CREATURE_BITFLAG_IS_CARNIVORE) != 0
    }
    
    /******************************************************************************************************************************************/
    /// returns the current biome
    #[inline(always)]
    fn get_biome(terrain_map: &Vec<u8>, pos: &Coordinate) -> u8 {
        let x = (pos.x as usize).clamp(0, (c::WORLD_WIDTH - 1) as usize);
        let y = (pos.y as usize).clamp(0, (c::WORLD_HEIGHT - 1) as usize);
        let index = y * (c::WORLD_WIDTH as usize) + x;
        terrain_map[index]
    }

    /******************************************************************************************************************************************/
    /// check whether there's a border ahead in the current direction
    #[inline(always)]
    fn check_border_ahead(pos: &Coordinate, orientation: f32, max_speed: f32) -> bool {
        let look_ahead_x = pos.x + orientation.cos() * max_speed * 1.1; // 10% over the distance reachable with sprint
        let look_ahead_y = pos.y + orientation.sin() * max_speed * 1.1;
        look_ahead_x < 0.0 || look_ahead_x >= c::WORLD_WIDTH as f32 || look_ahead_y < 0.0 || look_ahead_y >= c::WORLD_HEIGHT as f32
    }

    /******************************************************************************************************************************************/
    /// check whether there's heavy terrain ahead
    #[inline(always)]
    fn check_heavy_terrain_ahead(pos: &Coordinate, orientation: f32, terrain_map: &Vec<u8>, max_speed: f32) -> (bool, bool) {
        let look_ahead_distance = (max_speed * 2.0).max(1.1415) * 1.5; // twice the distance reachable with sprint

        let left_cone_angle  = orientation + std::f32::consts::FRAC_PI_4; // 45 degrees to the left
        let right_cone_angle = orientation - std::f32::consts::FRAC_PI_4; // 45 degrees to the right

        let left_cone_x  = pos.x + left_cone_angle.cos() * look_ahead_distance;
        let left_cone_y  = pos.y + left_cone_angle.sin() * look_ahead_distance;
        let right_cone_x = pos.x + right_cone_angle.cos() * look_ahead_distance;
        let right_cone_y = pos.y + right_cone_angle.sin() * look_ahead_distance;

        let left_terrain  = Self::get_biome(terrain_map, &Coordinate { x: left_cone_x , y: left_cone_y  }) == 1;
        let right_terrain = Self::get_biome(terrain_map, &Coordinate { x: right_cone_x, y: right_cone_y }) == 1;

        (left_terrain, right_terrain)
    }

    /******************************************************************************************************************************************/
    /// check whether there's carrion at the current location
    /// returns (can_eat & index for dead_creatures)
    fn can_eat_carrion(carrion_map: &SpatialHashmap, pos: &Coordinate, dead_creatures: &VecDeque<(Coordinate,f32,f32)>) -> (bool, usize) {
        let carrion_ids = carrion_map.get_creatures_in_cell(*pos);
        for &id in &carrion_ids {
            if let Some((carrion_pos, _, size)) = dead_creatures.get(id) {
                if *size > 0.0 {  // Nur essbar, wenn Größe > 0
                    let distance_sqr = (carrion_pos.x - pos.x).powi(2) + (carrion_pos.y - pos.y).powi(2);
                    if distance_sqr < size.powi(2) + 1.0 { // within reach (size of carrion + small margin)
                        return (true, id);
                    }
                }
            }
        }
        (false, 0)
    }
    /******************************************************************************************************************************************/
    /// check whether there's carrion ahead
    /// returns (carrion_ahead_left, carrion_ahead_center, carrion_ahead_right)
    #[inline(always)]
    fn check_carrion_nearby(pos: &Coordinate, orientation: f32, carrion_map: &SpatialHashmap, dead_creatures: &VecDeque<(Coordinate,f32,f32)>, max_speed: f32) -> (bool, bool, bool) {
        let look_ahead_distance = (max_speed * 2.0).max(1.1415);
        let left_cone_angle  = orientation + std::f32::consts::FRAC_PI_4; // 45 degrees to the left
        let right_cone_angle = orientation - std::f32::consts::FRAC_PI_4; // 45 degrees to the right

        let left_cone  = Coordinate {
            x: pos.x + left_cone_angle.cos() * look_ahead_distance,
            y: pos.y + left_cone_angle.sin() * look_ahead_distance
        };
        let right_cone = Coordinate {
            x: pos.x + right_cone_angle.cos() * look_ahead_distance,
            y: pos.y + right_cone_angle.sin() * look_ahead_distance
        };

        let carrion_ids = carrion_map.get_creatures_in_cell_with_neighbors(*pos);
        // pos, orientation, size
        // let local_carrion = carrion_ids.iter().filter_map(|&id| dead_creatures.get(id));
        
        let mut carrion_ahead_left = false;
        let mut carrion_ahead_right = false;
        let mut carrion_ahead_center = false;

        for carrion_id in carrion_ids {
            if carrion_ahead_left && carrion_ahead_right && carrion_ahead_center { break; } // we found all
            if let Some((carrion_pos, _, size)) = dead_creatures.get(carrion_id) {
                if *size <= 0.0 { continue; } // only eatable if size > 0
                let dx = carrion_pos.x - pos.x;
                let dy = carrion_pos.y - pos.y;
                let distance_sqr = dx * dx + dy * dy;
                let look_ahead_sq = (look_ahead_distance + size).powi(2);
                if distance_sqr > look_ahead_sq { continue; } // out of reach
                
                
                if !carrion_ahead_left {
                    let dxl = carrion_pos.x - left_cone.x;
                    let dyl = carrion_pos.y - left_cone.y;
                    if dxl * dxl + dyl * dyl < look_ahead_sq {
                        carrion_ahead_left = true;
                    }
                }
                
                if !carrion_ahead_right {
                    let dxl = carrion_pos.x - right_cone.x;
                    let dyl = carrion_pos.y - right_cone.y;
                    if dxl * dxl + dyl * dyl < look_ahead_sq {
                        carrion_ahead_right = true;
                    }
                }

                if !carrion_ahead_center {
                    let forward_dist = dx * orientation.cos() + dy * orientation.sin();
                    if forward_dist > 0.0 && forward_dist <= look_ahead_distance + *size {
                        let lateral = -dx * orientation.sin() + dy * orientation.cos();
                        if lateral.abs() <= *size {
                            carrion_ahead_center = true;
                        }
                    }
                }
                
            }
        }

        (carrion_ahead_left, carrion_ahead_center, carrion_ahead_right)
    }

    /******************************************************************************************************************************************/
    /// check whether there's food ahead in the current direction (left, center, right)
    #[inline(always)]
    fn check_food_ahead(pos: &Coordinate, orientation: f32, food_map: &Vec<u8>, max_speed: f32) -> (bool, bool, bool) {
        let look_ahead_distance = (max_speed * 2.0).max(1.1415); // 100% over the distance reachable with sprint, or at least 1.1415
        // let look_ahead_x = pos.x + orientation.cos() * look_ahead_distance;
        // let look_ahead_y = pos.y + orientation.sin() * look_ahead_distance;

        let left_cone_angle  = orientation + std::f32::consts::FRAC_PI_4; // 45 degrees to the left
        let right_cone_angle = orientation - std::f32::consts::FRAC_PI_4; // 45 degrees to the right

        let left_cone_x  = pos.x + left_cone_angle.cos() * look_ahead_distance;
        let left_cone_y  = pos.y + left_cone_angle.sin() * look_ahead_distance;
        let right_cone_x = pos.x + right_cone_angle.cos() * look_ahead_distance;
        let right_cone_y = pos.y + right_cone_angle.sin() * look_ahead_distance;

        let left_food  = Self::has_food(food_map, &Coordinate { x: left_cone_x, y: left_cone_y }).0;
        let right_food = Self::has_food(food_map, &Coordinate { x: right_cone_x, y: right_cone_y }).0;

        // center will go far more ahead, but stops at the next food
        let mut center_x = pos.x;
        let mut center_y = pos.y;
        for _ in 1..=20 {
            center_x += orientation.cos() * look_ahead_distance;
            center_y += orientation.sin() * look_ahead_distance;
            if Self::has_food(food_map, &Coordinate { x: center_x, y: center_y }).0 {
                return (left_food, true, right_food);
            }
        }
        (left_food, false, right_food)
    }

    /******************************************************************************************************************************************/
    /// create a map for biome structure and food-distribution
    #[inline(always)]
    fn create_map(terrain_map: &mut Vec<u8>, fertility_map: &mut Vec<u8>, _rng: &mut StdRng) {
        
        let mut hotspots: Vec<(Coordinate, f32)> = Vec::new();
        for _ in 0..20 {
            hotspots.push((Coordinate {
                x: _rng.gen_range(25.0..((c::WORLD_WIDTH  - 25) as f32)),
                y: _rng.gen_range(25.0..((c::WORLD_HEIGHT - 25) as f32)),
            }, _rng.gen_range(50.0..125.0)));
        }
        
        for x in 0..c::WORLD_WIDTH {
            for y in 0..c::WORLD_HEIGHT {
                // determine the biome
                // let mut biome: u8 = 0;
                let mut is_swamp = false;
                let fx = x as f32;
                let fy = y as f32;

                let base = fx + fy;
                let noise = 
                    (fx * 0.15 + fy * 0.05).sin() * 10.0 +
                    (fx * 0.4 - fy * 0.3).sin() * 4.0 +
                    (fx * 0.8 + fy * 0.9).cos() * 3.0 -
                    (fx * 0.01).sin() * 30.0;
                if base + noise > ((c::WORLD_HEIGHT + c::WORLD_WIDTH) as f32 / 2.0) {
                    is_swamp = true;
                }

                // calculate fertility based on distance to hotspots and biome
                let mut fertility: u8 = 0;
                
                let min_fertility: u8;
                let max_fertility: u8;
                let strength_factor: f32;

                if is_swamp {
                    min_fertility = 0;
                    max_fertility = 255;
                    strength_factor = 0.5;
                } else {
                    min_fertility = 50;
                    max_fertility = 100;
                    strength_factor = 1.25;
                }

                for (hotspot_pos, hotspot_strength) in &hotspots {
                    let dx = fx - hotspot_pos.x;
                    let dy = fy - hotspot_pos.y;
                    let distance_sqr = dx * dx + dy * dy + 1.0; // add 1 to prevent division by zero
                    let biome_strength_sqr = hotspot_strength * hotspot_strength * strength_factor * strength_factor;
                    fertility = fertility.saturating_add((biome_strength_sqr / distance_sqr).min(255.0) as u8);
                }
                fertility = fertility.clamp(min_fertility, max_fertility);

                let index: usize = (y as usize) * (c::WORLD_WIDTH as usize) + (x as usize);
                terrain_map[index] = if is_swamp { 1 } else { 0 };
                fertility_map[index] = fertility;
            }
        }
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
