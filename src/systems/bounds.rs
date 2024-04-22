use bevy::prelude::*;

use crate::particle::{Particle, SPHERE_RADIUS};

pub const BOUNDS: u16 = 100;
const FLOAT_BOUNDS: f32 = BOUNDS as f32;
const COLLISION_DAMPING: f32 = 0.8;

pub fn spawn_bounds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(Color::rgb(1.0, 0.0, 0.0));

    // left
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::from_size(Vec3::new(1.0, FLOAT_BOUNDS * 2.0, 1.0))),
        material: material.clone(),
        transform: Transform::from_translation(Vec3::new(-FLOAT_BOUNDS, 0.0, 0.0)),
        ..Default::default()
    });

    // right
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::from_size(Vec3::new(1.0, FLOAT_BOUNDS * 2.0, 1.0))),
        material: material.clone(),
        transform: Transform::from_translation(Vec3::new(FLOAT_BOUNDS, 0.0, 0.0)),
        ..Default::default()
    });

    // bottom
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::from_size(Vec3::new(FLOAT_BOUNDS * 2.0, 1.0, 1.0))),
        material: material.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, -FLOAT_BOUNDS, 0.0)),
        ..Default::default()
    });

    // top
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::from_size(Vec3::new(FLOAT_BOUNDS * 2.0, 1.0, 1.0))),
        material: material.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, FLOAT_BOUNDS, 0.0)),
        ..Default::default()
    });
}

pub fn check_bounds(mut particles: Query<(&mut Transform, &mut Particle)>) {
    for (mut transform, mut particle) in particles.iter_mut() {
        let particle_translation = transform.translation;

        let bounds_size = FLOAT_BOUNDS - SPHERE_RADIUS;

        if particle_translation.x.abs() > bounds_size {
            particle.velocity.x *= -COLLISION_DAMPING;
            particle.position.x = bounds_size.copysign(particle.position.x);
            transform.translation.x = particle.position.x;
        }

        if particle_translation.y.abs() > bounds_size {
            particle.velocity.y *= -COLLISION_DAMPING;
            particle.position.y = bounds_size.copysign(particle.position.y);
            transform.translation.y = particle.position.y;
        }
    }
}
