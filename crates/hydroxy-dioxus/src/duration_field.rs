//! `DurationField`: a length of time typed into hours, minutes and (optionally)
//! seconds segments. Digits fill a segment and move on when no further digit
//! could fit; Up/Down step and wrap, Left/Right move between segments.
//!
//! Segments take their digits from key events, which every desktop keyboard
//! and most soft keyboards send.

use dioxus::prelude::*;
use hydroxy_headless::duration_field::{
    self as h, Action, DurationFieldCommand, DurationFieldEvent, DurationFieldInput, DurationFieldMachine,
};
use hydroxy_statechart::Machine;

use crate::focus::FocusMap;

/// A labelled duration entered by typing. The value is whole seconds.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn DurationField(
    /// Visible label.
    label: String,
    /// Controlled value in seconds. Pair with `on_change`. Must be `Some` at start to control.
    #[props(default)]
    value: ReadSignal<Option<u32>>,
    /// Initial value in seconds when uncontrolled.
    #[props(default)]
    default_value: Option<u32>,
    /// Show a seconds segment.
    #[props(default = true)]
    show_seconds: bool,
    /// Block editing.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Focusable but not editable.
    #[props(default)]
    read_only: bool,
    /// Form field name: submits the total seconds through a hidden input.
    #[props(default)]
    name: Option<String>,
    /// Called with the total seconds, or `None` once every segment is empty.
    on_change: Option<EventHandler<Option<u32>>>,
) -> Element {
    let input = use_memo(move || DurationFieldInput {
        value: value(),
        default_value,
        show_seconds,
        disabled: disabled(),
        read_only,
    });
    let mut state = use_signal(|| DurationFieldMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = DurationFieldMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });

    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let focus = use_hook(FocusMap::new);
    // Segments are found by their id without the root, so the handlers stay `Copy`.
    let key = |segment| h::segment_id("", segment);

    let send = move |event: DurationFieldEvent| {
        let mut state = state;
        let step = DurationFieldMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state
            && *state.peek() != next
        {
            state.set(next);
        }
        for command in step.commands {
            match command {
                DurationFieldCommand::ValueChange(seconds) => {
                    if let Some(handler) = on_change {
                        handler.call(seconds);
                    }
                }
                DurationFieldCommand::FocusSegment(segment) => focus.focus(&key(segment)),
            }
        }
    };

    let s = state.read();
    let attrs: Vec<_> = input.read().segments().iter().map(|seg| (*seg, h::segment_attrs(&root, &s, *seg))).collect();
    let form_value = h::form_value(&s);
    drop(s);
    let first_id = attrs[0].1.id.clone();
    let is_disabled = disabled();
    let label_id = h::label_id(&root);

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-disabled": is_disabled.then_some(""),
            label {
                "data-scope": h::SCOPE,
                "data-part": "label",
                id: label_id.clone(),
                r#for: first_id,
                "{label}"
            }
            div { "data-scope": h::SCOPE, "data-part": "field", role: "group", "aria-labelledby": label_id,
                for (i, (seg, a)) in attrs.into_iter().enumerate() {
                    if i > 0 {
                        span { "data-scope": h::SCOPE, "data-part": "separator", "aria-hidden": "true", {h::SEPARATOR} }
                    }
                    input {
                        r#type: "text",
                        inputmode: "numeric",
                        autocomplete: "off",
                        spellcheck: "false",
                        role: "spinbutton",
                        "data-scope": h::SCOPE,
                        "data-part": "segment",
                        id: a.id.clone(),
                        "aria-label": a.label,
                        "aria-valuenow": a.valuenow,
                        "aria-valuemin": a.valuemin,
                        "aria-valuemax": a.valuemax,
                        "aria-valuetext": a.valuetext,
                        disabled: is_disabled,
                        readonly: read_only,
                        value: a.text,
                        onmounted: move |e| focus.register(key(seg), e.data()),
                        onfocus: move |_| send(DurationFieldEvent::Focus),
                        onkeydown: move |e| {
                            let m = e.modifiers();
                            let modified = m.ctrl() || m.meta() || m.alt();
                            match h::keydown(&e.key().to_string(), modified, seg, &input.peek()) {
                                Some(Action::Send(event)) => {
                                    e.prevent_default();
                                    send(event);
                                }
                                Some(Action::Focus(next)) => {
                                    e.prevent_default();
                                    focus.focus(&key(next));
                                }
                                Some(Action::Ignore) => e.prevent_default(),
                                None => {}
                            }
                        },
                    }
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name, value: form_value }
            }
        }
    }
}
