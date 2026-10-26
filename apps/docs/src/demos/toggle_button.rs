use leptos::prelude::*;
use oxide_leptos::icon::{Icon, IconName};
use oxide_leptos::toggle_button::ToggleButton;

pub fn demo() -> AnyView {
    let (starred, set_starred) = signal(false);

    view! {
        <div style="display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-3);">
            // Controlled.
            <ToggleButton pressed=starred on_pressed_change=move |v| set_starred.set(v)>
                <Icon name=IconName::Star size=14 />
                {move || if starred.get() { "Starred" } else { "Star" }}
            </ToggleButton>

            // Icon only: give it a label.
            <ToggleButton label="Pin" default_pressed=true>
                <Icon name=IconName::Mark size=14 />
            </ToggleButton>

            <ToggleButton>"Show sources"</ToggleButton>
            <ToggleButton disabled=true>"Disabled"</ToggleButton>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "toggle-button",
    name: "Toggle button",
    group: "Forms",
    summary: "A button that stays pressed, such as Bold or Pin. Use a switch for settings that read as on and off.",
    demo,
    leptos: include_str!("toggle_button.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/toggle_button.rs"),
    parts: &[&["ToggleButton", "root", "Native <button aria-pressed>. data-state=pressed|unpressed, disabled"]],
    keys: &[&["Space / Enter", "Press or release the focused button"], &["Tab", "Moves to the next control"]],
    props: &[
        &["pressed", "Option<bool>", "Controlled value. Pair with on_pressed_change"],
        &["default_pressed", "bool", "Initial value when uncontrolled. Default false"],
        &["disabled", "bool", "Ignore presses"],
        &["label", "String", "aria-label, for icon-only buttons"],
        &["on_pressed_change", "Callback<bool>", "Called with the new value"],
        &["children", "View", "The button's content"],
    ],
};
