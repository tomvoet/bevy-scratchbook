use std::ops::Range;

use bevy::{prelude::*, window::PrimaryWindow};
use rand::Rng;

use crate::{hashcell::HashCell, systems::bounds::BOUNDS, ui::UiState};

use super::Particle;

pub const STEP: usize = 2;
const PADDING: i16 = 60;
const BALL_ITERATOR: Range<i16> = (-(BOUNDS as i16) + PADDING)..((BOUNDS as i16) - PADDING);
const SPHERE_COLOR: Color = Color::rgb(0.0, 0.0, 1.0);

pub fn spawn_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Particle::mesh());
    let material = materials.add(SPHERE_COLOR);

    // each particle should move outwards from the center, faster particles should be further out

    let center = Vec3::ZERO;
    let max_distance = BALL_ITERATOR.end as f32;

    let rng = &mut rand::thread_rng();

    for x in BALL_ITERATOR.step_by(STEP) {
        for y in BALL_ITERATOR.step_by(STEP) {
            let pos = Vec3::new(x as f32, y as f32, 0.0);
            // if a particle is further out, it should move faster
            let velocity = (pos - center).normalize() * pos.length() / max_distance;
            // add some randomness to the velocity
            let velocity = Vec3::new(
                velocity.x + rng.gen_range(-0.3..0.3),
                velocity.y + rng.gen_range(-0.3..0.3),
                0.0,
            );

            // but reduce vertical velocity
            let velocity = Vec3::new(velocity.x, velocity.y * 0.5, velocity.z);
            let particle = Particle::new(pos, velocity);

            commands.spawn((
                PbrBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    transform: Transform::from_translation(pos),
                    ..Default::default()
                },
                particle,
            ));
        }
    }
}

pub fn apply_color(
    mut commands: Commands,
    mut particles: Query<(Option<Entity>, &Particle)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, particle) in particles.iter_mut() {
        if let Some(entity) = entity {
            commands
                .entity(entity)
                .insert(materials.add(particle.color()));
        }
    }
}

pub fn generate_hashcells(
    particles: Query<(Entity, &Particle)>,
    mut gen_hashcells: EventWriter<crate::hashcell::GeneratedHashCell>,
    ui_state: Res<UiState>,
) {
    // entity, weil constant look up https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Query.html oder particle??

    // replace with ui_state bounds, also use smoothing radius as cell size

    let cells = HashCell::new(ui_state.smoothing_radius, particles);

    gen_hashcells.send(crate::hashcell::GeneratedHashCell(cells));
}

pub fn calculate_densities(
    mut particles: Query<&mut Particle>,
    mut gen_hashcells: EventReader<crate::hashcell::GeneratedHashCell>,
    ui_state: Res<crate::ui::UiState>,
) {
    let hashcell = &gen_hashcells.read().next().unwrap().0;

    let smoothing_radius = ui_state.smoothing_radius;

    particles.par_iter_mut().for_each(move |mut particle| {
        particle.predicted_position = particle.position + particle.velocity * crate::TIMESTEP;

        let neighbours = hashcell.find_neighbor_particles(particle.predicted_position);

        particle.density = particle.calculate_density(neighbours.as_slice(), smoothing_radius);
    });
}

pub fn apply_pressure_force(
    mut particles: Query<&mut Particle>,
    mut gen_hashcells: EventReader<crate::hashcell::GeneratedHashCell>,
    ui_state: Res<crate::ui::UiState>,
) {
    let hashcell = &gen_hashcells.read().next().unwrap().0;

    particles.par_iter_mut().for_each(|mut particle| {
        let neighbours = hashcell.find_neighbor_particles(particle.position);

        let pressure_force = particle.calculate_pressure_force(
            &neighbours,
            ui_state.smoothing_radius,
            ui_state.target_density,
            ui_state.pressure_multiplier,
        );
        let pressure_acceleration = pressure_force / particle.density.max(0.0001);

        particle.apply_force(pressure_acceleration * crate::TIMESTEP);
    });
}

const UPPER_RIGHT_CORNER_WINDOW: Vec2 = Vec2::new(926.0, 72.0);
const LOWER_LEFT_CORNER_WINDOW: Vec2 = Vec2::new(352.0, 646.0);
const CENTER: Vec2 = Vec2::new(639.0, 359.0);

// convert to coords in relation to BOUNDS
fn convert_to_bounds_coords(position: Vec2) -> Vec2 {
    let x = (position.x - CENTER.x) / (UPPER_RIGHT_CORNER_WINDOW.x - CENTER.x) * BOUNDS as f32;
    let y = (position.y - CENTER.y) / (LOWER_LEFT_CORNER_WINDOW.y - CENTER.y) * BOUNDS as f32;

    Vec2::new(x, -y)
}

pub fn cursor_interaction(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut gen_hashcells: EventReader<crate::hashcell::GeneratedHashCell>,
    mut commands: Commands,
) {
    let left_click = mouse_button_input.just_pressed(MouseButton::Left)
        || mouse_button_input.pressed(MouseButton::Left);
    let right_click = mouse_button_input.just_pressed(MouseButton::Right)
        || mouse_button_input.pressed(MouseButton::Right);

    if left_click || right_click {
        if let Some(position) = q_windows.single().cursor_position() {
            let position = convert_to_bounds_coords(position);

            if position.x.abs() < BOUNDS as f32 && position.y.abs() < BOUNDS as f32 {
                let hashcell = &gen_hashcells.read().next().unwrap().0;

                let neighbours = hashcell.find_neighbors(Vec3::new(position.x, position.y, 0.0));

                for neighbour in neighbours {
                    let direction = neighbour.1.position - Vec3::new(position.x, position.y, 0.0);

                    let mut force = direction.normalize() * direction.length().powi(2) * 0.005;

                    if right_click {
                        force *= -1.0;
                    }

                    commands
                        .entity(neighbour.0)
                        .insert(neighbour.1.to_new_particle_with_force(force));
                }
            }
        }
    }
}
