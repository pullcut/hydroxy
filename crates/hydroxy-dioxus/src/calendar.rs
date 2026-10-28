//! `Calendar`: an inline month grid. Arrow keys move a day or a week, Page
//! Up/Down a month (with Shift, a year), Home/End to the week's ends.

use dioxus::prelude::*;
use hydroxy_headless::calendar::{self as h, CalendarCommand, CalendarEvent, CalendarInput, CalendarMachine};
use hydroxy_statechart::Machine;

use crate::icon::{Icon, IconName};

pub use hydroxy_headless::calendar::Date;

/// A month grid for picking one date.
#[component]
pub fn Calendar(
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
    /// Days that can be focused but not picked.
    #[props(default)]
    unavailable: ReadSignal<Vec<Date>>,
    /// First day of the week, 0 = Sunday … 6 = Saturday. Default Monday.
    #[props(default = 1)]
    week_start: u8,
    /// Today's date. Default: the local date.
    #[props(default)]
    today: Option<Date>,
    /// Block every interaction.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Accessible name for the whole calendar. Default "Calendar".
    #[props(default)]
    label: Option<String>,
    /// Called with the picked date.
    on_change: Option<EventHandler<Date>>,
) -> Element {
    let today = use_hook(|| today.unwrap_or_else(h::today));
    let input = use_memo(move || CalendarInput {
        value: value(),
        default_value,
        today,
        week_start,
        min: min(),
        max: max(),
        unavailable: unavailable(),
        disabled: disabled(),
    });
    rsx! {
        CalendarView { input, controlled: false, label: label.unwrap_or_else(|| "Calendar".into()), on_change }
    }
}

/// The calendar body. `controlled` forces controlled mode even when the value
/// starts empty, for owners (the date picker) that always hold the value.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub(crate) fn CalendarView(
    input: Memo<CalendarInput>,
    controlled: bool,
    label: String,
    on_change: Option<EventHandler<Date>>,
) -> Element {
    let mut state = use_signal(|| {
        let mut initial = CalendarMachine::initial(&input.peek());
        initial.controlled |= controlled;
        initial
    });
    use_effect(move || {
        let synced = CalendarMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced
            && *state.peek() != synced
        {
            state.set(synced);
        }
    });

    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    // Focus moves once the new cells are rendered.
    let focus_request = use_signal(|| None::<Date>);
    {
        let root = root.clone();
        use_effect(move || {
            if let Some(date) = focus_request() {
                let id = h::day_id(&root, date);
                let _ = document::eval(&format!("requestAnimationFrame(() => document.getElementById({id:?})?.focus());"));
            }
        });
    }

    let send = move |event: CalendarEvent| {
        let (mut state, mut focus_request) = (state, focus_request);
        let step = CalendarMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state
            && *state.peek() != next
        {
            state.set(next);
        }
        for command in step.commands {
            match command {
                CalendarCommand::ValueChange(date) => {
                    if let Some(handler) = on_change {
                        handler.call(date);
                    }
                }
                CalendarCommand::FocusDay(date) => focus_request.set(Some(date)),
            }
        }
    };

    let (days, can_prev, can_next, month, week_start, disabled) = {
        let i = input.read();
        let s = state.read();
        (h::days(&root, &i, &s), h::can_shift(&i, &s, -1), h::can_shift(&i, &s, 1), h::month_label(s.focused), i.week_start, i.disabled)
    };
    let label_id = h::label_id(&root);

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            id: root.clone(),
            role: "group",
            "aria-label": label,
            "data-disabled": disabled.then_some(""),
            div { "data-scope": h::SCOPE, "data-part": "header",
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "previous",
                    "aria-label": "Previous month",
                    disabled: !can_prev,
                    onclick: move |_| send(CalendarEvent::ShiftMonth(-1)),
                    Icon { name: IconName::ChevronRight }
                }
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "label",
                    id: label_id.clone(),
                    "aria-live": "polite",
                    "{month}"
                }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "next",
                    "aria-label": "Next month",
                    disabled: !can_next,
                    onclick: move |_| send(CalendarEvent::ShiftMonth(1)),
                    Icon { name: IconName::ChevronRight }
                }
            }
            table { role: "grid", "data-scope": h::SCOPE, "data-part": "grid", "aria-labelledby": label_id,
                thead {
                    tr {
                        for (short, long) in h::weekdays(week_start) {
                            th { scope: "col", abbr: long, "data-scope": h::SCOPE, "data-part": "weekday", "{short}" }
                        }
                    }
                }
                tbody {
                    for (w, week) in days.chunks(7).enumerate() {
                        tr { key: "{w}", "data-scope": h::SCOPE, "data-part": "week",
                            for (i, day) in week.iter().cloned().enumerate() {
                                td {
                                    key: "{i}",
                                    role: "gridcell",
                                    "data-scope": h::SCOPE,
                                    "data-part": "cell",
                                    "aria-selected": day.selected.then_some("true"),
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
                                        onclick: move |_| send(CalendarEvent::Select(day.date)),
                                        onfocus: move |_| send(CalendarEvent::Focus(day.date)),
                                        onkeydown: move |e| {
                                            let shift = e.modifiers().shift();
                                            if let Some(event) = h::day_keydown(&e.key().to_string(), shift) {
                                                e.prevent_default();
                                                send(event);
                                            }
                                        },
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
