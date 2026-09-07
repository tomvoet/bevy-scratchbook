use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::bridge::SimStats;

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Update, publish);

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Startup, overlay::spawn)
            .add_systems(Update, overlay::update);
    }
}

fn fps(diagnostics: &DiagnosticsStore) -> Option<f64> {
    diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
}

/// Only present when an outside panel asked for it, so native just skips this.
fn publish(diagnostics: Res<DiagnosticsStore>, stats: Option<Res<SimStats>>) {
    let Some(stats) = stats else { return };
    if let Some(fps) = fps(&diagnostics) {
        stats.set_fps(fps as f32);
    }
}

/// The web build draws this in leptos instead, which keeps bevy_ui off wasm.
#[cfg(not(target_arch = "wasm32"))]
mod overlay {
    use super::*;

    #[derive(Component)]
    pub struct FpsText;

    pub fn spawn(mut commands: Commands) {
        commands.spawn((
            Text::new("-- fps"),
            TextFont {
                font_size: px(16).into(),
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.9, 0.7)),
            Node {
                position_type: PositionType::Absolute,
                top: px(8),
                right: px(12),
                ..default()
            },
            FpsText,
        ));
    }

    pub fn update(diagnostics: Res<DiagnosticsStore>, mut text: Single<&mut Text, With<FpsText>>) {
        if let Some(fps) = fps(&diagnostics) {
            ***text = format!("{fps:.0} fps");
        }
    }
}
