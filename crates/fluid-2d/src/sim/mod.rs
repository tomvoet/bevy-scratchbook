use bevy::prelude::*;

pub mod bounds;
pub mod grid;
pub mod kernels;
pub mod spawn;
pub mod step;

pub const PARTICLE_RADIUS: f32 = 1.0;
/// Grid spacing at spawn. Resting density and smoothing radius come from it.
pub const SPACING: f32 = 2.0;
#[cfg(not(target_arch = "wasm32"))]
pub const SIM_HZ: f64 = 240.0;
/// Halved on the web: no threads there, so `par_iter_mut` runs serially.
#[cfg(target_arch = "wasm32")]
pub const SIM_HZ: f64 = 120.0;

/// Stable stiffness scales with `SIM_HZ^2`, so the defaults follow the rate.
pub const STIFFNESS: f32 = (SIM_HZ * SIM_HZ / (240.0 * 240.0)) as f32;
/// Caps fixed-step catch-up so one slow frame can't snowball into a stall.
const MAX_CATCH_UP: std::time::Duration = std::time::Duration::from_millis(50);

/// Marker. Position is the `Transform`.
#[derive(Component, Default)]
pub struct Particle;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Velocity(pub Vec2);

/// Where the particle is headed. Densities and forces get evaluated here.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PredictedPosition(pub Vec2);

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct GridSlot(pub u32);

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Density {
    pub value: f32,
    pub near: f32,
    /// `value` counting fluid neighbours only, so a droplet reads apart from
    /// water that merely sits next to a surface.
    pub fluid: f32,
}

/// A circle particles bounce off. Position is the `Transform`. `velocity` is
/// set while you drag it, so it shoves fluid instead of teleporting through.
#[derive(Component, Clone, Copy, Debug)]
pub struct Obstacle {
    pub radius: f32,
    pub velocity: Vec2,
}

impl Obstacle {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            velocity: Vec2::ZERO,
        }
    }
}

/// Only this many get rendered. See `render::MAX_OBSTACLES`.
pub const MAX_OBSTACLES: usize = 16;

#[derive(Message, Clone, Copy)]
pub enum ObstacleCommand {
    ResetPreset,
    Clear,
}

#[derive(Bundle, Default)]
pub struct ParticleBundle {
    pub particle: Particle,
    pub velocity: Velocity,
    pub predicted_position: PredictedPosition,
    pub density: Density,
    pub grid_slot: GridSlot,
    pub transform: Transform,
}

impl ParticleBundle {
    pub fn at_rest(position: Vec2) -> Self {
        Self {
            predicted_position: PredictedPosition(position),
            transform: Transform::from_translation(position.extend(0.0)),
            ..default()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SpawnLayout {
    #[default]
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
    /// Strength of attraction below target density, 0..1.
    pub cohesion: f32,
    pub viscosity: f32,
    /// Acceleration is `-air_drag * |v| * v`.
    pub air_drag: f32,
    pub interaction_radius: f32,
    pub interaction_strength: f32,
    pub spawn_layout: SpawnLayout,
    /// Radius for pillars placed with the mouse.
    pub pillar_radius: f32,
}

impl Default for SimParams {
    fn default() -> Self {
        Self {
            gravity: 100.0,
            mass: kernels::MASS,
            smoothing_radius: kernels::SMOOTHING_RADIUS,
            target_density: kernels::TARGET_DENSITY,
            // Stable up to roughly 400_000 at 240 Hz.
            pressure_multiplier: 300_000.0 * STIFFNESS,
            near_pressure_multiplier: 8_000.0 * STIFFNESS,
            // Full-strength attraction lets a lifted pillar drag a column of
            // water up with it. Near pressure is lowered to match.
            cohesion: 0.3,
            viscosity: 30.0,
            air_drag: 0.006,
            interaction_radius: 30.0,
            interaction_strength: 500.0,
            spawn_layout: SpawnLayout::DamBreak,
            pillar_radius: 10.0,
        }
    }
}

#[derive(Message, Default)]
pub struct ResetParticles;

/// Phases of one fixed simulation step, in order.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimSet {
    ExternalForces,
    Neighbours,
    Interactions,
    Integrate,
}

/// Where the cursor is pulling (positive) or pushing (negative) this frame.
#[derive(Resource, Default)]
pub struct CursorInteraction(pub Option<(Vec2, f32)>);

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimParams>()
            .init_resource::<CursorInteraction>()
            .init_resource::<grid::SpatialGrid>()
            .init_resource::<bounds::Boundary>()
            .add_message::<ResetParticles>()
            .add_message::<ObstacleCommand>()
            .insert_resource(Time::<Fixed>::from_hz(SIM_HZ))
            .insert_resource(Time::<Virtual>::from_max_delta(MAX_CATCH_UP))
            .add_systems(Startup, spawn::spawn_initial)
            .add_systems(Update, (spawn::reset, spawn::handle_obstacle_commands))
            .configure_sets(
                FixedUpdate,
                (
                    SimSet::ExternalForces,
                    SimSet::Neighbours,
                    SimSet::Interactions,
                    SimSet::Integrate,
                )
                    .chain(),
            )
            .add_systems(
                FixedUpdate,
                (
                    step::apply_external_forces.in_set(SimSet::ExternalForces),
                    (
                        step::sync_boundary,
                        step::build_grid,
                        step::calculate_densities,
                        step::refresh_densities,
                    )
                        .chain()
                        .in_set(SimSet::Neighbours),
                    step::apply_interactions.in_set(SimSet::Interactions),
                    (step::integrate, step::collide_obstacles)
                        .chain()
                        .in_set(SimSet::Integrate),
                ),
            );
    }
}
