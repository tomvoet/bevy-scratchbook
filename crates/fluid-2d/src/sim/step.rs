use bevy::prelude::*;

use super::{
    bounds,
    grid::{GridEntry, SpatialGrid},
    kernels::{
        density_to_pressure, near_density_to_pressure, poly6, spiky_pow2, spiky_pow2_derivative,
        spiky_pow3, spiky_pow3_derivative,
    },
    CursorInteraction, Density, GridSlot, Obstacle, PredictedPosition, SimParams, Velocity,
};

const LOOKAHEAD: f32 = (2.0 / super::SIM_HZ) as f32;
const MIN_DENSITY: f32 = 1e-4;
/// How hard a surface may lift water, as a fraction of gravity.
const LIFT: f32 = 1.0;
/// How much viscous drag a surface applies. Full no-slip brakes water sliding
/// down a wall to a crawl, since the band reads as a half space of still fluid.
const WALL_GRIP: f32 = 0.25;

pub fn apply_external_forces(
    mut particles: Query<(&Transform, &mut Velocity, &mut PredictedPosition)>,
    params: Res<SimParams>,
    time: Res<Time>,
    cursor: Res<CursorInteraction>,
) {
    let dt = time.delta_secs();
    let gravity = Vec2::new(0.0, -params.gravity);
    let interaction = cursor.0;
    let radius = params.interaction_radius;
    let air_drag = params.air_drag;

    particles
        .par_iter_mut()
        .for_each(|(transform, mut velocity, mut predicted)| {
            let position = transform.translation.truncate();
            let v = velocity.0;
            // Quadratic drag mainly brakes fast thin splashes.
            let mut acceleration = gravity - v * v.length() * air_drag;

            if let Some((point, strength)) = interaction {
                let offset = point - position;
                let distance = offset.length();
                if distance < radius {
                    let direction = if distance > 0.0 {
                        offset / distance
                    } else {
                        Vec2::ZERO
                    };
                    let falloff = 1.0 - distance / radius;
                    acceleration += (direction * strength - v) * falloff;
                }
            }

            velocity.0 += acceleration * dt;
            predicted.0 = position + velocity.0 * LOOKAHEAD;
        });
}

pub fn sync_boundary(
    mut boundary: ResMut<bounds::Boundary>,
    obstacles: Query<(&Transform, &Obstacle)>,
    params: Res<SimParams>,
) {
    boundary.sync(params.mass, params.target_density, params.smoothing_radius);
    // Pillars sit where the fluid is looking, same as the predicted positions.
    boundary.fill_pillars(obstacles.iter().map(|(transform, obstacle)| {
        (
            transform.translation.truncate() + obstacle.velocity * LOOKAHEAD,
            obstacle.radius,
            obstacle.velocity,
        )
    }));
}

pub fn build_grid(
    mut grid: ResMut<SpatialGrid>,
    mut particles: Query<(Entity, &PredictedPosition, &Velocity, &Density, &mut GridSlot)>,
    boundary: Res<bounds::Boundary>,
    params: Res<SimParams>,
) {
    grid.rebuild(
        params.smoothing_radius,
        particles
            .iter()
            .map(|(entity, position, velocity, density, _)| GridEntry {
                entity,
                position: position.0,
                velocity: velocity.0,
                density: *density,
            })
            // Walls last, so the slots below still line up with the query.
            .chain(boundary.positions().iter().map(|&position| GridEntry {
                entity: Entity::PLACEHOLDER,
                position,
                velocity: Vec2::ZERO,
                density: Density::default(),
            }))
            .chain(
                boundary
                    .pillars()
                    .iter()
                    .map(|&(position, velocity)| GridEntry {
                        entity: Entity::PLACEHOLDER,
                        position,
                        velocity,
                        density: Density::default(),
                    }),
            ),
    );

    // Same query as above, so the slots line up with it.
    for (slot, (.., mut grid_slot)) in grid.slots().iter().zip(particles.iter_mut()) {
        grid_slot.0 = *slot;
    }
}

pub fn refresh_densities(mut grid: ResMut<SpatialGrid>, particles: Query<(&GridSlot, &Density)>) {
    for (slot, density) in &particles {
        grid.set_density(slot.0, *density);
    }
}

pub fn calculate_densities(
    mut particles: Query<(&PredictedPosition, &mut Density)>,
    grid: Res<SpatialGrid>,
    params: Res<SimParams>,
) {
    let h = params.smoothing_radius;
    let mass = params.mass;

    particles
        .par_iter_mut()
        .for_each(|(position, mut density)| {
            let mut sum = Density::default();

            grid.for_each_neighbour(position.0, |other, distance| {
                let weight = mass * spiky_pow2(distance, h);
                sum.value += weight;
                sum.near += mass * spiky_pow3(distance, h);
                if other.entity != Entity::PLACEHOLDER {
                    sum.fluid += weight;
                }
            });

            *density = sum;
        });
}

/// Pressure, near pressure, and viscosity in one neighbour sweep.
pub fn apply_interactions(
    mut particles: Query<(Entity, &PredictedPosition, &Density, &mut Velocity)>,
    grid: Res<SpatialGrid>,
    params: Res<SimParams>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let h = params.smoothing_radius;
    let mass = params.mass;
    let lift = params.gravity * LIFT;
    let alone = mass * spiky_pow2(0.0, h);
    let (target, k, k_near, cohesion, viscosity) = (
        params.target_density,
        params.pressure_multiplier,
        params.near_pressure_multiplier,
        params.cohesion,
        params.viscosity,
    );

    particles
        .par_iter_mut()
        .for_each(|(entity, position, density, mut velocity)| {
            let position = position.0;
            let v = velocity.0;
            let own = *density;
            let pressure = density_to_pressure(own.value, target, k, cohesion);
            let near_pressure = near_density_to_pressure(own.near, k_near);

            let mut force = Vec2::ZERO;
            let mut pull = Vec2::ZERO;
            let mut viscous = Vec2::ZERO;

            grid.for_each_neighbour(position, |other, distance| {
                if other.entity == entity {
                    return;
                }

                let direction = if distance > 0.0 {
                    (other.position - position) / distance
                } else {
                    Vec2::from_angle(rand::random::<f32>() * std::f32::consts::TAU)
                };

                // A surface mirrors whoever looks at it, so it never reads
                // denser than the fluid touching it.
                let wall = other.entity == Entity::PLACEHOLDER;
                let neighbour = if wall { own } else { other.density };

                let shared_pressure =
                    (pressure + density_to_pressure(neighbour.value, target, k, cohesion)) / 2.0;
                let shared_near_pressure =
                    (near_pressure + near_density_to_pressure(neighbour.near, k_near)) / 2.0;

                // Derivatives are negative: positive pressure pushes away from `other`.
                let term = direction * spiky_pow2_derivative(distance, h) * shared_pressure * mass
                    / neighbour.value.max(MIN_DENSITY);
                if wall && shared_pressure < 0.0 {
                    pull += term;
                } else {
                    force += term;
                }
                force +=
                    direction * spiky_pow3_derivative(distance, h) * shared_near_pressure * mass
                        / neighbour.near.max(MIN_DENSITY);
                let grip = if wall { WALL_GRIP } else { 1.0 };
                viscous += (other.velocity - v) * poly6(distance, h) * grip;
            });

            // Only water that is part of a body wets a surface, so a lone
            // droplet falls off the ceiling instead of hanging there.
            let wetness = ((own.fluid - alone * 1.2) / (alone * 0.9)).clamp(0.0, 1.0);
            let density = own.value.max(MIN_DENSITY);
            let mut pull = pull * wetness;
            // And what is left can only lift so hard, so a sheet drains too.
            let limit = lift * density;
            if pull.y > limit {
                pull *= limit / pull.y;
            }
            let acceleration = (force + pull) / density + viscous * viscosity;
            velocity.0 += acceleration * dt;
        });
}

pub fn integrate(mut particles: Query<(&mut Transform, &mut Velocity)>, time: Res<Time>) {
    let dt = time.delta_secs();

    particles
        .par_iter_mut()
        .for_each(|(mut transform, mut velocity)| {
            let mut position = transform.translation.truncate() + velocity.0 * dt;
            bounds::resolve_collision(&mut position, &mut velocity.0);
            transform.translation = position.extend(0.0);
        });
}

pub fn collide_obstacles(
    obstacles: Query<(&Transform, &Obstacle)>,
    mut particles: Query<(&mut Transform, &mut Velocity), Without<Obstacle>>,
) {
    let circles: Vec<(Vec2, Obstacle)> = obstacles
        .iter()
        .map(|(transform, obstacle)| (transform.translation.truncate(), *obstacle))
        .collect();
    if circles.is_empty() {
        return;
    }

    particles
        .par_iter_mut()
        .for_each(|(mut transform, mut velocity)| {
            let mut position = transform.translation.truncate();
            for &(center, obstacle) in &circles {
                bounds::resolve_circle_collision(
                    &mut position,
                    &mut velocity.0,
                    center,
                    obstacle.radius,
                    obstacle.velocity,
                );
            }
            transform.translation = position.extend(0.0);
        });
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::sim::{
        bounds::BOUNDS, Particle, ParticleBundle, SimPlugin, Velocity, SIM_HZ, SPACING,
    };

    /// An empty tank holding only what `place` puts in it.
    fn ceiling_scene(place: &dyn Fn(i32, i32) -> Vec2) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1.0 / SIM_HZ,
            )))
            .init_resource::<ButtonInput<KeyCode>>()
            .add_plugins(SimPlugin);
        app.finish();
        app.update();

        let spawned: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<Particle>>()
            .iter(app.world())
            .collect();
        for entity in spawned {
            app.world_mut().entity_mut(entity).despawn();
        }
        for x in 0..7 {
            for y in 0..4 {
                app.world_mut().spawn(ParticleBundle::at_rest(place(x, y)));
            }
        }
        app
    }

    #[test]
    fn droplets_dont_stick_to_the_ceiling() {
        let mut app = ceiling_scene(&|x, y| {
            Vec2::new(x as f32 * 25.0 - 75.0, BOUNDS - 1.5 - y as f32 * 25.0)
        });

        for _ in 0..((0.5 * SIM_HZ) as usize) {
            app.update();
        }

        let highest = app
            .world_mut()
            .query_filtered::<&Transform, With<Particle>>()
            .iter(app.world())
            .map(|t| t.translation.y)
            .fold(f32::MIN, f32::max);
        assert!(highest < BOUNDS - 8.0, "still hanging at {highest}");
    }

    /// A sheet reads as a body, so wetness alone won't drop it.
    #[test]
    fn sheets_dont_stick_to_the_ceiling() {
        let mut app = ceiling_scene(&|x, y| {
            Vec2::new(x as f32 * SPACING - 6.0, BOUNDS - 1.5 - y as f32 * SPACING)
        });
        for _ in 0..((1.0 * SIM_HZ) as usize) {
            app.update();
        }
        let highest = app
            .world_mut()
            .query_filtered::<&Transform, With<Particle>>()
            .iter(app.world())
            .map(|t| t.translation.y)
            .fold(f32::MIN, f32::max);
        assert!(highest < BOUNDS - 8.0, "sheet still hanging at {highest}");
    }

    #[test]
    fn water_slides_down_a_wall() {
        let mut app = ceiling_scene(&|x, y| {
            Vec2::new(BOUNDS - 1.0 - x as f32 * SPACING, 20.0 - y as f32 * SPACING)
        });
        for _ in 0..(SIM_HZ as usize) {
            app.update();
        }
        let outer: Vec<f32> = app
            .world_mut()
            .query_filtered::<&Transform, With<Particle>>()
            .iter(app.world())
            .map(|t| t.translation.truncate())
            .filter(|p| p.x > BOUNDS - 1.0 - SPACING / 2.0)
            .map(|p| p.y)
            .collect();
        let fell = 20.0 - outer.iter().sum::<f32>() / outer.len() as f32;
        // Frictionless walls give about 49 here, full no-slip about 35.
        assert!(fell > 40.0, "wall column only fell {fell}");
    }

    /// Guards against a surface doing net work on the fluid, which shows up as
    /// a convection cell under a pillar that never runs down.
    #[test]
    fn the_pool_settles_around_the_pillars() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1.0 / SIM_HZ,
            )))
            .init_resource::<ButtonInput<KeyCode>>()
            .add_plugins(SimPlugin);
        app.finish();

        for _ in 0..((20.0 * SIM_HZ) as usize) {
            app.update();
        }

        // Under the smaller preset pillar.
        let churn: Vec<f32> = app
            .world_mut()
            .query_filtered::<(&Transform, &Velocity), With<Particle>>()
            .iter(app.world())
            .filter(|(t, _)| {
                let p = t.translation.truncate();
                p.x > 53.0 && p.x < 77.0 && p.y > -62.0 && p.y < -47.0
            })
            .map(|(_, v)| v.0.length())
            .collect();
        let mean = churn.iter().sum::<f32>() / churn.len() as f32;
        assert!(mean < 4.0, "still churning at {mean}");
    }
}
