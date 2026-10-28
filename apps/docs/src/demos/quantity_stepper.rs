use hydroxy_leptos::quantity_stepper::QuantityStepper;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (oat, set_oat) = signal(Some(2.0));

    view! {
        <div style="display: grid; gap: var(--space-7); justify-items: start;">
            // Controlled, between 1 and 10.
            <QuantityStepper label="Quantity of oat milk" min=1.0 max=10.0 value=oat on_change=move |v| set_oat.set(Some(v)) />
            <span>{move || format!("{} in the basket", oat.get().unwrap_or_default())}</span>

            // Uncontrolled, in fives, submitted as `crates`.
            <QuantityStepper label="Quantity of crates" default_value=5.0 step=5.0 name="crates" />

            <QuantityStepper label="Quantity locked" default_value=3.0 disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "quantity-stepper",
    name: "Quantity stepper",
    group: "Forms",
    summary: "A compact minus, value and plus control for counts. The number field's behaviour in a tighter layout.",
    demo,
    leptos: include_str!("quantity_stepper.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/quantity_stepper.rs"),
    parts: &[
        &["QuantityStepper", "root", "Pill with role=group. data-disabled"],
        &["QuantityStepper", "decrement", "Step down. Disabled at min"],
        &["QuantityStepper", "control", "Text input with role=spinbutton and aria-valuenow/min/max"],
        &["QuantityStepper", "increment", "Step up. Disabled at max"],
    ],
    keys: &[
        &["ArrowUp / ArrowDown", "Up or down one step"],
        &["PageUp / PageDown", "Up or down ten steps"],
        &["Home / End", "Jump to min or max"],
        &["Enter / blur", "Commit typed text, clamped to the range"],
    ],
    props: &[
        &["label", "String", "Required. Accessible name, not shown. Also names the buttons"],
        &["value", "Option<f64>", "Controlled value. Pair with on_change"],
        &["default_value", "f64", "Initial value when uncontrolled. Default 1"],
        &["min / max", "f64 / Option<f64>", "Bounds. min defaults to 0"],
        &["step", "f64", "Amount per press. Default 1"],
        &["disabled", "bool", "Block changes"],
        &["name", "String", "Form field name; submits the number"],
        &["on_change", "Callback<f64>", "Called with the new value"],
    ],
};
