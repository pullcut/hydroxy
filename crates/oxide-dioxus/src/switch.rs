//! `Switch`: an on/off control for settings that apply immediately.
//! Children become the label. Renders the connector's anatomy as one component.

use dioxus::prelude::*;
use oxide_headless::switch::{self as h, ChangeReason, SwitchCommand, SwitchEvent, SwitchInput, SwitchMachine};
use oxide_statechart::Machine;

/// On/off control. Uncontrolled by default; pass `checked` to control it.
#[component]
pub fn Switch(
    /// Controlled value. Pair with `on_checked_change`.
    #[props(default)]
    checked: ReadSignal<Option<bool>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_checked: bool,
    /// Ignore presses and drop out of the tab order.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Form field name. When set, a checked switch submits `value`.
    #[props(default)]
    name: Option<String>,
    /// Value submitted with the form.
    #[props(default = "on".to_string())]
    value: String,
    /// Called with the new value whenever it changes or is requested.
    on_checked_change: Option<EventHandler<bool>>,
    /// The label.
    children: Element,
) -> Element {
    let input = use_memo(move || SwitchInput { checked: checked(), default_checked, disabled: disabled() });
    let mut state = use_signal(|| SwitchMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = SwitchMachine::sync(&state.peek(), &input.read());
        state.set(synced);
    });

    let mut send = move |event: SwitchEvent| {
        let step = SwitchMachine::transition(&state.peek(), event, &input.peek());
        state.set(step.state);
        for command in step.commands {
            let SwitchCommand::CheckedChange { checked, .. } = command;
            if let Some(handler) = on_checked_change {
                handler.call(checked);
            }
        }
    };

    let s = state.read();
    let i = input.read();
    let data_state = h::data_state(&s);
    let a = h::control_attrs(&s, &i);
    let hidden = h::hidden_input(&s, &i, name.as_deref(), &value);

    rsx! {
        label {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-state": data_state,
            "data-disabled": a.disabled.then_some(""),
            button {
                r#type: "button",
                role: a.role,
                "aria-checked": a.aria_checked.to_string(),
                disabled: a.disabled,
                "data-scope": h::SCOPE,
                "data-part": "control",
                "data-state": data_state,
                // Dioxus click events carry no `detail`, so keyboard presses report Pointer too.
                onclick: move |_| send(SwitchEvent::Toggle(ChangeReason::Pointer)),
                span { "data-scope": h::SCOPE, "data-part": "thumb", "data-state": data_state, "aria-hidden": "true" }
            }
            span { "data-scope": h::SCOPE, "data-part": "label", {children} }
            if let Some(h) = hidden {
                input { r#type: "hidden", name: h.name, value: h.value }
            }
        }
    }
}
