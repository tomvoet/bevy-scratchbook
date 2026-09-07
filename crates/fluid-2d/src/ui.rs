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
    bridge::apply,
    input::UiPanel,
    controls::{edit, Action, Control, ControlKind, Params, Section, SECTIONS},
    render::RenderSettings,
    sim::{ObstacleCommand, ResetParticles, SimParams, SpawnLayout},
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

#[derive(Component, Clone, Copy, Default)]
struct Bound(Option<&'static Control>);

#[derive(Component, Clone, Copy, Default)]
struct LayoutChoice(Option<SpawnLayout>);

#[derive(Component, Clone, Copy, Default)]
struct ActionButton(Option<Action>);

fn spawn_panel(mut commands: Commands, sim: Res<SimParams>, render: Res<RenderSettings>) {
    commands.spawn_scene(panel(Params::new(*sim, *render)));
}

fn slider_row(control: &'static Control, params: &Params) -> impl Scene {
    let ControlKind::Slider {
        range,
        step,
        precision,
        get,
    } = control.kind
    else {
        unreachable!()
    };
    let (min, max) = range;
    let value = *get(&mut { *params });
    let name = control.name;
    let bound = Bound(Some(control));
    let step = SliderStep(step);
    let precision = SliderPrecision(precision);
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
                template_value(bound)
                template_value(step)
                template_value(precision)
                on(slider_changed)
            ),
        ]
    }
}

fn toggle_row(control: &'static Control) -> impl Scene {
    let caption = control.name;
    let bound = Bound(Some(control));
    bsn! {
        @FeathersCheckbox { @caption: bsn! { Text(caption) ThemedText } }
        template_value(bound)
        on(checkbox_changed)
    }
}

fn choice_row(options: &'static [(&'static str, SpawnLayout)]) -> impl Scene {
    let radios: Vec<Box<dyn Scene>> = options
        .iter()
        .map(|&(caption, layout)| {
            let choice = LayoutChoice(Some(layout));
            Box::new(bsn! {
                @FeathersRadio { @caption: bsn! { Text(caption) ThemedText } }
                template_value(choice)
            }) as Box<dyn Scene>
        })
        .collect();

    bsn! {
        RadioGroup
        Node { display: Display::Flex, flex_direction: FlexDirection::Row, column_gap: px(8) }
        on(layout_changed)
        Children [ {radios} ]
    }
}

fn actions_row(actions: &'static [(&'static str, Action)]) -> impl Scene {
    let buttons: Vec<Box<dyn Scene>> = actions
        .iter()
        .map(|&(caption, action)| {
            let button = ActionButton(Some(action));
            Box::new(bsn! {
                @FeathersButton { @caption: bsn! { Text(caption) ThemedText } }
                template_value(button)
                on(action_pressed)
            }) as Box<dyn Scene>
        })
        .collect();

    bsn! {
        Node { display: Display::Flex, flex_direction: FlexDirection::Row, column_gap: px(6) }
        Children [ {buttons} ]
    }
}

fn control_row(control: &'static Control, params: &Params) -> Box<dyn Scene> {
    match control.kind {
        ControlKind::Slider { .. } => Box::new(slider_row(control, params)),
        ControlKind::Toggle { .. } => Box::new(toggle_row(control)),
        ControlKind::Choice { options, .. } => Box::new(choice_row(options)),
        ControlKind::Actions(actions) => Box::new(actions_row(actions)),
    }
}

fn section_rows(section: &'static Section, params: &Params) -> Vec<Box<dyn Scene>> {
    let mut rows: Vec<Box<dyn Scene>> = vec![Box::new(label(section.title))];
    rows.extend(
        section
            .help
            .iter()
            .map(|&line| Box::new(label_small(line)) as Box<dyn Scene>),
    );
    rows.extend(
        section
            .controls
            .iter()
            .map(|control| control_row(control, params)),
    );
    rows
}

fn panel(params: Params) -> impl Scene {
    let rows: Vec<Box<dyn Scene>> = SECTIONS
        .iter()
        .flat_map(|section| section_rows(section, &params))
        .collect();

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
                Children [ {rows} ]
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
    bound: Query<&Bound>,
    mut sim: ResMut<SimParams>,
    mut render: ResMut<RenderSettings>,
    mut commands: Commands,
) {
    let Ok(Bound(Some(control))) = bound.get(change.source) else {
        return;
    };
    if let ControlKind::Slider { get, .. } = control.kind {
        edit(&mut sim, &mut render, |params| *get(params) = change.value);
    }
    commands
        .entity(change.source)
        .insert(SliderValue(change.value));
}

fn checkbox_changed(
    change: On<ValueChange<bool>>,
    bound: Query<&Bound>,
    mut sim: ResMut<SimParams>,
    mut render: ResMut<RenderSettings>,
) {
    let Ok(Bound(Some(control))) = bound.get(change.source) else {
        return;
    };
    if let ControlKind::Toggle { get } = control.kind {
        edit(&mut sim, &mut render, |params| *get(params) = change.value);
    }
}

fn layout_changed(
    change: On<ValueChange<Entity>>,
    choices: Query<&LayoutChoice>,
    mut sim: ResMut<SimParams>,
) {
    if let Ok(LayoutChoice(Some(layout))) = choices.get(change.value) {
        sim.spawn_layout = *layout;
    }
}

fn action_pressed(
    activate: On<Activate>,
    buttons: Query<&ActionButton>,
    mut sim: ResMut<SimParams>,
    mut render: ResMut<RenderSettings>,
    mut reset: MessageWriter<ResetParticles>,
    mut obstacles: MessageWriter<ObstacleCommand>,
) {
    if let Ok(ActionButton(Some(action))) = buttons.get(activate.entity) {
        apply(
            *action,
            &mut sim,
            &mut render,
            &mut reset,
            &mut obstacles,
        );
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
    sliders: Query<(Entity, &Bound, &SliderValue)>,
    checkboxes: Query<(Entity, &Bound, Has<Checked>)>,
    radios: Query<(Entity, &LayoutChoice, Has<Checked>)>,
    mut commands: Commands,
) {
    let params = Params::new(*sim, *render);

    for (entity, bound, current) in &sliders {
        let Some(control) = bound.0 else { continue };
        let ControlKind::Slider { get, .. } = control.kind else {
            continue;
        };
        let value = *get(&mut { params });
        if (current.0 - value).abs() > f32::EPSILON {
            commands.entity(entity).insert(SliderValue(value));
        }
    }
    for (entity, bound, checked) in &checkboxes {
        let Some(control) = bound.0 else { continue };
        let ControlKind::Toggle { get } = control.kind else {
            continue;
        };
        set_checked(&mut commands, entity, *get(&mut { params }), checked);
    }
    for (entity, choice, checked) in &radios {
        let Some(layout) = choice.0 else { continue };
        set_checked(
            &mut commands,
            entity,
            layout == sim.spawn_layout,
            checked,
        );
    }
}

fn set_checked(commands: &mut Commands, entity: Entity, wanted: bool, checked: bool) {
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
