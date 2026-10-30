use hydroxy_leptos::password_field::{PasswordField, PasswordMode, Rule, RuleKind};
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let (valid, set_valid) = signal(false);
    let rules = vec![
        Rule::new(RuleKind::MinLength(12)),
        Rule::new(RuleKind::Uppercase),
        Rule::new(RuleKind::Digit),
        Rule::new(RuleKind::Symbol),
        Rule::new(RuleKind::NotCommon),
        Rule::new(RuleKind::MinScore(3)),
    ];

    view! {
        <div style="display: grid; gap: var(--space-7); max-width: 380px;">
            // Choosing a password: meter, live checklist, and the person's own details count against it.
            <div style="display: grid; gap: var(--space-2);">
                <PasswordField
                    label="New password"
                    rules=rules
                    user_inputs=vec!["Priya Raman".to_string(), "priya.raman@northwind.io".to_string()]
                    on_valid_change=move |v| set_valid.set(v)
                />
                <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">
                    {move || if valid.get() { "Ready to save" } else { "Meets every rule when the list is all ticked" }}
                </p>
            </div>

            // Signing in: autocomplete="current-password", no meter.
            <PasswordField label="Current password" mode=PasswordMode::Current hint="Press Caps Lock to see the warning." />
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "password-field",
    name: "Password field",
    group: "Forms",
    summary: "A password input with show and hide, a Caps Lock warning, and for new passwords a strength meter and a live checklist of rules.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("password_field.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/password_field.rs"),
    parts: &[
        &[
            "PasswordField",
            "root",
            "Wrapper. data-mode=\"new\" or \"current\", data-valid when every rule passes, data-disabled",
        ],
        &["PasswordField", "label", "<label> for the input"],
        &["PasswordField", "field", "Bordered box holding the input and the toggle"],
        &[
            "PasswordField",
            "control",
            "Native <input>, type password or text. autocomplete new-password or current-password, passwordrules from the rules",
        ],
        &["PasswordField", "toggle", "<button aria-pressed> named \"Show password\", aria-controls the input"],
        &["PasswordField", "caps-lock", "role=\"status\" warning, data-active while Caps Lock is on"],
        &["PasswordField", "meter", "role=\"meter\" 0 to 4 with aria-valuetext. data-strength very-weak to strong"],
        &["PasswordField", "meter-bar", "Four bars, data-on when lit"],
        &["PasswordField", "strength", "Polite live text: \"Strength: Fair\", plus a warning when weak"],
        &[
            "PasswordField",
            "rules / rule",
            "Checklist. Each rule has data-met and says \"met\" or \"not met\" to screen readers",
        ],
    ],
    keys: &[
        &["Tab", "Moves from the input to the show/hide button"],
        &["Enter / Space", "On the button: show or hide the password"],
    ],
    props: &[
        &["label", "String", "Visible label. Default \"Password\""],
        &["mode", "PasswordMode", "New (meter, rules) or Current (sign-in). Default New"],
        &["value", "Option<String>", "Controlled value. Pair with on_change"],
        &["on_change", "Callback<String>", "Called on every edit"],
        &["on_valid_change", "Callback<bool>", "Called on every edit with whether every rule passes"],
        &["rules", "Vec<Rule>", "The checklist. Default: 12 characters, mixed case, a number, not common"],
        &["user_inputs", "Vec<String>", "Name, email and similar words that make a password easy to guess"],
        &["hint", "String", "Help text under the field"],
        &["name", "String", "Form field name"],
        &["disabled / readonly / required", "bool", "Native input states"],
    ],
    events: &[],
};
