

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
    pub target_bit: u8,
}


#[derive(Clone, Copy)]
pub struct ExecNeuron {
    pub mask: u64,
    pub threshold: u8,
    // #[allow(dead_code)]
    // pub kind: NeuronKind,
    pub target_bit: u8,
}
