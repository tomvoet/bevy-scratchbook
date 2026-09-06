use bevy::{
    feathers::{
        controls::{
            FeathersButton, FeathersCheckbox, FeathersRadio, FeathersScrollbar, FeathersSlider,
        },
        dark_theme::create_dark_theme,
        display::{label, label_small},
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
        tokens, FeathersPlugins,
    },
    picking::hover::Hovered,
    prelude::*,
    ui::Checked,
    ui_widgets::{
        Activate, ControlOrientation, RadioGroup, ScrollArea, SliderPrecision, SliderStep,
        SliderValue, ValueChange,
    },
};

use crate::{
    render::RenderSettings,
    sim::{kernels::TARGET_DENSITY, ObstacleCommand, ResetParticles, SimParams, SpawnLayout},
};

const PANEL_WIDTH: f32 = 300.0;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FeathersPlugins)
            .insert_resource(UiTheme(create_dark_theme()))
            .add_systems(Startup, spawn_panel)
            .add_systems(Update, sync_widgets.run_if(params_changed));
    }
}

/// Root of the control panel. `Hovered` on it tells input to ignore clicks.
#[derive(Component, Default, Clone)]
pub struct UiPanel;

/// Which parameter a slider edits.
#[derive(Clone, Copy)]
enum Target {
    Sim(fn(&mut SimParams) -> &mut f32),
    Render(fn(&mut RenderSettings) -> &mut f32),
}

impl Target {
    fn read(self, sim: &SimParams, render: &RenderSettings) -> f32 {
        match self {
            Target::Sim(get) => *get(&mut sim.clone()),
            Target::Render(get) => *get(&mut render.clone()),
        }
    }

    fn write(self, value: f32, sim: &mut SimParams, render: &mut RenderSettings) {
        match self {
            Target::Sim(get) => *get(sim) = value,
            Target::Render(get) => *get(render) = value,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
struct SliderBinding(Option<Target>);

#[derive(Component, Clone, Copy, Default)]
struct CheckboxBinding(Option<fn(&mut RenderSettings) -> &mut bool>);

#[derive(Component, Clone, Copy, Default)]
struct LayoutChoice(SpawnLayout);

struct SliderSpec {
    name: &'static str,
    range: (f32, f32),
    step: f32,
    precision: i32,
    target: Target,
}

const fn sim(
    name: &'static str,
    range: (f32, f32),
    step: f32,
    precision: i32,
    get: fn(&mut SimParams) -> &mut f32,
) -> SliderSpec {
    SliderSpec {
        name,
        range,
        step,
        precision,
        target: Target::Sim(get),
    }
}

const fn render(
    name: &'static str,
    range: (f32, f32),
    step: f32,
    precision: i32,
    get: fn(&mut RenderSettings) -> &mut f32,
) -> SliderSpec {
    SliderSpec {
        name,
        range,
        step,
        precision,
        target: Target::Render(get),
    }
}

const SIM_SLIDERS: &[SliderSpec] = &[
    sim("Pillar Radius", (3.0, 30.0), 0.5, 1, |p| {
        &mut p.pillar_radius
    }),
    sim("Gravity", (0.0, 300.0), 1.0, 0, |p| &mut p.gravity),
    sim("Mass", (0.1, 10.0), 0.1, 1, |p| &mut p.mass),
    // Also the grid cell size, so it must stay > 0.
    sim("Smoothing Radius", (1.0, 20.0), 0.1, 1, |p| {
        &mut p.smoothing_radius
    }),
    sim(
        "Target Density",
        (0.0, 4.0 * TARGET_DENSITY),
        0.01,
        2,
        |p| &mut p.target_density,
    ),
    sim(
        "Pressure Multiplier",
        (10_000.0, 1_000_000.0),
        10_000.0,
        0,
        |p| &mut p.pressure_multiplier,
    ),
    sim(
        "Near Pressure Multiplier",
        (100.0, 100_000.0),
        100.0,
        0,
        |p| &mut p.near_pressure_multiplier,
    ),
    sim("Cohesion", (0.0, 1.0), 0.01, 2, |p| &mut p.cohesion),
    sim("Viscosity", (0.0, 100.0), 1.0, 0, |p| &mut p.viscosity),
    sim("Wall Bounce", (0.0, 1.0), 0.01, 2, |p| {
        &mut p.collision_damping
    }),
    sim("Wall Friction", (0.0, 30.0), 0.5, 1, |p| {
        &mut p.wall_friction
    }),
    sim("Air Drag", (0.0, 0.01), 0.0001, 4, |p| &mut p.air_drag),
    sim("Cursor Radius", (5.0, 100.0), 1.0, 0, |p| {
        &mut p.interaction_radius
    }),
    sim("Cursor Strength", (0.0, 3000.0), 10.0, 0, |p| {
        &mut p.interaction_strength
    }),
];

const RENDER_SLIDERS: &[SliderSpec] = &[
    render("Blob Size", (2.0, 16.0), 0.1, 1, |r| &mut r.blob_size),
    render("Threshold", (0.05, 0.95), 0.01, 2, |r| &mut r.threshold),
    render("Edge Softness", (0.0, 0.2), 0.005, 3, |r| &mut r.softness),
    render("Rim Width", (0.0, 0.5), 0.01, 2, |r| &mut r.rim_width),
    render("Rim Brightness", (0.0, 1.0), 0.01, 2, |r| {
        &mut r.rim_brightness
    }),
    render("Shading", (0.0, 2.0), 0.01, 2, |r| &mut r.diffuse),
    render("Specular", (0.0, 2.0), 0.01, 2, |r| &mut r.specular),
    render("Shininess", (2.0, 64.0), 1.0, 0, |r| &mut r.shininess),
    render("Normal Strength", (0.5, 20.0), 0.1, 1, |r| {
        &mut r.normal_strength
    }),
    render("Gradient Radius", (1.0, 24.0), 0.5, 1, |r| {
        &mut r.gradient_radius
    }),
    render("Light Depth", (0.05, 1.0), 0.01, 2, |r| &mut r.lit_depth),
    render("Glow", (0.0, 4.0), 0.05, 2, |r| &mut r.glow),
    render("Foam Speed", (20.0, 300.0), 1.0, 0, |r| &mut r.foam_speed),
    render("Foam Opacity", (0.0, 1.0), 0.01, 2, |r| &mut r.foam_opacity),
];

fn spawn_panel(mut commands: Commands, sim: Res<SimParams>, render: Res<RenderSettings>) {
    commands.spawn_scene(panel(*sim, *render));
}

fn slider_row(spec: &SliderSpec, value: f32) -> impl Scene {
    let (min, max) = spec.range;
    let name = spec.name;
    let binding = SliderBinding(Some(spec.target));
    let step = SliderStep(spec.step);
    let precision = SliderPrecision(spec.precision);
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
        }
        Children [
            label_small(name),
            (
                @FeathersSlider { @min: {min}, @max: {max}, @value: {value} }
                template_value(binding)
                template_value(step)
                template_value(precision)
                on(slider_changed)
            ),
        ]
    }
}

fn slider_rows(
    specs: &[SliderSpec],
    sim: &SimParams,
    render: &RenderSettings,
) -> Vec<Box<dyn Scene>> {
    specs
        .iter()
        .map(|spec| Box::new(slider_row(spec, spec.target.read(sim, render))) as Box<dyn Scene>)
        .collect()
}

fn checkbox_row(caption: &'static str, get: fn(&mut RenderSettings) -> &mut bool) -> impl Scene {
    let binding = CheckboxBinding(Some(get));
    bsn! {
        @FeathersCheckbox { @caption: bsn! { Text(caption) ThemedText } }
        template_value(binding)
        on(checkbox_changed)
    }
}

fn button(caption: &'static str) -> impl Scene {
    bsn! {
        @FeathersButton { @caption: bsn! { Text(caption) ThemedText } }
    }
}

fn panel(sim: SimParams, render: RenderSettings) -> impl Scene {
    let sim_rows = slider_rows(SIM_SLIDERS, &sim, &render);
    let render_rows = slider_rows(RENDER_SLIDERS, &sim, &render);
    let dam_break = LayoutChoice(SpawnLayout::DamBreak);
    let drop = LayoutChoice(SpawnLayout::Drop);

    bsn! {
        UiPanel
        Hovered
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            bottom: px(0),
            width: px(PANEL_WIDTH),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            padding: UiRect { right: px(10) },
        }
        ThemeBackgroundColor(tokens::PANE_BODY_BG)
        Children [
            (
                #scroll
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    overflow: Overflow::scroll_y(),
                    padding: px(8),
                    row_gap: px(6),
                    flex_grow: 1.0,
                }
                ScrollArea
                Children [
                    label("Fluid"),
                    label_small("Left drag: pull. Right drag: push."),
                    label_small("Drag pillars to move them. Shift+click: add. Right-click: remove."),
                    (
                        RadioGroup
                        Node { display: Display::Flex, flex_direction: FlexDirection::Row, column_gap: px(8) }
                        on(layout_changed)
                        Children [
                            (
                                @FeathersRadio { @caption: bsn! { Text("Dam break") ThemedText } }
                                template_value(dam_break)
                            ),
                            (
                                @FeathersRadio { @caption: bsn! { Text("Drop") ThemedText } }
                                template_value(drop)
                            ),
                        ]
                    ),
                    (
                        Node { display: Display::Flex, flex_direction: FlexDirection::Row, column_gap: px(6) }
                        Children [
                            (button("Reset particles") on(|_: On<Activate>, mut reset: MessageWriter<ResetParticles>| {
                                reset.write_default();
                            })),
                            (button("Reset pillars") on(|_: On<Activate>, mut commands: MessageWriter<ObstacleCommand>| {
                                commands.write(ObstacleCommand::ResetPreset);
                            })),
                            (button("Clear pillars") on(|_: On<Activate>, mut commands: MessageWriter<ObstacleCommand>| {
                                commands.write(ObstacleCommand::Clear);
                            })),
                        ]
                    ),
                    {sim_rows},
                    label("Rendering"),
                    checkbox_row("Surface rendering", |r| &mut r.show_surface),
                    checkbox_row("Foam", |r| &mut r.foam),
                    {render_rows},
                    (button("Reset parameters") on(|_: On<Activate>, mut sim: ResMut<SimParams>, mut render: ResMut<RenderSettings>| {
                        *sim = SimParams { spawn_layout: sim.spawn_layout, ..default() };
                        *render = RenderSettings::default();
                    })),
                ]
            ),
            (
                @FeathersScrollbar { @target: #scroll, @orientation: {ControlOrientation::Vertical} }
                Node {
                    position_type: PositionType::Absolute,
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    width: px(6),
                }
            ),
        ]
    }
}

fn slider_changed(
    change: On<ValueChange<f32>>,
    bindings: Query<&SliderBinding>,
    mut sim: ResMut<SimParams>,
    mut render: ResMut<RenderSettings>,
    mut commands: Commands,
) {
    let Ok(SliderBinding(Some(target))) = bindings.get(change.source) else {
        return;
    };
    target.write(change.value, &mut sim, &mut render);
    commands
        .entity(change.source)
        .insert(SliderValue(change.value));
}

fn checkbox_changed(
    change: On<ValueChange<bool>>,
    bindings: Query<&CheckboxBinding>,
    mut render: ResMut<RenderSettings>,
) {
    if let Ok(CheckboxBinding(Some(get))) = bindings.get(change.source) {
        *get(&mut render) = change.value;
    }
}

fn layout_changed(
    change: On<ValueChange<Entity>>,
    choices: Query<&LayoutChoice>,
    mut sim: ResMut<SimParams>,
) {
    if let Ok(choice) = choices.get(change.value) {
        sim.spawn_layout = choice.0;
    }
}

fn params_changed(sim: Res<SimParams>, render: Res<RenderSettings>) -> bool {
    sim.is_changed() || render.is_changed()
}

/// Pushes resource values back into the widgets, so buttons and code that
/// change parameters are reflected in the panel.
fn sync_widgets(
    sim: Res<SimParams>,
    render: Res<RenderSettings>,
    sliders: Query<(Entity, &SliderBinding, &SliderValue)>,
    checkboxes: Query<(Entity, &CheckboxBinding, Has<Checked>)>,
    radios: Query<(Entity, &LayoutChoice, Has<Checked>)>,
    mut commands: Commands,
) {
    for (entity, binding, current) in &sliders {
        let Some(target) = binding.0 else { continue };
        let value = target.read(&sim, &render);
        if (current.0 - value).abs() > f32::EPSILON {
            commands.entity(entity).insert(SliderValue(value));
        }
    }
    for (entity, binding, checked) in &checkboxes {
        let Some(get) = binding.0 else { continue };
        let wanted = *get(&mut render.clone());
        match (wanted, checked) {
            (true, false) => {
                commands.entity(entity).insert(Checked);
            }
            (false, true) => {
                commands.entity(entity).remove::<Checked>();
            }
            _ => {}
        }
    }
    for (entity, choice, checked) in &radios {
        let wanted = choice.0 == sim.spawn_layout;
        match (wanted, checked) {
            (true, false) => {
                commands.entity(entity).insert(Checked);
            }
            (false, true) => {
                commands.entity(entity).remove::<Checked>();
            }
            _ => {}
        }
    }
}
