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

/// Marker. Position is the `Transform`.
#[derive(Component, Default)]
pub struct Particle;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Velocity(pub Vec2);

/// Where the particle will be shortly; densities and forces are evaluated here.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct PredictedPosition(pub Vec2);

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Density {
    pub value: f32,
    pub near: f32,
}

/// A circle particles collide with. Position is the `Transform`; `velocity`
/// is set while it is being dragged so it pushes fluid rather than
/// teleporting through it.
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

/// At most this many are rendered; see `render::MAX_OBSTACLES`.
pub const MAX_OBSTACLES: usize = 16;

#[derive(Event, Clone, Copy)]
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
    pub transform: TransformBundle,
}

impl ParticleBundle {
    pub fn at_rest(position: Vec2) -> Self {
        Self {
            predicted_position: PredictedPosition(position),
            transform: TransformBundle::from_transform(Transform::from_translation(
                position.extend(0.0),
            )),
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
    /// Strength of attraction below target density, 0..1.
    pub cohesion: f32,
    pub viscosity: f32,
    pub collision_damping: f32,
    /// 1/s
    pub wall_friction: f32,
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
            // Stable at `SIM_HZ` up to roughly 400_000.
            pressure_multiplier: 300_000.0,
            near_pressure_multiplier: 8_000.0,
            // Full-strength attraction lets a lifted pillar pull a column of
            // water up with it; near pressure is lowered to match.
            cohesion: 0.3,
            viscosity: 30.0,
            collision_damping: 0.8,
            wall_friction: 5.0,
            air_drag: 0.006,
            interaction_radius: 30.0,
            interaction_strength: 500.0,
            spawn_layout: SpawnLayout::DamBreak,
            pillar_radius: 10.0,
        }
    }
}

#[derive(Event, Default)]
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
            .add_event::<ResetParticles>()
            .add_event::<ObstacleCommand>()
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
                    // Rebuilt after densities so the pressure pass sees them.
                    (
                        step::build_grid,
                        step::calculate_densities,
                        step::build_grid,
                    )
                        .chain()
                        .in_set(SimSet::Neighbours),
                    (step::apply_pressure_forces, step::apply_viscosity)
                        .chain()
                        .in_set(SimSet::Interactions),
                    (step::integrate, step::collide_obstacles)
                        .chain()
                        .in_set(SimSet::Integrate),
                ),
            );
    }
}
