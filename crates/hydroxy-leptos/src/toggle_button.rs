//! `<ToggleButton>`: a button that stays pressed, such as Bold or Pin.
//! A native `<button aria-pressed>`; children are its content.

use hydroxy_headless::switch::click_reason;
use hydroxy_headless::toggle_button::{
    self as h, ToggleButtonCommand, ToggleButtonEvent, ToggleButtonInput, ToggleButtonMachine,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

/// A pressed / unpressed button. Uncontrolled by default; pass `pressed` to control it.
#[component]
pub fn ToggleButton(
    /// Controlled value. Pair with `on_pressed_change`.
    #[prop(optional, into)]
    pressed: MaybeProp<bool>,
    /// Initial value when uncontrolled.
    #[prop(optional)]
    default_pressed: bool,
    /// Ignore presses.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Accessible name, for icon-only buttons.
    #[prop(optional, into)]
    label: Option<String>,
    /// Called with the new value whenever it changes or is requested.
    #[prop(optional, into)]
    on_pressed_change: Option<Callback<bool>>,
    /// The button's content.
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| ToggleButtonInput {
        checked: pressed.get(),
        default_checked: default_pressed,
        disabled: disabled.get().unwrap_or(false),
    });
    let state = RwSignal::new(input.with_untracked(ToggleButtonMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| ToggleButtonMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let send = move |event: ToggleButtonEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| ToggleButtonMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            let ToggleButtonCommand::CheckedChange { checked, .. } = command;
            if let Some(cb) = on_pressed_change {
                cb.run(checked);
            }
        }
    };

    let attrs = Memo::new(move |_| input.with(|i| state.with(|s| h::root_attrs(s, i))));

    view! {
        <button
            type="button"
            aria-pressed=move || attrs.get().aria_pressed.to_string()
            aria-label=label
            disabled=move || attrs.get().disabled
            data-scope=h::SCOPE
            data-part="root"
            data-state=move || attrs.get().data_state
            on:click=move |ev| send(ToggleButtonEvent::Toggle(click_reason(ev.detail())))
        >
            {children()}
        </button>
    }
}
