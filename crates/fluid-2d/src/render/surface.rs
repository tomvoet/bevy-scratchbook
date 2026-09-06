use bevy::{
    asset::embedded_asset,
    camera::{visibility::RenderLayers, ClearColorConfig, RenderTarget},
    ecs::schedule::common_conditions::resource_changed,
    mesh::Mesh2d,
    prelude::*,
    render::render_resource::{AsBindGroup, TextureFormat},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin, MeshMaterial2d},
};

use crate::sim::{bounds::BOUNDS, Obstacle};

use super::{
    framed_projection, obstacle_uniform, obstacles_changed, scene::CORNER_RADIUS, RenderSettings,
    FIELD_LAYER, MAX_OBSTACLES, VIEW_HEIGHT,
};

const FIELD_RESOLUTION: u32 = 1024;

pub struct SurfacePlugin;

impl Plugin for SurfacePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/fluid_surface.wgsl");
        app.add_plugins(Material2dPlugin::<FluidSurfaceMaterial>::default())
            .add_systems(Startup, setup_surface)
            .add_systems(
                Update,
                (
                    (toggle_surface, update_material).run_if(resource_changed::<RenderSettings>),
                    update_obstacles.run_if(obstacles_changed),
                ),
            );
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct FluidSurfaceMaterial {
    #[texture(0)]
    #[sampler(1)]
    field: Handle<Image>,
    #[uniform(2)]
    params: Vec4,
    #[uniform(3)]
    clip: Vec4,
    #[uniform(4)]
    lighting: Vec4,
    #[uniform(5)]
    style: Vec4,
    #[uniform(6)]
    obstacles: [Vec4; MAX_OBSTACLES],
}

impl Material2d for FluidSurfaceMaterial {
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://fluid_2d/render/shaders/fluid_surface.wgsl".into()
    }
}

impl FluidSurfaceMaterial {
    fn apply(&mut self, s: &RenderSettings) {
        self.params = Vec4::new(s.threshold, s.softness, s.rim_width, s.rim_brightness);
        self.clip = Vec4::new(
            BOUNDS / VIEW_HEIGHT,
            s.normal_strength,
            s.gradient_radius,
            s.lit_depth,
        );
        self.lighting = Vec4::new(
            s.diffuse,
            s.specular,
            s.shininess,
            1.0 / FIELD_RESOLUTION as f32,
        );
        self.style.x = CORNER_RADIUS / VIEW_HEIGHT;
        self.style.y = s.glow;
    }
}

#[derive(Component)]
struct SurfaceQuad;

#[derive(Resource)]
struct SurfaceMaterialHandle(Handle<FluidSurfaceMaterial>);

fn setup_surface(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FluidSurfaceMaterial>>,
    settings: Res<RenderSettings>,
) {
    let field = images.add(Image::new_target_texture(
        FIELD_RESOLUTION,
        FIELD_RESOLUTION,
        TextureFormat::Bgra8UnormSrgb,
        None,
    ));

    commands.spawn((
        Camera2d,
        framed_projection(),
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        RenderTarget::Image(field.clone().into()),
        RenderLayers::layer(FIELD_LAYER),
    ));

    let mut material = FluidSurfaceMaterial {
        field,
        params: Vec4::ZERO,
        clip: Vec4::ZERO,
        lighting: Vec4::ZERO,
        style: Vec4::ZERO,
        obstacles: [Vec4::ZERO; MAX_OBSTACLES],
    };
    material.apply(&settings);
    let material = materials.add(material);
    commands.insert_resource(SurfaceMaterialHandle(material.clone()));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(VIEW_HEIGHT, VIEW_HEIGHT))),
        MeshMaterial2d(material),
        Transform::from_xyz(0.0, 0.0, -1.0),
        SurfaceQuad,
    ));
}

fn toggle_surface(
    settings: Res<RenderSettings>,
    mut quads: Query<&mut Visibility, With<SurfaceQuad>>,
) {
    for mut visibility in &mut quads {
        *visibility = if settings.show_surface {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn update_material(
    settings: Res<RenderSettings>,
    handle: Res<SurfaceMaterialHandle>,
    mut materials: ResMut<Assets<FluidSurfaceMaterial>>,
) {
    if let Some(mut material) = materials.get_mut(&handle.0) {
        material.apply(&settings);
    }
}

fn update_obstacles(
    obstacles: Query<(&Transform, &Obstacle)>,
    handle: Res<SurfaceMaterialHandle>,
    mut materials: ResMut<Assets<FluidSurfaceMaterial>>,
) {
    if let Some(mut material) = materials.get_mut(&handle.0) {
        (material.obstacles, material.style.z) = obstacle_uniform(&obstacles);
    }
}
