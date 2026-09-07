use std::time::Duration;

use fluid_2d::SimStats;
use leptos::prelude::*;

/// Polls the sim rather than pushing, so a slow frame never blocks the panel.
#[component]
pub fn Fps(stats: SimStats) -> impl IntoView {
    let fps = RwSignal::new(0.0f32);
    set_interval(
        move || fps.set(stats.fps()),
        Duration::from_millis(500),
    );

    view! {
        <div class="pointer-events-none absolute right-3 top-2 font-mono text-sm text-emerald-300">
            {move || format!("{:.0} fps", fps.get())}
        </div>
    }
}
