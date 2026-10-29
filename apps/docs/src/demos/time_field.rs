use hydroxy_leptos::time_field::TimeField;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (start, set_start) = signal("09:30".to_string());
    let error = move || (start.get().as_str() < "08:00").then(|| "Office opens at 08:00".to_string());

    view! {
        <div style="display: grid; gap: var(--space-8);">
            // Controlled, with validation.
            <TimeField
                label="Start time"
                value=start
                on_input=move |v| set_start.set(v)
                hint="When the daily digest is sent"
                error=Signal::derive(error)
            />

            // Uncontrolled, to the second.
            <TimeField label="Reminder" default_value="17:45:00" step=1 name="reminder" />

            <TimeField label="Cut-off" default_value="23:00" disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "time-field",
    name: "Time field",
    group: "Forms",
    summary: "A time of day. A styled native time input: segmented entry, the locale's 12- or 24-hour clock and a phone picker for free.",
    demo,
    leptos: include_str!("time_field.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/time_field.rs"),
    parts: &[
        &["TimeField", "root", "Wrapper. data-invalid, data-disabled"],
        &["TimeField", "label", "<label for> the control"],
        &["TimeField", "control", "Native <input type=time>. Segments get an accent chip when focused"],
        &["TimeField", "hint", "Help text, linked by aria-describedby"],
        &["TimeField", "error", "Replaces the hint and sets aria-invalid"],
    ],
    keys: &[
        &["ArrowUp / ArrowDown", "Spin the focused segment"],
        &["ArrowLeft / ArrowRight", "Previous / next segment"],
        &["0-9, A / P", "Type into the focused segment"],
        &["Backspace", "Clear the focused segment"],
    ],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["value", "Option<String>", "Controlled \"HH:MM\" (24-hour). Pair with on_input"],
        &["default_value", "String", "Initial value when uncontrolled"],
        &["min / max", "String", "Allowed range, as \"HH:MM\""],
        &["step", "u32", "Seconds. Below 60 adds a seconds segment"],
        &["on_input", "Callback<String>", "Called as segments change. Empty while incomplete"],
        &["hint / error", "String", "Text below the field. An error replaces the hint"],
        &["disabled / required", "bool", "Native disabled and required"],
        &["name", "String", "Form field name"],
    ],
    events: &[],
};
