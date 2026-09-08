use bevy::{picking::hover::Hovered, prelude::*, window::PrimaryWindow};

use crate::{
    render::MainCamera,
    sim::{bounds::BOUNDS, spawn, CursorInteraction, Obstacle, SimParams, MAX_OBSTACLES},
};

/// How long a finger sits still to count as a hold, and how far it may wander.
const HOLD: f32 = 0.4;
const HOLD_SLOP: f32 = 4.0;

/// Root of the native panel. If it is `Hovered` it tells input to ignore clicks.
/// On the web the leptos panel sits beside the canvas, so nothing matches.
#[derive(Component, Default, Clone)]
pub struct UiPanel;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DraggedObstacle>()
            .init_resource::<Pointer>()
            .init_resource::<Hold>()
            .add_systems(
                Update,
                (
                    update_pointer,
                    edit_obstacles,
                    hold_obstacles,
                    drag_obstacle,
                    update_cursor_interaction,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Button {
    Left,
    Right,
}

#[derive(Clone, Copy)]
struct Press {
    at: Vec2,
    button: Button,
    started: bool,
    touch: bool,
}

/// This frame's press, mouse or touch alike.
#[derive(Resource, Default)]
struct Pointer(Option<Press>);

/// The obstacle being dragged and the cursor's offset from its centre.
#[derive(Resource, Default)]
struct DraggedObstacle(Option<(Entity, Vec2)>);

/// Where a finger went down and when, while it is still a hold candidate.
#[derive(Resource, Default)]
struct Hold(Option<(Vec2, f32)>);

fn spawn_pillar(
    commands: &mut Commands,
    at: Vec2,
    params: &SimParams,
    obstacles: &Query<(Entity, &mut Transform, &mut Obstacle)>,
) {
    let inside_tank = at.abs().max_element() < BOUNDS - params.pillar_radius;
    if inside_tank && obstacles.iter().count() < MAX_OBSTACLES {
        spawn::spawn_obstacle(commands, at, params.pillar_radius);
    }
}

fn update_pointer(
    mut pointer: ResMut<Pointer>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let fingers: Vec<Vec2> = touches.iter().map(|touch| touch.position()).collect();
    // Several fingers work from their midpoint, so the fluid follows the grip.
    let press = match fingers.as_slice() {
        [] => mouse_press(&mouse, &windows),
        [only] => Some((*only, Button::Left)),
        many => Some((
            many.iter().copied().sum::<Vec2>() / many.len() as f32,
            Button::Right,
        )),
    };

    let touch = !fingers.is_empty();
    let started = if touch {
        touches.any_just_pressed()
    } else {
        mouse.any_just_pressed([MouseButton::Left, MouseButton::Right])
    };

    pointer.0 = press.and_then(|(screen, button)| {
        let (camera, transform) = cameras.single().ok()?;
        Some(Press {
            at: camera.viewport_to_world_2d(transform, screen).ok()?,
            button,
            started,
            touch,
        })
    });
}

fn mouse_press(
    mouse: &ButtonInput<MouseButton>,
    windows: &Query<&Window, With<PrimaryWindow>>,
) -> Option<(Vec2, Button)> {
    let button = match (
        mouse.pressed(MouseButton::Left),
        mouse.pressed(MouseButton::Right),
    ) {
        (true, _) => Button::Left,
        (_, true) => Button::Right,
        _ => return None,
    };
    Some((windows.single().ok()?.cursor_position()?, button))
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
/// Touch has neither, so it does both of those by holding still instead.
fn edit_obstacles(
    mut commands: Commands,
    mut drag: ResMut<DraggedObstacle>,
    pointer: Res<Pointer>,
    keys: Res<ButtonInput<KeyCode>>,
    params: Res<SimParams>,
    panel: Query<&Hovered, With<UiPanel>>,
    obstacles: Query<(Entity, &mut Transform, &mut Obstacle)>,
) {
    let Some(press) = pointer
        .0
        .filter(|press| press.started && !pointer_over_ui(&panel))
    else {
        return;
    };
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    match (press.button, obstacle_under(press.at, &obstacles)) {
        (Button::Left, Some((entity, center))) if !shift => {
            drag.0 = Some((entity, center - press.at));
        }
        (Button::Left, None) if shift => {
            spawn_pillar(&mut commands, press.at, &params, &obstacles);
        }
        (Button::Right, Some((entity, _))) if !press.touch => {
            commands.entity(entity).despawn();
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn hold_obstacles(
    mut commands: Commands,
    mut hold: ResMut<Hold>,
    mut drag: ResMut<DraggedObstacle>,
    pointer: Res<Pointer>,
    params: Res<SimParams>,
    time: Res<Time>,
    panel: Query<&Hovered, With<UiPanel>>,
    obstacles: Query<(Entity, &mut Transform, &mut Obstacle)>,
) {
    let candidate = pointer
        .0
        .filter(|press| press.touch && press.button == Button::Left && !pointer_over_ui(&panel));
    let Some(press) = candidate else {
        hold.0 = None;
        return;
    };
    if press.started {
        hold.0 = Some((press.at, time.elapsed_secs()));
    }

    let Some((from, since)) = hold.0 else { return };
    if from.distance(press.at) > HOLD_SLOP {
        hold.0 = None;
        return;
    }
    if time.elapsed_secs() - since < HOLD {
        return;
    }

    hold.0 = None;
    match obstacle_under(from, &obstacles) {
        Some((entity, _)) => {
            commands.entity(entity).despawn();
            drag.0 = None;
        }
        None => spawn_pillar(&mut commands, from, &params, &obstacles),
    }
}

fn drag_obstacle(
    mut drag: ResMut<DraggedObstacle>,
    pointer: Res<Pointer>,
    time: Res<Time>,
    mut obstacles: Query<(Entity, &mut Transform, &mut Obstacle)>,
) {
    let press = pointer.0.filter(|press| press.button == Button::Left);
    if press.is_none() {
        drag.0 = None;
    }

    // Any obstacle not being dragged is at rest.
    for (entity, _, mut obstacle) in &mut obstacles {
        let dragged = drag.0.is_some_and(|(dragged, _)| dragged == entity);
        if !dragged && obstacle.velocity != Vec2::ZERO {
            obstacle.velocity = Vec2::ZERO;
        }
    }

    let (Some((entity, offset)), Some(press)) = (drag.0, press) else {
        return;
    };
    let Ok((_, mut transform, mut obstacle)) = obstacles.get_mut(entity) else {
        return;
    };

    let limit = Vec2::splat(BOUNDS - obstacle.radius);
    let previous = transform.translation.truncate();
    let center = (press.at + offset).clamp(-limit, limit);
    if center != previous {
        transform.translation = center.extend(0.0);
    }
    let velocity = (center - previous) / time.delta_secs().max(1e-6);
    if obstacle.velocity != velocity {
        obstacle.velocity = velocity;
    }
}

fn update_cursor_interaction(
    mut cursor: ResMut<CursorInteraction>,
    pointer: Res<Pointer>,
    drag: Res<DraggedObstacle>,
    params: Res<SimParams>,
    keys: Res<ButtonInput<KeyCode>>,
    panel: Query<&Hovered, With<UiPanel>>,
    obstacles: Query<(Entity, &mut Transform, &mut Obstacle)>,
) {
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let blocked = shift || drag.0.is_some() || pointer_over_ui(&panel);

    let interaction = pointer
        .0
        .filter(|press| !blocked && obstacle_under(press.at, &obstacles).is_none())
        .map(|press| {
            let strength = match press.button {
                Button::Left => params.interaction_strength,
                Button::Right => -params.interaction_strength,
            };
            (press.at, strength)
        });

    if cursor.0 != interaction {
        cursor.0 = interaction;
    }
}
