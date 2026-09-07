use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(window()),
                ..default()
            }),
            fluid_2d::FluidSimPlugin,
            fluid_2d::FeathersUiPlugin,
        ))
        .run();
}

fn window() -> Window {
    Window {
        title: "Fluid Sim".into(),
        ..default()
    }
}
