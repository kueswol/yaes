use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use crate::constants as c;


/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// DNA
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct Dna {
    pub bytes: Vec<u8>,
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// public methods for DNA
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

impl Dna {
    /******************************************************************************************************************************************/
    /// creates a random DNA sequence of the given length (in bytes)
    pub fn random(length: usize, rng: &mut impl Rng) -> Self {
        let mut bytes = Vec::new();

    
        // byte structure for a single neuron (8 bytes):
        // byte 1: mask for input bits 1-8
        // byte 2: mask for input bits 9-16
        // byte 3: mask for input bits 17-24
        // byte 4: mask for input bits 25-32
        // byte 5: mask for input bits 33-40
        // byte 6: kind "Output"
        // byte 7: threshold
        // byte 8: target_bit (used in `(chunk[7] % 32) + 1)`)
        
        let type_hi1: u8 = 0;
        // let type_hi2: u8 = 2;
        let type_out: u8 = 4;

        // scaffold neuron, hidden1, reads "can_eat", "energy_low" & "energy_medium" and outputs to bit 64
        //                  output, forwards hidden1's bit 64 to action bit 3 (eat)
        bytes.extend_from_slice(&[ 0b00000011, 0b00000000, 0b00000100, 0b00000000, 0b00000000, type_hi1, 2_u8, 64_u8]);
        bytes.extend_from_slice(&[ 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b10000000, type_out, 1_u8,  2_u8]);
        
        // scaffold neuron, hidden1, reads "can_reproduce" & "energy_high" and outputs to bit 63
        //                  output, forwards hidden1's bit 63 to action bit 1 (reproduce)
        bytes.extend_from_slice(&[ 0b00000100, 0b01000000, 0b00000000, 0b00000000, 0b00000000, type_hi1, 2_u8, 63_u8]);
        bytes.extend_from_slice(&[ 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b01000000, type_out, 1_u8,  0_u8]);
        
        // scaffold neuron, hidden1, reads "energy_low" and outputs to bit 62
        //                  output, forwards hidden1's bit 62 to action bit 4 (move)
        bytes.extend_from_slice(&[ 0b00000001, 0b00000000, 0b00000000, 0b00000000, 0b00000000, type_hi1, 1_u8, 62_u8]);
        bytes.extend_from_slice(&[ 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00100000, type_out, 1_u8,  3_u8]);
        
        // additional output neurons triggering move and turning - with random mask and threshold
        bytes.extend_from_slice(&[rng.gen_range(0..=0xFF), rng.gen_range(0..=0xFF), rng.gen_range(0..=0xFF),
                                  rng.gen_range(0..=0xFF), rng.gen_range(0..=0xFF),
                                  type_out, rng.gen_range(5..=12), 3_u8]);
        bytes.extend_from_slice(&[rng.gen_range(0..=0xFF), rng.gen_range(0..=0xFF), rng.gen_range(0..=0xFF),
                                  rng.gen_range(0..=0xFF), rng.gen_range(0..=0xFF),
                                  type_out, rng.gen_range(4..=10), rng.gen_range(8_u8..=9_u8)]); // target_bit 9 oder 10
        
        // we'll use 64 bytes for the scaffold genes, so we need to subtract that from the random part of the DNA
        // a length of 64 bytes (=8 Neurons) is assumed as the bare minimum - therefor we make sure, we're having at least 128 bytes
        let mut len: usize = 128 - 64;
        if length > len { len = length - 64; }
        
        let mut random_bytes = vec![0u8; len];
        rng.fill(&mut random_bytes[..]);
        bytes.extend_from_slice(&random_bytes);

        Self { bytes }
    }

    /******************************************************************************************************************************************/
    /// mutates the DNA by randomly flipping bits, inserting new genes, or deleting existing genes
    pub fn mutate(&mut self, rng: &mut impl Rng) {
        // // some bit wise mutations
        // for byte in &mut self.bytes {
        //     if rng.gen_bool(0.05) {
        //         let bit = 1 << rng.gen_range(0..8);
        //         *byte ^= bit;
        //     }
        // }

        for chunk in &mut self.bytes.chunks_exact_mut(8).skip(8) {
            // chunks 0-4 are used for the mask
            if rng.gen_bool(c::MUTATE_CHANCE_BIT_FLIP_MASK) {
                let byte_index = rng.gen_range(0..=4);
                let bit = 1 << rng.gen_range(0..8);
                chunk[byte_index] ^= bit;
            }
            // chunk 5 is the type - we don't touch it for now

            // chunk 6 is the threshold - we can mutate it a bit by raiing or lowering it by 1
            if rng.gen_bool(c::MUTATE_CHANCE_CHANGE_THRESHOLD) {
                if rng.gen_bool(0.5) { chunk[6] = chunk[6].saturating_add(1); }
                else                 { chunk[6] = chunk[6].saturating_sub(1); }
            }
            
            // chunk 7 (last one) is the target bit
            if rng.gen_bool(c::MUTATE_CHANCE_CHANGE_TARGET_BIT) {
                if rng.gen_bool(0.5) { chunk[7] = chunk[7].saturating_add(1); }
                else                 { chunk[7] = chunk[7].saturating_sub(1); }
            }
        }
        // Insert mutation (new gene)
        let max_neurons = 50;
        let min_neurons = 8;

        if self.bytes.len() < (max_neurons * 8) && rng.gen_bool(c::MUTATE_CHANCE_GAINING_NEW_NEURON) {
            let mut new_gene = [0u8; 8];
            rng.fill(&mut new_gene);
            self.bytes.extend_from_slice(&new_gene);
        }

        // Delete mutation (loose a gene)
        if self.bytes.len() > (min_neurons * 8) && rng.gen_bool(c::MUTATE_CHANCE_LOOSING_NEW_NEURON) {
            let gene_index = rng.gen_range(0..(self.bytes.len() / 8));
            let start = gene_index * 8;
            self.bytes.drain(start..start + 8);
        }
    }

    /******************************************************************************************************************************************/
    /// base64 encode the DNA for a more compact representation
    pub fn to_compact_string(&self) -> String {
        general_purpose::STANDARD_NO_PAD.encode(&self.bytes)
    }

    /******************************************************************************************************************************************/
    /// define the DNA from a base64 encoded string
    pub fn from_compact_string(s: &str) -> Self {
        let bytes = general_purpose::STANDARD_NO_PAD
            .decode(s)
            .expect("Invalid DNA string");

        Self { bytes }
    }
}

/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// traits for DNA
/// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

impl ToString for Dna {
    fn to_string(&self) -> String {
        let mut result = String::new();

        for (i, chunk) in self.bytes.chunks(32).enumerate() {
            result.push_str(&format!("G{:03}: ", i));

            for byte in chunk {
                result.push_str(&format!("{:02X}", byte));
            }

            if chunk.len() == 32 {
                result.push('\n');
            } else {
                result.push_str("(incomplete)\n");
            }
        }

        result
    }
}