use dioxus::prelude::*;
use hydroxy_dioxus::calendar::{Calendar, Date};

#[component]
pub fn CalendarDemo() -> Element {
    let mut date = use_signal(|| Date::new(2026, 11, 12));
    let picked = date().map_or_else(|| "Nothing picked".to_string(), |d| format!("Picked {d}"));

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: var(--space-9); align-items: start;",
            // Controlled.
            div { style: "display: grid; gap: var(--space-4);",
                Calendar { label: "Delivery date", value: date(), on_change: move |d| date.set(Some(d)) }
                span { "{picked}" }
            }

            // Uncontrolled, weeks from Sunday, a bounded range with two days blocked.
            Calendar {
                label: "Review slot",
                week_start: 0,
                min: Date::new(2026, 11, 2),
                max: Date::new(2026, 11, 27),
                unavailable: vec![Date::new(2026, 11, 11).unwrap(), Date::new(2026, 11, 18).unwrap()],
            }
        }
    }
}
