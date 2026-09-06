use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;

use crate::render::MainCamera;

use super::{
    bounds,
    grid::{GridEntry, SpatialGrid},
    kernels::{
        density_to_pressure, near_density_to_pressure, poly6, spiky_pow2, spiky_pow2_derivative,
        spiky_pow3, spiky_pow3_derivative,
    },
    Particle, SimParams,
};

const LOOKAHEAD: f32 = 1.0 / 120.0;
const MIN_DENSITY: f32 = 1e-4;

pub fn apply_external_forces(
    mut particles: Query<&mut Particle>,
    params: Res<SimParams>,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut egui: EguiContexts,
) {
    let dt = time.delta_seconds();
    let gravity = Vec2::new(0.0, -params.gravity);

    let interaction = if egui.ctx_mut().is_pointer_over_area() {
        None
    } else {
        let strength = if mouse.pressed(MouseButton::Left) {
            params.interaction_strength
        } else if mouse.pressed(MouseButton::Right) {
            -params.interaction_strength
        } else {
            0.0
        };
        let (camera, camera_transform) = cameras.single();
        windows
            .single()
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
            .filter(|_| strength != 0.0)
            .map(|point| (point, strength))
    };
    let radius = params.interaction_radius;
    let air_drag = params.air_drag;

    particles.par_iter_mut().for_each(|mut p| {
        // Quadratic drag mainly brakes fast thin splashes.
        let mut acceleration = gravity - p.velocity * p.velocity.length() * air_drag;

        if let Some((point, strength)) = interaction {
            let offset = point - p.position;
            let distance = offset.length();
            if distance < radius {
                let direction = if distance > 0.0 { offset / distance } else { Vec2::ZERO };
                let falloff = 1.0 - distance / radius;
                acceleration += (direction * strength - p.velocity) * falloff;
            }
        }

        p.velocity += acceleration * dt;
        p.predicted_position = p.position + p.velocity * LOOKAHEAD;
    });
}

pub fn build_grid(
    mut grid: ResMut<SpatialGrid>,
    particles: Query<(Entity, &Particle)>,
    params: Res<SimParams>,
) {
    grid.rebuild(
        params.smoothing_radius,
        particles.iter().map(|(entity, p)| GridEntry {
            entity,
            position: p.predicted_position,
            velocity: p.velocity,
            density: p.density,
            near_density: p.near_density,
        }),
    );
}

pub fn calculate_densities(
    mut particles: Query<&mut Particle>,
    grid: Res<SpatialGrid>,
    params: Res<SimParams>,
) {
    let h = params.smoothing_radius;
    let mass = params.mass;

    particles.par_iter_mut().for_each(|mut p| {
        let mut density = 0.0;
        let mut near_density = 0.0;

        grid.for_each_neighbour(p.predicted_position, |_, distance| {
            density += mass * spiky_pow2(distance, h);
            near_density += mass * spiky_pow3(distance, h);
        });

        p.density = density;
        p.near_density = near_density;
    });
}

pub fn apply_pressure_forces(
    mut particles: Query<(Entity, &mut Particle)>,
    grid: Res<SpatialGrid>,
    params: Res<SimParams>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let h = params.smoothing_radius;
    let mass = params.mass;
    let (target, k, k_near) = (
        params.target_density,
        params.pressure_multiplier,
        params.near_pressure_multiplier,
    );

    particles.par_iter_mut().for_each(|(entity, mut p)| {
        let position = p.predicted_position;
        let pressure = density_to_pressure(p.density, target, k);
        let near_pressure = near_density_to_pressure(p.near_density, k_near);

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

            let shared_pressure = (pressure + density_to_pressure(other.density, target, k)) / 2.0;
            let shared_near_pressure =
                (near_pressure + near_density_to_pressure(other.near_density, k_near)) / 2.0;

            // Derivatives are negative: positive pressure pushes away from `other`.
            force += direction * spiky_pow2_derivative(distance, h) * shared_pressure * mass
                / other.density.max(MIN_DENSITY);
            force += direction * spiky_pow3_derivative(distance, h) * shared_near_pressure * mass
                / other.near_density.max(MIN_DENSITY);
        });

        let acceleration = force / p.density.max(MIN_DENSITY);
        p.velocity += acceleration * dt;
    });
}

pub fn apply_viscosity(
    mut particles: Query<(Entity, &mut Particle)>,
    grid: Res<SpatialGrid>,
    params: Res<SimParams>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let h = params.smoothing_radius;

    particles.par_iter_mut().for_each(|(entity, mut p)| {
        let position = p.predicted_position;
        let velocity = p.velocity;
        let mut force = Vec2::ZERO;

        grid.for_each_neighbour(position, |other, distance| {
            if other.entity != entity {
                force += (other.velocity - velocity) * poly6(distance, h);
            }
        });

        p.velocity += force * params.viscosity * dt;
    });
}

pub fn integrate(
    mut particles: Query<(&mut Particle, &mut Transform)>,
    time: Res<Time>,
    params: Res<SimParams>,
) {
    let dt = time.delta_seconds();

    particles.par_iter_mut().for_each(|(mut p, mut transform)| {
        let step = p.velocity * dt;
        p.position += step;
        bounds::resolve_collision(&mut p, params.collision_damping, params.wall_friction, dt);
        transform.translation = p.position.extend(0.0);
    });
}
