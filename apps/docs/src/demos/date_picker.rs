use hydroxy_leptos::date_picker::{Date, DatePicker};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (due, set_due) = signal(Date::new(2026, 11, 12));

    view! {
        <div style="display: grid; gap: var(--space-7); max-width: 320px;">
            // Controlled. Type a date as YYYY-MM-DD or pick one from the calendar.
            <DatePicker label="Due date" value=due on_change=move |d| set_due.set(d) />
            <span>
                {move || due.get().map_or_else(|| "No due date".to_string(), |d| format!("Due {d}"))}
            </span>

            // Uncontrolled, submitted with a form as `start`, nothing before 1 November.
            <DatePicker label="Start date" name="start" min=Date::new(2026, 11, 1) />

            <DatePicker label="Signed on" default_value=Date::new(2026, 3, 4).unwrap() disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "date-picker",
    name: "Date picker",
    group: "Forms",
    summary: "A date field: type an ISO date, or open a calendar in a popover. Closes on pick or Esc and hands focus back.",
    demo,
    leptos: include_str!("date_picker.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/date_picker.rs"),
    parts: &[
        &["DatePicker", "root", "Wrapper. data-disabled, data-invalid"],
        &["DatePicker", "label", "<label for> the text input"],
        &["DatePicker", "field", "Bordered row, and the popover's CSS anchor"],
        &[
            "DatePicker",
            "input",
            "Native text input taking YYYY-MM-DD. aria-invalid while the text isn't a usable date",
        ],
        &["DatePicker", "trigger", "Button opening the calendar. aria-expanded, data-state"],
        &["DatePicker", "content", "Native popover=auto, role=dialog, holding a Calendar"],
    ],
    keys: &[
        &["Enter (in the field)", "Tidy the text back to the current date"],
        &["Enter / Space (on the button)", "Open or close the calendar"],
        &["Arrows / PageUp / PageDown / Home / End", "Move around the calendar, as in Calendar"],
        &["Enter / Space (on a day)", "Pick it, close, focus the button"],
        &["Escape", "Close and focus the button"],
    ],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["value", "Option<Date>", "Controlled value. Pair with on_change"],
        &["default_value", "Option<Date>", "Initial value when uncontrolled"],
        &["min / max", "Option<Date>", "Selectable range, for typing and picking"],
        &["unavailable", "Vec<Date>", "Days that can't be picked"],
        &["week_start", "u8", "First day of the week, 0 = Sunday. Default 1 (Monday)"],
        &["today", "Option<Date>", "Override today. Default the browser's local date"],
        &["disabled", "bool", "Block editing and the calendar"],
        &["name", "String", "Form field name; submits YYYY-MM-DD"],
        &["on_change", "Callback<Option<Date>>", "Called with the new date, or None when cleared"],
    ],
};
