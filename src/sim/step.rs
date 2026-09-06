use bevy::prelude::*;

use super::{
    bounds,
    grid::{GridEntry, SpatialGrid},
    kernels::{
        density_to_pressure, near_density_to_pressure, poly6, spiky_pow2, spiky_pow2_derivative,
        spiky_pow3, spiky_pow3_derivative,
    },
    CursorInteraction, Density, Obstacle, PredictedPosition, SimParams, Velocity,
};

const LOOKAHEAD: f32 = 1.0 / 120.0;
const MIN_DENSITY: f32 = 1e-4;

pub fn apply_external_forces(
    mut particles: Query<(&Transform, &mut Velocity, &mut PredictedPosition)>,
    params: Res<SimParams>,
    time: Res<Time>,
    cursor: Res<CursorInteraction>,
) {
    let dt = time.delta_seconds();
    let gravity = Vec2::new(0.0, -params.gravity);
    let interaction = cursor.0;
    let radius = params.interaction_radius;
    let air_drag = params.air_drag;

    particles
        .par_iter_mut()
        .for_each(|(transform, mut velocity, mut predicted)| {
            let position = transform.translation.truncate();
            let v = velocity.0;
            // Quadratic drag mainly brakes fast thin splashes.
            let mut acceleration = gravity - v * v.length() * air_drag;

            if let Some((point, strength)) = interaction {
                let offset = point - position;
                let distance = offset.length();
                if distance < radius {
                    let direction = if distance > 0.0 {
                        offset / distance
                    } else {
                        Vec2::ZERO
                    };
                    let falloff = 1.0 - distance / radius;
                    acceleration += (direction * strength - v) * falloff;
                }
            }

            velocity.0 += acceleration * dt;
            predicted.0 = position + velocity.0 * LOOKAHEAD;
        });
}

pub fn build_grid(
    mut grid: ResMut<SpatialGrid>,
    particles: Query<(Entity, &PredictedPosition, &Velocity, &Density)>,
    params: Res<SimParams>,
) {
    grid.rebuild(
        params.smoothing_radius,
        particles
            .iter()
            .map(|(entity, position, velocity, density)| GridEntry {
                entity,
                position: position.0,
                velocity: velocity.0,
                density: *density,
            }),
    );
}

pub fn calculate_densities(
    mut particles: Query<(&PredictedPosition, &mut Density)>,
    grid: Res<SpatialGrid>,
    params: Res<SimParams>,
) {
    let h = params.smoothing_radius;
    let mass = params.mass;

    particles
        .par_iter_mut()
        .for_each(|(position, mut density)| {
            let mut sum = Density::default();

            grid.for_each_neighbour(position.0, |_, distance| {
                sum.value += mass * spiky_pow2(distance, h);
                sum.near += mass * spiky_pow3(distance, h);
            });

            *density = sum;
        });
}

pub fn apply_pressure_forces(
    mut particles: Query<(Entity, &PredictedPosition, &Density, &mut Velocity)>,
    grid: Res<SpatialGrid>,
    params: Res<SimParams>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let h = params.smoothing_radius;
    let mass = params.mass;
    let (target, k, k_near, cohesion) = (
        params.target_density,
        params.pressure_multiplier,
        params.near_pressure_multiplier,
        params.cohesion,
    );

    particles
        .par_iter_mut()
        .for_each(|(entity, position, density, mut velocity)| {
            let position = position.0;
            let pressure = density_to_pressure(density.value, target, k, cohesion);
            let near_pressure = near_density_to_pressure(density.near, k_near);

            let mut force = Vec2::ZERO;

            grid.for_each_neighbour(position, |other, distance| {
                if other.entity == entity {
                    return;
                }

                let direction = if distance > 0.0 {
                    (other.position - position) / distance
                } else {
                    Vec2::from_angle(rand::random::<f32>() * std::f32::consts::TAU)
                };

                let shared_pressure = (pressure
                    + density_to_pressure(other.density.value, target, k, cohesion))
                    / 2.0;
                let shared_near_pressure =
                    (near_pressure + near_density_to_pressure(other.density.near, k_near)) / 2.0;

                // Derivatives are negative: positive pressure pushes away from `other`.
                force += direction * spiky_pow2_derivative(distance, h) * shared_pressure * mass
                    / other.density.value.max(MIN_DENSITY);
                force +=
                    direction * spiky_pow3_derivative(distance, h) * shared_near_pressure * mass
                        / other.density.near.max(MIN_DENSITY);
            });

            let acceleration = force / density.value.max(MIN_DENSITY);
            velocity.0 += acceleration * dt;
        });
}

pub fn apply_viscosity(
    mut particles: Query<(Entity, &PredictedPosition, &mut Velocity)>,
    grid: Res<SpatialGrid>,
    params: Res<SimParams>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let h = params.smoothing_radius;

    particles
        .par_iter_mut()
        .for_each(|(entity, position, mut velocity)| {
            let v = velocity.0;
            let mut force = Vec2::ZERO;

            grid.for_each_neighbour(position.0, |other, distance| {
                if other.entity != entity {
                    force += (other.velocity - v) * poly6(distance, h);
                }
            });

            velocity.0 += force * params.viscosity * dt;
        });
}

pub fn integrate(
    mut particles: Query<(&mut Transform, &mut Velocity)>,
    time: Res<Time>,
    params: Res<SimParams>,
) {
    let dt = time.delta_seconds();

    particles
        .par_iter_mut()
        .for_each(|(mut transform, mut velocity)| {
            let mut position = transform.translation.truncate() + velocity.0 * dt;
            bounds::resolve_collision(
                &mut position,
                &mut velocity.0,
                params.collision_damping,
                params.wall_friction,
                dt,
            );
            transform.translation = position.extend(0.0);
        });
}

pub fn collide_obstacles(
    obstacles: Query<(&Transform, &Obstacle)>,
    mut particles: Query<(&mut Transform, &mut Velocity), Without<Obstacle>>,
    params: Res<SimParams>,
    time: Res<Time>,
) {
    let circles: Vec<(Vec2, f32)> = obstacles
        .iter()
        .map(|(transform, obstacle)| (transform.translation.truncate(), obstacle.radius))
        .collect();
    if circles.is_empty() {
        return;
    }
    let dt = time.delta_seconds();

    particles
        .par_iter_mut()
        .for_each(|(mut transform, mut velocity)| {
            let mut position = transform.translation.truncate();
            for &(center, radius) in &circles {
                bounds::resolve_circle_collision(
                    &mut position,
                    &mut velocity.0,
                    center,
                    radius,
                    params.collision_damping,
                    params.wall_friction,
                    dt,
                );
            }
            transform.translation = position.extend(0.0);
        });
}
