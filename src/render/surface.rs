use bevy::{
    ecs::schedule::common_conditions::resource_changed,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages,
        },
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use crate::sim::{bounds::BOUNDS, Obstacle};

use super::{
    framed_camera, obstacle_uniform, obstacles_changed, scene::CORNER_RADIUS, RenderSettings,
    FIELD_LAYER, MAX_OBSTACLES, VIEW_HEIGHT,
};

const FIELD_RESOLUTION: u32 = 1024;

pub struct SurfacePlugin;

impl Plugin for SurfacePlugin {
    fn build(&self, app: &mut App) {
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
    fn fragment_shader() -> ShaderRef {
        "shaders/fluid_surface.wgsl".into()
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
    let size = Extent3d {
        width: FIELD_RESOLUTION,
        height: FIELD_RESOLUTION,
        depth_or_array_layers: 1,
    };
    let mut field = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("fluid field"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    field.resize(size);
    let field = images.add(field);

    let mut field_camera = framed_camera();
    field_camera.camera.order = -1;
    field_camera.camera.target = RenderTarget::Image(field.clone());
    field_camera.camera.clear_color = ClearColorConfig::Custom(Color::NONE);
    commands.spawn((field_camera, RenderLayers::layer(FIELD_LAYER)));

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
        MaterialMesh2dBundle {
            mesh: meshes.add(Rectangle::new(VIEW_HEIGHT, VIEW_HEIGHT)).into(),
            material,
            transform: Transform::from_xyz(0.0, 0.0, -1.0),
            ..default()
        },
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
    if let Some(material) = materials.get_mut(&handle.0) {
        material.apply(&settings);
    }
}

fn update_obstacles(
    obstacles: Query<(&Transform, &Obstacle)>,
    handle: Res<SurfaceMaterialHandle>,
    mut materials: ResMut<Assets<FluidSurfaceMaterial>>,
) {
    if let Some(material) = materials.get_mut(&handle.0) {
        (material.obstacles, material.style.z) = obstacle_uniform(&obstacles);
    }
}
