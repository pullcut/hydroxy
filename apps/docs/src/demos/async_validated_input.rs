use leptos::prelude::*;
use oxide_leptos::async_validated_input::{AsyncValidatedInput, sleep, validator};

pub fn demo() -> AnyView {
    // Pretend the server takes a moment to check each name.
    let username = validator(|name: String| async move {
        sleep(700).await;
        if ["admin", "oxide", "tom"].contains(&name.trim().to_lowercase().as_str()) {
            Err(format!("\"{}\" is taken. Try another.", name.trim()))
        } else if name.trim().len() < 3 {
            Err("Use at least 3 characters.".to_string())
        } else {
            Ok(())
        }
    });

    view! {
        <div style="display: grid; gap: var(--space-8); max-width: 420px;">
            <AsyncValidatedInput
                label="Workspace address"
                validator=username
                hint="Try \"admin\" or \"oxide\" to see a rejection."
                valid_text="That one is free."
                placeholder="your-team"
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "async-validated-input",
    name: "Async validated input",
    group: "Forms",
    summary: "A text field that checks its value with your async validator once typing pauses. It shows checking, valid and invalid states and ignores answers about text that has since changed.",
    demo,
    leptos: include_str!("async_validated_input.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/async_validated_input.rs"),
    parts: &[
        &["AsyncValidatedInput", "root", "Wrapper around Input. data-status is idle, validating, valid or invalid; aria-busy while validating"],
        &["AsyncValidatedInput", "live", "role=status region that says the checking, valid or error text"],
        &["Input", "hint / error", "The state text. An error replaces the hint and sets aria-invalid on the field"],
    ],
    keys: &[
        &["Typing", "Restarts the wait. The validator runs once typing pauses for the debounce time"],
    ],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["validator", "Validator", "Required. Async fn from the text to Ok(()) or Err(message). Build one with validator()"],
        &["value", "Option<String>", "Controlled value. Pair with on_change"],
        &["default_value", "String", "Initial value when uncontrolled"],
        &["on_change", "Callback<String>", "Called on every keystroke"],
        &["debounce_ms", "u32", "Quiet time before validating. Default 400"],
        &["hint", "String", "Help text while there is nothing to report"],
        &["validating_text / valid_text", "String", "Words for those states. Default \"Checking…\" and \"Looks good.\""],
        &["placeholder / name / disabled", "", "As on Input"],
    ],
};
