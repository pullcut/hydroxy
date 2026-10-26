//! `AvailabilityPicker`: a week grid of time slots painted by dragging or by keyboard.
//!
//! Painting uses mouse events: press on a cell, drag across others, release.
//! A tap toggles one cell; dragging with a finger is not supported.

use std::collections::BTreeSet;

use dioxus::prelude::*;
use oxide_headless::availability_picker::{
    self as h, AvailabilityCommand, AvailabilityEvent, AvailabilityInput, AvailabilityPickerMachine,
};
use oxide_statechart::Machine;

use crate::focus::FocusMap;

pub use h::{Date, Slot};

/// A week of time slots. Press on a cell and drag to paint a block available;
/// start on an available cell to clear instead. Space toggles the focused cell,
/// and Shift with an arrow key paints as you move.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn AvailabilityPicker(
    /// Accessible name for the grid.
    label: String,
    /// The first day shown.
    start: Date,
    /// Number of days. Default 7.
    #[props(default = 7)]
    days: u8,
    /// Hour of day the first row starts at. Default 9.
    #[props(default = 9)]
    start_hour: u8,
    /// Hour of day the last row ends at. Default 17.
    #[props(default = 17)]
    end_hour: u8,
    /// Length of one slot in minutes. Default 30.
    #[props(default = 30)]
    slot_minutes: u16,
    /// The time zone the slots are in, shown above the grid, such as "Europe/London".
    timezone: String,
    /// Controlled slots. Pair with `on_value_change`.
    #[props(default)]
    value: ReadSignal<Option<BTreeSet<Slot>>>,
    /// Initial slots when uncontrolled.
    #[props(default)]
    default_value: BTreeSet<Slot>,
    /// Block edits.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Called with the full set of available slots after each change.
    on_value_change: Option<EventHandler<BTreeSet<Slot>>>,
) -> Element {
    let root = use_hook(|| oxide_headless::next_id(h::SCOPE));
    let input = use_memo(move || AvailabilityInput {
        start,
        days,
        first_minute: u16::from(start_hour) * 60,
        last_minute: u16::from(end_hour) * 60,
        slot_minutes,
        value: value(),
        default_value: default_value.clone(),
        disabled: disabled(),
    });
    let mut state = use_signal(|| AvailabilityPickerMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = AvailabilityPickerMachine::sync(&state.peek(), &input.read());
        state.set(synced);
    });
    let focus = use_hook(FocusMap::new);

    let mut send = move |event: AvailabilityEvent| {
        let step = AvailabilityPickerMachine::transition(&state.peek(), event, &input.peek());
        state.set(step.state);
        for command in step.commands {
            match command {
                AvailabilityCommand::ValueChange(v) => {
                    if let Some(handler) = on_value_change {
                        handler.call(v);
                    }
                }
                AvailabilityCommand::FocusCell(c, r) => focus.focus(&format!("{c}-{r}")),
            }
        }
    };

    let grid = input.read().clone();
    let (cols, rows) = (grid.cols(), grid.rows());
    let shown = h::display(&state.read(), &grid);
    let cursor = state.read().cursor;
    let painting = state.read().paint.is_some();
    let is_disabled = disabled();
    let lines = h::summary_lines(&shown, slot_minutes);

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-disabled": is_disabled.then_some(""),
            "data-painting": painting.then_some(""),
            p { "data-scope": h::SCOPE, "data-part": "timezone", "Times shown in {timezone}" }
            div {
                "data-scope": h::SCOPE,
                "data-part": "grid",
                role: "grid",
                "aria-label": "{label}",
                "aria-colcount": "{cols}",
                "aria-rowcount": "{rows + 1}",
                "aria-multiselectable": "true",
                style: "--columns: {cols};",
                onmouseup: move |_| send(AvailabilityEvent::PointerUp),
                onmouseleave: move |_| send(AvailabilityEvent::Cancel),
                div { "data-scope": h::SCOPE, "data-part": "row", role: "row",
                    div { "data-scope": h::SCOPE, "data-part": "corner", role: "columnheader" }
                    for c in 0..cols {
                        if let Some(slot) = grid.slot(c, 0) {
                            div { key: "{c}", "data-scope": h::SCOPE, "data-part": "day", role: "columnheader", {h::day_label(slot.date)} }
                        }
                    }
                }
                for r in 0..rows {
                    div { key: "{r}", "data-scope": h::SCOPE, "data-part": "row", role: "row",
                        div {
                            "data-scope": h::SCOPE,
                            "data-part": "time",
                            role: "rowheader",
                            "data-hour": grid.slot(0, r).is_some_and(|s| s.minute % 60 == 0).then_some(""),
                            if let Some(s) = grid.slot(0, r).filter(|s| s.minute % 60 == 0) {
                                {h::time_label(s.minute)}
                            }
                        }
                        for c in 0..cols {
                            if let Some(slot) = grid.slot(c, r) {
                                {
                                    let available = shown.contains(&slot);
                                    rsx! {
                                        div {
                                            key: "{c}-{r}",
                                            id: h::cell_id(&root, c, r),
                                            "data-scope": h::SCOPE,
                                            "data-part": "cell",
                                            role: "gridcell",
                                            tabindex: if cursor == (c, r) { "0" } else { "-1" },
                                            "aria-selected": available.to_string(),
                                            "aria-label": h::cell_label(slot, slot_minutes, available),
                                            "aria-disabled": is_disabled.then_some("true"),
                                            "data-available": available.then_some(""),
                                            onmounted: move |e| focus.register(format!("{c}-{r}"), e.data()),
                                            onmousedown: move |e| {
                                                if e.trigger_button() == Some(dioxus::html::input_data::MouseButton::Primary) {
                                                    e.prevent_default();
                                                    send(AvailabilityEvent::PointerDown(c, r));
                                                    focus.focus(&format!("{c}-{r}"));
                                                }
                                            },
                                            onmouseenter: move |_| send(AvailabilityEvent::PointerEnter(c, r)),
                                            onfocus: move |_| send(AvailabilityEvent::Focus(c, r)),
                                            onkeydown: move |e| {
                                                let m = e.modifiers();
                                                let event = h::keydown(&e.key().to_string(), m.shift(), m.ctrl() || m.meta());
                                                if let Some(event) = event {
                                                    e.prevent_default();
                                                    send(event);
                                                }
                                            },
                                            onkeyup: move |e| {
                                                if e.key().to_string() == "Shift" {
                                                    send(AvailabilityEvent::KeyEnd);
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "live", "aria-live": "polite",
                if lines.is_empty() {
                    p { "data-scope": h::SCOPE, "data-part": "summary-empty", "No availability yet. Drag across the grid to add some." }
                } else {
                    ul { "data-scope": h::SCOPE, "data-part": "summary",
                        for (i , line) in lines.iter().enumerate() {
                            li { key: "{i}", "{line}" }
                        }
                    }
                }
            }
        }
    }
}
