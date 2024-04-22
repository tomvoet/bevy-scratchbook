use std::hash::Hash;

use bevy::prelude::*;
use rand::Rng;

use self::smoothing::{density_to_pressure, smoothing_kernel_derivative, MASS, TARGET_DENSITY};

mod smoothing;
pub mod systems;

pub const SPHERE_RADIUS: f32 = 1.0;

#[derive(Debug, Default, Component, Clone)]
pub struct Particle {
    pub id: uuid::Uuid,
    pub position: Vec3,
    pub velocity: Vec3,
    pub density: f32,
    pub predicted_position: Vec3,
}

impl PartialEq for Particle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Particle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for Particle {}

impl Particle {
    pub fn new(position: Vec3, velocity: Vec3) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            velocity,
            position,
            density: TARGET_DENSITY,
            predicted_position: position,
        }
    }

    pub fn random_velocity(position: Vec3) -> Self {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-1.0..1.0);
        let y = rng.gen_range(-1.0..1.0);

        let velocity = Vec3::new(x, y, 0.0);

        Self::new(position, velocity)
    }

    fn mesh() -> Mesh {
        Sphere::new(SPHERE_RADIUS).mesh().build()
    }

    pub fn apply_drag(&mut self, drag: f32) {
        self.velocity *= 1.0 - drag;
    }

    pub fn clamp_velocity(&mut self, max_speed: f32) {
        self.velocity = self.velocity.clamp_length_max(max_speed);
    }

    pub fn try_sleep(&mut self) {
        if self.velocity.length() < 0.05 {
            self.velocity = Vec3::ZERO;
        }
    }

    pub fn apply_force(&mut self, force: Vec3) {
        self.velocity += force;
    }

    pub(super) fn color(&self) -> Color {
        let hue = (self.velocity.length().min(4.0) / 4.0) * 120.0 + 240.0;
        Color::hsl(hue, 1.0, 0.5)
    }

    pub(super) fn calculate_density(
        &mut self,
        particles: &[&Particle],
        smoothing_radius: f32,
    ) -> f32 {
        let mut density = 0.0;

        for particle in particles.iter() {
            let distance = self.position.distance(particle.position);
            let influence = smoothing::smoothing_kernel(distance, smoothing_radius);
            density += MASS * influence
        }

        density
    }

    pub(super) fn calculate_pressure_force(
        &self,
        particles: &[&Particle],
        smoothing_radius: f32,
        target_density: f32,
        pressure_multiplier: f32,
    ) -> Vec3 {
        let mut pressure_force = Vec3::ZERO;

        for particle in particles.iter()
        /*.filter(|p| {
            let distance = self.position.distance(p.position);
            distance < SMOOTHING_RADIUS && p.id != self.id
        })  */
        {
            if particle.id == self.id {
                continue;
            }

            let distance = self.position.distance(particle.position);

            let direction = if distance > 0.0 {
                (particle.position - self.position).normalize()
            } else {
                let random = Vec3::new(rand::random(), rand::random(), 0.);
                random.normalize()
            };

            let slope = smoothing_kernel_derivative(distance, smoothing_radius);
            let density = particle.density;

            let shared_pressure =
                self.calculate_shared_pressure(particle, target_density, pressure_multiplier);
            let non_zero_density = density.max(0.0001);

            pressure_force -= shared_pressure * slope * direction * MASS / non_zero_density;
        }

        //particles
        //    .par_iter()
        //    .fold(
        //        || Vec3::ZERO,
        //        |mut acc_force, particle| {
        //            let distance = self.position.distance(particle.position);
        //            if distance < SMOOTHING_RADIUS && particle.id != self.id {
        //                let direction = if distance > 0.0 {
        //                    (particle.position - self.position).normalize()
        //                } else {
        //                    Vec3::new(rand::random(), rand::random(), 0.).normalize()
        //                };
        //
        //                let slope = smoothing_kernel_derivative(distance, smoothing_radius);
        //                let density = particle.density;
        //                let shared_pressure = self.calculate_shared_pressure(
        //                    particle,
        //                    target_density,
        //                    pressure_multiplier,
        //                );
        //                let non_zero_density = density.max(0.0001);
        //
        //                acc_force -= shared_pressure * slope * direction * MASS / non_zero_density;
        //            }
        //            acc_force
        //        },
        //    )
        //    .sum::<Vec3>()
        pressure_force
    }

    pub fn calculate_shared_pressure(
        &self,
        particle: &Particle,
        target_density: f32,
        pressure_multiplier: f32,
    ) -> f32 {
        let pressure_a = density_to_pressure(self.density, target_density, pressure_multiplier);
        let pressure_b = density_to_pressure(particle.density, target_density, pressure_multiplier);
        (pressure_a + pressure_b) / 2.0
    }

    pub fn to_new_particle_with_force(&self, force: Vec3) -> Self {
        let mut new_particle = self.clone();
        new_particle.apply_force(force);
        new_particle
    }
}
