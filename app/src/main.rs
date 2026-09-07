use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(window()),
                ..default()
            }),
            fluid_2d::FluidSimPlugin,
        ))
        .run();
}

#[cfg(not(target_arch = "wasm32"))]
fn window() -> Window {
    Window {
        title: "Fluid Sim".into(),
        ..default()
    }
}

/// On the web, render into the page's canvas and follow its size.
#[cfg(target_arch = "wasm32")]
fn window() -> Window {
    Window {
        canvas: Some("#sim".into()),
        fit_canvas_to_parent: true,
        ..default()
    }
}
