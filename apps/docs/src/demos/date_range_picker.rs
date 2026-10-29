use hydroxy_leptos::date_range_picker::{Date, DateRange, DateRangePicker};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (stay, set_stay) =
        signal(Some(DateRange::new(Date::new(2026, 11, 12).unwrap(), Date::new(2026, 11, 18).unwrap())));

    view! {
        <div style="display: grid; gap: var(--space-7);">
            // Controlled. Click a start day, then an end day.
            <DateRangePicker label="Stay" value=stay on_change=move |r| set_stay.set(Some(r)) />
            <span>
                {move || {
                    stay.get()
                        .map_or_else(
                            || "No dates".to_string(),
                            |r| format!("{} to {}", r.start, r.end),
                        )
                }}
            </span>

            // Uncontrolled, nothing before 1 November, submitted as `trip-start` and `trip-end`.
            <DateRangePicker label="Trip" name="trip" min=Date::new(2026, 11, 1) />

            <DateRangePicker label="Booked" disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "date-range-picker",
    name: "Date range picker",
    group: "Forms",
    summary: "Pick a start and end day on two months at once. The days between light up as you move, and it closes on the second click.",
    demo,
    leptos: include_str!("date_range_picker.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/date_range_picker.rs"),
    parts: &[
        &["DateRangePicker", "root", "Wrapper. data-disabled"],
        &["DateRangePicker", "label-text", "<label for> the trigger"],
        &["DateRangePicker", "field", "The popover's CSS anchor"],
        &["DateRangePicker", "trigger", "Button showing the range. aria-expanded, data-state, data-empty"],
        &["DateRangePicker", "content", "Native popover=auto, role=dialog, holding two months"],
        &["DateRangePicker", "month / header / label", "One month, its title row and live label"],
        &["DateRangePicker", "previous / next", "Month buttons, on the first and second month"],
        &["DateRangePicker", "grid / weekday / week", "role=grid table of days, named by the month label"],
        &[
            "DateRangePicker",
            "cell",
            "aria-selected inside the range. data-range-start, data-range-end, data-in-range, data-preview",
        ],
        &[
            "DateRangePicker",
            "day",
            "Button. data-today, data-unavailable, data-outside. Only the focused day is tabbable",
        ],
    ],
    keys: &[
        &["Enter / Space (on the button)", "Open or close the calendar"],
        &["Enter / Space (on a day)", "First press sets the start; second sets the end, closes and focuses the button"],
        &["Arrows / Home / End", "Move a day or a week, or to the week's ends. With a start set, the range follows"],
        &["PageUp / PageDown", "Move a month. With Shift, a year"],
        &["Escape", "Close, drop an unfinished range and focus the button"],
    ],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["value", "Option<DateRange>", "Controlled range. Pair with on_change"],
        &["default_value", "Option<DateRange>", "Initial range when uncontrolled"],
        &["min / max", "Option<Date>", "Selectable range"],
        &["unavailable", "Vec<Date>", "Days that can't be picked"],
        &["week_start", "u8", "First day of the week, 0 = Sunday. Default 1 (Monday)"],
        &["today", "Option<Date>", "Override today. Default the browser's local date"],
        &["placeholder", "String", "Button text while empty. Default \"Select dates\""],
        &["disabled", "bool", "Block opening"],
        &["name", "String", "Submits name-start and name-end as YYYY-MM-DD"],
        &["on_change", "Callback<DateRange>", "Called with the completed range"],
    ],
    events: &[],
};
