use bevy::prelude::*;
use rand::Rng;

use super::{
    bounds::BOUNDS, Obstacle, ObstacleCommand, Particle, ParticleBundle, ResetParticles, SimParams,
    SpawnLayout, SPACING,
};

const DROP_SIZE: (usize, usize) = (80, 50);
const DAM_SIZE: (usize, usize) = (50, 80);
/// Preset pillars: centre and radius.
const OBSTACLES: [(Vec2, f32); 2] = [
    (Vec2::new(15.0, -75.0), 14.0),
    (Vec2::new(65.0, -40.0), 9.0),
];

pub fn spawn_initial(mut commands: Commands, params: Res<SimParams>) {
    spawn_particles(&mut commands, params.spawn_layout);
    spawn_preset_obstacles(&mut commands);
}

pub fn spawn_obstacle(commands: &mut Commands, center: Vec2, radius: f32) {
    commands.spawn((
        Obstacle::new(radius),
        TransformBundle::from_transform(Transform::from_translation(center.extend(0.0))),
    ));
}

fn spawn_preset_obstacles(commands: &mut Commands) {
    for (center, radius) in OBSTACLES {
        spawn_obstacle(commands, center, radius);
    }
}

pub fn handle_obstacle_commands(
    mut commands: Commands,
    mut events: EventReader<ObstacleCommand>,
    obstacles: Query<Entity, With<Obstacle>>,
) {
    for command in events.read() {
        for entity in &obstacles {
            commands.entity(entity).despawn();
        }
        if let ObstacleCommand::ResetPreset = command {
            spawn_preset_obstacles(&mut commands);
        }
    }
}

pub fn reset(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut events: EventReader<ResetParticles>,
    params: Res<SimParams>,
    particles: Query<Entity, With<Particle>>,
) {
    let requested = events.read().next().is_some();
    if !keys.just_pressed(KeyCode::Space) && !requested {
        return;
    }
    for entity in &particles {
        commands.entity(entity).despawn_recursive();
    }
    spawn_particles(&mut commands, params.spawn_layout);
}

fn spawn_particles(commands: &mut Commands, layout: SpawnLayout) {
    let mut rng = rand::thread_rng();
    let gap = 2.0 * SPACING;

    let ((w, h), center) = match layout {
        SpawnLayout::DamBreak => {
            let half = Vec2::new(DAM_SIZE.0 as f32 - 1.0, DAM_SIZE.1 as f32 - 1.0) * SPACING / 2.0;
            (DAM_SIZE, -Vec2::splat(BOUNDS) + half + gap)
        }
        SpawnLayout::Drop => {
            let half_y = (DROP_SIZE.1 as f32 - 1.0) * SPACING / 2.0;
            (DROP_SIZE, Vec2::new(0.0, BOUNDS - half_y - gap))
        }
    };
    let half = Vec2::new(w as f32 - 1.0, h as f32 - 1.0) * SPACING / 2.0;

    let mut bundles = Vec::with_capacity(w * h);
    for x in 0..w {
        for y in 0..h {
            let jitter = Vec2::new(rng.gen_range(-0.1..0.1), rng.gen_range(-0.1..0.1)) * SPACING;
            let position = Vec2::new(x as f32, y as f32) * SPACING - half + center + jitter;
            bundles.push(ParticleBundle::at_rest(position));
        }
    }
    commands.spawn_batch(bundles);
}
