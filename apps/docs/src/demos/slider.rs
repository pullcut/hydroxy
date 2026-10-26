use leptos::prelude::*;
use oxide_leptos::slider::Slider;

pub fn demo() -> AnyView {
    let (budget, set_budget) = signal(1200.0);

    view! {
        <div style="display: grid; gap: var(--space-8); max-width: 420px;">
            // Controlled, with a formatted readout.
            <Slider
                label="Monthly budget"
                min=0.0
                max=5000.0
                step=100.0
                value=budget
                on_input=move |v| set_budget.set(v)
                format=|v: f64| format!("£{v}")
            />

            // Uncontrolled.
            <Slider
                label="Summary length"
                min=1.0
                max=10.0
                default_value=4.0
                format=|v: f64| format!("{v} paragraphs")
                name="length"
            />

            <Slider label="Sources per answer" default_value=3.0 max=10.0 disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "slider",
    name: "Slider",
    group: "Forms",
    summary: "Pick a value from a range. A native range input, so the keyboard, assistive tech and forms work without extra code.",
    demo,
    leptos: include_str!("slider.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/slider.rs"),
    parts: &[
        &["Slider", "root", "Wrapper. data-disabled"],
        &["Slider", "header", "Label row"],
        &["Slider", "label", "<label for> the control"],
        &["Slider", "value", "The current value in mono. Hidden from assistive tech, which reads the control"],
        &["Slider", "control", "Native <input type=range>. --ox-slider-fill drives the filled track"],
    ],
    keys: &[
        &["ArrowRight / ArrowUp", "Up one step"],
        &["ArrowLeft / ArrowDown", "Down one step"],
        &["PageUp / PageDown", "Up or down a larger step"],
        &["Home / End", "Minimum / maximum"],
    ],
    props: &[
        &["label", "String", "Required. Visible label and accessible name"],
        &["min / max / step", "f64", "Range and step. Default 0, 100 and 1"],
        &["value", "Option<f64>", "Controlled value. Pair with on_input"],
        &["default_value", "f64", "Initial value when uncontrolled. Defaults to min"],
        &["on_input", "Callback<f64>", "Called as the thumb moves"],
        &["format", "Callback<f64, String>", "Readout text, also used as aria-valuetext"],
        &["disabled", "bool", "Block changes"],
        &["name", "String", "Form field name"],
    ],
};
