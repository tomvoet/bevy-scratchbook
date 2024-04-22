use crate::particle::Particle;
use bevy::prelude::*;

#[derive(Resource)]
pub struct Drag(pub f32);

pub fn drag_system(mut particles: Query<&mut Particle>, drag: Res<Drag>) {
    //for mut particle in particles.iter_mut() {
    //    particle.apply_drag(drag.0);
    //    //particle.try_sleep();
    //}
    particles.par_iter_mut().for_each(|mut particle| {
        particle.apply_drag(drag.0);
        //particle.try_sleep();
    });
}
