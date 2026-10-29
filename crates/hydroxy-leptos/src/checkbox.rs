//! `<Checkbox>`: a `<label>` wrapping a `role="checkbox"` button and its label text.

use hydroxy_headless::checkbox::{self as h, CheckboxCommand, CheckboxEvent, CheckboxInput, CheckboxMachine};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::drive::drive;

pub use hydroxy_headless::checkbox::{ChangeReason, CheckedState};

/// Tri-state checkbox. Children are the label.
#[component]
pub fn Checkbox(
    /// Controlled state. Pair with `on_checked_change`.
    #[prop(optional, into)]
    checked: MaybeProp<CheckedState>,
    /// Initial state when uncontrolled.
    #[prop(optional)]
    default_checked: CheckedState,
    /// Ignore clicks and keys.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Form field name. When set, a checked box submits `value`.
    #[prop(optional, into)]
    name: Option<String>,
    /// Submitted value. Default `on`.
    #[prop(optional, into, default = "on".into())]
    value: String,
    /// Called with the proposed state when the user toggles.
    #[prop(optional, into)]
    on_checked_change: Option<Callback<bool>>,
    /// The label.
    children: Children,
) -> impl IntoView {
    let input = Memo::new(move |_| CheckboxInput {
        checked: checked.get(),
        default_checked,
        disabled: disabled.get().unwrap_or(false),
    });
    let state = RwSignal::new(input.with_untracked(CheckboxMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| CheckboxMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let send = move |event| {
        // Events can still arrive after unmount.
        for command in drive::<CheckboxMachine, _>(state, input, event) {
            let CheckboxCommand::CheckedChange { checked, .. } = command;
            if let Some(cb) = on_checked_change {
                cb.run(checked);
            }
        }
    };
    let attrs = Memo::new(move |_| input.with(|i| state.with(|s| h::control_attrs(s, i))));
    let state_attr = move || attrs.with(|x| x.data_state);
    let hidden = move || {
        let name = name.clone()?;
        state.with(h::submits).then(|| view! { <input type="hidden" name=name value=value.clone() /> })
    };

    view! {
        <label
            data-scope=h::SCOPE
            data-part="root"
            data-state=state_attr
            data-disabled=move || attrs.with(|x| x.disabled).then_some("")
        >
            <button
                type="button"
                role="checkbox"
                aria-checked=move || attrs.with(|x| x.aria_checked)
                disabled=move || attrs.with(|x| x.disabled)
                data-scope=h::SCOPE
                data-part="control"
                data-state=state_attr
                on:click=move |_| send(CheckboxEvent::Toggle(ChangeReason::Pointer))
                on:keydown=move |ev| {
                    if h::blocks_key(&ev.key()) {
                        ev.prevent_default();
                    }
                }
            >
                <span data-scope=h::SCOPE data-part="indicator" data-state=state_attr aria-hidden="true">
                    <svg viewBox="0 0 16 16">
                        <path data-glyph="check" d="M3.5 8.5l3 3 6-7" />
                        <path data-glyph="dash" d="M4 8h8" />
                    </svg>
                </span>
            </button>
            <span data-scope=h::SCOPE data-part="label">{children()}</span>
            {hidden}
        </label>
    }
}
