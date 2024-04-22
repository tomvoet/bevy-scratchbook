use super::SPHERE_RADIUS;

pub const SMOOTHING_RADIUS: f32 = SPHERE_RADIUS + 12.;
pub const MASS: f32 = 2.0;

pub(super) fn smoothing_kernel(distance: f32, smoothing_radius: f32) -> f32 {
    if distance >= smoothing_radius {
        return 0.0;
    }

    let volume = std::f32::consts::FRAC_PI_6 * smoothing_radius.powi(4);
    (smoothing_radius - distance).powi(2) / volume
}

pub(super) fn smoothing_kernel_derivative(distance: f32, smoothing_radius: f32) -> f32 {
    if distance >= smoothing_radius {
        return 0.0;
    }
    // 12: minus weg?
    let scale = 12.0 / (std::f32::consts::PI * smoothing_radius.powi(4));
    (smoothing_radius - distance) * scale
}

pub const TARGET_DENSITY: f32 = 0.02;

pub fn density_to_pressure(density: f32, target_density: f32, pressure_multiplier: f32) -> f32 {
    pressure_multiplier * (density - target_density)
}
