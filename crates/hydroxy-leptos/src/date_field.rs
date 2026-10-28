//! `<DateField>`: a date typed into day, month and year segments. Digits fill a
//! segment and move on when no further digit could fit; Up/Down step and wrap,
//! Left/Right move between segments, Backspace empties one.

use leptos::prelude::*;
use hydroxy_headless::date_field::{
    self as h, Action, DateFieldCommand, DateFieldEvent, DateFieldInput, DateFieldMachine, DateSegment,
};
use hydroxy_statechart::Machine;

use crate::calendar::use_today;
use crate::focus::focus_id;

pub use hydroxy_headless::calendar::Date;
pub use hydroxy_headless::date_field::DateOrder;

/// A labelled date entered by typing.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn DateField(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Controlled value. Pair with `on_change`. Must be `Some` at start to control.
    #[prop(optional, into)]
    value: MaybeProp<Date>,
    /// Initial value when uncontrolled.
    #[prop(optional)]
    default_value: Option<Date>,
    /// Earliest valid date. Earlier ones mark the field invalid.
    #[prop(optional, into)]
    min: MaybeProp<Date>,
    /// Latest valid date.
    #[prop(optional, into)]
    max: MaybeProp<Date>,
    /// Today's date: where an empty segment starts when stepped. Default: the browser's local date.
    #[prop(optional)]
    today: Option<Date>,
    /// Segment order. Default day, month, year.
    #[prop(optional)]
    order: DateOrder,
    /// Block editing.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Focusable but not editable.
    #[prop(optional)]
    read_only: bool,
    /// Form field name: submits `YYYY-MM-DD` through a hidden input.
    #[prop(optional, into)]
    name: Option<String>,
    /// Called with the date once all segments make an allowed one, and with `None` when that stops being true.
    #[prop(optional, into)]
    on_change: Option<Callback<Option<Date>>>,
) -> impl IntoView {
    let today = use_today(today);
    let input = Memo::new(move |_| DateFieldInput {
        value: value.get(),
        default_value,
        min: min.get(),
        max: max.get(),
        today: today.get(),
        order,
        disabled: disabled.get().unwrap_or(false),
        read_only,
    });
    let state = RwSignal::new(input.with_untracked(DateFieldMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| DateFieldMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });

    let root = crate::id::next_id(h::SCOPE);
    let id = StoredValue::new(root.clone());
    let send = move |event: DateFieldEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| DateFieldMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state
            && state.with_untracked(|s| *s != next)
        {
            state.set(next);
        }
        for command in step.commands {
            match command {
                DateFieldCommand::ValueChange(date) => {
                    if let Some(cb) = on_change {
                        cb.run(date);
                    }
                }
                DateFieldCommand::FocusSegment(segment) => focus_id(&h::segment_id(&id.get_value(), segment)),
            }
        }
    };

    let is_disabled = move || disabled.get().unwrap_or(false);
    let invalid = move || input.with(|i| state.with(|s| s.invalid(i)));
    let segment = move |seg: DateSegment| {
        let attrs = Memo::new(move |_| state.with(|s| h::segment_attrs(&id.get_value(), s, seg)));
        view! {
            <input
                type="text"
                inputmode="numeric"
                autocomplete="off"
                spellcheck="false"
                role="spinbutton"
                data-scope=h::SCOPE
                data-part="segment"
                id=move || attrs.get().id
                aria-label=move || attrs.get().label
                aria-valuenow=move || attrs.get().valuenow
                aria-valuemin=move || attrs.get().valuemin
                aria-valuemax=move || attrs.get().valuemax
                aria-valuetext=move || attrs.get().valuetext
                aria-invalid=move || invalid().then_some("true")
                style=format!("width: {}ch;", attrs.with_untracked(|a| a.width))
                disabled=is_disabled
                readonly=read_only
                prop:value=move || attrs.get().text
                on:focus=move |_| send(DateFieldEvent::Focus)
                on:keydown=move |ev| {
                    let modified = ev.ctrl_key() || ev.meta_key() || ev.alt_key();
                    match h::keydown(&ev.key(), modified, seg, order) {
                        Some(Action::Send(event)) => {
                            ev.prevent_default();
                            send(event);
                        }
                        Some(Action::Focus(next)) => {
                            ev.prevent_default();
                            focus_id(&h::segment_id(&id.get_value(), next));
                        }
                        Some(Action::Ignore) => ev.prevent_default(),
                        None => {}
                    }
                }
                on:input=move |ev| {
                    // Soft keyboards may report no key: take the typed digit, then put the text back.
                    let el = event_target::<web_sys::HtmlInputElement>(&ev);
                    if let Some(digit) = h::input_digit(&el.value()) {
                        send(DateFieldEvent::Digit { segment: seg, digit });
                    }
                    el.set_value(&state.with_untracked(|s| s.text(seg)));
                }
            />
        }
    };

    let segments = order.segments();
    let first = h::segment_id(&root, segments[0]);
    let label_id = h::label_id(&root);
    
    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-disabled=move || is_disabled().then_some("")
            data-invalid=move || invalid().then_some("")
        >
            <label data-scope=h::SCOPE data-part="label" id=label_id.clone() for=first>{label}</label>
            <div data-scope=h::SCOPE data-part="field" role="group" aria-labelledby=label_id>
                {segment(segments[0])}
                <span data-scope=h::SCOPE data-part="separator" aria-hidden="true">{h::SEPARATOR}</span>
                {segment(segments[1])}
                <span data-scope=h::SCOPE data-part="separator" aria-hidden="true">{h::SEPARATOR}</span>
                {segment(segments[2])}
            </div>
            {name.map(|name| {
                view! {
                    <input
                        type="hidden"
                        name=name
                        prop:value=move || input.with(|i| state.with(|s| h::form_value(s, i)))
                    />
                }
            })}
        </div>
    }
}
