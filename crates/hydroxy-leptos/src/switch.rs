//! `<Switch>`: an on/off control for settings that apply immediately.
//! Children become the label. Renders the connector's anatomy as one component.

use hydroxy_headless::switch::{self as h, SwitchCommand, SwitchEvent, SwitchInput, SwitchMachine};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::drive::drive;

/// On/off control. Uncontrolled by default; pass `checked` to control it.
#[component]
pub fn Switch(
    /// Controlled value. Pair with `on_checked_change`.
    #[prop(optional, into)]
    checked: MaybeProp<bool>,
    /// Initial value when uncontrolled.
    #[prop(optional)]
    default_checked: bool,
    /// Ignore presses and drop out of the tab order.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Form field name. When set, a checked switch submits `value`.
    #[prop(optional, into)]
    name: Option<String>,
    /// Value submitted with the form. Default `"on"`.
    #[prop(optional, into)]
    value: Option<String>,
    /// Called with the new value whenever it changes or is requested.
    #[prop(optional, into)]
    on_checked_change: Option<Callback<bool>>,
    /// The label.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let input = Memo::new(move |_| SwitchInput {
        checked: checked.get(),
        default_checked,
        disabled: disabled.get().unwrap_or(false),
    });
    let state = RwSignal::new(input.with_untracked(SwitchMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| SwitchMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let send = move |event: SwitchEvent| {
        // Events can still arrive after unmount.
        for command in drive::<SwitchMachine, _>(state, input, event) {
            let SwitchCommand::CheckedChange { checked, .. } = command;
            if let Some(cb) = on_checked_change {
                cb.run(checked);
            }
        }
    };

    let data_state = move || state.with(h::data_state);
    let control = Memo::new(move |_| input.with(|i| state.with(|s| h::control_attrs(s, i))));
    let value = value.unwrap_or_else(|| "on".to_string());
    let hidden = move || input.with(|i| state.with(|s| h::hidden_input(s, i, name.as_deref(), &value)));

    view! {
        <label
            data-scope=h::SCOPE
            data-part="root"
            data-state=data_state
            data-disabled=move || control.with(|x| x.disabled).then_some("")
        >
            <button
                type="button"
                role=move || control.with(|x| x.role)
                aria-checked=move || control.with(|x| x.aria_checked).to_string()
                disabled=move || control.with(|x| x.disabled)
                data-scope=h::SCOPE
                data-part="control"
                data-state=data_state
                on:click=move |ev| send(SwitchEvent::Toggle(h::click_reason(ev.detail())))
            >
                <span data-scope=h::SCOPE data-part="thumb" data-state=data_state aria-hidden="true"></span>
            </button>
            {children.map(|c| view! { <span data-scope=h::SCOPE data-part="label">{c()}</span> })}
            {move || hidden().map(|h| view! { <input type="hidden" name=h.name value=h.value /> })}
        </label>
    }
}
