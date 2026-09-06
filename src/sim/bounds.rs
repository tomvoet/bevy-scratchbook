use super::{Particle, PARTICLE_RADIUS};

pub const BOUNDS: f32 = 100.0;

/// `friction` is a rate in 1/s, applied while in contact.
pub fn resolve_collision(particle: &mut Particle, damping: f32, friction: f32, dt: f32) {
    let limit = BOUNDS - PARTICLE_RADIUS;
    let slide = (-friction * dt).exp();

    if particle.position.x.abs() > limit {
        particle.position.x = limit.copysign(particle.position.x);
        particle.velocity.x *= -damping;
        particle.velocity.y *= slide;
    }

    if particle.position.y.abs() > limit {
        particle.position.y = limit.copysign(particle.position.y);
        particle.velocity.y *= -damping;
        particle.velocity.x *= slide;
    }
}
