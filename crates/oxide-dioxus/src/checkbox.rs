//! `Checkbox`: a `label` wrapping a `role="checkbox"` button and its label text.

use dioxus::prelude::*;
use oxide_headless::checkbox::{
    self as h, CheckboxCommand, CheckboxEvent, CheckboxInput, CheckboxMachine, CheckboxState,
};
use oxide_statechart::Machine;

pub use oxide_headless::checkbox::{ChangeReason, CheckedState};

fn send(
    mut state: Signal<CheckboxState>,
    input: Memo<CheckboxInput>,
    on_checked_change: Option<EventHandler<bool>>,
    event: CheckboxEvent,
) {
    let step = CheckboxMachine::transition(&state.peek(), event, &input.peek());
    if let Some(next) = step.state {
        state.set(next);
    }
    for command in step.commands {
        let CheckboxCommand::CheckedChange { checked, .. } = command;
        if let Some(handler) = on_checked_change {
            handler.call(checked);
        }
    }
}

/// Tri-state checkbox. Children are the label.
#[component]
pub fn Checkbox(
    /// Controlled state. Pair with `on_checked_change`.
    #[props(default)]
    checked: ReadSignal<Option<CheckedState>>,
    /// Initial state when uncontrolled.
    #[props(default)]
    default_checked: CheckedState,
    /// Ignore clicks and keys.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Form field name. When set, a checked box submits `value`.
    name: Option<String>,
    /// Submitted value. Default `on`.
    #[props(default = "on".to_string())]
    value: String,
    /// Called with the proposed state when the user toggles.
    on_checked_change: Option<EventHandler<bool>>,
    /// The label.
    children: Element,
) -> Element {
    let input = use_memo(move || CheckboxInput { checked: checked(), default_checked, disabled: disabled() });
    let mut state = use_signal(|| CheckboxMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = CheckboxMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let a = h::control_attrs(&state.read(), &input.read());
    let hidden = name.filter(|_| h::submits(&state.read()));

    rsx! {
        label {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-state": a.data_state,
            "data-disabled": a.disabled.then_some(""),
            button {
                r#type: "button",
                role: a.role,
                "aria-checked": a.aria_checked,
                disabled: a.disabled,
                "data-scope": h::SCOPE,
                "data-part": "control",
                "data-state": a.data_state,
                onclick: move |_| send(state, input, on_checked_change, CheckboxEvent::Toggle(ChangeReason::Pointer)),
                onkeydown: move |e| {
                    if h::blocks_key(&e.key().to_string()) {
                        e.prevent_default();
                    }
                },
                span {
                    "data-scope": h::SCOPE,
                    "data-part": "indicator",
                    "data-state": a.data_state,
                    "aria-hidden": "true",
                    svg { view_box: "0 0 16 16",
                        path { "data-glyph": "check", d: "M3.5 8.5l3 3 6-7" }
                        path { "data-glyph": "dash", d: "M4 8h8" }
                    }
                }
            }
            span { "data-scope": h::SCOPE, "data-part": "label", {children} }
            if let Some(name) = hidden {
                input { r#type: "hidden", name, value }
            }
        }
    }
}
