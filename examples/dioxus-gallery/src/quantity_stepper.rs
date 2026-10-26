use dioxus::prelude::*;
use oxide_dioxus::quantity_stepper::QuantityStepper;

#[component]
pub fn QuantityStepperDemo() -> Element {
    let mut oat = use_signal(|| Some(2.0));
    let shown = format!("{} in the basket", oat().unwrap_or_default());

    rsx! {
        div { style: "display: grid; gap: var(--space-7); justify-items: start;",
            // Controlled, between 1 and 10.
            QuantityStepper {
                label: "Quantity of oat milk",
                min: 1.0,
                max: 10.0,
                value: oat(),
                on_change: move |v| oat.set(Some(v)),
            }
            span { "{shown}" }

            // Uncontrolled, in fives, submitted as `crates`.
            QuantityStepper { label: "Quantity of crates", default_value: 5.0, step: 5.0, name: "crates" }

            QuantityStepper { label: "Quantity locked", default_value: 3.0, disabled: true }
        }
    }
}
