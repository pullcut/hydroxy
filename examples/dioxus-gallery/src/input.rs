use dioxus::prelude::*;
use oxide_dioxus::input::{Input, Size};

#[component]
pub fn InputDemo() -> Element {
    let mut email = use_signal(|| String::from("jo.marsh"));
    // Validate as the user types; an error replaces the hint.
    let email_error =
        (!email().is_empty() && !email().contains('@')).then(|| "Enter an email address with an @.".to_string());

    rsx! {
        div { style: "display: grid; gap: var(--space-7); max-width: 360px;",
            Input {
                label: "Project name",
                placeholder: "Q3 customer research",
                hint: "Shown to everyone in the workspace.",
            }
            Input {
                label: "Monthly budget",
                mono: true,
                prefix: "£",
                suffix: "/ month",
                default_value: "1,200.00",
            }
            Input {
                label: "Invite by email",
                input_type: "email",
                value: email(),
                on_input: move |v| email.set(v),
                hint: "They join as a viewer.",
                error: email_error,
            }
            Input {
                label: "Workspace ID",
                size: Size::Sm,
                mono: true,
                default_value: "ws_0b7f21",
                disabled: true,
            }
        }
    }
}
