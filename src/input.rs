use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;

use crate::{
    render::MainCamera,
    sim::{bounds::BOUNDS, CursorInteraction, Obstacle, SimParams},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DraggedObstacle>()
            .add_systems(Update, (drag_obstacles, update_cursor_interaction).chain());
    }
}

/// The obstacle being dragged and the cursor's offset from its centre.
#[derive(Resource, Default)]
struct DraggedObstacle(Option<(Entity, Vec2)>);

fn cursor_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    cameras: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<Vec2> {
    let (camera, camera_transform) = cameras.single();
    windows
        .single()
        .cursor_position()
        .and_then(|position| camera.viewport_to_world_2d(camera_transform, position))
}

fn drag_obstacles(
    mut drag: ResMut<DraggedObstacle>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut egui: EguiContexts,
    mut obstacles: Query<(Entity, &mut Transform, &Obstacle)>,
) {
    let cursor = cursor_world_position(&windows, &cameras);

    if mouse.just_pressed(MouseButton::Left) && !egui.ctx_mut().is_pointer_over_area() {
        if let Some(cursor) = cursor {
            drag.0 = obstacles.iter().find_map(|(entity, transform, obstacle)| {
                let center = transform.translation.truncate();
                (center.distance(cursor) < obstacle.radius).then_some((entity, center - cursor))
            });
        }
    }
    if !mouse.pressed(MouseButton::Left) {
        drag.0 = None;
    }

    if let (Some((entity, offset)), Some(cursor)) = (drag.0, cursor) {
        if let Ok((_, mut transform, obstacle)) = obstacles.get_mut(entity) {
            let limit = Vec2::splat(BOUNDS - obstacle.radius);
            let center = (cursor + offset).clamp(-limit, limit);
            transform.translation = center.extend(0.0);
        }
    }
}

fn update_cursor_interaction(
    mut cursor: ResMut<CursorInteraction>,
    drag: Res<DraggedObstacle>,
    params: Res<SimParams>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut egui: EguiContexts,
) {
    let strength = if mouse.pressed(MouseButton::Left) {
        params.interaction_strength
    } else if mouse.pressed(MouseButton::Right) {
        -params.interaction_strength
    } else {
        0.0
    };

    let blocked = strength == 0.0 || drag.0.is_some() || egui.ctx_mut().is_pointer_over_area();
    let interaction = if blocked {
        None
    } else {
        cursor_world_position(&windows, &cameras).map(|point| (point, strength))
    };

    if cursor.0 != interaction {
        cursor.0 = interaction;
    }
}
