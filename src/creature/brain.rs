use rand::Rng;

/// ---------------------------------------------------------------------------------------------------------
/// Evolvierbare Struktur (Genom)
/// ---------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub enum NeuronKind {
    Hidden1,
    Hidden2,
    Output,
}

#[derive(Clone)]
pub struct GeneNeuron {
    pub mask: u64,
    pub threshold: u8,
    pub kind: NeuronKind,
    pub target_bit: u8, // wohin wird geschrieben?
}

impl GeneNeuron {
    pub fn mutate(&mut self, rng: &mut impl Rng) {
        // Mutation: zufällige Änderung von Maske, Schwelle oder Zielbit
        if rng.gen_bool(0.1) { // 10% Chance für Mutation
            self.mask ^= 1 << rng.gen_range(0..64); // Flippe ein zufälliges Bit in der Maske
        }
        if rng.gen_bool(0.1) {
            let delta: i8 = if rng.gen_bool(0.5) { 1 } else { -1 };
            self.threshold = self.threshold.saturating_add_signed(delta);
        }
        if rng.gen_bool(0.05) {
            self.target_bit = (self.target_bit + rng.gen_range(1..5)) % 64; // Ändere das Zielbit
        }
    }
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

    /// Fügt ein neues Neuron hinzu (Mutation)
    pub fn add_neuron(&mut self, mask: u64, threshold: u8, kind: NeuronKind,target_bit: u8) {
        self.neurons.push(GeneNeuron {
            mask,
            threshold,
            kind,
            target_bit,
        });
    }

    pub fn mutate(&mut self, rng: &mut impl Rng) {
        for neuron in self.neurons.iter_mut() {
            neuron.mutate(rng);
        }
    }
}

/// ---------------------------------------------------------------------------------------------------------
/// Optimierte Runtime-Struktur
/// ---------------------------------------------------------------------------------------------------------
pub const BIT_OFFSET_HIDDEN1: u8 = 16;
pub const BIT_OFFSET_HIDDEN2: u8 = 32;
pub const BIT_OFFSET_OUTPUTS: u8 = 48;

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
                mask: gene.mask,
                threshold: gene.threshold,
                kind: gene.kind,
                target_bit: gene.target_bit + match gene.kind {
                    NeuronKind::Hidden1 => BIT_OFFSET_HIDDEN1,
                    NeuronKind::Hidden2 => BIT_OFFSET_HIDDEN2,
                    NeuronKind::Output => BIT_OFFSET_OUTPUTS,
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
    /// outputs: 16 least significant bits of a u64
    ///     ---- ---- ---- ----
    ///     val3 val2 val1 act.
    pub fn tick(&self, input: u64) -> u64 {
        const DEBUG_OUTPUT: bool = false;
        let mut state = input;
        
        if DEBUG_OUTPUT {
            println!("--- Brain Tick ----------------------------------------------------------");
            println!("INPUT:    state: {}", self.format_u64(&state));
        }
        
        // Hidden 1 Phase
        for neuron in &self.hidden1 {
            let active = state & neuron.mask;
            let sum = active.count_ones() as u8;

            let fire = (sum >= neuron.threshold) as u64;
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
            output_bits |= fire << (neuron.target_bit & 63);

            if DEBUG_OUTPUT {
                println!("OUTPUT:   mask : {}, target: {:001}, threshold: {}, sum: {}", self.format_u64(&neuron.mask), neuron.target_bit, neuron.threshold, sum);
                println!("    output_bits: {}", self.format_u64(&output_bits));
                println!("          state: {}", self.format_u64(&state));
            }
        }

        output_bits >> BIT_OFFSET_OUTPUTS // shift the most significant bits reflecting the actual output down
    }

    #[allow(dead_code)]
    fn format_u64(&self, my_u64: &u64) -> String {
        let b = format!("{:064b}", my_u64); 
        format!("{} {} | {} {} {} {} | {} {}", &b[0..8], &b[8..16], &b[16..24], &b[24..32], &b[32..40], &b[40..48], &b[48..56], &b[56..64])
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