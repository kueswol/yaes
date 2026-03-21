use crate::{constants as c, utils::Coordinate};

pub struct SpatialHashmap {
    cells: Vec<Vec<usize>>,
    cell_count_x: usize,
}

impl SpatialHashmap {
    pub fn new() -> Self {
        let cell_count_x = (c::WORLD_WIDTH / c::SPATIAL_HASHMAP_CELL_SIZE) as usize;
        let cells: Vec<Vec<usize>> = vec![
            Vec::new();
            ((c::WORLD_WIDTH * c::WORLD_HEIGHT)
                / (c::SPATIAL_HASHMAP_CELL_SIZE * c::SPATIAL_HASHMAP_CELL_SIZE))
                as usize
        ];
        SpatialHashmap { cells, cell_count_x }
    }

    pub fn insert(&mut self, creature_id: usize, position: Coordinate) {
        let index = self.get_cell_index(position);
        self.cells[index].push(creature_id);
    }
    // pub fn remove(&mut self, creature_id: usize, position: Coordinate) {
    //     let index = self.get_cell_index(position);
    //     if let Some(pos) = self.cells[index].iter().position(|&id| id == creature_id) {
    //         self.cells[index].remove(pos);
    //     }
    // }

    #[inline(always)]
    pub fn get_cell_index(&self, position: Coordinate) -> usize {
        let grid_width = (c::WORLD_WIDTH / c::SPATIAL_HASHMAP_CELL_SIZE) as usize;  // 20
        let grid_height = (c::WORLD_HEIGHT / c::SPATIAL_HASHMAP_CELL_SIZE) as usize;  // 20 (falls du es brauchst)

        let x_index = (position.x as usize / c::SPATIAL_HASHMAP_CELL_SIZE  as usize).min(grid_width - 1);
        let y_index = (position.y as usize / c::SPATIAL_HASHMAP_CELL_SIZE  as usize).min(grid_height - 1);

        (y_index * grid_width + x_index) as usize
    }   

    #[allow(dead_code)]
    pub fn get_creatures_in_cell(&self, position: Coordinate) -> Vec<usize> {
        let index = self.get_cell_index(position);
        self.cells[index].clone()
    }

    #[allow(dead_code)]
    pub fn get_creatures_in_cell_with_neighbors(&self, position: Coordinate) -> Vec<usize> {
        let mut return_vec: Vec<usize> = Vec::new();
        let mut indexes: Vec<usize> = Vec::with_capacity(9);

        // the current cell is always included
        indexes.push(self.get_cell_index(position));

        // the left ones, if we're far enough away from the left border
        if position.x >= c::SPATIAL_HASHMAP_CELL_SIZE as f32 {
            // top left
            if position.y >= c::SPATIAL_HASHMAP_CELL_SIZE as f32 { indexes.push(indexes[0] - self.cell_count_x - 1); }
            // left
            indexes.push(indexes[0] - 1);
            // bottom left
            if position.y <= (c::WORLD_HEIGHT - c::SPATIAL_HASHMAP_CELL_SIZE) as f32 { indexes.push(indexes[0] + self.cell_count_x - 1); }
        }
        // the right ones, if we're far enough away from the right border
        if position.x <= (c::WORLD_WIDTH - c::SPATIAL_HASHMAP_CELL_SIZE) as f32 {
            // top right
            if position.y >= c::SPATIAL_HASHMAP_CELL_SIZE as f32 { indexes.push(indexes[0] - self.cell_count_x + 1); }
            // right
            indexes.push(indexes[0] + 1);
            // bottom right
            if position.y <= (c::WORLD_HEIGHT - c::SPATIAL_HASHMAP_CELL_SIZE) as f32 {indexes.push(indexes[0] + self.cell_count_x + 1); }
        }
        // the top one, if we're far enough away from the top border
        if position.y >= c::SPATIAL_HASHMAP_CELL_SIZE as f32 {indexes.push(indexes[0] - self.cell_count_x); }
        // the bottom one, if we're far enough away from the bottom border
        if position.y <= (c::WORLD_HEIGHT - c::SPATIAL_HASHMAP_CELL_SIZE) as f32 { indexes.push(indexes[0] + self.cell_count_x); }
        
        indexes.sort_unstable();
        indexes.dedup();
        for index in indexes {
            return_vec.extend(&self.cells[index]);
        }
        
        return_vec.sort_unstable();
        return_vec.dedup();
        return_vec
    }

}

#[test]
fn test_spatial_hashmap() {
    let mut hashmap = SpatialHashmap::new();
    hashmap.insert(0, Coordinate { x:   0.0_f32, y:   0.0_f32 }); // should be cell index 0
    hashmap.insert(1, Coordinate { x:   9.9_f32, y:   9.9_f32 }); // should be cell index 0
    hashmap.insert(2, Coordinate { x:   0.0_f32, y:  11.0_f32 }); // should be cell index 20
    hashmap.insert(3, Coordinate { x:  11.0_f32, y:   0.0_f32 }); // should be cell index 1
    hashmap.insert(4, Coordinate { x:  11.0_f32, y:  11.0_f32 }); // should be cell index 21
    hashmap.insert(5, Coordinate { x:  22.0_f32, y:   0.0_f32 }); // should be cell index 2
    hashmap.insert(6, Coordinate { x:  33.0_f32, y:   0.0_f32 }); // should be cell index 3
    hashmap.insert(7, Coordinate { x:  44.0_f32, y:   0.0_f32 }); // should be cell index 4
    hashmap.insert(8, Coordinate { x: 199.9_f32, y: 199.9_f32 }); // should be cell index 399
    
    // assert_eq!(hashmap.get_creatures_in_cell               (Coordinate { x:   5.5_f32, y:   5.5_f32 }).len(), 2); 
    // assert_eq!(hashmap.get_creatures_in_cell               (Coordinate { x:   5.5_f32, y:   5.5_f32 }).len(), 2); 
    assert_eq!(hashmap.get_creatures_in_cell_with_neighbors(Coordinate { x:   5.5_f32, y:   5.5_f32 }).len(), 5); 
    // assert_eq!(hashmap.get_creatures_in_cell               (Coordinate { x:  15.5_f32, y:   5.5_f32 }).len(), 1); 
    assert_eq!(hashmap.get_creatures_in_cell_with_neighbors(Coordinate { x:  15.5_f32, y:   5.5_f32 }).len(), 6); 
    assert_eq!(hashmap.get_creatures_in_cell_with_neighbors(Coordinate { x: 195.5_f32, y: 195.5_f32 }).len(), 1); 

    // println!("{:?}", hashmap.get_creatures_in_cell_with_neighbors(Coordinate { x: 5.0, y: 5.0 }));
} 