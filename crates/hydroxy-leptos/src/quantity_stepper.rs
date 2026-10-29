//! `<QuantityStepper>`: a compact − / value / + control for counts such as
//! items in a basket. It is the number field's machine in a tighter layout.

use crate::drive::drive;
use hydroxy_headless::Direction;
use hydroxy_headless::quantity_stepper::{
    self as h, NumberFieldCommand, NumberFieldEvent, NumberFieldInput, NumberFieldMachine,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

/// A whole-number stepper. Type a number (commits on Enter or blur) or use the
/// buttons and arrow keys; the value stays within `min`..`max`.
#[component]
pub fn QuantityStepper(
    /// Accessible name, such as "Quantity of Oat milk". Not shown.
    #[prop(into)]
    label: String,
    /// Controlled value. Pair with `on_change`. Must be `Some` at start to control.
    #[prop(optional, into)]
    value: MaybeProp<f64>,
    /// Initial value when uncontrolled. Default 1.
    #[prop(default = 1.0)]
    default_value: f64,
    /// Lowest value. Default 0.
    #[prop(default = 0.0)]
    min: f64,
    /// Highest value.
    #[prop(optional, into)]
    max: Option<f64>,
    /// Amount per press. Default 1.
    #[prop(default = 1.0)]
    step: f64,
    /// Block changes.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Form field name. Submits the number.
    #[prop(optional, into)]
    name: Option<String>,
    /// Called with the new value whenever it commits or a change is requested.
    #[prop(optional, into)]
    on_change: Option<Callback<f64>>,
) -> impl IntoView {
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let input = Memo::new(move |_| NumberFieldInput {
        value: value.get(),
        default_value: Some(default_value),
        min: Some(min),
        max,
        step,
        disabled: disabled.get().unwrap_or(false),
        ..NumberFieldInput::default()
    });
    let state = RwSignal::new(input.with_untracked(NumberFieldMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| NumberFieldMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let send = move |event: NumberFieldEvent| {
        // Blur can arrive after unmount.
        for NumberFieldCommand::ValueChange(v) in drive::<NumberFieldMachine, _>(state, input, event) {
            if let (Some(cb), Some(v)) = (on_change, v) {
                cb.run(v);
            }
        }
    };

    let control = Memo::new(move |_| input.with(|i| state.with(|s| h::control_attrs(s, i))));
    let can_down = move || input.with(|i| state.with(|s| h::can_step(s, i, Direction::Prev)));
    let can_up = move || input.with(|i| state.with(|s| h::can_step(s, i, Direction::Next)));
    let is_disabled = move || disabled.get().unwrap_or(false);
    let down_label = format!("Decrease {}", label.to_lowercase());
    let up_label = format!("Increase {}", label.to_lowercase());
    let group_label = label.clone();

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            role="group"
            aria-label=group_label
            data-disabled=move || is_disabled().then_some("")
        >
            <button
                type="button"
                data-scope=h::SCOPE
                data-part="decrement"
                aria-label=down_label
                aria-controls=ids.control.clone()
                disabled=move || !can_down()
                on:click=move |_| send(NumberFieldEvent::Step { direction: Direction::Prev, large: false })
            >
                <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
                    <path d="M3.5 8h9" />
                </svg>
            </button>
            <input
                type="text"
                inputmode="numeric"
                autocomplete="off"
                id=ids.control.clone()
                data-scope=h::SCOPE
                data-part="control"
                role=move || control.with(|x| x.role)
                aria-label=label
                aria-valuenow=move || control.with(|x| x.aria_valuenow)
                aria-valuemin=move || control.with(|x| x.aria_valuemin)
                aria-valuemax=move || control.with(|x| x.aria_valuemax)
                disabled=is_disabled
                prop:value=move || state.with(|s| s.draft.clone())
                on:input=move |ev| send(NumberFieldEvent::SetText(event_target_value(&ev)))
                on:keydown=move |ev| {
                    if let Some(event) = input.try_with_untracked(|i| h::keydown(&ev.key(), i)).flatten() {
                        ev.prevent_default();
                        send(event);
                    }
                }
                on:blur=move |_| send(NumberFieldEvent::Commit)
            />
            <button
                type="button"
                data-scope=h::SCOPE
                data-part="increment"
                aria-label=up_label
                aria-controls=ids.control
                disabled=move || !can_up()
                on:click=move |_| send(NumberFieldEvent::Step { direction: Direction::Next, large: false })
            >
                <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
                    <path d="M3.5 8h9M8 3.5v9" />
                </svg>
            </button>
            {name.map(|name| view! { <input type="hidden" name=name disabled=is_disabled prop:value=move || state.with(h::form_value) /> })}
        </div>
    }
}
