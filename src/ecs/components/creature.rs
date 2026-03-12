// use std::hash::{Hash, Hasher};

// /// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// /// Entities (Creatures) for the ECS
// /// -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------


// #[derive(Copy, Clone, Eq, PartialEq, Debug)]
// pub struct Creature {
//     /// The index/slot number. Gets recycled when entities are deleted.
//     pub id: u32,
//     /// Incremented each time this slot is reused. Prevents ABA problem.
//     pub generation: u32,
// }

// impl Creature {
//     #[inline]
//     pub fn new(id: u32, generation: u32) -> Self {
//         Creature { id, generation }
//     }
// }

// impl Hash for Creature {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         // Both fields must contribute to hash for HashMap usage
//         state.write_u32(self.id);
//         state.write_u32(self.generation);
//     }
// }