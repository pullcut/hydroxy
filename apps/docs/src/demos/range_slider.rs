use leptos::prelude::*;
use hydroxy_leptos::range_slider::RangeSlider;

pub fn demo() -> AnyView {
    let (price, set_price) = signal([200.0, 800.0]);

    view! {
        <div style="display: grid; gap: var(--space-8); max-width: 420px;">
            // Controlled, with a gap the thumbs keep between them.
            <RangeSlider
                label="Price"
                max=1000.0
                step=10.0
                min_gap=50.0
                value=price
                on_value_change=move |v| set_price.set(v)
                format=|v: f64| format!("£{v}")
                name="price"
            />

            // Uncontrolled.
            <RangeSlider label="Hours" max=24.0 default_value=[9.0, 17.0] />

            <RangeSlider label="Disabled" default_value=[20.0, 60.0] disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "range-slider",
    name: "Range slider",
    group: "Forms",
    summary: "Pick a low and a high value on one track. Each thumb stops at the other, and the track moves whichever thumb is nearest.",
    demo,
    leptos: include_str!("range_slider.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/range_slider.rs"),
    parts: &[
        &["RangeSlider", "root", "role=group, labelled by the label. data-disabled"],
        &["(inside)", "header", "Label row"],
        &["(inside)", "label", "Visible label"],
        &["(inside)", "value", "\"low – high\" in mono. Hidden from assistive tech, which reads the thumbs"],
        &["(inside)", "track", "Press to move the nearest thumb there and drag it. Captures the pointer"],
        &["(inside)", "range", "The filled band between the thumbs"],
        &["(inside)", "thumb", "role=slider. aria-valuemin/max stop at the other thumb. data-thumb=min|max, data-dragging"],
        &["(inside)", "hidden input", "Two <input type=hidden>, low then high, when named"],
    ],
    keys: &[
        &["ArrowRight / ArrowUp", "Focused thumb up one step"],
        &["ArrowLeft / ArrowDown", "Focused thumb down one step"],
        &["PageUp / PageDown", "Up or down ten steps"],
        &["Home / End", "As low or as high as the thumb can go"],
        &["Tab", "Low thumb, then high thumb"],
    ],
    props: &[
        &["label", "String", "Required. Visible label and group name"],
        &["min / max / step", "f64", "Range and step. Default 0, 100 and 1"],
        &["min_gap", "f64", "Smallest distance kept between the thumbs. Default 0"],
        &["value", "Option<[f64; 2]>", "Controlled pair. Pair with on_value_change"],
        &["default_value", "[f64; 2]", "Initial pair when uncontrolled. Defaults to [min, max]"],
        &["on_value_change", "Callback<[f64; 2]>", "Called as either thumb moves"],
        &["format", "Callback<f64, String>", "Readout text, also used as aria-valuetext"],
        &["thumb_labels", "[String; 2]", "Thumb names. Default \"Minimum\" and \"Maximum\""],
        &["disabled", "bool", "Block changes"],
        &["name", "String", "Form field name. Submits both values"],
    ],
};
