//! `<AvailabilityPicker>`: a week grid of time slots painted by dragging or by keyboard.

use std::collections::BTreeSet;

use hydroxy_headless::availability_picker::{
    self as h, AvailabilityCommand, AvailabilityEvent, AvailabilityInput, AvailabilityPickerMachine, AvailabilityState,
};
use hydroxy_statechart::Machine;
use leptos::either::Either;
use leptos::prelude::*;

use crate::drive::drive;
use crate::focus::focus_id;
use crate::modifiers::key_mods;

pub use h::{Date, Slot};

/// A week of time slots. Press on a cell and drag to paint a block available;
/// start on an available cell to clear instead. Space toggles the focused cell,
/// and Shift with an arrow key paints as you move.
#[component]
pub fn AvailabilityPicker(
    /// Accessible name for the grid.
    #[prop(into)]
    label: String,
    /// The first day shown.
    start: Date,
    /// Number of days. Default 7.
    #[prop(default = 7)]
    days: u8,
    /// Hour of day the first row starts at. Default 9.
    #[prop(default = 9)]
    start_hour: u8,
    /// Hour of day the last row ends at. Default 17.
    #[prop(default = 17)]
    end_hour: u8,
    /// Length of one slot in minutes. Default 30.
    #[prop(default = 30)]
    slot_minutes: u16,
    /// The time zone the slots are in, shown above the grid, such as "Europe/London".
    #[prop(into)]
    timezone: String,
    /// Controlled slots. Pair with `on_value_change`.
    #[prop(optional, into)]
    value: MaybeProp<BTreeSet<Slot>>,
    /// Initial slots when uncontrolled.
    #[prop(optional)]
    default_value: BTreeSet<Slot>,
    /// Block edits.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Called with the full set of available slots after each change.
    #[prop(optional, into)]
    on_value_change: Option<Callback<BTreeSet<Slot>>>,
) -> impl IntoView {
    let root = StoredValue::new(crate::id::next_id(h::SCOPE));
    let input = Memo::new(move |_| AvailabilityInput {
        start,
        days,
        first_minute: u16::from(start_hour) * 60,
        last_minute: u16::from(end_hour) * 60,
        slot_minutes,
        value: value.get(),
        default_value: default_value.clone(),
        disabled: disabled.get(),
    });
    let state = RwSignal::new(input.with_untracked(AvailabilityPickerMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| AvailabilityPickerMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let shown = Memo::new(move |_| input.with(|i| state.with(|s| h::display(s, i))));

    let send = {
        move |event: AvailabilityEvent| {
            for command in drive::<AvailabilityPickerMachine, _>(state, input, event) {
                match command {
                    AvailabilityCommand::ValueChange(v) => {
                        if let Some(cb) = on_value_change {
                            cb.run(v);
                        }
                    }
                    AvailabilityCommand::FocusCell(c, r) => focus_id(&h::cell_id(&root.get_value(), c, r)),
                }
            }
        }
    };

    let (cols, rows) = input.with_untracked(|i| (i.cols(), i.rows()));
    let grid = input.with_untracked(Clone::clone);
    let heads = (0..cols)
        .filter_map(|c| grid.slot(c, 0))
        .map(|s| {
            view! { <div data-scope=h::SCOPE data-part="day" role="columnheader">{h::day_label(s.date)}</div> }
        })
        .collect_view();
    let body = (0..rows)
        .map(|r| {
            let first = grid.slot(0, r);
            let hour_start = first.is_some_and(|s| s.minute % 60 == 0);
            let time = first.map(|s| h::time_label(s.minute));
            let cells = (0..cols)
                .filter_map(|c| grid.slot(c, r).map(|slot| (c, slot)))
                .map(|(c, slot)| {
                    let id = h::cell_id(&root.get_value(), c, r);
                    let target = id.clone();
                    let available = move || shown.with(|v| v.contains(&slot));
                    view! {
                        <div
                            id=id
                            data-scope=h::SCOPE
                            data-part="cell"
                            role="gridcell"
                            tabindex=move || if state.with(|s: &AvailabilityState| s.cursor == (c, r)) { "0" } else { "-1" }
                            aria-selected=move || available().to_string()
                            aria-label=move || h::cell_label(slot, slot_minutes, available())
                            aria-disabled=move || disabled.get().then_some("true")
                            data-available=move || available().then_some("")
                            on:mousedown=move |ev| {
                                if ev.button() == 0 {
                                    ev.prevent_default();
                                    send(AvailabilityEvent::PointerDown(c, r));
                                    focus_id(&target);
                                }
                            }
                            on:mouseenter=move |_| send(AvailabilityEvent::PointerEnter(c, r))
                            on:focus=move |_| send(AvailabilityEvent::Focus(c, r))
                            on:keydown=move |ev| {
                                if let Some(event) = h::keydown(&ev.key(), key_mods(&ev)) {
                                    ev.prevent_default();
                                    send(event);
                                }
                            }
                            on:keyup=move |ev| {
                                if ev.key() == "Shift" {
                                    send(AvailabilityEvent::KeyEnd);
                                }
                            }
                        ></div>
                    }
                })
                .collect_view();
            view! {
                <div data-scope=h::SCOPE data-part="row" role="row">
                    <div data-scope=h::SCOPE data-part="time" role="rowheader" data-hour=hour_start.then_some("")>
                        {time.filter(|_| hour_start)}
                    </div>
                    {cells}
                </div>
            }
        })
        .collect_view();

    let summary = move || {
        let lines = input.with(|i| shown.with(|v| h::summary_lines(v, i.slot_minutes)));
        if lines.is_empty() {
            Either::Left(
                view! { <p data-scope=h::SCOPE data-part="summary-empty">"No availability yet. Drag across the grid to add some."</p> },
            )
        } else {
            Either::Right(view! {
                <ul data-scope=h::SCOPE data-part="summary">
                    {lines.into_iter().map(|l| view! { <li>{l}</li> }).collect_view()}
                </ul>
            })
        }
    };

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-disabled=move || disabled.get().then_some("")
            data-painting=move || state.with(|s| s.paint.is_some()).then_some("")
        >
            <p data-scope=h::SCOPE data-part="timezone">{format!("Times shown in {timezone}")}</p>
            <div
                data-scope=h::SCOPE
                data-part="grid"
                role="grid"
                aria-label=label
                aria-colcount=cols
                aria-rowcount=rows + 1
                aria-multiselectable="true"
                style=format!("--columns: {cols};")
                on:mouseup=move |_| send(AvailabilityEvent::PointerUp)
                on:mouseleave=move |_| send(AvailabilityEvent::Cancel)
            >
                <div data-scope=h::SCOPE data-part="row" role="row">
                    <div data-scope=h::SCOPE data-part="corner" role="columnheader"></div>
                    {heads}
                </div>
                {body}
            </div>
            <div data-scope=h::SCOPE data-part="live" aria-live="polite">{summary}</div>
        </div>
    }
}
