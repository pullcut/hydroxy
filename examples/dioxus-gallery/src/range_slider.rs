use dioxus::prelude::*;
use hydroxy_dioxus::range_slider::RangeSlider;

#[component]
pub fn RangeSliderDemo() -> Element {
    let mut price = use_signal(|| [200.0, 800.0]);

    rsx! {
        div { style: "display: grid; gap: var(--space-8); max-width: 420px;",
            RangeSlider {
                label: "Price",
                max: 1000.0,
                step: 10.0,
                min_gap: 50.0,
                value: Some(price()),
                on_value_change: move |v| price.set(v),
                format: |v: f64| format!("£{v}"),
                name: "price",
            }
            RangeSlider { label: "Hours", max: 24.0, default_value: [9.0, 17.0] }
            RangeSlider { label: "Disabled", default_value: [20.0, 60.0], disabled: true }
        }
    }
}
