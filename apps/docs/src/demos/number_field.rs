use hydroxy_leptos::number_field::NumberField;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (seats, set_seats) = signal(Some(12.0));

    view! {
        <div style="display: grid; gap: var(--space-8); max-width: 320px;">
            // Controlled, bounded.
            <NumberField
                label="Seats"
                min=1.0
                max=50.0
                value=seats
                on_value_change=move |v| set_seats.set(v)
            />

            // Uncontrolled money: two decimals, a prefix and a large step.
            <NumberField
                label="Monthly budget"
                default_value=1200.0
                min=0.0
                step=50.0
                large_step=500.0
                fraction_digits=2
                prefix="£"
                name="budget"
            />

            <NumberField label="Discount" default_value=10.0 suffix="%" disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "number-field",
    name: "Number field",
    group: "Forms",
    summary: "Type or step a number. Drafts commit on Enter or blur, clamped to the range; arrows and the buttons step on a grid.",
    demo,
    leptos: include_str!("number_field.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/number_field.rs"),
    parts: &[
        &["NumberField", "root", "Wrapper. data-disabled, data-readonly"],
        &["NumberField", "label", "<label for> the control"],
        &["NumberField", "field", "Bordered row holding the input and steppers"],
        &["NumberField", "affix", "Optional prefix or suffix text"],
        &["NumberField", "control", "Text input with role=spinbutton and aria-valuenow/min/max"],
        &["NumberField", "steppers", "The button pair"],
        &["NumberField", "decrement", "Step down. Disabled at min. Out of the tab order"],
        &["NumberField", "increment", "Step up. Disabled at max. Out of the tab order"],
    ],
    keys: &[
        &["ArrowUp / ArrowDown", "Up or down one step"],
        &["PageUp / PageDown", "Up or down a large step"],
        &["Home / End", "Minimum / maximum, when set"],
        &["Enter", "Commit the typed value"],
    ],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["value", "Option<f64>", "Controlled value. Pair with on_value_change"],
        &["default_value", "Option<f64>", "Initial value when uncontrolled. Empty without one"],
        &["min / max", "Option<f64>", "Bounds. Typed values are clamped"],
        &["step / large_step", "f64", "Step, default 1; large step, default ten steps"],
        &["fraction_digits", "Option<u8>", "Fixed decimal places for display and rounding"],
        &["prefix / suffix", "String", "Text inside the field, such as £ or %"],
        &["on_value_change", "Callback<Option<f64>>", "Called on commit or step. None is empty"],
        &["disabled / read_only", "bool", "Block changes"],
        &["name", "String", "Form field name. Submits the committed number"],
        &["placeholder", "String", "Shown while empty"],
    ],
};
