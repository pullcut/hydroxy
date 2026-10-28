use leptos::prelude::*;
use hydroxy_leptos::color_picker::{ColorPicker, Rgba};

pub fn demo() -> AnyView {
    let (brand, set_brand) = signal(Some(Rgba::rgb(0x3b, 0x82, 0xf6)));

    view! {
        <div style="display: flex; flex-wrap: wrap; gap: var(--space-8); align-items: flex-start;">
            // Controlled. Drag the area and sliders, type hex, or pick a swatch.
            <div style="display: grid; gap: var(--space-5);">
                <ColorPicker label="Brand colour" value=brand on_change=move |c| set_brand.set(Some(c)) />
                <span>{move || brand.get().map(|c| c.hex(false)).unwrap_or_default()}</span>
            </div>

            // Uncontrolled with an alpha slider, submitted as `overlay`.
            <ColorPicker label="Overlay" alpha=true default_value=Rgba { r: 239, g: 68, b: 68, a: 160 } name="overlay" />

            <ColorPicker label="Locked" default_value=Rgba::rgb(34, 197, 94) swatches=Vec::new() disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "color-picker",
    name: "Color picker",
    group: "Forms",
    summary: "A saturation and brightness area, a hue slider, optional opacity, a hex field and swatches. Pointer or keyboard.",
    demo,
    leptos: include_str!("color_picker.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/color_picker.rs"),
    parts: &[
        &["ColorPicker", "root", "role=group. Carries --picker-hue, --picker-color and --picker-alpha for the gradients"],
        &["ColorPicker", "area / area-thumb", "Saturation across, brightness up. The thumb is role=slider with aria-valuetext"],
        &["ColorPicker", "hue / hue-thumb", "Hue slider, 0 to 360"],
        &["ColorPicker", "alpha / alpha-thumb", "Opacity slider, 0 to 100. Only with alpha"],
        &["ColorPicker", "preview", "The current colour over a checkerboard"],
        &["ColorPicker", "hex", "Text input. A valid colour applies as you type; aria-invalid otherwise"],
        &["ColorPicker", "swatches / swatch", "Preset buttons with aria-pressed on the current one"],
    ],
    keys: &[
        &["Arrows (area)", "Saturation and brightness by one point. With Shift, ten"],
        &["PageUp / PageDown (area)", "Brightness by ten"],
        &["Arrows (hue, opacity)", "One degree or point. With Shift or PageUp/PageDown, ten"],
        &["Home / End (hue, opacity)", "Lowest or highest"],
        &["Enter (hex)", "Tidy the text to the current colour"],
    ],
    props: &[
        &["label", "String", "Required. Accessible name for the picker"],
        &["value", "Option<Rgba>", "Controlled colour. Pair with on_change"],
        &["default_value", "Option<Rgba>", "Initial colour when uncontrolled. Default black"],
        &["alpha", "bool", "Offer an opacity slider and #rrggbbaa hex"],
        &["swatches", "Option<Vec<Rgba>>", "Preset colours. Default a palette of eight; an empty Vec shows none"],
        &["disabled", "bool", "Block changes"],
        &["name", "String", "Form field name; submits the hex text"],
        &["on_change", "Callback<Rgba>", "Called as the colour changes"],
    ],
};
