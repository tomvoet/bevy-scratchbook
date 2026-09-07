use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, spawn_counter)
            .add_systems(Update, update_counter);
    }
}

#[derive(Component)]
struct FpsText;

fn spawn_counter(mut commands: Commands) {
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

fn update_counter(
    diagnostics: Res<DiagnosticsStore>,
    mut text: Single<&mut Text, With<FpsText>>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed());
    let ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed());
    if let (Some(fps), Some(ms)) = (fps, ms) {
        ***text = format!("{fps:.0} fps  {ms:.1} ms");
    }
}
