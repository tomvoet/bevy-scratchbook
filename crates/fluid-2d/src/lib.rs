// The `AsBindGroup` derive on the materials needs more room than the default.
#![recursion_limit = "256"]

use bevy::prelude::*;

mod bridge;
pub mod controls;
mod fps;
mod input;
mod render;
mod sim;
#[cfg(not(target_arch = "wasm32"))]
mod ui;

pub use bridge::{control_channel, ControlBridgePlugin, ControlSender, SimStats};
pub use render::RenderSettings;
pub use sim::{ObstacleCommand, ResetParticles, SimParams, SpawnLayout};

/// The simulation, rendering, and mouse input. Bring your own panel.
pub struct FluidSimPlugin;

impl Plugin for FluidSimPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            sim::SimPlugin,
            fps::FpsPlugin,
            render::RenderPlugin,
            input::InputPlugin,
        ));
    }
}

/// The bevy feathers panel, used on native. The web build drives the same
/// parameters from leptos instead, which keeps bevy_ui out of the wasm binary.
#[cfg(not(target_arch = "wasm32"))]
pub struct FeathersUiPlugin;

#[cfg(not(target_arch = "wasm32"))]
impl Plugin for FeathersUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ui::UiPlugin);
    }
}
