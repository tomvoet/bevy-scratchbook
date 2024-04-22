use std::collections::HashSet;

use bevy::{math::I64Vec3, prelude::*, utils::HashMap};

use crate::particle::Particle;

// 3x3 grid of neighbours 2D
const NEIGHBOURS: [I64Vec3; 9] = [
    I64Vec3::new(-1, -1, 0),
    I64Vec3::new(0, -1, 0),
    I64Vec3::new(1, -1, 0),
    I64Vec3::new(-1, 0, 0),
    I64Vec3::new(0, 0, 0),
    I64Vec3::new(1, 0, 0),
    I64Vec3::new(-1, 1, 0),
    I64Vec3::new(0, 1, 0),
    I64Vec3::new(1, 1, 0),
];

#[derive(Event)]
pub struct GeneratedHashCell(pub HashCell);

pub struct HashCell {
    map: HashMap<I64Vec3, Vec<(Entity, Particle)>>,
    cell_size: f32,
}

impl HashCell {
    pub fn new(cell_size: f32, particles: Query<(Entity, &Particle)>) -> Self {
        let mut map = HashMap::default();

        //for particle in particles.iter() {
        //    let cell_coords = Self::position_to_cell_coords(particle.position, cell_size);
        //    map.entry(cell_coords)
        //        .or_insert_with(Vec::new)
        //        .push(particle.clone());
        //}
        //
        //Self { map, cell_size }

        for (entity, particle) in particles.iter() {
            let cell_coords = Self::position_to_cell_coords(particle.position, cell_size);
            map.entry(cell_coords)
                .or_insert_with(Vec::new)
                .push((entity, particle.clone()));
        }

        Self { map, cell_size }
    }

    pub fn position_to_cell_coords(position: Vec3, cell_size: f32) -> I64Vec3 {
        let x = (position.x / cell_size) as i64;
        let y = (position.y / cell_size) as i64;

        I64Vec3::new(x, y, 0)
    }

    pub fn find_neighbor_particles(&self, position: Vec3) -> Vec<&Particle> {
        let center_coords = Self::position_to_cell_coords(position, self.cell_size);
        let sqr_radius = self.cell_size.powi(2);

        let mut neighbors = HashSet::new();

        for neighbour in NEIGHBOURS.iter() {
            let cell = center_coords + *neighbour;

            if let Some(particles) = self.map.get(&cell) {
                for particle in particles {
                    let distance = particle.1.position.distance_squared(position);

                    if distance < sqr_radius {
                        neighbors.insert(particle);
                    }
                }
            }
        }

        neighbors.into_iter().map(|p| &p.1).collect()
    }

    pub fn find_neighbor_entities(&self, position: Vec3) -> Vec<Entity> {
        let center_coords = Self::position_to_cell_coords(position, self.cell_size);
        let sqr_radius = self.cell_size.powi(2);

        let mut neighbors = HashSet::new();

        for neighbour in NEIGHBOURS.iter() {
            let cell = center_coords + *neighbour;

            if let Some(particles) = self.map.get(&cell) {
                for particle in particles {
                    let distance = particle.1.position.distance_squared(position);

                    if distance < sqr_radius {
                        neighbors.insert(particle.0);
                    }
                }
            }
        }

        neighbors.into_iter().collect()
    }

    pub fn find_neighbors(&self, position: Vec3) -> Vec<(Entity, Particle)> {
        let center_coords = Self::position_to_cell_coords(position, self.cell_size);
        let sqr_radius = self.cell_size.powi(2);

        let mut neighbors = HashSet::new();

        for neighbour in NEIGHBOURS.iter() {
            let cell = center_coords + *neighbour;

            if let Some(particles) = self.map.get(&cell) {
                for particle in particles {
                    let distance = particle.1.position.distance_squared(position);

                    if distance < sqr_radius {
                        neighbors.insert(particle.clone());
                    }
                }
            }
        }

        neighbors.into_iter().collect()
    }
}

//#[derive(Debug)]
//pub struct HashCell {
//    //particles: Box<[Particle]>,
//    entries: Box<[HashCellEntry]>,
//    start_indexes: Box<[u16]>,
//    // assume square for now
//    bounds: f32,
//    cell_size: f32,
//}

//#[derive(Debug)]
//pub struct HashCellEntry {
//    pub particle: Particle,
//    pub cell_key: u16,
//}
//
//impl HashCell {
//    fn position_to_cell_coords(position: Vec3, cell_size: f32) -> I64Vec3 {
//        let x = (position.x / cell_size) as i64;
//        let y = (position.y / cell_size) as i64;
//
//        I64Vec3::new(x, y, 0)
//    }
//
//    // bad typecasting prevents overflow (-1 zweierkomplement ca u32::MAX)
//    fn hash_cell(cell: I64Vec3) -> u64 {
//        let a = (cell.x * 15823) as u32;
//        let b = (cell.y * 9737333) as u32;
//
//        a as u64 + b as u64
//    }
//
//    fn get_key_from_hash(hash: u32, length: u32) -> u32 {
//        hash % length
//    }
//
//    pub fn new(bounds: f32, cell_size: f32, particles: Query<&Particle>) -> Self {
//        let particles = particles.iter().cloned().collect::<Vec<Particle>>();
//
//        let len = particles.len();
//
//        let mut cells = Vec::with_capacity(len);
//
//        for particle in particles.into_iter() {
//            let cell = Self::position_to_cell_coords(particle.position, cell_size);
//            let hash = Self::hash_cell(cell);
//            let key = Self::get_key_from_hash(hash as u32, len as u32);
//
//            cells.push(HashCellEntry {
//                particle,
//                cell_key: key as u16,
//            });
//        }
//
//        cells.sort_by(|a, b| a.cell_key.cmp(&b.cell_key));
//
//        let mut start_indexes = Vec::with_capacity(len);
//
//        // start index = u16::MAX if cell key does not occur, else index of first particle in cell
//        let mut last_key = u16::MAX;
//        for (i, entry) in cells.iter().enumerate() {
//            if entry.cell_key != last_key {
//                start_indexes.push(i as u16);
//                last_key = entry.cell_key;
//            } else {
//                start_indexes.push(u16::MAX);
//                last_key = u16::MAX;
//            }
//        }
//
//        Self {
//            entries: cells.into_boxed_slice(),
//            start_indexes: start_indexes.into_boxed_slice(),
//            bounds,
//            cell_size,
//        }
//    }
//
//    pub fn find_neighbors(&self, position: Vec3, print: bool) -> Vec<&Particle> {
//        let center_coords = Self::position_to_cell_coords(position, self.cell_size);
//        let sqr_radius = self.cell_size.powi(2);
//
//        let mut neighbors = HashSet::new();
//
//        let cell_count = self.entries.len() as u16;
//
//        let print = if print {
//            println!("Position: {:?}", position);
//            println!("Center Coords: {:?}", center_coords);
//            println!("Start Indexes: {:?}", self.start_indexes);
//            for entry in self.entries.iter() {
//                print!("{:?} {:?},", entry.particle.position, entry.cell_key);
//            }
//            true
//        } else {
//            false
//        };
//
//        for neighbour in NEIGHBOURS.iter() {
//            let cell = center_coords + *neighbour;
//            let hash = Self::hash_cell(cell);
//            let key = Self::get_key_from_hash(hash as u32, cell_count as u32);
//            let cell_start = self.start_indexes[key as usize];
//
//            if print {
//                println!(
//                    "Cell: {:?}, Hash: {}, Key: {}, Start: {}",
//                    cell, hash, key, cell_start
//                );
//            }
//
//            for i in cell_start..cell_count {
//                // if we are in a different cell, break
//                if key != self.entries[i as usize].cell_key as u32 {
//                    if print {
//                        println!("Breaking at {}", i);
//                    }
//                    break;
//                }
//
//                let particle = &self.entries[i as usize].particle;
//                let distance = particle.position.distance_squared(position);
//
//                if print {
//                    println!(
//                        "Distance: {}, Position: {:?}, Particle: {:?}",
//                        distance, position, particle.position
//                    );
//                }
//
//                if distance < sqr_radius {
//                    neighbors.insert(particle);
//                }
//            }
//        }
//
//        neighbors.into_iter().collect()
//    }
//
//    pub fn get_all_entries(&self) -> Vec<&HashCellEntry> {
//        self.entries.iter().collect()
//    }
//
//    pub fn get_length(&self) -> usize {
//        self.entries.len()
//    }
//}
