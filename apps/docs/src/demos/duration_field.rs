use leptos::prelude::*;
use oxide_leptos::duration_field::DurationField;

pub fn demo() -> AnyView {
    let (seconds, set_seconds) = signal(Some(5415));

    view! {
        <div style="display: grid; gap: var(--space-7); max-width: 320px;">
            // Controlled. Total seconds in, total seconds out.
            <DurationField label="Estimate" value=seconds on_change=move |s| set_seconds.set(s) />
            <span>
                {move || seconds.get().map_or_else(|| "Empty".to_string(), |s| format!("{s} seconds"))}
            </span>

            // Uncontrolled, hours and minutes only, submitted as `timebox`.
            <DurationField label="Timebox" default_value=1800 show_seconds=false name="timebox" />

            <DurationField label="Logged" default_value=3600 disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "duration-field",
    name: "Duration field",
    group: "Forms",
    summary: "A length of time typed into hours, minutes and seconds. The value is whole seconds.",
    demo,
    leptos: include_str!("duration_field.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/duration_field.rs"),
    parts: &[
        &["DurationField", "root", "Wrapper. data-disabled"],
        &["DurationField", "label", "<label for> the first segment"],
        &["DurationField", "field", "Bordered row with role=group, named by the label"],
        &["DurationField", "segment", "Text input with role=spinbutton and aria-valuenow/min/max/text: hours, minutes, optional seconds"],
        &["DurationField", "separator", "The colon between segments. aria-hidden"],
    ],
    keys: &[
        &["0-9", "Type into the segment. It moves on once no further digit could fit"],
        &["ArrowUp / ArrowDown", "Step the segment and wrap (0-59, hours 0-99)"],
        &["Home / End", "Lowest or highest value for the segment"],
        &["ArrowLeft / ArrowRight", "Previous or next segment"],
        &["Backspace / Delete", "Empty the segment"],
    ],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["value", "Option<u32>", "Controlled value in seconds. Pair with on_change"],
        &["default_value", "Option<u32>", "Initial seconds when uncontrolled"],
        &["show_seconds", "bool", "Show the seconds segment. Default true"],
        &["disabled / read_only", "bool", "Block editing"],
        &["name", "String", "Form field name; submits total seconds"],
        &["on_change", "Callback<Option<u32>>", "Called with total seconds, or None once every part is empty. Empty parts count as zero"],
    ],
};
