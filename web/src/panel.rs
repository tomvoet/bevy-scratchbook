use fluid_2d::{
    controls::{Control, ControlKind, Section, SECTIONS},
    ControlSender,
};
use leptos::prelude::*;

use crate::{
    bus::Bus,
    widgets::{Actions, Choice, Slider, Toggle},
};

#[component]
pub fn Panel(sender: ControlSender, open: RwSignal<bool>) -> impl IntoView {
    provide_context(Bus::new(sender));

    view! {
        <aside
            class="absolute inset-y-0 left-0 z-10 w-[320px] max-w-[85vw] shrink-0 overflow-y-auto bg-slate-900/95 p-4 text-slate-200 sm:static sm:bg-slate-900/80"
            class=("hidden", move || !open.get())
        >
            <div class="flex items-center justify-between">
                <h1 class="text-xs font-semibold uppercase tracking-wide text-slate-400">
                    "Controls"
                </h1>
                <button
                    class="rounded px-2 py-1 text-xs text-slate-400 hover:bg-slate-700 hover:text-slate-200"
                    on:click=move |_| open.set(false)
                >
                    "Hide"
                </button>
            </div>
            {SECTIONS.iter().map(|section| view! { <SectionView section=section /> }).collect_view()}
        </aside>
    }
}

#[component]
fn SectionView(section: &'static Section) -> impl IntoView {
    view! {
        <details open=true class="group mt-4">
            <summary class="flex cursor-pointer select-none list-none items-center gap-2 text-base font-semibold [&::-webkit-details-marker]:hidden">
                <span class="text-xs text-slate-400 transition-transform group-open:rotate-90">
                    "\u{25b6}"
                </span>
                {section.title}
            </summary>
            {section
                .help
                .iter()
                .map(|line| view! { <p class="mt-1 text-xs text-slate-400">{*line}</p> })
                .collect_view()}
            {section
                .controls
                .iter()
                .map(|control| view! { <ControlView control=control /> })
                .collect_view()}
        </details>
    }
}

#[component]
fn ControlView(control: &'static Control) -> impl IntoView {
    match control.kind {
        ControlKind::Slider { .. } => view! { <Slider control=control /> }.into_any(),
        ControlKind::Toggle { .. } => view! { <Toggle control=control /> }.into_any(),
        ControlKind::Choice { options, .. } => view! { <Choice options=options /> }.into_any(),
        ControlKind::Actions(actions) => view! { <Actions actions=actions /> }.into_any(),
    }
}
