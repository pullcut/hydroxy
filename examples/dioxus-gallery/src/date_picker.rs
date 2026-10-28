use dioxus::prelude::*;
use hydroxy_dioxus::date_picker::{Date, DatePicker};

#[component]
pub fn DatePickerDemo() -> Element {
    let mut due = use_signal(|| Date::new(2026, 11, 12));
    let shown = due().map_or_else(|| "No due date".to_string(), |d| format!("Due {d}"));

    rsx! {
        div { style: "display: grid; gap: var(--space-7); max-width: 320px;",
            // Controlled. Type a date as YYYY-MM-DD or pick one from the calendar.
            DatePicker { label: "Due date", value: due(), on_change: move |d| due.set(d) }
            span { "{shown}" }

            // Uncontrolled, submitted with a form as `start`, nothing before 1 November.
            DatePicker { label: "Start date", name: "start", min: Date::new(2026, 11, 1) }

            DatePicker { label: "Signed on", default_value: Date::new(2026, 3, 4).unwrap(), disabled: true }
        }
    }
}
