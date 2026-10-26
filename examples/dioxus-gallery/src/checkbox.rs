use dioxus::prelude::*;
use oxide_dioxus::checkbox::{Checkbox, CheckedState};

#[component]
pub fn CheckboxDemo() -> Element {
    let mut interviews = use_signal(|| true);
    let mut tickets = use_signal(|| false);
    // The parent is indeterminate while its children disagree.
    let all = match (interviews(), tickets()) {
        (true, true) => CheckedState::Checked,
        (false, false) => CheckedState::Unchecked,
        _ => CheckedState::Indeterminate,
    };

    rsx! {
        div { style: "display: grid; gap: var(--space-4);",
            Checkbox {
                checked: Some(all),
                on_checked_change: move |on| {
                    interviews.set(on);
                    tickets.set(on);
                },
                "All sources"
            }
            div { style: "display: grid; gap: var(--space-4); padding-left: var(--space-8);",
                Checkbox {
                    checked: Some(interviews().into()),
                    on_checked_change: move |on| interviews.set(on),
                    name: "sources",
                    value: "interviews",
                    "interviews-q3.pdf"
                }
                Checkbox {
                    checked: Some(tickets().into()),
                    on_checked_change: move |on| tickets.set(on),
                    name: "sources",
                    value: "tickets",
                    "tickets-sept.csv"
                }
            }
            Checkbox { default_checked: CheckedState::Checked, "Email me when the summary is ready" }
            Checkbox { disabled: true, "Share with the workspace" }
        }
    }
}
