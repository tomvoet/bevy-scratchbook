use bevy::prelude::*;

pub mod bounds;
pub mod drag;
pub mod velocity;

#[derive(Default)]
pub struct ParticlePlugin {
    pub drag: f32,
}

impl ParticlePlugin {
    pub fn drag(mut self, drag: f32) -> Self {
        self.drag = drag;
        self
    }
}

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(drag::Drag(self.drag))
            .add_systems(Startup, bounds::spawn_bounds)
            .add_systems(
                Update,
                (
                    velocity::velocity_system,
                    drag::drag_system,
                    bounds::check_bounds,
                )
                    .chain(),
            );
    }
}
