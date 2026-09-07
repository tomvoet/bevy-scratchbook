use fluid_2d::controls::{Control, ControlKind};
use leptos::prelude::*;

use crate::bus::Bus;

#[component]
pub fn Slider(control: &'static Control) -> impl IntoView {
    let bus = expect_context::<Bus>();
    let ControlKind::Slider {
        range: (min, max),
        step,
        precision,
        get,
    } = control.kind
    else {
        unreachable!()
    };

    let value = move || *get(&mut bus.params.get());
    let shown = move || format!("{:.*}", precision.max(0) as usize, value());
    let on_input = {
        let bus = bus.clone();
        move |ev| {
            let Ok(next) = event_target_value(&ev).parse::<f32>() else {
                return;
            };
            bus.edit(|params| *get(params) = next);
        }
    };

    view! {
        <label class="mt-3 block">
            <span class="flex justify-between text-xs text-slate-400">
                <span>{control.name}</span>
                <span class="font-mono text-slate-200">{shown}</span>
            </span>
            <input
                type="range"
                class="mt-1 w-full accent-sky-400"
                min=min
                max=max
                step=step
                prop:value=value
                on:input=on_input
            />
        </label>
    }
}
