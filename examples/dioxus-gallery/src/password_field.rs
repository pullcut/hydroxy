use dioxus::prelude::*;
use oxide_dioxus::password_field::{PasswordField, PasswordMode, Rule, RuleKind};

#[component]
pub fn PasswordFieldDemo() -> Element {
    let mut valid = use_signal(|| false);
    let rules = vec![
        Rule::new(RuleKind::MinLength(12)),
        Rule::new(RuleKind::Uppercase),
        Rule::new(RuleKind::Digit),
        Rule::new(RuleKind::Symbol),
        Rule::new(RuleKind::NotCommon),
        Rule::new(RuleKind::MinScore(3)),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-7); max-width: 380px;",
            div { style: "display: grid; gap: var(--space-2);",
                PasswordField {
                    label: "New password",
                    rules,
                    user_inputs: vec!["Priya Raman".to_string(), "priya.raman@northwind.io".to_string()],
                    on_valid_change: move |v| valid.set(v),
                }
                p { style: "margin: 0; font: var(--text-xs); color: var(--text-3);",
                    if valid() { "Ready to save" } else { "Meets every rule when the list is all ticked" }
                }
            }
            PasswordField { label: "Current password", mode: PasswordMode::Current, hint: "Press Caps Lock to see the warning." }
        }
    }
}
