use bevy::prelude::*;

use super::PARTICLE_RADIUS;

pub const BOUNDS: f32 = 100.0;

/// `friction` is a rate in 1/s, applied while in contact.
pub fn resolve_collision(
    position: &mut Vec2,
    velocity: &mut Vec2,
    damping: f32,
    friction: f32,
    dt: f32,
) {
    let limit = BOUNDS - PARTICLE_RADIUS;
    let slide = (-friction * dt).exp();

    if position.x.abs() > limit {
        position.x = limit.copysign(position.x);
        velocity.x *= -damping;
        velocity.y *= slide;
    }

    if position.y.abs() > limit {
        position.y = limit.copysign(position.y);
        velocity.y *= -damping;
        velocity.x *= slide;
    }
}
