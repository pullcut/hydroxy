use dioxus::prelude::*;
use hydroxy_dioxus::switch::Switch;

#[component]
pub fn SwitchDemo() -> Element {
    let mut alerts = use_signal(|| true);

    rsx! {
        div { style: "display: grid; gap: var(--space-5);",
            // Controlled.
            Switch {
                checked: Some(alerts()),
                on_checked_change: move |v| alerts.set(v),
                name: "alerts",
                "Email me when a summary is ready"
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);",
                if alerts() { "Alerts on" } else { "Alerts off" }
            }

            // Uncontrolled, and disabled.
            Switch { default_checked: false, "Share drafts with the team" }
            Switch { default_checked: true, disabled: true, "Keep source files" }
        }
    }
}
