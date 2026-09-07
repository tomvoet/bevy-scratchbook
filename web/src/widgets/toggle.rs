use fluid_2d::controls::{Control, ControlKind};
use leptos::prelude::*;

use crate::bus::Bus;

#[component]
pub fn Toggle(control: &'static Control) -> impl IntoView {
    let bus = expect_context::<Bus>();
    let ControlKind::Toggle { get } = control.kind else {
        unreachable!()
    };
    let checked = move || *get(&mut bus.params.get());
    let on_change = {
        let bus = bus.clone();
        move |ev| {
            let next = event_target_checked(&ev);
            bus.edit(|params| *get(params) = next);
        }
    };

    view! {
        <label class="mt-2 flex items-center gap-2 text-sm">
            <input type="checkbox" class="accent-sky-400" prop:checked=checked on:change=on_change />
            {control.name}
        </label>
    }
}
