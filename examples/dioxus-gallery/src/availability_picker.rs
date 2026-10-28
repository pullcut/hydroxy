use std::collections::BTreeSet;

use dioxus::prelude::*;
use hydroxy_dioxus::availability_picker::{AvailabilityPicker, Date, Slot};

#[component]
pub fn AvailabilityPickerDemo() -> Element {
    // A Monday. Dates are plain values, so the demo reads no clock.
    let monday = Date::new(2026, 11, 2).unwrap_or_else(|| Date::from_days(0));
    let mut slots = use_signal(BTreeSet::<Slot>::new);

    rsx! {
        div { style: "display: grid; gap: var(--space-8); max-width: 640px;",
            // Controlled. Drag across the grid, or focus a cell and press Space.
            AvailabilityPicker {
                label: "Your availability this week",
                start: monday,
                days: 5,
                timezone: "Europe/London",
                value: Some(slots()),
                on_value_change: move |v| slots.set(v),
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);",
                "{slots().len()} half-hour slots selected"
            }
        }
    }
}
