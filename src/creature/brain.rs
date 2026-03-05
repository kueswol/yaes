use super::genome::Genome;
use super::neurons::*;
use crate::constants as c;


/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// Brain
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct Brain {
    hidden1: Box<[ExecNeuron]>,
    hidden2: Box<[ExecNeuron]>,
    output: Box<[ExecNeuron]>,
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// public functions for Brain
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

impl Brain {
    /******************************************************************************************************************************************/
    /// Constructor
    pub fn new() -> Brain {
        Self::recompile(&Genome::new())
    }
    
    /******************************************************************************************************************************************/
    /// "Compiler": builds the brain based on a genome
    pub fn recompile(genome: &Genome) -> Self {
        let mut hidden1 = Vec::new();
        let mut hidden2 = Vec::new();
        let mut output = Vec::new();

        for gene in &genome.neurons {
            let exec = ExecNeuron {
                mask: (gene.mask << match gene.kind {
                    NeuronKind::Hidden1 => c::NEURON_HIDDEN1_MASK_OFFSET,
                    NeuronKind::Hidden2 => c::NEURON_HIDDEN2_MASK_OFFSET,
                    NeuronKind::Output  => c::NEURON_OUTPUT_MASK_OFFSET,
                }) & match gene.kind {
                    NeuronKind::Hidden1 => c::NEURON_HIDDEN1_MASK_SCOPE,
                    NeuronKind::Hidden2 => c::NEURON_HIDDEN2_MASK_SCOPE,
                    NeuronKind::Output  => c::NEURON_OUTPUT_MASK_SCOPE,
                },
                threshold: gene.threshold,
                kind: gene.kind,
                target_bit: gene.target_bit + match gene.kind {
                    NeuronKind::Hidden1 => c::NEURON_HIDDEN1_TARGET_OFFSET,
                    NeuronKind::Hidden2 => c::NEURON_HIDDEN2_TARGET_OFFSET,
                    NeuronKind::Output  => c::NEURON_OUTPUT_TARGET_OFFSET,
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

    /******************************************************************************************************************************************/
    /// a brain tick
    /// input: bitmap of sensory inputs
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
            output_bits >> c::NEURON_OUTPUT_TARGET_OFFSET, // shift the most significant bits reflecting the actual output down
            fired_count
        )
    }

    /******************************************************************************************************************************************/
    /// helper for debug output
    #[allow(dead_code)]
    fn format_u64(&self, my_u64: &u64) -> String {
        let b = format!("{:064b}", my_u64); 
        format!("{} {} | {} {} {} | {} {} {}", &b[0..8], &b[8..16], &b[16..24], &b[24..32], &b[32..40], &b[40..48], &b[48..56], &b[56..64])
    }
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// traits for Brain
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

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