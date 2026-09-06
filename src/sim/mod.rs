use bevy::prelude::*;

pub mod bounds;
pub mod grid;
pub mod kernels;
pub mod spawn;
pub mod step;

pub const PARTICLE_RADIUS: f32 = 1.0;
/// Grid spacing at spawn; the resting density and smoothing radius derive from it.
pub const SPACING: f32 = 2.0;
pub const SIM_HZ: f64 = 240.0;
/// Caps fixed-step catch-up so one slow frame can't snowball into a stall.
const MAX_CATCH_UP: std::time::Duration = std::time::Duration::from_millis(50);

#[derive(Debug, Default, Component, Clone)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub predicted_position: Vec2,
    pub density: f32,
    pub near_density: f32,
}

impl Particle {
    pub fn at_rest(position: Vec2) -> Self {
        Self {
            position,
            predicted_position: position,
            ..default()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpawnLayout {
    DamBreak,
    Drop,
}

#[derive(Resource, Clone, Copy, PartialEq, Debug)]
pub struct SimParams {
    pub gravity: f32,
    pub mass: f32,
    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub near_pressure_multiplier: f32,
    pub viscosity: f32,
    pub collision_damping: f32,
    /// 1/s
    pub wall_friction: f32,
    /// Acceleration is `-air_drag * |v| * v`.
    pub air_drag: f32,
    pub interaction_radius: f32,
    pub interaction_strength: f32,
    pub spawn_layout: SpawnLayout,
}

impl Default for SimParams {
    fn default() -> Self {
        Self {
            gravity: 100.0,
            mass: kernels::MASS,
            smoothing_radius: kernels::SMOOTHING_RADIUS,
            target_density: kernels::TARGET_DENSITY,
            // Stable at `SIM_HZ` up to roughly 400_000.
            pressure_multiplier: 300_000.0,
            near_pressure_multiplier: 15_000.0,
            viscosity: 30.0,
            collision_damping: 0.8,
            wall_friction: 5.0,
            air_drag: 0.006,
            interaction_radius: 30.0,
            interaction_strength: 500.0,
            spawn_layout: SpawnLayout::DamBreak,
        }
    }
}

#[derive(Event, Default)]
pub struct ResetParticles;

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimParams>()
            .init_resource::<grid::SpatialGrid>()
            .add_event::<ResetParticles>()
            .insert_resource(Time::<Fixed>::from_hz(SIM_HZ))
            .add_systems(Startup, |mut time: ResMut<Time<Virtual>>| {
                time.set_max_delta(MAX_CATCH_UP);
            })
            .add_systems(Startup, spawn::spawn_initial)
            .add_systems(Update, spawn::reset)
            .add_systems(
                FixedUpdate,
                (
                    step::apply_external_forces,
                    // Rebuilt after densities so the pressure pass sees them.
                    step::build_grid,
                    step::calculate_densities,
                    step::build_grid,
                    step::apply_pressure_forces,
                    step::apply_viscosity,
                    step::integrate,
                )
                    .chain(),
            );
    }
}
