use bevy::{
    camera::{visibility::RenderLayers, Hdr, OrthographicProjection, Projection, ScalingMode},
    post_process::bloom::Bloom,
    prelude::*,
    ui::IsDefaultUiCamera,
};

use crate::sim::{bounds::BOUNDS, Obstacle};

pub mod particles;
pub mod scene;
pub mod surface;

/// Visible height in world units; also the side length of the surface quad.
pub const VIEW_HEIGHT: f32 = BOUNDS * 2.0 + 50.0;
/// Particles live here; the field camera sees only this layer.
pub const FIELD_LAYER: usize = 1;
const CLEAR_COLOR: Color = Color::srgb(0.015, 0.018, 0.03);

/// Uniform array capacity for obstacle circles in the shaders.
pub const MAX_OBSTACLES: usize = crate::sim::MAX_OBSTACLES;

#[derive(Component)]
pub struct MainCamera;

/// Obstacle circles as `(x, y, radius, 0)` in quad uv space, plus the count.
pub fn obstacle_uniform(
    obstacles: &Query<(&Transform, &Obstacle)>,
) -> ([Vec4; MAX_OBSTACLES], f32) {
    let mut circles = [Vec4::ZERO; MAX_OBSTACLES];
    let mut count = 0;
    for (transform, obstacle) in obstacles.iter().take(MAX_OBSTACLES) {
        let p = transform.translation.truncate() / VIEW_HEIGHT;
        circles[count] = Vec4::new(0.5 + p.x, 0.5 - p.y, obstacle.radius / VIEW_HEIGHT, 0.0);
        count += 1;
    }
    (circles, count as f32)
}

/// Run condition: an obstacle was added, moved, or removed this frame.
pub fn obstacles_changed(
    changed: Query<(), (With<Obstacle>, Or<(Added<Obstacle>, Changed<Transform>)>)>,
    mut removed: RemovedComponents<Obstacle>,
) -> bool {
    !changed.is_empty() || removed.read().next().is_some()
}

#[derive(Resource, Clone, Copy, PartialEq, Debug)]
pub struct RenderSettings {
    pub show_surface: bool,
    pub blob_size: f32,
    pub threshold: f32,
    pub softness: f32,
    pub rim_width: f32,
    pub rim_brightness: f32,
    pub diffuse: f32,
    pub specular: f32,
    pub shininess: f32,
    pub normal_strength: f32,
    pub gradient_radius: f32,
    pub lit_depth: f32,
    pub glow: f32,
    pub foam: bool,
    /// Speed at which a particle is fully foamy.
    pub foam_speed: f32,
    pub foam_opacity: f32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            show_surface: true,
            blob_size: 7.0,
            threshold: 0.5,
            softness: 0.05,
            rim_width: 0.15,
            rim_brightness: 0.3,
            diffuse: 0.8,
            specular: 0.8,
            shininess: 16.0,
            normal_strength: 6.0,
            gradient_radius: 12.0,
            lit_depth: 0.5,
            glow: 1.0,
            foam: true,
            foam_speed: 90.0,
            foam_opacity: 0.8,
        }
    }
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderSettings>()
            .insert_resource(ClearColor(CLEAR_COLOR))
            .add_plugins((
                particles::ParticleRenderPlugin,
                scene::ScenePlugin,
                surface::SurfacePlugin,
            ))
            .add_systems(Startup, spawn_camera);
    }
}

/// A projection framing `VIEW_HEIGHT` world units around the origin.
pub fn framed_projection() -> Projection {
    Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: VIEW_HEIGHT,
        },
        ..OrthographicProjection::default_2d()
    })
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        framed_projection(),
        Hdr,
        Bloom {
            intensity: 0.2,
            ..Bloom::NATURAL
        },
        MainCamera,
        IsDefaultUiCamera,
        RenderLayers::layer(0),
    ));
}
