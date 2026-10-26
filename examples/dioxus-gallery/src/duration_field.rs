use dioxus::prelude::*;
use oxide_dioxus::duration_field::DurationField;

#[component]
pub fn DurationFieldDemo() -> Element {
    let mut seconds = use_signal(|| Some(5415));
    let shown = seconds().map_or_else(|| "Empty".to_string(), |s| format!("{s} seconds"));

    rsx! {
        div { style: "display: grid; gap: var(--space-7); max-width: 320px;",
            // Controlled. Total seconds in, total seconds out.
            DurationField { label: "Estimate", value: seconds(), on_change: move |s| seconds.set(s) }
            span { "{shown}" }

            // Uncontrolled, hours and minutes only, submitted as `timebox`.
            DurationField { label: "Timebox", default_value: 1800, show_seconds: false, name: "timebox" }

            DurationField { label: "Logged", default_value: 3600, disabled: true }
        }
    }
}
