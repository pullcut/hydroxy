//! `ToggleButton`: a button that stays pressed, such as Bold or Pin.
//! A native `<button aria-pressed>`; children are its content.

use dioxus::prelude::*;
use hydroxy_headless::toggle_button::{
    self as h, ChangeReason, ToggleButtonCommand, ToggleButtonEvent, ToggleButtonInput, ToggleButtonMachine,
};
use hydroxy_statechart::Machine;

/// A pressed / unpressed button. Uncontrolled by default; pass `pressed` to control it.
#[component]
pub fn ToggleButton(
    /// Controlled value. Pair with `on_pressed_change`.
    #[props(default)]
    pressed: ReadSignal<Option<bool>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_pressed: bool,
    /// Ignore presses.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Accessible name, for icon-only buttons.
    #[props(default)]
    label: Option<String>,
    /// Called with the new value whenever it changes or is requested.
    on_pressed_change: Option<EventHandler<bool>>,
    /// The button's content.
    children: Element,
) -> Element {
    let input = use_memo(move || ToggleButtonInput {
        checked: pressed(),
        default_checked: default_pressed,
        disabled: disabled(),
    });
    let mut state = use_signal(|| ToggleButtonMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = ToggleButtonMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let mut send = move |event: ToggleButtonEvent| {
        let step = ToggleButtonMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            let ToggleButtonCommand::CheckedChange { checked, .. } = command;
            if let Some(handler) = on_pressed_change {
                handler.call(checked);
            }
        }
    };

    let a = h::root_attrs(&state.read(), &input.read());
    rsx! {
        button {
            r#type: "button",
            "aria-pressed": a.aria_pressed.to_string(),
            "aria-label": label,
            disabled: a.disabled,
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-state": a.data_state,
            // Dioxus click events carry no `detail`, so keyboard presses report Pointer too.
            onclick: move |_| send(ToggleButtonEvent::Toggle(ChangeReason::Pointer)),
            {children}
        }
    }
}
