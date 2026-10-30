use hydroxy_leptos::availability_picker::{AvailabilityPicker, Date, Slot};
use leptos::prelude::*;
use std::collections::BTreeSet;

#[component]
pub fn Demo() -> impl IntoView {
    // A Monday. Dates are plain values, so the demo reads no clock.
    let monday = Date::new(2026, 11, 2).unwrap_or_else(|| Date::from_days(0));
    let (slots, set_slots) = signal(BTreeSet::<Slot>::new());

    view! {
        <div style="display: grid; gap: var(--space-8); max-width: 640px;">
            // Controlled. Drag across the grid, or focus a cell and press Space.
            <AvailabilityPicker
                label="Your availability this week"
                start=monday
                days=5
                timezone="Europe/London"
                value=slots
                on_value_change=move |v| set_slots.set(v)
            />
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">
                {move || format!("{} half-hour slots selected", slots.get().len())}
            </p>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "availability-picker",
    name: "Availability picker",
    group: "Forms",
    summary: "A week grid of time slots. Drag to paint a block available or clear it, or paint with the keyboard. Shows the time zone and a plain-language summary.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("availability_picker.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/availability_picker.rs"),
    parts: &[
        &["AvailabilityPicker", "root", "Wraps everything. data-painting during a drag"],
        &["(inside)", "timezone", "\"Times shown in Europe/London\""],
        &["(inside)", "grid", "role=grid. A CSS grid with a time column and one column per day"],
        &["(inside)", "day", "role=columnheader, such as \"Mon 2\""],
        &["(inside)", "time", "role=rowheader, shown on the hour"],
        &[
            "(inside)",
            "cell",
            "role=gridcell, aria-selected when available, data-available. Roving tabindex; the name reads \"Mon 2, 09:00 to 09:30, available\"",
        ],
        &["(inside)", "summary", "aria-live list of the chosen ranges, one line per day"],
    ],
    keys: &[
        &["Arrow keys", "Move between cells"],
        &["Space / Enter", "Toggle the focused cell"],
        &["Shift + Arrow keys", "Paint while moving: the first cell decides add or clear"],
        &["Home / End", "First or last cell in the row (Ctrl: of the grid)"],
        &["PageUp / PageDown", "Top or bottom of the column"],
        &["Pointer drag", "Paint a rectangle. Touch: tap to toggle a cell"],
    ],
    props: &[
        &["label", "String", "Required. Accessible name for the grid"],
        &["start", "Date", "Required. The first day shown"],
        &["timezone", "String", "Required. Shown above the grid; the slots are in this zone"],
        &["days", "u8", "Number of days. Default 7"],
        &["start_hour / end_hour", "u8", "Hours the rows span. Default 9 to 17"],
        &["slot_minutes", "u16", "Length of a slot. Default 30"],
        &[
            "value",
            "Option<BTreeSet<Slot>>",
            "Controlled slots, each a Date and a start minute. Pair with on_value_change",
        ],
        &["default_value", "BTreeSet<Slot>", "Initial slots when uncontrolled"],
        &["disabled", "bool", "Block edits"],
        &["on_value_change", "Callback<BTreeSet<Slot>>", "Called with every available slot after each change"],
    ],
    events: &[],
};
