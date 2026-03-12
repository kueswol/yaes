use crate::{
    constants as c,
    ecs::components::{genome::Genome, *},
    utils::*,
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use rayon::prelude::*;

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// The World
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

pub struct World {
    // the world wide root of all "randomness"
    rng: rand::rngs::StdRng,
    
    // statistics
    pub tick_counter: u64,
    pub seed: u64,
    pub deaths: u64,
    pub births: u64,
    pub successfully_reproducing_dna: Vec<String>,
    // entity management
    next_creature_id: usize,
    /// bitmap:
    ///    0b0001 = exists/alive
    ///    0b0010 = can_reproduce
    creatures: Vec<u8>,

    // light components
    positions: Vec<Coordinate>,
    energies: Vec<f32>,
    ages: Vec<u32>,
    brain_inputs: Vec<u64>,
    brain_outputs: Vec<u64>,

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
        Self {
            rng: StdRng::seed_from_u64(rng_seed),
            tick_counter: 0,
            seed: rng_seed,
            deaths: 0,
            births: 0,
            successfully_reproducing_dna: Vec::new(),
            next_creature_id: 0,
            creatures: Vec::with_capacity(c::MAX_POPULATION),
            positions: Vec::with_capacity(c::MAX_POPULATION),
            energies: Vec::with_capacity(c::MAX_POPULATION),
            ages: Vec::with_capacity(c::MAX_POPULATION),
            brain_inputs: Vec::with_capacity(c::MAX_POPULATION),
            brain_outputs: Vec::with_capacity(c::MAX_POPULATION),
            brains: Vec::with_capacity(c::MAX_POPULATION),
            dnas: Vec::with_capacity(c::MAX_POPULATION),
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
        self.schedule_actions();
        self.handle_action_eat();
        self.handle_action_move();
        self.handle_action_sleep();
        self.handle_action_reproduce();
        self.handle_energy_costs();
        self.handle_deaths();

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
        let new_creature_position: Coordinate =
            position.unwrap_or_else(|| Coordinate { x: 50.0, y: 50.0 });
        let new_creature_energy: f32 = 50.0;
        let new_creature_birthtick: u32 = self.tick_counter as u32;
        let new_creature_brain_input: u64 = 0;
        let new_creature_brain_output: u64 = 0;

        let new_creature_dna: Dna = dna.unwrap_or_else(|| Dna::random(256, &mut self.rng));
        let new_creature_genome: Genome = Genome::from_dna(&new_creature_dna);
        let new_creature_brain: Brain = Brain::recompile(&new_creature_genome);

        // we trust, that all vectors are aligned, so new creatures and its components will just be pushed at the end of the vectors
        self.creatures.push(0b0000_0001);
        self.positions.push(new_creature_position);
        self.energies.push(new_creature_energy);
        self.ages.push(new_creature_birthtick);
        self.brain_inputs.push(new_creature_brain_input);
        self.brain_outputs.push(new_creature_brain_output);
        self.brains.push(new_creature_brain);
        self.dnas.push(new_creature_dna);

        true // return successfully spawned
    }

    /******************************************************************************************************************************************/
    /// Delete a creature
    pub fn delete_creature(&mut self, id: usize) {
        self.creatures.swap_remove(id);
        self.positions.swap_remove(id);
        self.energies.swap_remove(id);
        self.ages.swap_remove(id);
        self.brain_inputs.swap_remove(id);
        self.brain_outputs.swap_remove(id);
        self.brains.swap_remove(id);
        self.dnas.swap_remove(id);

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
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// The World's private functions
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
    ///   [19] unused,                  [20] unused,                  [21] unused,
    ///   [22] unused,                  [23] unused,                  [24] unused
    fn update_brain_inputs(&mut self) {
        // for each creature, gather sensory data and update brain_inputs
        let energies = &self.energies;
        let positions = &self.positions;
        let creatures = &self.creatures;
        let brain_outputs = &self.brain_outputs;
        let ages = &self.ages;
        let tick_counter = self.tick_counter;

        self.brain_inputs
            .par_iter_mut()
            .with_min_len(100)
            .enumerate()
            .for_each(move |(entity_id, input)| {
                *input |= Self::encode_3bucket(
                    energies[entity_id],
                    [c::BRAIN_INPUTS_ENERGY_LOW, c::BRAIN_INPUTS_ENERGY_MID],
                ) << 0;
                *input |= Self::encode_3bucket(
                    positions[entity_id].x,
                    [c::BRAIN_INPUTS_POSX_LEFT, c::BRAIN_INPUTS_POSX_CENTER],
                ) << 3;
                *input |= Self::encode_3bucket(
                    positions[entity_id].y,
                    [c::BRAIN_INPUTS_POSY_TOP, c::BRAIN_INPUTS_POSY_CENTER],
                ) << 6;

                let last_action = Self::decode_output(brain_outputs[entity_id]).0;
                *input |= ((last_action == CreatureAction::Idle) as u64) << 9;
                *input |= ((last_action == CreatureAction::Sleep) as u64) << 10;
                *input |= ((last_action == CreatureAction::Move) as u64) << 11;
                *input |= ((last_action == CreatureAction::Eat) as u64) << 12;
                *input |= ((last_action == CreatureAction::Reproduce) as u64) << 13;

                *input |= (((creatures[entity_id] & c::CREATURE_CAN_REPRODUCE) != 0) as u64) << 14;

                *input |= Self::encode_3bucket(
                    ((tick_counter as u32) - ages[entity_id]) as f32,
                    [
                        c::BRAIN_INPUTS_AGE_LOW as f32,
                        c::BRAIN_INPUTS_AGE_MID as f32,
                    ],
                ) << 15;
            });

        // *inputs |= something else << 18;
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
                || (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()), // Lokaler Buffer pro Thread
                |(mut moves, mut eats, mut sleeps, mut reproduces, mut energy_costs),
                 (entity_id, &brain_output)| {
                    let (action, value1, value2, value3, fired_neurons_count) =
                        Self::decode_output(brain_output);
                    let mut energy_cost =
                        (fired_neurons_count as f32) * c::ENERGY_COST_FIRED_NEURON;
                    match action {
                        CreatureAction::Move => {
                            let direction = (value1 as u16 | (value2 as u16) << 8) as f32
                                / (u16::MAX as f32)
                                * 360.0;
                            let speed = value3 as f32 / (u8::MAX as f32) * c::CREATURE_MAX_SPEED;
                            moves.push((entity_id, CreatureEvent::Move { direction, speed }));
                        }
                        CreatureAction::Eat => {
                            eats.push((entity_id, CreatureEvent::Eat));
                        }
                        CreatureAction::Sleep => {
                            sleeps.push((entity_id, CreatureEvent::Sleep));
                        }
                        CreatureAction::Reproduce => {
                            if self.energies[entity_id] >= c::ENERGY_COST_REPRODUCE {
                                reproduces.push((entity_id, CreatureEvent::Reproduce));
                            }
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
    /// schedule the creatures' actions based on their brain outputs
    fn handle_action_eat(&mut self) {
        let pending_eat =
            std::mem::replace(&mut self.pending_eat, Vec::with_capacity(c::MAX_POPULATION));
        for (entity_id, _action) in pending_eat {
            let pos = &self.positions[entity_id];
            if pos.x >= 75.0 && pos.x < 25.0 && pos.y >= 75.0 && pos.y < 25.0 {
                self.pending_energy_costs.push((entity_id, -20.0)); // negative cost = energy gain
            }
        }
    }

    /******************************************************************************************************************************************/
    /// schedule the creatures' actions based on their brain outputs
    fn handle_action_move(&mut self) {
        for (entity_id, action) in &self.pending_move {
            match action {
                CreatureEvent::Move { direction, speed } => {
                    let radians = direction.to_radians();
                    let dx = radians.cos() * speed;
                    let dy = radians.sin() * speed;

                    self.positions[*entity_id].x =
                        (self.positions[*entity_id].x + dx).clamp(0.0, c::WORLD_WIDTH as f32);
                    self.positions[*entity_id].y =
                        (self.positions[*entity_id].y + dy).clamp(0.0, c::WORLD_HEIGHT as f32);

                    self.pending_energy_costs
                        .push((*entity_id, c::ENERGY_COST_MOVE));
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
            let age = (self.tick_counter as u32) - self.ages[entity_id];
            if age < c::REPRODUCE_AGE_MIN || age > c::REPRODUCE_AGE_MAX {
                continue;
            }
            if self.energies[entity_id] < c::ENERGY_COST_REPRODUCE {
                continue;
            }
            let mut new_dna = self.dnas[entity_id].clone();
            self.successfully_reproducing_dna.push(new_dna.to_compact_string());

            new_dna.mutate(&mut self.rng);
            let new_position = self.positions[entity_id];

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

        // for (entity_id, energy_cost) in &self.pending_energy_costs {
        //     self.energies[*entity_id] = (self.energies[*entity_id] - energy_cost).clamp(-1.0, 100.0);
        //     if self.energies[*entity_id] <= 0.0 {
        //         self.pending_deaths.push(*entity_id);
        //     }
        // }
        // self.pending_energy_costs.clear();
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
    fn decode_output(output: u64) -> (CreatureAction, u8, u8, u8, u16) {
        // extract the value parameters
        let value1 =
            ((output & c::OUTPUT_VALUE1_MASK) >> c::OUTPUT_VALUE1_MASK.trailing_zeros()) as u8;
        let value2 =
            ((output & c::OUTPUT_VALUE2_MASK) >> c::OUTPUT_VALUE2_MASK.trailing_zeros()) as u8;
        let value3 =
            ((output & c::OUTPUT_VALUE3_MASK) >> c::OUTPUT_VALUE3_MASK.trailing_zeros()) as u8;
        let fired_neurons_count: u16 = ((output & c::OUTPUT_FIRED_NEURONS_MASK)
            >> c::OUTPUT_FIRED_NEURONS_MASK.trailing_zeros())
            as u16;

        // extract the bits that determine the action
        let action_bits = (output & c::OUTPUT_ACTION_MASK) as u8;

        // if it is zero we define it as Idle
        let action = if action_bits == 0 {
            CreatureAction::Idle
        } else {
            // we're focusing on the highest action bit
            let highest_bit = 7 - action_bits.leading_zeros() as u8;

            match 1 << highest_bit {
                0b00000001 => CreatureAction::Reproduce,
                0b00000010 => CreatureAction::Sleep,
                0b00000100 => CreatureAction::Eat,
                0b00001000 => CreatureAction::Move,
                0b00010000 => CreatureAction::Idle,
                0b00100000 => CreatureAction::Idle,
                0b01000000 => CreatureAction::Idle,
                0b10000000 => CreatureAction::Idle,
                _ => CreatureAction::Idle,
            }
        };

        (action, value1, value2, value3, fired_neurons_count)
    }

    /******************************************************************************************************************************************/
    /// calculates the tick-age of a creature
    #[inline(always)]
    #[allow(dead_code)]
    fn get_age(&mut self, creature_id: usize) -> u32 {
        (self.tick_counter as u32) - self.ages[creature_id]
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
