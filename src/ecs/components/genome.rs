use super::dna::Dna;
use super::neurons::*;
use crate::constants as c;

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// Genome
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct Genome {
    pub neurons: Vec<GeneNeuron>,
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// public functions for Genome
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

impl Genome {
    pub fn new() -> Self {
        Self {
            neurons: Vec::new(),
        }
    }

    /******************************************************************************************************************************************/
    /// Constructor: builds a genome from a DNA sequence
    pub fn from_dna(dna: &Dna) -> Self {
        let mut neurons = Vec::new();

        // chunk explanation:
        // - bytes 0-4: mask (which inputs this neuron reads, we mutate)
        // - byte 5: kind (what type of neuron this is)
        // - byte 6: threshold (how many inputs need to be active for the neuron to fire)
        // - byte 7: target_bit (which bit this neuron outputs to)

        for chunk in dna.bytes.chunks_exact(8).skip(1) { //frist 8 bytes are reserved for looks
            let mut mask_part =
                (chunk[0] as u64) |
                ((chunk[1] as u64) << 8) |
                ((chunk[2] as u64) << 16) |
                ((chunk[3] as u64) << 24) |
                ((chunk[4] as u64) << 32);
            if mask_part == 0 {
                mask_part = 1 << (chunk[0] % 8);
            }

            let kind = match chunk[5] % 9 {
                0 => NeuronKind::Hidden1,
                1 => NeuronKind::Hidden1,
                2 => NeuronKind::Hidden2,
                3 => NeuronKind::Hidden2,
                4 => NeuronKind::Output,
                5 => NeuronKind::Output,
                6 => NeuronKind::Hidden2,
                7 => NeuronKind::Hidden2,
                _ => NeuronKind::Hidden1,
            };
            
            // let threshold = (chunk[6] % 16) + 1;
            let threshold = match kind {
                NeuronKind::Hidden1 => (chunk[6] % (c::NEURON_HIDDEN1_MASK_SCOPE.count_ones() / 4) as u8) + 1,
                NeuronKind::Hidden2 => (chunk[6] % (c::NEURON_HIDDEN2_MASK_SCOPE.count_ones() / 4) as u8) + 1,
                NeuronKind::Output  => (chunk[6] % (c::NEURON_OUTPUT_MASK_SCOPE.count_ones()  / 4) as u8) + 1,
            };

            // let target_bit = chunk[7] % 64;
            let target_bit = match kind {
                NeuronKind::Hidden1 => chunk[7] % (c::NEURON_HIDDEN2_MASK_SCOPE.count_ones() + 1) as u8,
                NeuronKind::Hidden2 => chunk[7] % (c::NEURON_OUTPUT_MASK_SCOPE.count_ones()  + 1) as u8,
                // NeuronKind::Output  => chunk[7] % c::NEURON_OUTPUT_MASK_SCOPE.count_ones() as u8,
                NeuronKind::Output  => chunk[7] % 32,
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

}
