use dioxus::prelude::*;
use hydroxy_dioxus::color_picker::{ColorPicker, Rgba};

#[component]
pub fn ColorPickerDemo() -> Element {
    let mut brand = use_signal(|| Some(Rgba::rgb(0x3b, 0x82, 0xf6)));
    let shown = brand().map(|c| c.hex(false)).unwrap_or_default();

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: var(--space-8); align-items: flex-start;",
            // Controlled. Drag the area and sliders, type hex, or pick a swatch.
            div { style: "display: grid; gap: var(--space-5);",
                ColorPicker { label: "Brand colour", value: brand(), on_change: move |c| brand.set(Some(c)) }
                span { "{shown}" }
            }

            // Uncontrolled with an alpha slider, submitted as `overlay`.
            ColorPicker {
                label: "Overlay",
                alpha: true,
                default_value: Rgba { r: 239, g: 68, b: 68, a: 160 },
                name: "overlay",
            }

            ColorPicker { label: "Locked", default_value: Rgba::rgb(34, 197, 94), swatches: Vec::new(), disabled: true }
        }
    }
}
