use crate::{
    render::RenderSettings,
    sim::{kernels::TARGET_DENSITY, SimParams, SpawnLayout, STIFFNESS},
};

/// Everything the panel can edit, in one place so the accessors below don't
/// need to care which resource a value ends up in.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Params {
    pub sim: SimParams,
    pub render: RenderSettings,
}

impl Params {
    pub fn new(sim: SimParams, render: RenderSettings) -> Self {
        Self { sim, render }
    }

    /// Back to defaults, but stay on the layout the user picked.
    pub fn reset(&mut self) {
        self.sim = SimParams {
            spawn_layout: self.sim.spawn_layout,
            ..SimParams::default()
        };
        self.render = RenderSettings::default();
    }
}

/// Buttons. The first three turn into messages, the last one edits `Params`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    ResetParticles,
    ResetPillars,
    ClearPillars,
    ResetParams,
}

pub enum ControlKind {
    Slider {
        range: (f32, f32),
        step: f32,
        precision: i32,
        get: fn(&mut Params) -> &mut f32,
    },
    Toggle {
        get: fn(&mut Params) -> &mut bool,
    },
    Choice {
        options: &'static [(&'static str, SpawnLayout)],
        get: fn(&mut Params) -> &mut SpawnLayout,
    },
    /// A row of buttons, kept together so both panels lay them out the same.
    Actions(&'static [(&'static str, Action)]),
}

pub struct Control {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: ControlKind,
}

impl Control {
    pub fn read_f32(&self, params: &Params) -> Option<f32> {
        match self.kind {
            ControlKind::Slider { get, .. } => Some(*get(&mut { *params })),
            _ => None,
        }
    }
}

/// Applies `edit` to both resources at once, writing back only what changed so
/// change detection stays honest.
pub fn edit(sim: &mut SimParams, render: &mut RenderSettings, edit: impl FnOnce(&mut Params)) {
    let mut params = Params::new(*sim, *render);
    edit(&mut params);
    if *sim != params.sim {
        *sim = params.sim;
    }
    if *render != params.render {
        *render = params.render;
    }
}

pub struct Section {
    pub title: &'static str,
    /// Lines shown under the title, for the mouse hints.
    pub help: &'static [&'static str],
    pub controls: &'static [Control],
}

const fn slider(
    id: &'static str,
    name: &'static str,
    range: (f32, f32),
    step: f32,
    precision: i32,
    get: fn(&mut Params) -> &mut f32,
) -> Control {
    Control {
        id,
        name,
        kind: ControlKind::Slider {
            range,
            step,
            precision,
            get,
        },
    }
}

const fn toggle(id: &'static str, name: &'static str, get: fn(&mut Params) -> &mut bool) -> Control {
    Control {
        id,
        name,
        kind: ControlKind::Toggle { get },
    }
}

pub const SECTIONS: &[Section] = &[
    Section {
        title: "Fluid",
        help: &[
            "Left drag: pull. Right drag: push.",
            "Drag pillars to move them. Shift+click: add. Right-click: remove.",
        ],
        controls: FLUID,
    },
    Section {
        title: "Rendering",
        help: &[],
        controls: RENDERING,
    },
];

const LAYOUTS: &[(&str, SpawnLayout)] = &[
    ("Dam break", SpawnLayout::DamBreak),
    ("Drop", SpawnLayout::Drop),
];

const FLUID: &[Control] = &[
    Control {
        id: "layout",
        name: "Layout",
        kind: ControlKind::Choice {
            options: LAYOUTS,
            get: |p| &mut p.sim.spawn_layout,
        },
    },
    Control {
        id: "actions",
        name: "",
        kind: ControlKind::Actions(&[
            ("Reset particles", Action::ResetParticles),
            ("Reset pillars", Action::ResetPillars),
            ("Clear pillars", Action::ClearPillars),
        ]),
    },
    slider("pillar_radius", "Pillar Radius", (3.0, 30.0), 0.5, 1, |p| {
        &mut p.sim.pillar_radius
    }),
    slider("gravity", "Gravity", (0.0, 300.0), 1.0, 0, |p| {
        &mut p.sim.gravity
    }),
    slider("mass", "Mass", (0.1, 10.0), 0.1, 1, |p| &mut p.sim.mass),
    // Also the grid cell size, so it must stay > 0.
    slider(
        "smoothing_radius",
        "Smoothing Radius",
        (1.0, 20.0),
        0.1,
        1,
        |p| &mut p.sim.smoothing_radius,
    ),
    slider(
        "target_density",
        "Target Density",
        (0.0, 4.0 * TARGET_DENSITY),
        0.01,
        2,
        |p| &mut p.sim.target_density,
    ),
    slider(
        "pressure",
        "Pressure Multiplier",
        (10_000.0 * STIFFNESS, 1_000_000.0 * STIFFNESS),
        10_000.0 * STIFFNESS,
        0,
        |p| &mut p.sim.pressure_multiplier,
    ),
    slider(
        "near_pressure",
        "Near Pressure Multiplier",
        (100.0 * STIFFNESS, 100_000.0 * STIFFNESS),
        100.0 * STIFFNESS,
        0,
        |p| &mut p.sim.near_pressure_multiplier,
    ),
    slider("cohesion", "Cohesion", (0.0, 1.0), 0.01, 2, |p| {
        &mut p.sim.cohesion
    }),
    slider("viscosity", "Viscosity", (0.0, 100.0), 1.0, 0, |p| {
        &mut p.sim.viscosity
    }),
    slider("wall_bounce", "Wall Bounce", (0.0, 1.0), 0.01, 2, |p| {
        &mut p.sim.collision_damping
    }),
    slider("wall_friction", "Wall Friction", (0.0, 30.0), 0.5, 1, |p| {
        &mut p.sim.wall_friction
    }),
    slider("air_drag", "Air Drag", (0.0, 0.01), 0.0001, 4, |p| {
        &mut p.sim.air_drag
    }),
    slider("cursor_radius", "Cursor Radius", (5.0, 100.0), 1.0, 0, |p| {
        &mut p.sim.interaction_radius
    }),
    slider(
        "cursor_strength",
        "Cursor Strength",
        (0.0, 3000.0),
        10.0,
        0,
        |p| &mut p.sim.interaction_strength,
    ),
];

const RENDERING: &[Control] = &[
    toggle("surface", "Surface rendering", |p| &mut p.render.show_surface),
    toggle("foam", "Foam", |p| &mut p.render.foam),
    slider("blob_size", "Blob Size", (2.0, 16.0), 0.1, 1, |p| {
        &mut p.render.blob_size
    }),
    slider("threshold", "Threshold", (0.05, 0.95), 0.01, 2, |p| {
        &mut p.render.threshold
    }),
    slider("softness", "Edge Softness", (0.0, 0.2), 0.005, 3, |p| {
        &mut p.render.softness
    }),
    slider("rim_width", "Rim Width", (0.0, 0.5), 0.01, 2, |p| {
        &mut p.render.rim_width
    }),
    slider("rim_brightness", "Rim Brightness", (0.0, 1.0), 0.01, 2, |p| {
        &mut p.render.rim_brightness
    }),
    slider("diffuse", "Shading", (0.0, 2.0), 0.01, 2, |p| {
        &mut p.render.diffuse
    }),
    slider("specular", "Specular", (0.0, 2.0), 0.01, 2, |p| {
        &mut p.render.specular
    }),
    slider("shininess", "Shininess", (2.0, 64.0), 1.0, 0, |p| {
        &mut p.render.shininess
    }),
    slider("normal", "Normal Strength", (0.5, 20.0), 0.1, 1, |p| {
        &mut p.render.normal_strength
    }),
    slider("gradient", "Gradient Radius", (1.0, 24.0), 0.5, 1, |p| {
        &mut p.render.gradient_radius
    }),
    slider("lit_depth", "Light Depth", (0.05, 1.0), 0.01, 2, |p| {
        &mut p.render.lit_depth
    }),
    slider("glow", "Glow", (0.0, 4.0), 0.05, 2, |p| &mut p.render.glow),
    slider("foam_speed", "Foam Speed", (20.0, 300.0), 1.0, 0, |p| {
        &mut p.render.foam_speed
    }),
    slider("foam_opacity", "Foam Opacity", (0.0, 1.0), 0.01, 2, |p| {
        &mut p.render.foam_opacity
    }),
    Control {
        id: "reset_params",
        name: "",
        kind: ControlKind::Actions(&[("Reset parameters", Action::ResetParams)]),
    },
];
