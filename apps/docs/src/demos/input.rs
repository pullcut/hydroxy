use leptos::prelude::*;
use hydroxy_leptos::input::*;

pub fn demo() -> AnyView {
    let (email, set_email) = signal(String::from("jo.marsh"));
    // Validate as the user types; an error replaces the hint.
    let email_error = move || {
        let v = email.get();
        (!v.is_empty() && !v.contains('@')).then(|| "Enter an email address with an @.".to_string())
    };

    view! {
        <div style="display: grid; gap: var(--space-7); max-width: 360px;">
            <Input
                label="Project name"
                placeholder="Q3 customer research"
                hint="Shown to everyone in the workspace."
            />
            <Input label="Monthly budget" mono=true prefix="£" suffix="/ month" default_value="1,200.00" />
            <Input
                label="Invite by email"
                input_type="email"
                value=email
                on_input=move |v| set_email.set(v)
                hint="They join as a viewer."
                error=Signal::derive(email_error)
            />
            <Input label="Workspace ID" size=Size::Sm mono=true default_value="ws_0b7f21" disabled=true />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "input",
    name: "Input",
    group: "Forms",
    summary: "A labelled text field with a hint or an error below it. Use mono for money and rates.",
    demo,
    leptos: include_str!("input.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/input.rs"),
    parts: &[
        &["Input", "root", "Wrapper. data-size, data-mono, data-invalid, data-disabled"],
        &["Input", "label", "<label for> the control"],
        &["Input", "field", "The bordered box. Takes the focus ring via :focus-within"],
        &["Input", "affix", "Prefix or suffix text inside the field"],
        &["Input", "control", "Native <input>. aria-describedby the hint or error, aria-invalid on error"],
        &["Input", "hint", "Help text. Hidden while there is an error"],
        &["Input", "error", "Error text"],
    ],
    keys: &[],
    props: &[
        &["label", "String", "Required. Visible label"],
        &["hint", "Option<String>", "Help text under the field"],
        &["error", "Option<String>", "Replaces the hint and marks the field invalid"],
        &["value", "Option<String>", "Controlled value. Pair with on_input"],
        &["default_value", "String", "Initial value when uncontrolled"],
        &["on_input", "Callback<String>", "Called on every keystroke"],
        &["input_type", "String", "Native type. Default text"],
        &["placeholder", "String", "Placeholder text"],
        &["name", "String", "Form field name"],
        &["disabled", "bool", "Block editing"],
        &["required", "bool", "Native required"],
        &["mono", "bool", "Monospaced text, for money and rates"],
        &["size", "Size", "Sm (36px), Md (44px, default) or Lg (52px)"],
        &["prefix / suffix", "String", "Text inside the field, such as £ or %"],
    ],
};
