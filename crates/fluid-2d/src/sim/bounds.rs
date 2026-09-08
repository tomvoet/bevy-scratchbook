use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::SPACING;

pub const BOUNDS: f32 = 100.0;

/// Frozen particles filling the solid side of every surface. Spacing matches
/// the resting fluid, so a surface is just fluid that doesn't move.
#[derive(Resource, Default)]
pub struct Boundary {
    positions: Vec<Vec2>,
    pillars: Vec<(Vec2, Vec2)>,
    spacing: f32,
    thickness: f32,
    layers: i32,
}

impl Boundary {
    pub fn positions(&self) -> &[Vec2] {
        &self.positions
    }

    pub fn pillars(&self) -> &[(Vec2, Vec2)] {
        &self.pillars
    }

    /// Rings rather than a lattice, so the pillar surface stays smooth.
    pub fn fill_pillars(&mut self, pillars: impl Iterator<Item = (Vec2, f32, Vec2)>) {
        self.pillars.clear();
        for (center, radius, velocity) in pillars {
            for layer in 0..self.layers {
                let ring = radius - (layer as f32 + 0.5) * self.spacing;
                if ring <= 0.0 {
                    break;
                }
                let count = (TAU * ring / self.spacing).round().max(3.0);
                let offset = layer as f32 * PI / count;
                for i in 0..count as i32 {
                    let angle = offset + TAU * i as f32 / count;
                    self.pillars
                        .push((center + Vec2::from_angle(angle) * ring, velocity));
                }
            }
        }
    }

    /// Rebuilds only when a slider it depends on moves.
    pub fn sync(&mut self, mass: f32, target_density: f32, smoothing_radius: f32) {
        // Lower bound keeps the particle count sane at extreme slider settings.
        let spacing = (mass / target_density.max(1e-4))
            .sqrt()
            .clamp(SPACING / 2.0, SPACING * 4.0);
        if spacing == self.spacing && smoothing_radius == self.thickness {
            return;
        }

        self.spacing = spacing;
        self.thickness = smoothing_radius;
        self.positions.clear();

        self.layers = (smoothing_radius / spacing).ceil().max(1.0) as i32;
        let outer = BOUNDS + self.layers as f32 * spacing;
        let along = |half: f32| {
            let n = (half / spacing).ceil() as i32;
            (-n..n).map(move |i| (i as f32 + 0.5) * spacing)
        };

        for layer in 0..self.layers {
            let out = BOUNDS + (layer as f32 + 0.5) * spacing;
            // Sides run the full height, so they own the corners outright.
            for y in along(outer) {
                self.positions.push(Vec2::new(-out, y));
                self.positions.push(Vec2::new(out, y));
            }
            for x in along(BOUNDS) {
                self.positions.push(Vec2::new(x, -out));
                self.positions.push(Vec2::new(x, out));
            }
        }
    }
}

/// Backstop for a pillar, resolved in the frame of the circle so a dragged one
/// still pushes. No bounce, so nothing parks on it.
pub fn resolve_circle_collision(
    position: &mut Vec2,
    velocity: &mut Vec2,
    center: Vec2,
    radius: f32,
    surface_velocity: Vec2,
) {
    let offset = *position - center;
    let distance = offset.length();
    let limit = radius;
    if distance >= limit {
        return;
    }

    let normal = if distance > 0.0 {
        offset / distance
    } else {
        Vec2::Y
    };
    *position = center + normal * limit;

    let into = (*velocity - surface_velocity).dot(normal);
    if into < 0.0 {
        *velocity -= normal * into;
    }
}

/// Backstop for anything fast enough to punch through the boundary particles.
/// Sits on the wall itself, so a resting particle never reaches it.
pub fn resolve_collision(position: &mut Vec2, velocity: &mut Vec2) {
    let limit = BOUNDS;

    if position.x > limit {
        position.x = limit;
        velocity.x = velocity.x.min(0.0);
    } else if position.x < -limit {
        position.x = -limit;
        velocity.x = velocity.x.max(0.0);
    }

    if position.y > limit {
        position.y = limit;
        velocity.y = velocity.y.min(0.0);
    } else if position.y < -limit {
        position.y = -limit;
        velocity.y = velocity.y.max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::kernels::{spiky_pow2, MASS, SMOOTHING_RADIUS, TARGET_DENSITY};

    fn density_at(point: Vec2, fluid: &[Vec2], boundary: &Boundary) -> f32 {
        fluid
            .iter()
            .chain(boundary.positions())
            .map(|p| MASS * spiky_pow2(p.distance(point), SMOOTHING_RADIUS))
            .sum()
    }

    #[test]
    fn band_stays_inside_the_grid() {
        for &(mass, target, h) in &[
            (0.1f32, 0.0f32, 1.0f32),
            (10.0, 1.0, 20.0),
            (0.1, 1.0, 20.0),
            (10.0, 0.01, 1.0),
            (MASS, TARGET_DENSITY, SMOOTHING_RADIUS),
        ] {
            let mut boundary = Boundary::default();
            boundary.sync(mass, target, h);
            let far = boundary
                .positions()
                .iter()
                .map(|p| p.abs().max_element())
                .fold(0.0f32, f32::max);
            assert!(
                far < BOUNDS + 32.0,
                "mass {mass} target {target} h {h}: {far}"
            );
        }
    }

    #[test]
    fn density_stays_flat_up_to_the_wall() {
        let mut boundary = Boundary::default();
        boundary.sync(MASS, TARGET_DENSITY, SMOOTHING_RADIUS);

        let n = (BOUNDS / SPACING) as i32;
        let coords: Vec<f32> = (-n..n).map(|i| (i as f32 + 0.5) * SPACING).collect();
        let fluid: Vec<Vec2> = coords
            .iter()
            .flat_map(|&x| coords.iter().map(move |&y| Vec2::new(x, y)))
            .collect();

        let bulk = density_at(Vec2::ZERO, &fluid, &boundary);
        assert!(
            (bulk - TARGET_DENSITY).abs() / TARGET_DENSITY < 0.05,
            "{bulk}"
        );

        let edge = BOUNDS - SPACING / 2.0;
        for probe in [Vec2::new(edge, 0.0), Vec2::new(edge, edge)] {
            let density = density_at(probe, &fluid, &boundary);
            assert!(
                (density - bulk).abs() / bulk < 0.05,
                "{probe} reads {density}, bulk is {bulk}"
            );
        }
    }
}
