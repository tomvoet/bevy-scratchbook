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
pub fn Panel(sender: ControlSender) -> impl IntoView {
    provide_context(Bus::new(sender));

    view! {
        <aside class="w-[320px] shrink-0 overflow-y-auto bg-slate-900/80 p-4 text-slate-200">
            {SECTIONS.iter().map(|section| view! { <SectionView section=section /> }).collect_view()}
        </aside>
    }
}

#[component]
fn SectionView(section: &'static Section) -> impl IntoView {
    view! {
        <h2 class="mt-5 text-base font-semibold first:mt-0">{section.title}</h2>
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
