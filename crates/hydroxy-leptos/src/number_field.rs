//! `<NumberField>`: a labelled number input with step buttons.

use hydroxy_headless::number_field::{
    self as h, NumberFieldCommand, NumberFieldEvent, NumberFieldInput, NumberFieldMachine,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

/// A number you can type or step. Typing commits on Enter or blur, clamped to
/// `min`..`max`; arrows, PageUp/PageDown and the buttons step at once.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn NumberField(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Controlled value. Pair with `on_value_change`. Must be `Some` at start to control.
    #[prop(optional, into)]
    value: MaybeProp<f64>,
    /// Initial value when uncontrolled. Starts empty without one.
    #[prop(optional, into)]
    default_value: Option<f64>,
    /// Lowest value.
    #[prop(optional, into)]
    min: Option<f64>,
    /// Highest value.
    #[prop(optional, into)]
    max: Option<f64>,
    /// Step for arrows and buttons. Default 1.
    #[prop(default = 1.0)]
    step: f64,
    /// Step for PageUp/PageDown. Default ten steps.
    #[prop(optional, into)]
    large_step: Option<f64>,
    /// Fixed decimal places, such as 2 for money.
    #[prop(optional, into)]
    fraction_digits: Option<u8>,
    /// Block changes and drop out of the tab order.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Focusable but not editable.
    #[prop(optional)]
    read_only: bool,
    /// Form field name. Submits the committed number.
    #[prop(optional, into)]
    name: Option<String>,
    /// Placeholder while empty.
    #[prop(optional, into)]
    placeholder: Option<String>,
    /// Text before the value inside the field, such as `£`.
    #[prop(optional, into)]
    prefix: Option<String>,
    /// Text after the value inside the field, such as `%`.
    #[prop(optional, into)]
    suffix: Option<String>,
    /// Called with the new value whenever it commits or a change is requested. `None` is empty.
    #[prop(optional, into)]
    on_value_change: Option<Callback<Option<f64>>>,
) -> impl IntoView {
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let input = Memo::new(move |_| NumberFieldInput {
        value: value.get(),
        default_value,
        min,
        max,
        step,
        large_step,
        fraction_digits,
        disabled: disabled.get().unwrap_or(false),
        read_only,
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
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| NumberFieldMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for NumberFieldCommand::ValueChange(v) in step.commands {
            if let Some(cb) = on_value_change {
                cb.run(v);
            }
        }
    };

    let control = Memo::new(move |_| input.with(|i| state.with(|s| h::control_attrs(s, i))));
    let can_down = move || input.with(|i| state.with(|s| h::can_step(s, i, false)));
    let can_up = move || input.with(|i| state.with(|s| h::can_step(s, i, true)));
    let is_disabled = move || disabled.get().unwrap_or(false);
    let affix = |text: Option<String>| text.map(|t| view! { <span data-scope=h::SCOPE data-part="affix">{t}</span> });

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-disabled=move || is_disabled().then_some("")
            data-readonly=read_only.then_some("")
        >
            <label data-scope=h::SCOPE data-part="label" for=ids.control.clone()>{label}</label>
            <div data-scope=h::SCOPE data-part="field">
                {affix(prefix)}
                <input
                    type="text"
                    inputmode="decimal"
                    autocomplete="off"
                    id=ids.control.clone()
                    data-scope=h::SCOPE
                    data-part="control"
                    role=move || control.get().role
                    aria-valuenow=move || control.get().aria_valuenow
                    aria-valuemin=move || control.get().aria_valuemin
                    aria-valuemax=move || control.get().aria_valuemax
                    placeholder=placeholder
                    disabled=is_disabled
                    readonly=read_only
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
                {affix(suffix)}
                <div data-scope=h::SCOPE data-part="steppers">
                    <button
                        type="button"
                        tabindex="-1"
                        data-scope=h::SCOPE
                        data-part="decrement"
                        aria-label="Decrease"
                        aria-controls=ids.control.clone()
                        disabled=move || !can_down()
                        on:click=move |_| send(NumberFieldEvent::Step { up: false, large: false })
                    >
                        <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
                            <path d="M3.5 8h9" />
                        </svg>
                    </button>
                    <button
                        type="button"
                        tabindex="-1"
                        data-scope=h::SCOPE
                        data-part="increment"
                        aria-label="Increase"
                        aria-controls=ids.control.clone()
                        disabled=move || !can_up()
                        on:click=move |_| send(NumberFieldEvent::Step { up: true, large: false })
                    >
                        <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
                            <path d="M3.5 8h9M8 3.5v9" />
                        </svg>
                    </button>
                </div>
            </div>
            {name.map(|name| view! { <input type="hidden" name=name disabled=is_disabled prop:value=move || state.with(h::form_value) /> })}
        </div>
    }
}
