// The `AsBindGroup` derive on the materials needs more room than the default.
#![recursion_limit = "256"]

use bevy::prelude::*;

mod fps;
mod input;
mod render;
mod sim;
mod ui;

/// The whole 2D SPH fluid: simulation, rendering, control panel, and mouse input.
pub struct FluidSimPlugin;

impl Plugin for FluidSimPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            sim::SimPlugin,
            fps::FpsPlugin,
            render::RenderPlugin,
            ui::UiPlugin,
            input::InputPlugin,
        ));
    }
}
