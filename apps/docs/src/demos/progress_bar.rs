use hydroxy_leptos::progress_bar::{Jewel, ProgressBar, ProgressSize};
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let (read, set_read) = signal(22.0);

    view! {
        <div style="display: grid; gap: var(--space-7);">
            // Controlled by the slider; the fill springs to each new width.
            <ProgressBar
                label="Reading interviews"
                value=Signal::derive(move || Some(read.get()))
                max=24.0
                value_text=Signal::derive(move || Some(format!("{} of 24", read.get())))
            />
            <input
                type="range"
                min="0"
                max="24"
                prop:value=move || read.get().to_string()
                on:input=move |ev| set_read.set(event_target_value(&ev).parse().unwrap_or(0.0))
                aria-label="Interviews read"
                style="accent-color: var(--accent);"
            />

            <ProgressBar label="Storage" value=64.0 jewel=Jewel::Emerald size=ProgressSize::Sm />

            // No value: indeterminate.
            <ProgressBar label="Drafting summary" />
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "progress-bar",
    name: "Progress bar",
    group: "Feedback",
    summary: "Shows how far along a task is. Leave out the value when the amount is unknown.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("progress_bar.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/progress_bar.rs"),
    parts: &[
        &["ProgressBar", "root", "<div>, data-state=loading|complete|indeterminate, data-size, data-jewel"],
        &["(inside)", "header", "Label and value row, when label is set"],
        &["(inside)", "label", "Names the bar (aria-labelledby)"],
        &["(inside)", "value", "The value text, hidden from assistive tech (the track exposes it)"],
        &["(inside)", "track", "role=progressbar, aria-valuemin/max/now, aria-valuetext"],
        &["(inside)", "indicator", "The fill, width in percent"],
    ],
    keys: &[],
    props: &[
        &["value", "Option<f64>", "Current value, clamped to 0..=max. None for indeterminate"],
        &["max", "f64", "Default 100"],
        &["label", "Option<String>", "Shown above the bar and names it"],
        &["value_text", "Option<String>", "Such as \"22 of 24\". Default a rounded percentage"],
        &["size", "ProgressSize", "Md (8px, default) or Sm (4px)"],
        &["jewel", "Option<Jewel>", "Fill colour. Default the accent"],
    ],
    events: &[],
};
