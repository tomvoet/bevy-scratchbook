use bevy::{picking::hover::Hovered, prelude::*, window::PrimaryWindow};

use crate::{
    render::MainCamera,
    sim::{bounds::BOUNDS, spawn, CursorInteraction, Obstacle, SimParams, MAX_OBSTACLES},
};

/// Root of the native panel. If it is `Hovered` it tells input to ignore clicks.
/// On the web the leptos panel sits beside the canvas, so nothing matches.
#[derive(Component, Default, Clone)]
pub struct UiPanel;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DraggedObstacle>()
            .add_systems(Update, (edit_obstacles, update_cursor_interaction).chain());
    }
}

/// The obstacle being dragged and the cursor's offset from its centre.
#[derive(Resource, Default)]
struct DraggedObstacle(Option<(Entity, Vec2)>);

fn cursor_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    cameras: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<Vec2> {
    let (camera, camera_transform) = cameras.single().ok()?;
    windows
        .single()
        .ok()?
        .cursor_position()
        .and_then(|position| camera.viewport_to_world_2d(camera_transform, position).ok())
}

fn pointer_over_ui(panel: &Query<&Hovered, With<UiPanel>>) -> bool {
    panel.iter().any(|hovered| hovered.0)
}

fn obstacle_under(
    cursor: Vec2,
    obstacles: &Query<(Entity, &mut Transform, &mut Obstacle)>,
) -> Option<(Entity, Vec2)> {
    obstacles.iter().find_map(|(entity, transform, obstacle)| {
        let center = transform.translation.truncate();
        (center.distance(cursor) < obstacle.radius).then_some((entity, center))
    })
}

/// Drag with the left button, add with shift+click, remove with right-click.
#[allow(clippy::too_many_arguments)]
fn edit_obstacles(
    mut commands: Commands,
    mut drag: ResMut<DraggedObstacle>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    params: Res<SimParams>,
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    panel: Query<&Hovered, With<UiPanel>>,
    mut obstacles: Query<(Entity, &mut Transform, &mut Obstacle)>,
) {
    let cursor = cursor_world_position(&windows, &cameras);
    let over_ui = pointer_over_ui(&panel);
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    if let (Some(cursor), false) = (cursor, over_ui) {
        let hit = obstacle_under(cursor, &obstacles);
        let inside_tank = cursor.abs().max_element() < BOUNDS - params.pillar_radius;

        if mouse.just_pressed(MouseButton::Left) {
            match hit {
                Some((entity, center)) if !shift => drag.0 = Some((entity, center - cursor)),
                None if shift && inside_tank && obstacles.iter().count() < MAX_OBSTACLES => {
                    spawn::spawn_obstacle(&mut commands, cursor, params.pillar_radius);
                }
                _ => {}
            }
        }
        if mouse.just_pressed(MouseButton::Right) {
            if let Some((entity, _)) = hit {
                commands.entity(entity).despawn();
            }
        }
    }

    if !mouse.pressed(MouseButton::Left) {
        drag.0 = None;
    }

    // Any obstacle not being dragged is at rest.
    for (entity, _, mut obstacle) in &mut obstacles {
        let dragged = drag.0.is_some_and(|(dragged, _)| dragged == entity);
        if !dragged && obstacle.velocity != Vec2::ZERO {
            obstacle.velocity = Vec2::ZERO;
        }
    }

    if let (Some((entity, offset)), Some(cursor)) = (drag.0, cursor) {
        if let Ok((_, mut transform, mut obstacle)) = obstacles.get_mut(entity) {
            let limit = Vec2::splat(BOUNDS - obstacle.radius);
            let previous = transform.translation.truncate();
            let center = (cursor + offset).clamp(-limit, limit);
            if center != previous {
                transform.translation = center.extend(0.0);
            }
            let velocity = (center - previous) / time.delta_secs().max(1e-6);
            if obstacle.velocity != velocity {
                obstacle.velocity = velocity;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update_cursor_interaction(
    mut cursor: ResMut<CursorInteraction>,
    drag: Res<DraggedObstacle>,
    params: Res<SimParams>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    panel: Query<&Hovered, With<UiPanel>>,
    obstacles: Query<(Entity, &mut Transform, &mut Obstacle)>,
) {
    let strength = if mouse.pressed(MouseButton::Left) {
        params.interaction_strength
    } else if mouse.pressed(MouseButton::Right) {
        -params.interaction_strength
    } else {
        0.0
    };
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let point = cursor_world_position(&windows, &cameras);
    let over_pillar = point.is_some_and(|p| obstacle_under(p, &obstacles).is_some());

    let blocked =
        strength == 0.0 || shift || over_pillar || drag.0.is_some() || pointer_over_ui(&panel);
    let interaction = if blocked {
        None
    } else {
        point.map(|point| (point, strength))
    };

    if cursor.0 != interaction {
        cursor.0 = interaction;
    }
}
