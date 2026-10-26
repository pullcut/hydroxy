//! `<DurationField>`: a length of time typed into hours, minutes and (optionally)
//! seconds segments. Digits fill a segment and move on when no further digit
//! could fit; Up/Down step and wrap, Left/Right move between segments.

use leptos::prelude::*;
use oxide_headless::duration_field::{
    self as h, Action, DurationFieldCommand, DurationFieldEvent, DurationFieldInput, DurationFieldMachine,
    DurationSegment,
};
use oxide_statechart::Machine;

use crate::focus::focus_id;

/// A labelled duration entered by typing. The value is whole seconds.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn DurationField(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Controlled value in seconds. Pair with `on_change`. Must be `Some` at start to control.
    #[prop(optional, into)]
    value: MaybeProp<u32>,
    /// Initial value in seconds when uncontrolled.
    #[prop(optional)]
    default_value: Option<u32>,
    /// Show a seconds segment. Default `true`.
    #[prop(default = true)]
    show_seconds: bool,
    /// Block editing.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Focusable but not editable.
    #[prop(optional)]
    read_only: bool,
    /// Form field name: submits the total seconds through a hidden input.
    #[prop(optional, into)]
    name: Option<String>,
    /// Called with the total seconds, or `None` once every segment is empty.
    #[prop(optional, into)]
    on_change: Option<Callback<Option<u32>>>,
) -> impl IntoView {
    let input = Memo::new(move |_| DurationFieldInput {
        value: value.get(),
        default_value,
        show_seconds,
        disabled: disabled.get().unwrap_or(false),
        read_only,
    });
    let state = RwSignal::new(input.with_untracked(DurationFieldMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| DurationFieldMachine::sync(s, i)));
        if state.with_untracked(|s| *s != synced) {
            state.set(synced);
        }
    });

    let root = crate::id::next_id(h::SCOPE);
    let id = StoredValue::new(root.clone());
    let send = move |event: DurationFieldEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| DurationFieldMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if state.with_untracked(|s| *s != step.state) {
            state.set(step.state);
        }
        for command in step.commands {
            match command {
                DurationFieldCommand::ValueChange(seconds) => {
                    if let Some(cb) = on_change {
                        cb.run(seconds);
                    }
                }
                DurationFieldCommand::FocusSegment(segment) => focus_id(&h::segment_id(&id.get_value(), segment)),
            }
        }
    };

    let is_disabled = move || disabled.get().unwrap_or(false);
    let segment = move |seg: DurationSegment| {
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
                disabled=is_disabled
                readonly=read_only
                prop:value=move || attrs.get().text
                on:focus=move |_| send(DurationFieldEvent::Focus)
                on:keydown=move |ev| {
                    let modified = ev.ctrl_key() || ev.meta_key() || ev.alt_key();
                    let action = input.with_untracked(|i| h::keydown(&ev.key(), modified, seg, i));
                    match action {
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
                        send(DurationFieldEvent::Digit { segment: seg, digit });
                    }
                    el.set_value(&state.with_untracked(|s| s.text(seg)));
                }
            />
        }
    };

    let first = h::segment_id(&root, DurationSegment::Hours);
    let label_id = h::label_id(&root);
    let separator = || view! { <span data-scope=h::SCOPE data-part="separator" aria-hidden="true">{h::SEPARATOR}</span> };

    view! {
        <div data-scope=h::SCOPE data-part="root" data-disabled=move || is_disabled().then_some("")>
            <label data-scope=h::SCOPE data-part="label" id=label_id.clone() for=first>{label}</label>
            <div data-scope=h::SCOPE data-part="field" role="group" aria-labelledby=label_id>
                {segment(DurationSegment::Hours)}
                {separator()}
                {segment(DurationSegment::Minutes)}
                {show_seconds.then(|| view! { {separator()} {segment(DurationSegment::Seconds)} })}
            </div>
            {name.map(|name| {
                view! { <input type="hidden" name=name prop:value=move || state.with(h::form_value) /> }
            })}
        </div>
    }
}
