//! `DateField`: a date typed into day, month and year segments. Digits fill a
//! segment and move on when no further digit could fit; Up/Down step and wrap,
//! Left/Right move between segments, Backspace empties one.
//!
//! Segments take their digits from key events, which every desktop keyboard
//! and most soft keyboards send.

use std::rc::Rc;

use dioxus::prelude::*;
use hydroxy_headless::date_field::{
    self as h, Action, DateFieldCommand, DateFieldEvent, DateFieldInput, DateFieldMachine,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::focus::FocusMap;
use crate::modifiers::mods;

pub use hydroxy_headless::calendar::Date;
pub use hydroxy_headless::date_field::DateOrder;

/// One segment, flattened so [`DateField`] and `DurationField` can render it the same way.
#[derive(Clone, PartialEq)]
pub(crate) struct SegmentView {
    pub id: String,
    pub label: &'static str,
    pub text: String,
    pub valuenow: Option<u32>,
    pub valuemin: u32,
    pub valuemax: u32,
    pub valuetext: String,
    /// Width in digits, when the field sizes its segments.
    pub width: Option<u8>,
}

/// The markup both segmented fields share: label, a group of spinbutton segments with separators,
/// and the optional hidden form input. The caller owns the machine and decides what each key does.
#[component]
pub(crate) fn SegmentField(
    scope: &'static str,
    separator: &'static str,
    label: String,
    label_id: String,
    segments: Vec<SegmentView>,
    invalid: bool,
    disabled: bool,
    read_only: bool,
    /// Called with the segment's index and element when it mounts.
    on_mounted: Callback<(usize, Rc<MountedData>)>,
    /// Called when any segment takes focus.
    on_focus: Callback<()>,
    /// Called with the segment's index and each key press on it.
    on_key: Callback<(usize, KeyboardEvent)>,
    name: Option<String>,
    form_value: String,
) -> Element {
    let first_id = segments.first().map(|a| a.id.clone());
    rsx! {
        div {
            "data-scope": scope,
            "data-part": "root",
            "data-disabled": disabled.then_some(""),
            "data-invalid": invalid.then_some(""),
            label { "data-scope": scope, "data-part": "label", id: label_id.clone(), r#for: first_id, "{label}" }
            div { "data-scope": scope, "data-part": "field", role: "group", "aria-labelledby": label_id,
                for (i, a) in segments.into_iter().enumerate() {
                    if i > 0 {
                        span { "data-scope": scope, "data-part": "separator", "aria-hidden": "true", "{separator}" }
                    }
                    input {
                        r#type: "text",
                        inputmode: "numeric",
                        autocomplete: "off",
                        spellcheck: "false",
                        role: "spinbutton",
                        "data-scope": scope,
                        "data-part": "segment",
                        id: a.id,
                        "aria-label": a.label,
                        "aria-valuenow": a.valuenow,
                        "aria-valuemin": a.valuemin,
                        "aria-valuemax": a.valuemax,
                        "aria-valuetext": a.valuetext,
                        "aria-invalid": invalid.then_some("true"),
                        style: a.width.map(|w| format!("width: {w}ch;")),
                        disabled,
                        readonly: read_only,
                        value: a.text,
                        onmounted: move |e| on_mounted.call((i, e.data())),
                        onfocus: move |_| on_focus.call(()),
                        onkeydown: move |e| on_key.call((i, e)),
                    }
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name, value: form_value }
            }
        }
    }
}

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
    let today = use_hook(|| today.unwrap_or_else(hydroxy_headless::calendar::today));
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
    let send = move |event: DateFieldEvent| {
        for command in drive::<DateFieldMachine>(state, input, event) {
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
    let segs = order.segments();
    let views: Vec<SegmentView> = segs
        .iter()
        .map(|seg| {
            let a = h::segment_attrs(&root, &s, *seg);
            SegmentView {
                id: a.id,
                label: a.label,
                text: a.text,
                valuenow: a.valuenow,
                valuemin: a.valuemin,
                valuemax: a.valuemax,
                valuetext: a.valuetext,
                width: Some(a.width),
            }
        })
        .collect();
    let (invalid, form_value) = (s.invalid(&input.read()), h::form_value(&s, &input.read()));
    drop(s);
    let (mounted_segs, key_segs) = (segs, segs);

    rsx! {
        SegmentField {
            scope: h::SCOPE,
            separator: h::SEPARATOR,
            label,
            label_id: h::label_id(&root),
            segments: views,
            invalid,
            disabled: disabled(),
            read_only,
            on_mounted: move |(i, el): (usize, Rc<MountedData>)| focus.register(key(mounted_segs[i]), el),
            on_focus: move |()| send(DateFieldEvent::Focus),
            on_key: move |(i, e): (usize, KeyboardEvent)| {
                let seg = key_segs[i];
                let m = e.modifiers();
                                match h::keydown(&e.key().to_string(), mods(m), seg, order) {
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
            name,
            form_value,
        }
    }
}
