use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::RenderLayers,
    ecs::schedule::common_conditions::resource_changed,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::sim::{Density, Particle, SimParams, Velocity, PARTICLE_RADIUS};

use super::{MainCamera, RenderSettings, FIELD_LAYER};

const COLOR_STOPS: [Vec3; 3] = [
    Vec3::new(0.08, 0.35, 1.0),
    Vec3::new(0.35, 0.85, 1.0),
    Vec3::new(0.95, 1.0, 1.0),
];
const COLOR_MAX_SPEED: f32 = 150.0;
const TEXTURE_SIZE: u32 = 32;
const FOAM_SIZE: f32 = 1.6;
/// Density below this fraction of target counts as spray.
const FOAM_SPARSE: f32 = 0.6;

pub struct ParticleRenderPlugin;

impl Plugin for ParticleRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_assets).add_systems(
            Update,
            (
                attach_sprites,
                apply_render_mode.run_if(resource_changed::<RenderSettings>),
                color_by_speed,
                update_foam,
            ),
        );
    }
}

/// A particle's foam sprite, a child drawn above the surface.
#[derive(Component)]
struct Foam;

#[derive(Resource)]
pub struct ParticleAssets {
    dot: Handle<Image>,
    blob: Handle<Image>,
}

impl ParticleAssets {
    fn sprite(&self, settings: &RenderSettings) -> (Handle<Image>, f32) {
        if settings.show_surface {
            (self.blob.clone(), settings.blob_size)
        } else {
            (self.dot.clone(), 2.0 * PARTICLE_RADIUS)
        }
    }
}

fn setup_assets(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.insert_resource(ParticleAssets {
        dot: images.add(radial_texture(|d| {
            (1.0 - d) * TEXTURE_SIZE as f32 / 2.0 + 0.5
        })),
        blob: images.add(radial_texture(|d| (1.0 - d * d).powi(2))),
    });
}

/// White with alpha `falloff(d)`, `d` in 0..1 from centre to edge.
fn radial_texture(falloff: impl Fn(f32) -> f32) -> Image {
    let center = TEXTURE_SIZE as f32 / 2.0;

    let mut data = Vec::with_capacity((TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize);
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            let offset = Vec2::new(x as f32 + 0.5 - center, y as f32 + 0.5 - center);
            let d = (offset.length() / center).min(1.0);
            let alpha = falloff(d).clamp(0.0, 1.0);
            data.extend_from_slice(&[255, 255, 255, (alpha * 255.0) as u8]);
        }
    }

    Image::new(
        Extent3d {
            width: TEXTURE_SIZE,
            height: TEXTURE_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn attach_sprites(
    mut commands: Commands,
    particles: Query<Entity, Added<Particle>>,
    assets: Res<ParticleAssets>,
    settings: Res<RenderSettings>,
) {
    let (texture, size) = assets.sprite(&settings);
    for entity in &particles {
        commands
            .entity(entity)
            .insert((
                Sprite {
                    image: texture.clone(),
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Visibility::default(),
                RenderLayers::layer(FIELD_LAYER),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Sprite {
                        image: assets.dot.clone(),
                        color: Color::NONE,
                        custom_size: Some(Vec2::splat(FOAM_SIZE)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.5),
                    Foam,
                    RenderLayers::layer(0),
                ));
            });
    }
}

fn apply_render_mode(
    settings: Res<RenderSettings>,
    assets: Res<ParticleAssets>,
    mut cameras: Query<&mut RenderLayers, With<MainCamera>>,
    mut particles: Query<&mut Sprite, With<Particle>>,
) {
    for mut layers in &mut cameras {
        *layers = if settings.show_surface {
            RenderLayers::layer(0)
        } else {
            RenderLayers::from_layers(&[0, FIELD_LAYER])
        };
    }

    let (texture, size) = assets.sprite(&settings);
    for mut sprite in &mut particles {
        sprite.image = texture.clone();
        sprite.custom_size = Some(Vec2::splat(size));
    }
}

fn color_by_speed(mut particles: Query<(&Velocity, &mut Sprite)>) {
    particles.par_iter_mut().for_each(|(velocity, mut sprite)| {
        let t = (velocity.0.length() / COLOR_MAX_SPEED).clamp(0.0, 1.0);
        let scaled = t * (COLOR_STOPS.len() - 1) as f32;
        let i = (scaled as usize).min(COLOR_STOPS.len() - 2);
        let rgb = COLOR_STOPS[i].lerp(COLOR_STOPS[i + 1], scaled - i as f32);
        sprite.color = Color::srgb(rgb.x, rgb.y, rgb.z);
    });
}

fn update_foam(
    mut foam: Query<(&ChildOf, &mut Sprite, &mut Visibility), With<Foam>>,
    particles: Query<(&Velocity, &Density)>,
    settings: Res<RenderSettings>,
    params: Res<SimParams>,
) {
    let sparse_density = FOAM_SPARSE * params.target_density;

    foam.par_iter_mut()
        .for_each(|(child_of, mut sprite, mut visibility)| {
            let Ok((velocity, density)) = particles.get(child_of.parent()) else {
                return;
            };
            let amount = if settings.foam {
                let fast = smoothstep(
                    0.5 * settings.foam_speed,
                    settings.foam_speed,
                    velocity.0.length(),
                );
                let sparse = smoothstep(sparse_density, 0.5 * sparse_density, density.value);
                fast.max(sparse) * settings.foam_opacity
            } else {
                0.0
            };
            // Invisible foam is hidden outright so the renderer never queues it.
            let wanted = if amount > 0.01 {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if *visibility != wanted {
                *visibility = wanted;
            }
            if amount > 0.01 {
                sprite.color = Color::srgba(1.0, 1.0, 1.0, amount);
            }
        });
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
