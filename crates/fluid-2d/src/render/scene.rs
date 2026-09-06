use bevy::{
    asset::{embedded_asset, RenderAssetUsages},
    color::ColorToComponents,
    mesh::Mesh2d,
    prelude::*,
    render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin, MeshMaterial2d},
};

use crate::sim::{bounds::BOUNDS, Obstacle};

use super::{obstacle_uniform, obstacles_changed, MAX_OBSTACLES, VIEW_HEIGHT};

const BACKGROUND_TOP: Color = Color::srgb(0.075, 0.09, 0.14);
const BACKGROUND_BOTTOM: Color = Color::srgb(0.015, 0.018, 0.03);
const TANK_FILL: Color = Color::srgb(0.05, 0.065, 0.1);
const WALL_COLOR: Color = Color::srgb(0.55, 0.62, 0.72);
pub const WALL_THICKNESS: f32 = 2.0;
pub const CORNER_RADIUS: f32 = 6.0;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/tank.wgsl");
        app.add_plugins(Material2dPlugin::<TankMaterial>::default())
            .add_systems(Startup, spawn_scene)
            .add_systems(Update, update_obstacles.run_if(obstacles_changed));
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TankMaterial {
    #[uniform(0)]
    shape: Vec4,
    #[uniform(1)]
    fill_color: LinearRgba,
    #[uniform(2)]
    wall_color: LinearRgba,
    /// x: 0 = fill, 1 = walls; y: obstacle count.
    #[uniform(3)]
    style: Vec4,
    #[uniform(4)]
    obstacles: [Vec4; MAX_OBSTACLES],
}

impl Material2d for TankMaterial {
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://fluid_2d/render/shaders/tank.wgsl".into()
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
            fill_color: TANK_FILL.into(),
            wall_color: WALL_COLOR.into(),
            style: Vec4::new(if walls { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0),
            obstacles: [Vec4::ZERO; MAX_OBSTACLES],
        }
    }
}

#[derive(Resource)]
struct WallMaterialHandle(Handle<TankMaterial>);

fn update_obstacles(
    obstacles: Query<(&Transform, &Obstacle)>,
    handle: Res<WallMaterialHandle>,
    mut materials: ResMut<Assets<TankMaterial>>,
) {
    if let Some(mut material) = materials.get_mut(&handle.0) {
        (material.obstacles, material.style.y) = obstacle_uniform(&obstacles);
    }
}

pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TankMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn((
        Sprite {
            image: images.add(gradient_texture(BACKGROUND_TOP, BACKGROUND_BOTTOM)),
            custom_size: Some(Vec2::new(VIEW_HEIGHT * 4.0, VIEW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -5.0),
    ));

    // Fill behind the fluid, walls above it.
    let quad = meshes.add(Rectangle::new(VIEW_HEIGHT, VIEW_HEIGHT));
    for (walls, z) in [(false, -2.0), (true, 1.0)] {
        let material = materials.add(TankMaterial::new(walls));
        if walls {
            commands.insert_resource(WallMaterialHandle(material.clone()));
        }
        commands.spawn((
            Mesh2d(quad.clone()),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, z),
        ));
    }
}

fn gradient_texture(top: Color, bottom: Color) -> Image {
    const STEPS: u32 = 256;
    let (top, bottom) = (top.to_srgba().to_vec4(), bottom.to_srgba().to_vec4());

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
