use base64::{engine::general_purpose, Engine as _};
use rand::Rng;

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

        // we added chunks of 8 bytes to encode some output genes which trigger actions
        // as a scaffold for the evolution to find useful actions more easily, but this is optional and can be removed for a more free-form evolution
        for target_bit in 0..4 {
            bytes.extend_from_slice(&[
                rng.gen_range(0..=0xFF),  // mask byte 0
                rng.gen_range(0..=0xFF),  // mask byte 1
                rng.gen_range(0..=0xFF),  // mask byte 2
                rng.gen_range(0..=0xFF),  // mask byte 3
                rng.gen_range(0..=0xFF),  // mask byte 4
                4,                        // kind "Output"
                rng.gen_range(5..=12),    // threshold
                target_bit as u8,         // target_bit (used in `(chunk[7] % 32) + 1)`)
            ]);
        }

        // we've used 32 bytes for the scaffold genes, so we need to subtract that from the random part of the DNA
        let mut len: usize = 128 - 32;
        if length > len { len = length - 32; }

        let mut random_bytes = vec![0u8; len];
        rng.fill(&mut random_bytes[..]);
        bytes.extend_from_slice(&random_bytes);

        Self { bytes }
    }

    /******************************************************************************************************************************************/
    /// mutates the DNA by randomly flipping bits, inserting new genes, or deleting existing genes
    pub fn mutate(&mut self, rng: &mut impl Rng) {
        // some bit wise mutations
        for byte in &mut self.bytes {
            if rng.gen_bool(0.05) {
                let bit = 1 << rng.gen_range(0..8);
                *byte ^= bit;
            }
        }

        // Insert mutation (new gene)
        let max_genes = 64;
        if (self.bytes.len() / 8) < max_genes && rng.gen_bool(0.02) {
            let mut new_gene = [0u8; 8];
            rng.fill(&mut new_gene);
            self.bytes.extend_from_slice(&new_gene);
        }

        // Delete mutation (loose a gene)
        if self.bytes.len() >= 8 && rng.gen_bool(0.02) {
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