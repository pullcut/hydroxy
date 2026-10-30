use hydroxy_leptos::date_field::{Date, DateField, DateOrder};
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let (born, set_born) = signal(Date::new(1990, 4, 17));

    view! {
        <div style="display: grid; gap: var(--space-7); max-width: 320px;">
            // Controlled. Type 17041990, or arrow through each part.
            <DateField label="Date of birth" value=born on_change=move |d| set_born.set(d) />
            <span>
                {move || born.get().map_or_else(|| "Incomplete date".to_string(), |d| format!("Born {d}"))}
            </span>

            // Uncontrolled, month first, nothing before 2026, submitted as `start`.
            <DateField label="Start" order=DateOrder::Mdy min=Date::new(2026, 1, 1) name="start" />

            <DateField label="Signed on" default_value=Date::new(2026, 3, 4).unwrap() disabled=true />
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "date-field",
    name: "Date field",
    group: "Forms",
    summary: "A date typed into day, month and year segments. Digits fill a part and move on; arrows step and wrap.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("date_field.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/date_field.rs"),
    parts: &[
        &["DateField", "root", "Wrapper. data-disabled, data-invalid"],
        &["DateField", "label", "<label for> the first segment"],
        &["DateField", "field", "Bordered row with role=group, named by the label"],
        &[
            "DateField",
            "segment",
            "Text input with role=spinbutton and aria-valuenow/min/max/text, one each for day, month, year",
        ],
        &["DateField", "separator", "The slash between segments. aria-hidden"],
    ],
    keys: &[
        &["0-9", "Type into the segment. It moves on once no further digit could fit"],
        &["ArrowUp / ArrowDown", "Step the segment and wrap. An empty one starts from today"],
        &["Home / End", "Lowest or highest value for the segment"],
        &["ArrowLeft / ArrowRight", "Previous or next segment"],
        &["Backspace / Delete", "Empty the segment"],
    ],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["value", "Option<Date>", "Controlled value. Pair with on_change"],
        &["default_value", "Option<Date>", "Initial value when uncontrolled"],
        &["min / max", "Option<Date>", "Allowed range. A date outside it marks the field invalid and reports None"],
        &["order", "DateOrder", "Dmy (default), Mdy or Ymd"],
        &["today", "Option<Date>", "Where an empty segment starts. Default the browser's local date"],
        &["disabled / read_only", "bool", "Block editing"],
        &["name", "String", "Form field name; submits YYYY-MM-DD"],
        &[
            "on_change",
            "Callback<Option<Date>>",
            "Called with the date once all three parts make an allowed one, and None when that stops being true",
        ],
    ],
    events: &[],
};
