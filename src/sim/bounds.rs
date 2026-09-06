use bevy::prelude::*;

use super::PARTICLE_RADIUS;

pub const BOUNDS: f32 = 100.0;

/// Keeps the particle outside a circle, with the same wall response.
pub fn resolve_circle_collision(
    position: &mut Vec2,
    velocity: &mut Vec2,
    center: Vec2,
    radius: f32,
    damping: f32,
    friction: f32,
    dt: f32,
) {
    let offset = *position - center;
    let distance = offset.length();
    let limit = radius + PARTICLE_RADIUS;
    if distance >= limit {
        return;
    }

    let normal = if distance > 0.0 {
        offset / distance
    } else {
        Vec2::Y
    };
    *position = center + normal * limit;

    let into = velocity.dot(normal);
    let tangential = *velocity - normal * into;
    let bounced = if into < 0.0 { -into * damping } else { into };
    *velocity = normal * bounced + tangential * (-friction * dt).exp();
}

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
