// The `AsBindGroup` derive on the materials needs more room than the default.
#![recursion_limit = "256"]

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

mod input;
mod render;
mod sim;
mod ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Fluid Sim".into(),
                    ..default()
                }),
                ..default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            sim::SimPlugin,
            render::RenderPlugin,
            ui::UiPlugin,
            input::InputPlugin,
        ))
        .run();
}
