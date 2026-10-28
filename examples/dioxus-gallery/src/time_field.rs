use dioxus::prelude::*;
use hydroxy_dioxus::time_field::TimeField;

#[component]
pub fn TimeFieldDemo() -> Element {
    let mut start = use_signal(|| "09:30".to_string());
    let error = (start().as_str() < "08:00").then(|| "Office opens at 08:00".to_string());

    rsx! {
        div { style: "display: grid; gap: var(--space-8);",
            TimeField {
                label: "Start time",
                value: start(),
                on_input: move |v| start.set(v),
                hint: "When the daily digest is sent",
                error,
            }
            TimeField { label: "Reminder", default_value: "17:45:00", step: 1, name: "reminder" }
            TimeField { label: "Cut-off", default_value: "23:00", disabled: true }
        }
    }
}
