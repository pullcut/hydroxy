use dioxus::prelude::*;
use hydroxy_dioxus::slider::Slider;

#[component]
pub fn SliderDemo() -> Element {
    let mut budget = use_signal(|| 1200.0);

    rsx! {
        div { style: "display: grid; gap: var(--space-8); max-width: 420px;",
            Slider {
                label: "Monthly budget",
                min: 0.0,
                max: 5000.0,
                step: 100.0,
                value: Some(budget()),
                on_input: move |v| budget.set(v),
                format: |v: f64| format!("£{v}"),
            }
            Slider {
                label: "Summary length",
                min: 1.0,
                max: 10.0,
                default_value: 4.0,
                format: |v: f64| format!("{v} paragraphs"),
                name: "length",
            }
            Slider { label: "Sources per answer", default_value: 3.0, max: 10.0, disabled: true }
        }
    }
}
