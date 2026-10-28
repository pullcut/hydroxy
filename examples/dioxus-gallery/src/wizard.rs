use dioxus::prelude::*;
use hydroxy_dioxus::input::Input;
use hydroxy_dioxus::wizard::{Wizard, WizardPanel};

#[component]
pub fn WizardDemo() -> Element {
    let mut name = use_signal(String::new);
    let mut error = use_signal(|| false);
    let mut done = use_signal(|| false);

    rsx! {
        div { style: "max-width: 520px;",
            Wizard {
                steps: vec!["Workspace".to_string(), "Sources".to_string(), "Review".to_string()],
                // The first step needs a name before Next moves on.
                can_advance: move |step: usize| {
                    let ok = step != 0 || !name.peek().trim().is_empty();
                    error.set(!ok);
                    ok
                },
                on_finish: move |()| done.set(true),
                WizardPanel { index: 0,
                    Input {
                        label: "Workspace name",
                        value: name(),
                        on_input: move |v| name.set(v),
                        error: error().then(|| "Give the workspace a name".to_string()),
                    }
                }
                WizardPanel { index: 1,
                    p { style: "margin: 0; color: var(--text-2);", "Connect Drive, Notion or Slack. You can add more later." }
                }
                WizardPanel { index: 2,
                    p { style: "margin: 0; color: var(--text-2);",
                        if done() { "Created {name}." } else { "Ready to create {name}." }
                    }
                }
            }
        }
    }
}
