use super::creature::Dna;

pub const NEURON_HIDDEN1_MASK_OFFSET: u8 = 0;
pub const NEURON_HIDDEN2_MASK_OFFSET: u8 = 24;
pub const NEURON_OUTPUT_MASK_OFFSET : u8 = 24;
pub const NEURON_HIDDEN1_MASK_SCOPE: u64 = 0b00000000_00000000_00000000_00000000_00000000_11111111_11111111_11111111;
pub const NEURON_HIDDEN2_MASK_SCOPE: u64 = 0b00000000_00000000_11111111_11111111_11111111_00000000_00000000_00000000;
pub const NEURON_OUTPUT_MASK_SCOPE : u64 = 0b11111111_11111111_11111111_11111111_11111111_00000000_00000000_00000000;
pub const NEURON_HIDDEN1_TARGET_OFFSET: u8 = 24;
pub const NEURON_HIDDEN2_TARGET_OFFSET: u8 = 48;
pub const NEURON_OUTPUT_TARGET_OFFSET : u8 = 0;
// pub const BIT_OFFSET_HIDDEN1: u8 = 16;
// pub const BIT_OFFSET_HIDDEN2: u8 = 32;
// pub const BIT_OFFSET_OUTPUTS: u8 = 48;

#[derive(Clone, Copy)]
pub enum NeuronKind {
    Hidden1,
    Hidden2,
    Output,
}

/// ---------------------------------------------------------------------------------------------------------
/// Evolvierbare Struktur (Genom)
/// ---------------------------------------------------------------------------------------------------------


#[derive(Clone)]
pub struct GeneNeuron {
    pub mask: u64,
    pub threshold: u8,
    pub kind: NeuronKind,
    pub target_bit: u8,
}

#[derive(Clone)]
pub struct Genome {
    pub neurons: Vec<GeneNeuron>,
}

impl Genome {
    pub fn new() -> Self {
        Self {
            neurons: Vec::new(),
        }
    }

    pub fn from_dna(dna: &Dna) -> Self {
        let mut neurons = Vec::new();

        for chunk in dna.bytes.chunks_exact(8) {
            let mut mask_part =
                (chunk[0] as u64) |
                ((chunk[1] as u64) << 8) |
                ((chunk[2] as u64) << 16) |
                ((chunk[3] as u64) << 24) |
                ((chunk[4] as u64) << 32);
            if mask_part == 0 {
                mask_part = 1 << (chunk[0] % 8);
            }

            let kind = match chunk[5] % 5 {
                0 => NeuronKind::Hidden1,
                1 => NeuronKind::Hidden1,
                2 => NeuronKind::Hidden2,
                3 => NeuronKind::Hidden2,
                _ => NeuronKind::Output,
            };
            
            // let threshold = (chunk[6] % 16) + 1;
            let threshold = match kind {
                NeuronKind::Hidden1 => (chunk[6] % (NEURON_HIDDEN1_MASK_SCOPE.count_ones() / 4) as u8) + 1,
                NeuronKind::Hidden2 => (chunk[6] % (NEURON_HIDDEN2_MASK_SCOPE.count_ones() / 4) as u8) + 1,
                NeuronKind::Output  => (chunk[6] % (NEURON_OUTPUT_MASK_SCOPE.count_ones()  / 4) as u8) + 1,
            };

            // let target_bit = chunk[7] % 64;
            let target_bit = match kind {
                NeuronKind::Hidden1 => chunk[7] % NEURON_HIDDEN2_MASK_SCOPE.count_ones() as u8,
                NeuronKind::Hidden2 => chunk[7] % NEURON_OUTPUT_MASK_SCOPE.count_ones() as u8,
                // NeuronKind::Output  => chunk[7] % NEURON_OUTPUT_MASK_SCOPE.count_ones() as u8,
                NeuronKind::Output  => (chunk[7] % 32) + 1,
            };

            neurons.push(GeneNeuron {
                mask: mask_part,
                threshold,
                kind,
                target_bit,
            });
        }

        Self { neurons }
    }

    // /// Fügt ein neues Neuron hinzu (Mutation)
    // pub fn add_neuron(&mut self, mask: u64, threshold: u8, kind: NeuronKind,target_bit: u8) {
    //     self.neurons.push(GeneNeuron {
    //         mask,
    //         threshold,
    //         kind,
    //         target_bit,
    //     });
    // }
}

/// ---------------------------------------------------------------------------------------------------------
/// Optimierte Runtime-Struktur
/// ---------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct ExecNeuron {
    mask: u64,
    threshold: u8,
    #[allow(dead_code)]
    kind: NeuronKind,
    target_bit: u8,
}

pub struct Brain {
    hidden1: Box<[ExecNeuron]>,
    hidden2: Box<[ExecNeuron]>,
    output: Box<[ExecNeuron]>,
}

impl Brain {
    pub fn new() -> Brain {
        Self::recompile(&Genome::new())
    }
   
    /// "Compiler": wandelt Genome → Runtime Brain
    pub fn recompile(genome: &Genome) -> Self {
        let mut hidden1 = Vec::new();
        let mut hidden2 = Vec::new();
        let mut output = Vec::new();

        for gene in &genome.neurons {
            let exec = ExecNeuron {
                mask: (gene.mask << match gene.kind {
                    NeuronKind::Hidden1 => NEURON_HIDDEN1_MASK_OFFSET,
                    NeuronKind::Hidden2 => NEURON_HIDDEN2_MASK_OFFSET,
                    NeuronKind::Output => NEURON_OUTPUT_MASK_OFFSET,
                }) & match gene.kind {
                    NeuronKind::Hidden1 => NEURON_HIDDEN1_MASK_SCOPE,
                    NeuronKind::Hidden2 => NEURON_HIDDEN2_MASK_SCOPE,
                    NeuronKind::Output => NEURON_OUTPUT_MASK_SCOPE,
                },
                threshold: gene.threshold,
                kind: gene.kind,
                target_bit: gene.target_bit + match gene.kind {
                    NeuronKind::Hidden1 => NEURON_HIDDEN1_TARGET_OFFSET,
                    NeuronKind::Hidden2 => NEURON_HIDDEN2_TARGET_OFFSET,
                    NeuronKind::Output => NEURON_OUTPUT_TARGET_OFFSET,
                },
            };

            match gene.kind {
                NeuronKind::Hidden1 => hidden1.push(exec),
                NeuronKind::Hidden2 => hidden2.push(exec),
                NeuronKind::Output => output.push(exec),
            }
        }

        Self {
            hidden1: hidden1.into_boxed_slice(),
            hidden2: hidden2.into_boxed_slice(),
            output: output.into_boxed_slice(),
        }
    }

    /// Führt einen Tick aus
    /// input: bitkodierte Sensorwerte
    /// outputs: 32 least significant bits of a u64
    ///     |--------|--------|--------|--------|
    ///     | value3 | value2 | value1 | action |
    /// second return value is the count of neurons that fired in this tick
    pub fn tick(&self, input: u64) -> (u64, u32) {
        const DEBUG_OUTPUT: bool = false;
        let mut fired_count: u32 = 0;
        let mut state: u64 = input;
        
        if DEBUG_OUTPUT {
            println!("--- Brain Tick ----------------------------------------------------------");
            println!("INPUT:    state: {}", self.format_u64(&state));
        }
        
        // Hidden 1 Phase
        for neuron in &self.hidden1 {
            let active = state & neuron.mask;
            let sum = active.count_ones() as u8;

            let fire = (sum >= neuron.threshold) as u64;
            fired_count += fire as u32;
            state |= fire << (neuron.target_bit & 63);
            
            if DEBUG_OUTPUT {
                println!("Hidden1:  mask : {}, target: {:001}, threshold: {}, sum: {}", self.format_u64(&neuron.mask), neuron.target_bit, neuron.threshold, sum);
                println!("          state: {}", self.format_u64(&state));
            }
        }
        
        // Hidden 2 Phase
        for neuron in &self.hidden2 {
            let active = state & neuron.mask;
            let sum = active.count_ones() as u8;

            let fire = (sum >= neuron.threshold) as u64;
            fired_count += fire as u32;
            state |= fire << (neuron.target_bit & 63);
            
            if DEBUG_OUTPUT {
                println!("Hidden2:  mask : {}, target: {:001}, threshold: {}, sum: {}", self.format_u64(&neuron.mask), neuron.target_bit, neuron.threshold, sum);
                println!("          state: {}", self.format_u64(&state));
            }
        }
        
        // Output Phase
        let mut output_bits = 0u64;
        
        for neuron in &self.output {
            let active = state & neuron.mask;
            let sum = active.count_ones() as u8;
            
            let fire = (sum >= neuron.threshold) as u64;
            fired_count += fire as u32;
            output_bits |= fire << (neuron.target_bit & 63);

            if DEBUG_OUTPUT {
                println!("OUTPUT:   mask : {}, target: {:001}, threshold: {}, sum: {}", self.format_u64(&neuron.mask), neuron.target_bit, neuron.threshold, sum);
                println!("    output_bits: {}", self.format_u64(&output_bits));
                println!("          state: {}", self.format_u64(&state));
            }
        }

        (
            output_bits >> NEURON_OUTPUT_TARGET_OFFSET, // shift the most significant bits reflecting the actual output down
            fired_count
        )
    }

    #[allow(dead_code)]
    fn format_u64(&self, my_u64: &u64) -> String {
        let b = format!("{:064b}", my_u64); 
        format!("{} {} | {} {} {} | {} {} {}", &b[0..8], &b[8..16], &b[16..24], &b[24..32], &b[32..40], &b[40..48], &b[48..56], &b[56..64])
    }
}

impl Default for Brain {
    fn default() -> Self {
        Brain::new()
    }
}

impl ToString for Brain {
    fn to_string(&self) -> String {
        format!("h1:{},h2:{},o:{}", self.hidden1.len(), self.hidden2.len(), self.output.len())
    }
}