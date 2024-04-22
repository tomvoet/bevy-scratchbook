use bevy::prelude::*;

use crate::particle::Particle;

pub const MAX_SPEED: f32 = 8.0;
pub const FACTOR: f32 = 50.0;

pub fn velocity_system(
    mut query: Query<(&mut Particle, &mut Transform)>,
    ui_state: Res<crate::ui::UiState>,
) {
    query
        .par_iter_mut()
        .for_each(|(mut particle, mut transform)| {
            particle.clamp_velocity(MAX_SPEED);
            transform.translation += particle.velocity * crate::TIMESTEP * FACTOR;
            particle.position = transform.translation;
            particle.apply_force(Vec3::new(0.0, -ui_state.gravity, 0.0) * crate::TIMESTEP * 0.1);
        });
}
