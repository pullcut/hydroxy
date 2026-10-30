use hydroxy_leptos::switch::Switch;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let (alerts, set_alerts) = signal(true);

    view! {
        // Controlled.
        <div style="display: grid; gap: var(--space-5);">
            <Switch checked=alerts on_checked_change=move |v| set_alerts.set(v) name="alerts">
                "Email me when a summary is ready"
            </Switch>
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">
                {move || if alerts.get() { "Alerts on" } else { "Alerts off" }}
            </p>

            // Uncontrolled, and disabled.
            <Switch default_checked=false>"Share drafts with the team"</Switch>
            <Switch default_checked=true disabled=true>"Keep source files"</Switch>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "switch",
    name: "Switch",
    group: "Forms",
    summary: "An on/off toggle for settings that apply immediately. Use a checkbox when the choice is submitted later.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("switch.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/switch.rs"),
    parts: &[
        &["Switch", "root", "A <label>, so clicking the text toggles. data-state, data-disabled"],
        &["(inside)", "control", "<button role=switch>, aria-checked, disabled, data-state=checked|unchecked"],
        &["(inside)", "thumb", "aria-hidden decoration that slides on"],
        &["(inside)", "label", "The children"],
        &["(inside)", "hidden input", "<input type=hidden name value> while checked, enabled and named"],
    ],
    keys: &[&["Space / Enter", "Toggles the focused switch"], &["Tab", "Moves to the next control"]],
    props: &[
        &["checked", "Option<bool>", "Controlled value. Pair with on_checked_change"],
        &["default_checked", "bool", "Initial value when uncontrolled. Default false"],
        &["disabled", "bool", "Ignore presses and leave the tab order"],
        &["name", "String", "Form field name. A checked switch submits value"],
        &["value", "String", "Submitted value. Default \"on\""],
        &["on_checked_change", "Callback<bool>", "Called with the new value"],
        &["children", "View", "The label"],
    ],
    events: &[],
};
