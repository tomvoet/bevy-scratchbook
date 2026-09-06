use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    render::RenderSettings,
    sim::{kernels::TARGET_DENSITY, ResetParticles, SimParams, SpawnLayout},
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_ui);
    }
}

fn draw_ui(
    mut contexts: EguiContexts,
    mut sim_params: ResMut<SimParams>,
    mut render_settings: ResMut<RenderSettings>,
    mut reset: EventWriter<ResetParticles>,
) {
    // Edit copies so the resources are only marked changed when a value moved.
    let mut sim = *sim_params;
    let mut render = *render_settings;

    egui::SidePanel::left("ui_panel").show(contexts.ctx_mut(), |ui| {
        ui.heading("Fluid");
        ui.label("Left drag: pull. Right drag: push. Drag pillars to move them.");
        ui.separator();

        ui.horizontal(|ui| {
            ui.radio_value(&mut sim.spawn_layout, SpawnLayout::DamBreak, "Dam break");
            ui.radio_value(&mut sim.spawn_layout, SpawnLayout::Drop, "Drop");
        });
        ui.checkbox(&mut sim.obstacles, "Obstacles");
        if ui.button("Reset particles (Space)").clicked() {
            reset.send_default();
        }
        ui.separator();

        ui.add(egui::Slider::new(&mut sim.gravity, 0.0..=300.0).text("Gravity"));
        ui.add(egui::Slider::new(&mut sim.mass, 0.1..=10.0).text("Mass"));
        // Also the grid cell size, so it must stay > 0.
        ui.add(egui::Slider::new(&mut sim.smoothing_radius, 1.0..=20.0).text("Smoothing Radius"));
        ui.add(
            egui::Slider::new(&mut sim.target_density, 0.0..=4.0 * TARGET_DENSITY)
                .text("Target Density"),
        );
        ui.add(
            egui::Slider::new(&mut sim.pressure_multiplier, 1.0..=1_000_000.0)
                .logarithmic(true)
                .text("Pressure Multiplier"),
        );
        ui.add(
            egui::Slider::new(&mut sim.near_pressure_multiplier, 1.0..=100_000.0)
                .logarithmic(true)
                .text("Near Pressure Multiplier"),
        );
        ui.add(egui::Slider::new(&mut sim.cohesion, 0.0..=1.0).text("Cohesion"));
        ui.add(egui::Slider::new(&mut sim.viscosity, 0.0..=100.0).text("Viscosity"));
        ui.add(egui::Slider::new(&mut sim.collision_damping, 0.0..=1.0).text("Wall Bounce"));
        ui.add(egui::Slider::new(&mut sim.wall_friction, 0.0..=30.0).text("Wall Friction"));
        ui.add(egui::Slider::new(&mut sim.air_drag, 0.0..=0.01).text("Air Drag"));
        ui.separator();
        ui.add(egui::Slider::new(&mut sim.interaction_radius, 5.0..=100.0).text("Cursor Radius"));
        ui.add(
            egui::Slider::new(&mut sim.interaction_strength, 0.0..=3000.0).text("Cursor Strength"),
        );

        ui.separator();
        ui.checkbox(&mut render.show_surface, "Surface rendering");
        ui.add(egui::Slider::new(&mut render.blob_size, 2.0..=16.0).text("Blob Size"));
        ui.add(egui::Slider::new(&mut render.threshold, 0.05..=0.95).text("Threshold"));
        ui.add(egui::Slider::new(&mut render.softness, 0.0..=0.2).text("Edge Softness"));
        ui.add(egui::Slider::new(&mut render.rim_width, 0.0..=0.5).text("Rim Width"));
        ui.add(egui::Slider::new(&mut render.rim_brightness, 0.0..=1.0).text("Rim Brightness"));
        ui.add(egui::Slider::new(&mut render.diffuse, 0.0..=2.0).text("Shading"));
        ui.add(egui::Slider::new(&mut render.specular, 0.0..=2.0).text("Specular"));
        ui.add(egui::Slider::new(&mut render.shininess, 2.0..=64.0).text("Shininess"));
        ui.add(egui::Slider::new(&mut render.normal_strength, 0.5..=20.0).text("Normal Strength"));
        ui.add(egui::Slider::new(&mut render.gradient_radius, 1.0..=24.0).text("Gradient Radius"));
        ui.add(egui::Slider::new(&mut render.lit_depth, 0.05..=1.0).text("Light Depth"));
        ui.add(egui::Slider::new(&mut render.glow, 0.0..=4.0).text("Glow"));
        ui.checkbox(&mut render.foam, "Foam");
        ui.add(egui::Slider::new(&mut render.foam_speed, 20.0..=300.0).text("Foam Speed"));
        ui.add(egui::Slider::new(&mut render.foam_opacity, 0.0..=1.0).text("Foam Opacity"));
        ui.separator();

        if ui.button("Reset parameters").clicked() {
            sim = SimParams {
                spawn_layout: sim.spawn_layout,
                ..default()
            };
            render = RenderSettings::default();
        }
    });

    if sim != *sim_params {
        *sim_params = sim;
    }
    if render != *render_settings {
        *render_settings = render;
    }
}
