use dioxus::prelude::*;
use hydroxy_dioxus::date_range_picker::{Date, DateRange, DateRangePicker};

#[component]
pub fn DateRangePickerDemo() -> Element {
    let mut stay = use_signal(|| Some(DateRange::new(Date::new(2026, 11, 12).unwrap(), Date::new(2026, 11, 18).unwrap())));
    let shown = stay().map_or_else(|| "No dates".to_string(), |r| format!("{} to {}", r.start, r.end));

    rsx! {
        div { style: "display: grid; gap: var(--space-7);",
            // Controlled. Click a start day, then an end day.
            DateRangePicker { label: "Stay", value: stay(), on_change: move |r| stay.set(Some(r)) }
            span { "{shown}" }

            // Uncontrolled, nothing before 1 November, submitted as `trip-start` and `trip-end`.
            DateRangePicker { label: "Trip", name: "trip", min: Date::new(2026, 11, 1) }

            DateRangePicker { label: "Booked", disabled: true }
        }
    }
}
