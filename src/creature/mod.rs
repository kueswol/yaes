mod creature;
mod brain;
mod dna;
mod neurons;
mod genome;

// publicly we only need the struct Creature, not the whole module
pub use creature::Creature;