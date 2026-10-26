//! `DateField`: a date typed into day, month and year segments. Digits fill a
//! segment and move on when no further digit could fit; Up/Down step and wrap,
//! Left/Right move between segments, Backspace empties one.
//!
//! Segments take their digits from key events, which every desktop keyboard
//! and most soft keyboards send.

use dioxus::prelude::*;
use oxide_headless::date_field::{
    self as h, Action, DateFieldCommand, DateFieldEvent, DateFieldInput, DateFieldMachine,
};
use oxide_statechart::Machine;

use crate::focus::FocusMap;

pub use oxide_headless::calendar::Date;
pub use oxide_headless::date_field::DateOrder;

/// A labelled date entered by typing.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn DateField(
    /// Visible label.
    label: String,
    /// Controlled value. Pair with `on_change`. Must be `Some` at start to control.
    #[props(default)]
    value: ReadSignal<Option<Date>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: Option<Date>,
    /// Earliest valid date. Earlier ones mark the field invalid.
    #[props(default)]
    min: ReadSignal<Option<Date>>,
    /// Latest valid date.
    #[props(default)]
    max: ReadSignal<Option<Date>>,
    /// Today's date: where an empty segment starts when stepped. Default: the local date.
    #[props(default)]
    today: Option<Date>,
    /// Segment order. Default day, month, year.
    #[props(default)]
    order: DateOrder,
    /// Block editing.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Focusable but not editable.
    #[props(default)]
    read_only: bool,
    /// Form field name: submits `YYYY-MM-DD` through a hidden input.
    #[props(default)]
    name: Option<String>,
    /// Called with the date once all segments make an allowed one, and with `None` when that stops being true.
    on_change: Option<EventHandler<Option<Date>>>,
) -> Element {
    let today = use_hook(|| today.unwrap_or_else(oxide_headless::calendar::today));
    let input = use_memo(move || DateFieldInput {
        value: value(),
        default_value,
        min: min(),
        max: max(),
        today,
        order,
        disabled: disabled(),
        read_only,
    });
    let mut state = use_signal(|| DateFieldMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = DateFieldMachine::sync(&state.peek(), &input.read());
        if *state.peek() != synced {
            state.set(synced);
        }
    });

    let root = use_hook(|| oxide_headless::next_id(h::SCOPE));
    let focus = use_hook(FocusMap::new);

    // Segments are found by their id without the root, so the handlers stay `Copy`.
    let key = |segment| h::segment_id("", segment);
    let send = move |event: DateFieldEvent| {
        let mut state = state;
        let step = DateFieldMachine::transition(&state.peek(), event, &input.peek());
        if *state.peek() != step.state {
            state.set(step.state);
        }
        for command in step.commands {
            match command {
                DateFieldCommand::ValueChange(date) => {
                    if let Some(handler) = on_change {
                        handler.call(date);
                    }
                }
                DateFieldCommand::FocusSegment(segment) => focus.focus(&key(segment)),
            }
        }
    };

    let s = state.read();
    let attrs: Vec<_> = order.segments().into_iter().map(|seg| (seg, h::segment_attrs(&root, &s, seg))).collect();
    let (invalid, form_value) = (s.invalid(&input.read()), h::form_value(&s, &input.read()));
    drop(s);
    let first_id = attrs[0].1.id.clone();
    let is_disabled = disabled();
    let label_id = h::label_id(&root);

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-disabled": is_disabled.then_some(""),
            "data-invalid": invalid.then_some(""),
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
                        "aria-invalid": invalid.then_some("true"),
                        style: "width: {a.width}ch;",
                        disabled: is_disabled,
                        readonly: read_only,
                        value: a.text,
                        onmounted: move |e| focus.register(key(seg), e.data()),
                        onfocus: move |_| send(DateFieldEvent::Focus),
                        onkeydown: move |e| {
                            let m = e.modifiers();
                            let modified = m.ctrl() || m.meta() || m.alt();
                            match h::keydown(&e.key().to_string(), modified, seg, order) {
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
