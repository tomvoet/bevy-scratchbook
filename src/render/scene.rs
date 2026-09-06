use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat},
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use crate::sim::bounds::BOUNDS;

use super::VIEW_HEIGHT;

const BACKGROUND_TOP: Color = Color::rgb(0.075, 0.09, 0.14);
const BACKGROUND_BOTTOM: Color = Color::rgb(0.015, 0.018, 0.03);
const TANK_FILL: Color = Color::rgb(0.05, 0.065, 0.1);
const WALL_COLOR: Color = Color::rgb(0.55, 0.62, 0.72);
pub const WALL_THICKNESS: f32 = 2.0;
pub const CORNER_RADIUS: f32 = 6.0;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<TankMaterial>::default())
            .add_systems(Startup, spawn_scene);
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TankMaterial {
    #[uniform(0)]
    shape: Vec4,
    #[uniform(1)]
    fill_color: Color,
    #[uniform(2)]
    wall_color: Color,
    #[uniform(3)]
    style: Vec4,
}

impl Material2d for TankMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/tank.wgsl".into()
    }
}

impl TankMaterial {
    fn new(walls: bool) -> Self {
        Self {
            shape: Vec4::new(
                BOUNDS / VIEW_HEIGHT,
                WALL_THICKNESS / VIEW_HEIGHT,
                CORNER_RADIUS / VIEW_HEIGHT,
                0.0,
            ),
            fill_color: TANK_FILL,
            wall_color: WALL_COLOR,
            style: Vec4::new(if walls { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0),
        }
    }
}

pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TankMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(SpriteBundle {
        texture: images.add(gradient_texture(BACKGROUND_TOP, BACKGROUND_BOTTOM)),
        sprite: Sprite {
            custom_size: Some(Vec2::new(VIEW_HEIGHT * 4.0, VIEW_HEIGHT)),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, -5.0),
        ..default()
    });

    // Fill behind the fluid, walls above it.
    let quad = meshes.add(Rectangle::new(VIEW_HEIGHT, VIEW_HEIGHT));
    for (walls, z) in [(false, -2.0), (true, 1.0)] {
        commands.spawn(MaterialMesh2dBundle {
            mesh: quad.clone().into(),
            material: materials.add(TankMaterial::new(walls)),
            transform: Transform::from_xyz(0.0, 0.0, z),
            ..default()
        });
    }
}

fn gradient_texture(top: Color, bottom: Color) -> Image {
    const STEPS: u32 = 256;
    let (top, bottom) = (top.rgba_to_vec4(), bottom.rgba_to_vec4());

    let mut data = Vec::with_capacity((STEPS * 4) as usize);
    for i in 0..STEPS {
        let t = i as f32 / (STEPS - 1) as f32;
        let c = top.lerp(bottom, t);
        data.extend_from_slice(&[
            (c.x * 255.0) as u8,
            (c.y * 255.0) as u8,
            (c.z * 255.0) as u8,
            255,
        ]);
    }

    Image::new(
        Extent3d {
            width: 1,
            height: STEPS,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
