use bevy::prelude::*;

use super::{bounds::BOUNDS, Density};

#[derive(Clone, Copy, Debug)]
pub struct GridEntry {
    pub entity: Entity,
    pub position: Vec2,
    pub velocity: Vec2,
    pub density: Density,
}

impl GridEntry {
    const PLACEHOLDER: Self = Self {
        entity: Entity::PLACEHOLDER,
        position: Vec2::ZERO,
        velocity: Vec2::ZERO,
        density: Density {
            value: 0.0,
            near: 0.0,
        },
    };
}

/// Padded, because predicted positions can leave the tank.
const EXTENT: f32 = BOUNDS + 32.0;

/// Cell size equals the smoothing radius, so neighbours are in the 3x3 block.
/// Entries are counting-sorted by cell, so a row of three cells is one slice.
#[derive(Resource, Default)]
pub struct SpatialGrid {
    cell_size: f32,
    width: i32,
    entries: Vec<GridEntry>,
    /// `starts[i]..starts[i + 1]` is cell `i`. Length is `width * width + 1`.
    starts: Vec<u32>,
    cursor: Vec<u32>,
    scratch: Vec<GridEntry>,
    /// Slot each entry landed in, in the order `rebuild` was given them.
    order: Vec<u32>,
}

impl SpatialGrid {
    pub fn rebuild(&mut self, cell_size: f32, entries: impl Iterator<Item = GridEntry>) {
        self.cell_size = cell_size.max(0.01);
        self.width = ((2.0 * EXTENT / self.cell_size).ceil() as i32).max(1);
        let cells = (self.width as usize).pow(2);

        let mut scratch = std::mem::take(&mut self.scratch);
        scratch.clear();
        scratch.extend(entries);

        self.starts.clear();
        self.starts.resize(cells + 1, 0);
        for entry in &scratch {
            let cell = self.cell_index(entry.position);
            self.starts[cell + 1] += 1;
        }
        for i in 0..cells {
            self.starts[i + 1] += self.starts[i];
        }

        self.cursor.clear();
        self.cursor.extend_from_slice(&self.starts[..cells]);

        self.entries.clear();
        self.entries.resize(scratch.len(), GridEntry::PLACEHOLDER);
        self.order.clear();
        for entry in &scratch {
            let cell = self.cell_index(entry.position);
            let slot = self.cursor[cell];
            self.entries[slot as usize] = *entry;
            self.cursor[cell] += 1;
            self.order.push(slot);
        }

        self.scratch = scratch;
    }

    pub fn slots(&self) -> &[u32] {
        &self.order
    }

    /// Nothing moves between the two passes, so the sort still holds.
    pub fn set_density(&mut self, slot: u32, density: Density) {
        if let Some(entry) = self.entries.get_mut(slot as usize) {
            entry.density = density;
        }
    }

    /// Clamped, so a far particle lands in an edge cell. The radius check drops it.
    fn cell_coords(&self, position: Vec2) -> (i32, i32) {
        let half = self.width as f32 / 2.0;
        let cell = (position / self.cell_size + half).floor();
        (
            (cell.x as i32).clamp(0, self.width - 1),
            (cell.y as i32).clamp(0, self.width - 1),
        )
    }

    fn cell_index(&self, position: Vec2) -> usize {
        let (x, y) = self.cell_coords(position);
        (y * self.width + x) as usize
    }

    /// Includes the querying particle itself.
    pub fn for_each_neighbour(&self, position: Vec2, mut visit: impl FnMut(&GridEntry, f32)) {
        let (x, y) = self.cell_coords(position);
        let sqr_radius = self.cell_size * self.cell_size;
        let first = (x - 1).max(0);
        let last = (x + 1).min(self.width - 1);

        for row in (y - 1).max(0)..=(y + 1).min(self.width - 1) {
            let row = row * self.width;
            let begin = self.starts[(row + first) as usize] as usize;
            let end = self.starts[(row + last) as usize + 1] as usize;
            for entry in &self.entries[begin..end] {
                let sqr_distance = entry.position.distance_squared(position);
                if sqr_distance < sqr_radius {
                    visit(entry, sqr_distance.sqrt());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn points(n: usize) -> Vec<Vec2> {
        let mut seed = 0x2545_f491u32;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed >> 8) as f32 / (1 << 24) as f32
        };
        (0..n)
            .map(|_| Vec2::new(next() * 240.0 - 120.0, next() * 240.0 - 120.0))
            .collect()
    }

    #[test]
    fn matches_brute_force() {
        for h in [1.0f32, 3.5, 6.0, 20.0] {
            let positions = points(600);
            let mut grid = SpatialGrid::default();
            grid.rebuild(
                h,
                positions.iter().map(|&position| GridEntry {
                    position,
                    ..GridEntry::PLACEHOLDER
                }),
            );

            for &query in &positions {
                let mut found: Vec<Vec2> = Vec::new();
                grid.for_each_neighbour(query, |entry, distance| {
                    assert!((distance - entry.position.distance(query)).abs() < 1e-3);
                    found.push(entry.position);
                });
                let mut expected: Vec<Vec2> = positions
                    .iter()
                    .copied()
                    .filter(|p| p.distance_squared(query) < h * h)
                    .collect();

                found.sort_by(|a, b| a.to_array().partial_cmp(&b.to_array()).unwrap());
                expected.sort_by(|a, b| a.to_array().partial_cmp(&b.to_array()).unwrap());
                assert_eq!(found, expected, "radius {h}");
            }
        }
    }

    #[test]
    fn slots_point_at_the_right_entry() {
        let positions = points(300);
        let mut grid = SpatialGrid::default();
        grid.rebuild(
            6.0,
            positions.iter().map(|&position| GridEntry {
                position,
                ..GridEntry::PLACEHOLDER
            }),
        );

        let slots = grid.slots().to_vec();
        assert_eq!(slots.len(), positions.len());
        for (k, &slot) in slots.iter().enumerate() {
            grid.set_density(
                slot,
                Density {
                    value: k as f32,
                    near: 0.0,
                },
            );
        }
        for (k, &slot) in slots.iter().enumerate() {
            assert_eq!(grid.entries[slot as usize].position, positions[k]);
            assert_eq!(grid.entries[slot as usize].density.value, k as f32);
        }
    }

    #[test]
    fn rebuild_drops_old_entries() {
        let mut grid = SpatialGrid::default();
        let positions = points(200);
        let entry = |&position: &Vec2| GridEntry {
            position,
            ..GridEntry::PLACEHOLDER
        };

        grid.rebuild(6.0, positions.iter().map(entry));
        grid.rebuild(6.0, positions[..10].iter().map(entry));

        let mut count = 0;
        for &query in &positions[..10] {
            grid.for_each_neighbour(query, |_, _| count += 1);
        }
        assert!(count >= 10);
        assert_eq!(grid.entries.len(), 10);
    }
}
