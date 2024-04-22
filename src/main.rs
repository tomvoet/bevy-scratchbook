use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::system::SystemId,
    prelude::*,
    utils::HashMap,
};
use bevy_egui::{egui::emath::Numeric, EguiPlugin};
use hashcell::GeneratedHashCell;
use particle::systems::{
    apply_pressure_force, calculate_densities, cursor_interaction, generate_hashcells,
    spawn_particles,
};
use systems::ParticlePlugin;
use ui::UIPlugin;

mod hashcell;
mod oneshot;
mod particle;
mod systems;
mod ui;

pub const TIMESTEP: f32 = 1.0 / 60.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            ParticlePlugin::default().drag(0.01),
            EguiPlugin,
            UIPlugin,
        ))
        .insert_resource(OneShot::default())
        .add_event::<GeneratedHashCell>()
        .add_systems(Startup, (spawn_camera, spawn_particles, setup))
        // Chain because, otherwise the particles will be despawned while the color is being applied
        .add_systems(
            Update,
            (
                reset_particles,
                (
                    generate_hashcells,
                    calculate_densities,
                    cursor_interaction,
                    apply_pressure_force,
                )
                    .chain(),
                //apply_color,
            ),
        )
        .run();
}

#[derive(Default, Resource)]
struct OneShot(HashMap<&'static str, SystemId>);

impl OneShot {
    fn insert(&mut self, key: &'static str, system_id: SystemId) {
        self.0.insert(key, system_id);
    }
}

fn setup(world: &mut World) {
    let reset_particles_system = world.register_system(spawn_particles);

    let mut oneshot = OneShot::default();

    oneshot.insert("reset_particles", reset_particles_system);

    world.insert_resource(oneshot);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 300.0)),
        ..default()
    });

    // add a light
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.5, 0.5, 0.5),
        brightness: 2500.0,
    });

    // add background color
    commands.insert_resource(ClearColor(Color::rgb(0.75, 0.75, 0.75)));

    // set timestep
    commands.insert_resource(Time::<Fixed>::from_seconds(TIMESTEP.to_f64()));
}

fn reset_particles(
    mut commands: Commands,
    keys: ResMut<ButtonInput<KeyCode>>,
    mut query: Query<Entity, With<particle::Particle>>,
    oneshot: Res<OneShot>,
) {
    if keys.just_pressed(KeyCode::Space) {
        if let Some(reset_particles_system) = oneshot.0.get("reset_particles") {
            for entity in query.iter_mut() {
                commands.entity(entity).despawn_recursive();
            }

            commands.run_system(*reset_particles_system);
        } else {
            eprintln!("Failed to get reset_particles system");
        }
    }
}
