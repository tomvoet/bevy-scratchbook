use bevy::{prelude::*, utils::HashMap};

use super::Density;

#[derive(Clone, Copy, Debug)]
pub struct GridEntry {
    pub entity: Entity,
    pub position: Vec2,
    pub velocity: Vec2,
    pub density: Density,
}

/// Cell size equals the smoothing radius, so neighbours are in the 3x3 block.
#[derive(Resource, Default)]
pub struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<IVec2, Vec<GridEntry>>,
}

impl SpatialGrid {
    pub fn rebuild(&mut self, cell_size: f32, entries: impl Iterator<Item = GridEntry>) {
        self.cell_size = cell_size;
        for cell in self.cells.values_mut() {
            cell.clear();
        }
        for entry in entries {
            let coords = self.cell_coords(entry.position);
            self.cells.entry(coords).or_default().push(entry);
        }
    }

    fn cell_coords(&self, position: Vec2) -> IVec2 {
        (position / self.cell_size).floor().as_ivec2()
    }

    /// Includes the querying particle itself.
    pub fn for_each_neighbour(&self, position: Vec2, mut visit: impl FnMut(&GridEntry, f32)) {
        let center = self.cell_coords(position);
        let sqr_radius = self.cell_size * self.cell_size;

        for dy in -1..=1 {
            for dx in -1..=1 {
                let Some(cell) = self.cells.get(&(center + IVec2::new(dx, dy))) else {
                    continue;
                };
                for entry in cell {
                    let sqr_distance = entry.position.distance_squared(position);
                    if sqr_distance < sqr_radius {
                        visit(entry, sqr_distance.sqrt());
                    }
                }
            }
        }
    }
}
