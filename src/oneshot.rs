//use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};
//
//struct OneShotPlugin {
//    systems: Vec<Box<dyn System<In = (), Out = ()>>>,
//}
//
//impl OneShotPlugin {
//    pub fn new() -> Self {
//        Self {
//            systems: Vec::new(),
//        }
//    }
//
//    pub fn add_system(&mut self, system: impl System<In = (), Out = ()>) {
//        self.systems.push(Box::new(system));
//    }
//}
//
//impl Plugin for OneShotPlugin {
//    fn build(&self, app: &mut App) {
//        app.init_resource::<OneShot>()
//            .add_systems(Startup, setup_oneshot)
//            .add_systems(Update, trigger_oneshot);
//    }
//}
//
//#[derive(Default, Resource)]
//struct OneShot(HashMap<&'static str, SystemId>);
//
//#[derive(Event)]
//pub struct TriggerOneShot(pub &'static str);
//
//impl OneShot {
//    fn insert(&mut self, key: &'static str, system_id: SystemId) {
//        self.0.insert(key, system_id);
//    }
//
//    pub fn register_system(&mut self, system: impl System) {}
//}
//
////fn setup_oneshot(world: &mut World) {
////    let reset_particles_system = world.register_system(particle::spawn_particles);
////
////    let mut oneshot = OneShot::default();
////
////    oneshot.insert("reset_particles", reset_particles_system);
////
////    world.insert_resource(oneshot);
////}
//
//pub fn register_systems(mut oneshot: ResMut<OneShot>) {
//    systems.iter().for_each(|system| {
//        oneshot.insert(system.name(), system.id());
//    });
//}
//
//fn trigger_oneshot(
//    mut commands: Commands,
//    oneshot: Res<OneShot>,
//    mut events: EventReader<TriggerOneShot>,
//) {
//    for TriggerOneShot(key) in events.read() {
//        if let Some(system_id) = oneshot.0.get(key) {
//            commands.run_system(*system_id);
//        } else {
//            eprintln!("Failed to get {} system", key);
//        }
//    }
//}
//
