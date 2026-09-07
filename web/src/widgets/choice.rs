use fluid_2d::SpawnLayout;
use leptos::prelude::*;

use crate::bus::Bus;

#[component]
pub fn Choice(options: &'static [(&'static str, SpawnLayout)]) -> impl IntoView {
    let bus = expect_context::<Bus>();

    view! {
        <div class="mt-3 flex gap-4 text-sm">
            {options
                .iter()
                .map(|&(label, layout)| {
                    let bus = bus.clone();
                    let checked = {
                        let bus = bus.clone();
                        move || bus.params.get().sim.spawn_layout == layout
                    };
                    view! {
                        <label class="flex items-center gap-1.5">
                            <input
                                type="radio"
                                name="layout"
                                class="accent-sky-400"
                                prop:checked=checked
                                on:change=move |_| bus.edit(|p| p.sim.spawn_layout = layout)
                            />
                            {label}
                        </label>
                    }
                })
                .collect_view()}
        </div>
    }
}
