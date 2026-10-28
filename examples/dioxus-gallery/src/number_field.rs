use dioxus::prelude::*;
use hydroxy_dioxus::number_field::NumberField;

#[component]
pub fn NumberFieldDemo() -> Element {
    let mut seats = use_signal(|| Some(12.0));

    rsx! {
        div { style: "display: grid; gap: var(--space-8); max-width: 320px;",
            NumberField {
                label: "Seats",
                min: 1.0,
                max: 50.0,
                value: seats(),
                on_value_change: move |v| seats.set(v),
            }
            NumberField {
                label: "Monthly budget",
                default_value: 1200.0,
                min: 0.0,
                step: 50.0,
                large_step: 500.0,
                fraction_digits: 2,
                prefix: "£",
                name: "budget",
            }
            NumberField { label: "Discount", default_value: 10.0, suffix: "%", disabled: true }
        }
    }
}
