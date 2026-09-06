use bevy::{
    core_pipeline::bloom::BloomSettings,
    prelude::*,
    render::{camera::ScalingMode, view::RenderLayers},
};

use crate::sim::bounds::BOUNDS;

pub mod particles;
pub mod scene;
pub mod surface;

/// Visible height in world units; also the side length of the surface quad.
pub const VIEW_HEIGHT: f32 = BOUNDS * 2.0 + 50.0;
/// Particles live here; the field camera sees only this layer.
pub const FIELD_LAYER: u8 = 1;
const CLEAR_COLOR: Color = Color::rgb(0.015, 0.018, 0.03);

#[derive(Component)]
pub struct MainCamera;

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

/// A camera framing `VIEW_HEIGHT` world units around the origin.
pub fn framed_camera() -> Camera2dBundle {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedVertical(VIEW_HEIGHT);
    camera
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = framed_camera();
    camera.camera.hdr = true;
    commands.spawn((
        camera,
        BloomSettings {
            intensity: 0.2,
            ..BloomSettings::NATURAL
        },
        MainCamera,
        RenderLayers::layer(0),
    ));
}
