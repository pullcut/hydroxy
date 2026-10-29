//! `DateRangePicker`: a button showing the chosen span that opens two months
//! in a native `popover="auto"` anchored under it. Click a start day, then an
//! end day; the days between light up as the pointer or arrow keys move.

use dioxus::prelude::*;
use hydroxy_headless::calendar::CalendarInput;
use hydroxy_headless::date_range_picker::{
    self as h, DateRangePickerCommand, DateRangePickerEvent, DateRangePickerInput, DateRangePickerMachine,
};
use hydroxy_statechart::Machine;

use crate::date_picker::{PickerPopover, use_picker_popover};
use crate::drive::drive;
use crate::focus::{FocusMap, focus_soon};
use crate::icon::{Icon, IconName};
use crate::modifiers::mods;

pub use hydroxy_headless::calendar::Date;
pub use hydroxy_headless::date_range_picker::DateRange;

const TRIGGER_KEY: &str = "trigger";

/// A range of dates picked from a two-month calendar popover.
#[component]
pub fn DateRangePicker(
    /// Visible label.
    label: String,
    /// Controlled value. Pair with `on_change`. Must be `Some` at start to control.
    #[props(default)]
    value: ReadSignal<Option<DateRange>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: Option<DateRange>,
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
    /// Text on the button while nothing is chosen. Default "Select dates".
    #[props(default)]
    placeholder: Option<String>,
    /// Block opening the calendar.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Form field name: submits `name-start` and `name-end` as `YYYY-MM-DD` through hidden inputs.
    #[props(default)]
    name: Option<String>,
    /// Called with the completed range.
    on_change: Option<EventHandler<DateRange>>,
) -> Element {
    let today = use_hook(|| today.unwrap_or_else(hydroxy_headless::calendar::today));
    let input = use_memo(move || DateRangePickerInput {
        value: value(),
        default_value,
        rules: CalendarInput {
            today,
            week_start,
            min: min(),
            max: max(),
            unavailable: unavailable(),
            disabled: disabled(),
            ..CalendarInput::default()
        },
    });
    let mut state = use_signal(|| DateRangePickerMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = DateRangePickerMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });

    let root_id = use_signal(|| hydroxy_headless::next_id(h::SCOPE));
    let root = root_id();
    let ids = h::ids(&root);
    let focus = use_hook(FocusMap::new);

    let send = move |event: DateRangePickerEvent| {
        for command in drive::<DateRangePickerMachine>(state, input, event) {
            match command {
                DateRangePickerCommand::ValueChange(range) => {
                    if let Some(handler) = on_change {
                        handler.call(range);
                    }
                }
                // Focus once the new month's cells are rendered.
                DateRangePickerCommand::FocusDay(date) => {
                    focus_soon(&h::day_id(&root_id.peek(), date));
                }
                DateRangePickerCommand::FocusTrigger => focus.focus(TRIGGER_KEY),
            }
        }
    };

    // The native popover follows the state; on open, focus the focused day.
    use_picker_popover(ids.content.clone(), h::FOCUS_ON_OPEN, move || state.read().open);

    let s = state.read();
    let i = input.read();
    let (open, data_state, empty) = (s.open, h::data_state(&s), s.value.value.is_none());
    let text =
        h::range_text(s.value.value).unwrap_or_else(|| placeholder.clone().unwrap_or_else(|| h::PLACEHOLDER.into()));
    let months = if open { Some(h::months(&root, &i, &s)) } else { None };
    let (can_prev, can_next) = (h::can_shift(&i, &s, -1), h::can_shift(&i, &s, 1));
    let [start, end] = h::form_values(&s);
    let (is_disabled, week_start) = (i.rules.disabled, i.rules.week_start);
    drop((s, i));

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-disabled": is_disabled.then_some(""),
            label { "data-scope": h::SCOPE, "data-part": "label-text", r#for: ids.trigger.clone(), "{label}" }
            div { "data-scope": h::SCOPE, "data-part": "field", style: h::field_style(&root),
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "trigger",
                    id: ids.trigger.clone(),
                    "aria-haspopup": "dialog",
                    "aria-expanded": open.to_string(),
                    "aria-controls": ids.content.clone(),
                    "popovertarget": ids.content,
                    "data-state": data_state,
                    "data-empty": empty.then_some(""),
                    disabled: is_disabled,
                    onmounted: move |e| focus.register(TRIGGER_KEY.to_string(), e.data()),
                    onclick: move |e| {
                        // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                        e.prevent_default();
                        send(DateRangePickerEvent::Toggle);
                    },
                    span { "data-scope": h::SCOPE, "data-part": "value", "{text}" }
                    Icon { name: IconName::ChevronDown }
                }
            }
            if let Some(name) = name {
                input { r#type: "hidden", name: "{name}-start", value: start }
                input { r#type: "hidden", name: "{name}-end", value: end }
            }
            PickerPopover {
                scope: h::SCOPE,
                id: ids.content.clone(),
                label: label,
                style: h::content_style(&root),
                data_state,
                on_key: move |key: String| match h::content_keydown(&key) {
                    Some(event) => {
                        send(event);
                        true
                    }
                    None => false,
                },
                on_dismiss: move |()| send(DateRangePickerEvent::Close { restore_focus: false }),
                // Mounted only while open, so each opening starts on the value (or today).
                if let Some(months) = months {
                    div { "data-scope": h::SCOPE, "data-part": "months",
                        for (m, month) in months.into_iter().enumerate() {
                            div { key: "{m}", "data-scope": h::SCOPE, "data-part": "month",
                                div { "data-scope": h::SCOPE, "data-part": "header",
                                    if m == 0 {
                                        button {
                                            r#type: "button",
                                            "data-scope": h::SCOPE,
                                            "data-part": "previous",
                                            "aria-label": "Previous month",
                                            disabled: !can_prev,
                                            onclick: move |_| send(DateRangePickerEvent::ShiftMonth(-1)),
                                            Icon { name: IconName::ChevronRight }
                                        }
                                    } else {
                                        span { "data-part": "spacer" }
                                    }
                                    div {
                                        "data-scope": h::SCOPE,
                                        "data-part": "label",
                                        id: month.label_id.clone(),
                                        "aria-live": "polite",
                                        "{month.label}"
                                    }
                                    if m == 1 {
                                        button {
                                            r#type: "button",
                                            "data-scope": h::SCOPE,
                                            "data-part": "next",
                                            "aria-label": "Next month",
                                            disabled: !can_next,
                                            onclick: move |_| send(DateRangePickerEvent::ShiftMonth(1)),
                                            Icon { name: IconName::ChevronRight }
                                        }
                                    } else {
                                        span { "data-part": "spacer" }
                                    }
                                }
                                table {
                                    role: "grid",
                                    "data-scope": h::SCOPE,
                                    "data-part": "grid",
                                    "aria-labelledby": month.label_id,
                                    thead {
                                        tr {
                                            for (short, long) in hydroxy_headless::calendar::weekdays(week_start) {
                                                th { scope: "col", abbr: long, "data-scope": h::SCOPE, "data-part": "weekday", "{short}" }
                                            }
                                        }
                                    }
                                    tbody {
                                        for (w, week) in month.days.chunks(7).enumerate() {
                                            tr { key: "{w}", "data-scope": h::SCOPE, "data-part": "week",
                                                for (d, day) in week.iter().cloned().enumerate() {
                                                    td {
                                                        key: "{d}",
                                                        role: "gridcell",
                                                        "data-scope": h::SCOPE,
                                                        "data-part": "cell",
                                                        "aria-selected": day.selected.then_some("true"),
                                                        "data-range-start": day.start.then_some(""),
                                                        "data-range-end": day.end.then_some(""),
                                                        "data-in-range": day.between.then_some(""),
                                                        "data-preview": day.preview.then_some(""),
                                                        button {
                                                            r#type: "button",
                                                            "data-scope": h::SCOPE,
                                                            "data-part": "day",
                                                            id: day.id,
                                                            tabindex: day.tabindex,
                                                            "aria-label": day.label,
                                                            "aria-current": day.today.then_some("date"),
                                                            "aria-disabled": day.unavailable.then_some("true"),
                                                            "aria-hidden": day.outside.then_some("true"),
                                                            disabled: day.disabled,
                                                            "data-selected": day.selected.then_some(""),
                                                            "data-today": day.today.then_some(""),
                                                            "data-unavailable": day.unavailable.then_some(""),
                                                            "data-outside": day.outside.then_some(""),
                                                            onclick: move |_| send(DateRangePickerEvent::Select(day.date)),
                                                            onpointerenter: move |_| send(DateRangePickerEvent::Hover(day.date)),
                                                            onfocus: move |_| send(DateRangePickerEvent::Focus(day.date)),
                                                            onkeydown: move |e| {
                                                                let shift = mods(e.modifiers());
                                                                if let Some(event) = h::day_keydown(&e.key().to_string(), shift) {
                                                                    e.prevent_default();
                                                                    send(event);
                                                                }
                                                            },
                                                            if !day.outside {
                                                                "{day.date.day()}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
