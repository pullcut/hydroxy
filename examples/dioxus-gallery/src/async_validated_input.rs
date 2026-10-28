use dioxus::prelude::*;
use hydroxy_dioxus::async_validated_input::{AsyncValidatedInput, sleep, validator};

#[component]
pub fn AsyncValidatedInputDemo() -> Element {
    // Pretend the server takes a moment to check each name.
    let username = use_hook(|| {
        validator(|name: String| async move {
            sleep(700).await;
            if ["admin", "hydroxy", "tom"].contains(&name.trim().to_lowercase().as_str()) {
                Err(format!("\"{}\" is taken. Try another.", name.trim()))
            } else if name.trim().len() < 3 {
                Err("Use at least 3 characters.".to_string())
            } else {
                Ok(())
            }
        })
    });

    rsx! {
        div { style: "display: grid; gap: var(--space-8); max-width: 420px;",
            AsyncValidatedInput {
                label: "Workspace address",
                validator: username,
                hint: "Try \"admin\" or \"hydroxy\" to see a rejection.",
                valid_text: "That one is free.",
                placeholder: "your-team",
            }
        }
    }
}
