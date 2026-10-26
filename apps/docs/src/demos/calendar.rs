use leptos::prelude::*;
use oxide_leptos::calendar::{Calendar, Date};

pub fn demo() -> AnyView {
    let (date, set_date) = signal(Date::new(2026, 11, 12));

    view! {
        <div style="display: flex; flex-wrap: wrap; gap: var(--space-9); align-items: start;">
            // Controlled.
            <div style="display: grid; gap: var(--space-4);">
                <Calendar label="Delivery date" value=date on_change=move |d| set_date.set(Some(d)) />
                <span>
                    {move || date.get().map_or_else(|| "Nothing picked".to_string(), |d| format!("Picked {d}"))}
                </span>
            </div>

            // Uncontrolled, weeks from Sunday, a bounded range with two days blocked.
            <Calendar
                label="Review slot"
                week_start=0
                min=Date::new(2026, 11, 2)
                max=Date::new(2026, 11, 27)
                unavailable=vec![Date::new(2026, 11, 11).unwrap(), Date::new(2026, 11, 18).unwrap()]
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "calendar",
    name: "Calendar",
    group: "Forms",
    summary: "An inline month grid for picking one date. Keyboard-first, with min/max bounds, blocked days and a today marker.",
    demo,
    leptos: include_str!("calendar.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/calendar.rs"),
    parts: &[
        &["Calendar", "root", "role=group with the calendar's name. data-disabled"],
        &["Calendar", "header", "Previous button, month label, next button"],
        &["Calendar", "previous / next", "Month buttons. Disabled when the month is out of range"],
        &["Calendar", "label", "Visible month, aria-live so changes are announced"],
        &["Calendar", "grid", "Native <table role=grid>, named by the label"],
        &["Calendar", "weekday", "<th> column heading, full name in abbr"],
        &["Calendar", "week / cell", "Row and role=gridcell; aria-selected on the chosen day"],
        &["Calendar", "day", "<button>. data-selected, data-today, data-unavailable, data-outside"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous / next day"],
        &["ArrowUp / ArrowDown", "Same day last / next week"],
        &["PageUp / PageDown", "Same day last / next month"],
        &["Shift + PageUp / PageDown", "Same day last / next year"],
        &["Home / End", "First / last day of the week"],
        &["Enter / Space", "Pick the focused day"],
    ],
    props: &[
        &["value", "Option<Date>", "Controlled value. Pair with on_change"],
        &["default_value", "Option<Date>", "Initial value when uncontrolled"],
        &["min / max", "Option<Date>", "Selectable range. Days outside it can't be focused"],
        &["unavailable", "Vec<Date>", "Days that can be focused but not picked"],
        &["week_start", "u8", "First day of the week, 0 = Sunday. Default 1 (Monday)"],
        &["today", "Option<Date>", "Override today. Default the browser's local date"],
        &["disabled", "bool", "Block every interaction"],
        &["label", "String", "Accessible name. Default \"Calendar\""],
        &["on_change", "Callback<Date>", "Called with the picked date"],
    ],
};
