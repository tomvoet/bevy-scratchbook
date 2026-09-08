use std::{cell::RefCell, rc::Rc};

use leptos::prelude::*;

use crate::{panel::Panel, widgets::Fps};

#[component]
pub fn Shell() -> impl IntoView {
    let (sender, stats, bridge) = fluid_2d::control_channel();
    let bridge = Rc::new(RefCell::new(Some(bridge)));

    // Runs once the canvas is in the DOM, which bevy needs before it starts.
    Effect::new(move |_| {
        if let Some(bridge) = bridge.borrow_mut().take() {
            start_sim(bridge);
        }
    });

    // Starts folded on a phone, where the panel would cover the sim.
    let open = RwSignal::new(is_wide());

    view! {
        <div class="relative flex h-screen w-screen">
            <Panel sender=sender open=open />
            <main class="relative min-w-0 flex-1">
                <canvas id="sim" class="block h-full w-full touch-none outline-none"></canvas>
                <button
                    class="absolute left-3 top-2 rounded bg-slate-800/80 px-3 py-1.5 text-xs text-slate-200 hover:bg-slate-700"
                    class=("hidden", move || open.get())
                    on:click=move |_| open.set(true)
                >
                    "Controls"
                </button>
                <Fps stats=stats />
            </main>
        </div>
    }
}

fn is_wide() -> bool {
    window()
        .inner_width()
        .ok()
        .and_then(|width| width.as_f64())
        .is_some_and(|width| width >= 640.0)
}

fn start_sim(bridge: fluid_2d::ControlBridgePlugin) {
    use bevy::prelude::*;

    // `spawn_app` on wasm, so this returns and leptos keeps running.
    bevy::app::App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#sim".into()),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
            fluid_2d::FluidSimPlugin,
            bridge,
        ))
        .run();
}
