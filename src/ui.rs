use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>().add_systems(Update, setup_ui);
    }
}

#[derive(Resource)]
pub struct UiState {
    pub mass: f32,
    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub gravity: f32,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            mass: 1.0,
            smoothing_radius: 14.0,
            target_density: 0.02,
            pressure_multiplier: 25.0,
            //gravity: 9.81,
            gravity: 0.0,
        }
    }
}

fn setup_ui(mut contexts: EguiContexts, mut state: ResMut<UiState>) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("ui_panel").show(ctx, |ui| {
        ui.heading("Particle System");
        ui.add(egui::Slider::new(&mut state.mass, 0.0..=10.0).text("Mass"));
        ui.add(egui::Slider::new(&mut state.smoothing_radius, 0.0..=20.0).text("Smoothing Radius"));
        ui.add(egui::Slider::new(&mut state.target_density, 0.0..=20.0).text("Target Density"));
        ui.add(
            egui::Slider::new(&mut state.pressure_multiplier, 0.0..=40.0)
                .text("Pressure Multiplier"),
        );
        ui.add(egui::Slider::new(&mut state.gravity, 0.0..=20.0).text("Gravity"));
    });
}
