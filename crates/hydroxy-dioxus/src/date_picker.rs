//! `DatePicker`: a text field that takes a typed `YYYY-MM-DD` date, and a
//! button that opens a [`Calendar`](crate::calendar::Calendar) in a native
//! `popover="auto"` anchored under the field.

use dioxus::prelude::*;
use hydroxy_headless::calendar::{self as cal, CalendarInput};
use hydroxy_headless::date_picker::{
    self as h, DatePickerCommand, DatePickerEvent, DatePickerInput, DatePickerMachine,
};
use hydroxy_statechart::Machine;

use crate::calendar::CalendarView;
use crate::focus::FocusMap;
use crate::icon::{Icon, IconName};
use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::calendar::Date;

/// `FocusMap` key for the trigger.
const TRIGGER_KEY: &str = "trigger";

/// A date field with a calendar popover.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn DatePicker(
    /// Visible label.
    label: String,
    /// Controlled value. Pair with `on_change`.
    #[props(default)]
    value: ReadSignal<Option<Date>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: Option<Date>,
    /// Earliest selectable date.
    #[props(default)]
    min: ReadSignal<Option<Date>>,
    /// Latest selectable date.
    #[props(default)]
    max: ReadSignal<Option<Date>>,
    /// Days that can't be picked.
    #[props(default)]
    unavailable: ReadSignal<Vec<Date>>,
    /// First day of the week, 0 = Sunday … 6 = Saturday. Default Monday.
    #[props(default = 1)]
    week_start: u8,
    /// Today's date. Default: the local date.
    #[props(default)]
    today: Option<Date>,
    /// Block editing and the calendar.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Form field name: submits the date as `YYYY-MM-DD` through a hidden input.
    #[props(default)]
    name: Option<String>,
    /// Called with the new date, or `None` when the field is cleared.
    on_change: Option<EventHandler<Option<Date>>>,
) -> Element {
    let today = use_hook(|| today.unwrap_or_else(cal::today));
    let rules = use_memo(move || CalendarInput {
        today,
        week_start,
        min: min(),
        max: max(),
        unavailable: unavailable(),
        disabled: disabled(),
        ..CalendarInput::default()
    });
    let input = use_memo(move || DatePickerInput { value: value(), default_value, rules: rules() });
    let mut state = use_signal(|| DatePickerMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = DatePickerMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });

    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let ids = h::ids(&root);
    let focus = use_hook(FocusMap::new);

    let send = move |event: DatePickerEvent| {
        let mut state = state;
        let step = DatePickerMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state
            && *state.peek() != next
        {
            state.set(next);
        }
        for command in step.commands {
            match command {
                DatePickerCommand::ValueChange(date) => {
                    if let Some(handler) = on_change {
                        handler.call(date);
                    }
                }
                DatePickerCommand::FocusTrigger => focus.focus(TRIGGER_KEY),
            }
        }
    };

    // The native popover follows the state; on open, focus the calendar's day.
    {
        let content = ids.content.clone();
        use_effect(move || {
            if state.read().open {
                show_popover(&content);
                let _ = document::eval(&format!(
                    "requestAnimationFrame(() => document.getElementById({content:?})?.querySelector({sel:?})?.focus());",
                    sel = h::FOCUS_ON_OPEN
                ));
            } else {
                hide_popover(&content);
            }
        });
    }

    // The calendar always shows the picker's value.
    let calendar_input = use_memo(move || CalendarInput { value: state.read().value, ..rules() });

    let s = state.read();
    let (open, text, invalid, data_state) = (s.open, s.text.clone(), s.invalid(), h::data_state(&s));
    let iso = s.value.map(|d| d.to_string()).unwrap_or_default();
    drop(s);
    let is_disabled = rules.read().disabled;
    let content_id = ids.content.clone();

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-disabled": is_disabled.then_some(""),
            "data-invalid": invalid.then_some(""),
            label { "data-scope": h::SCOPE, "data-part": "label", r#for: ids.input.clone(), "{label}" }
            div { "data-scope": h::SCOPE, "data-part": "field", style: h::field_style(&root),
                input {
                    r#type: "text",
                    "data-scope": h::SCOPE,
                    "data-part": "input",
                    id: ids.input,
                    inputmode: "numeric",
                    autocomplete: "off",
                    placeholder: h::PLACEHOLDER,
                    disabled: is_disabled,
                    "aria-invalid": invalid.then_some("true"),
                    value: text,
                    oninput: move |e| send(DatePickerEvent::Input(e.value())),
                    onblur: move |_| send(DatePickerEvent::Commit),
                    onkeydown: move |e| {
                        if let Some(event) = h::input_keydown(&e.key().to_string()) {
                            e.prevent_default();
                            send(event);
                        }
                    },
                }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "trigger",
                    id: ids.trigger,
                    "aria-label": "Choose {label.to_lowercase()}",
                    "aria-haspopup": "dialog",
                    "aria-expanded": open.to_string(),
                    "aria-controls": ids.content.clone(),
                    "popovertarget": ids.content.clone(),
                    "data-state": data_state,
                    disabled: is_disabled,
                    onmounted: move |e| focus.register(TRIGGER_KEY.to_string(), e.data()),
                    onclick: move |e| {
                        // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                        e.prevent_default();
                        send(DatePickerEvent::Toggle);
                    },
                    Icon { name: IconName::ChevronDown }
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name, value: iso }
            }
            div {
                id: ids.content,
                "popover": "auto",
                role: "dialog",
                "aria-label": label.clone(),
                style: h::content_style(&root),
                "data-overlay": "",
                "data-scope": h::SCOPE,
                "data-part": "content",
                "data-state": data_state,
                onkeydown: move |e| {
                    if let Some(event) = h::content_keydown(&e.key().to_string()) {
                        e.prevent_default();
                        send(event);
                    }
                },
                ontoggle: move |_| {
                    // The browser may have closed it (outside click): ask, then catch the machine up.
                    let js = format!("return document.getElementById({content_id:?})?.matches(':popover-open') ?? false");
                    spawn(async move {
                        if let Ok(false) = document::eval(&js).join::<bool>().await {
                            send(DatePickerEvent::Close { restore_focus: false });
                        }
                    });
                },
                // Mounted only while open, so each opening starts on the value (or today).
                if open {
                    CalendarView {
                        input: calendar_input,
                        controlled: true,
                        label: label.clone(),
                        on_change: move |date| send(DatePickerEvent::Select(date)),
                    }
                }
            }
        }
    }
}
