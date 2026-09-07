use fluid_2d::controls::Action;
use leptos::prelude::*;

use crate::bus::Bus;

#[component]
pub fn Actions(actions: &'static [(&'static str, Action)]) -> impl IntoView {
    let bus = expect_context::<Bus>();

    view! {
        <div class="mt-3 flex flex-wrap gap-2">
            {actions
                .iter()
                .map(|&(label, action)| {
                    let bus = bus.clone();
                    view! {
                        <button
                            class="flex-1 rounded bg-slate-700 px-2 py-1.5 text-xs hover:bg-slate-600"
                            on:click=move |_| bus.act(action)
                        >
                            {label}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}
