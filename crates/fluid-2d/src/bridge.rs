use std::sync::{
    atomic::{AtomicU32, Ordering},
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};

use bevy::prelude::*;

use crate::{
    controls::{edit, Action, Params},
    render::RenderSettings,
    sim::{ObstacleCommand, ResetParticles, SimParams},
};

pub enum ControlMessage {
    Params(Params),
    Action(Action),
}

/// Handed to a panel that lives outside the app, like the leptos one.
#[derive(Clone)]
pub struct ControlSender(Sender<ControlMessage>);

impl ControlSender {
    pub fn params(&self, params: Params) {
        let _ = self.0.send(ControlMessage::Params(params));
    }

    pub fn action(&self, action: Action) {
        let _ = self.0.send(ControlMessage::Action(action));
    }
}

/// Readback the other way, for a panel that wants to show frame timings.
/// Tenths of a frame per second, so it fits an atomic.
#[derive(Resource, Clone, Default)]
pub struct SimStats(Arc<AtomicU32>);

impl SimStats {
    pub fn fps(&self) -> f32 {
        self.0.load(Ordering::Relaxed) as f32 / 10.0
    }

    pub fn set_fps(&self, fps: f32) {
        self.0.store((fps * 10.0) as u32, Ordering::Relaxed);
    }
}

#[derive(Resource)]
struct Inbox(Mutex<Receiver<ControlMessage>>);

pub struct ControlBridgePlugin {
    receiver: Mutex<Option<Receiver<ControlMessage>>>,
    stats: SimStats,
}

/// Wires an outside panel to the sim. The sender goes to the panel, the plugin
/// to the app, so a second sim just makes its own pair.
pub fn control_channel() -> (ControlSender, SimStats, ControlBridgePlugin) {
    let (sender, receiver) = channel();
    let stats = SimStats::default();
    (
        ControlSender(sender),
        stats.clone(),
        ControlBridgePlugin {
            receiver: Mutex::new(Some(receiver)),
            stats,
        },
    )
}

impl Plugin for ControlBridgePlugin {
    fn build(&self, app: &mut App) {
        let receiver = self
            .receiver
            .lock()
            .unwrap()
            .take()
            .expect("control bridge added twice");
        app.insert_resource(Inbox(Mutex::new(receiver)))
            .insert_resource(self.stats.clone())
            .add_systems(Update, drain);
    }
}

fn drain(
    inbox: Res<Inbox>,
    mut sim: ResMut<SimParams>,
    mut render: ResMut<RenderSettings>,
    mut reset: MessageWriter<ResetParticles>,
    mut obstacles: MessageWriter<ObstacleCommand>,
) {
    let receiver = inbox.0.lock().unwrap();
    for message in receiver.try_iter() {
        match message {
            ControlMessage::Params(next) => edit(&mut sim, &mut render, |params| *params = next),
            ControlMessage::Action(action) => {
                apply(action, &mut sim, &mut render, &mut reset, &mut obstacles)
            }
        }
    }
}

/// Shared by both panels so a button means the same thing either way.
pub fn apply(
    action: Action,
    sim: &mut SimParams,
    render: &mut RenderSettings,
    reset: &mut MessageWriter<ResetParticles>,
    obstacles: &mut MessageWriter<ObstacleCommand>,
) {
    match action {
        Action::ResetParticles => {
            reset.write_default();
        }
        Action::ResetPillars => {
            obstacles.write(ObstacleCommand::ResetPreset);
        }
        Action::ClearPillars => {
            obstacles.write(ObstacleCommand::Clear);
        }
        Action::ResetParams => edit(sim, render, |params| params.reset()),
    }
}
