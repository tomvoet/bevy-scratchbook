// Each kernel integrates to 1 over the disc of radius `h`.

use std::f32::consts::PI;

use super::SPACING;

pub const SMOOTHING_RADIUS: f32 = 3.0 * SPACING;
pub const MASS: f32 = 1.0;

/// Resting density of the spawn grid.
pub const TARGET_DENSITY: f32 = MASS / (SPACING * SPACING);

/// `(h - d)^2`
pub fn spiky_pow2(d: f32, h: f32) -> f32 {
    if d >= h {
        return 0.0;
    }
    let scale = 6.0 / (PI * h.powi(4));
    (h - d).powi(2) * scale
}

pub fn spiky_pow2_derivative(d: f32, h: f32) -> f32 {
    if d >= h {
        return 0.0;
    }
    let scale = -12.0 / (PI * h.powi(4));
    (h - d) * scale
}

/// `(h - d)^3`, for the near-pressure term that prevents particle pairing.
pub fn spiky_pow3(d: f32, h: f32) -> f32 {
    if d >= h {
        return 0.0;
    }
    let scale = 10.0 / (PI * h.powi(5));
    (h - d).powi(3) * scale
}

pub fn spiky_pow3_derivative(d: f32, h: f32) -> f32 {
    if d >= h {
        return 0.0;
    }
    let scale = -30.0 / (PI * h.powi(5));
    (h - d).powi(2) * scale
}

/// `(h^2 - d^2)^3`
pub fn poly6(d: f32, h: f32) -> f32 {
    if d >= h {
        return 0.0;
    }
    let scale = 4.0 / (PI * h.powi(8));
    (h * h - d * d).powi(3) * scale
}

/// `cohesion` scales the negative (attractive) side, below target density.
pub fn density_to_pressure(
    density: f32,
    target_density: f32,
    pressure_multiplier: f32,
    cohesion: f32,
) -> f32 {
    let pressure = pressure_multiplier * (density - target_density);
    if pressure < 0.0 {
        pressure * cohesion
    } else {
        pressure
    }
}

pub fn near_density_to_pressure(near_density: f32, near_pressure_multiplier: f32) -> f32 {
    near_pressure_multiplier * near_density
}
