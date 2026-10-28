//! `<Calendar>`: an inline month grid. Arrow keys move a day or a week, Page
//! Up/Down a month (with Shift, a year), Home/End to the week's ends.

use leptos::prelude::*;
use oxide_headless::calendar::{self as h, CalendarCommand, CalendarEvent, CalendarInput, CalendarMachine};
use oxide_statechart::Machine;

use crate::focus::focus_id;
use crate::icon::{Icon, IconName};

pub use oxide_headless::calendar::Date;

/// Today's date as a signal. Server and first client render agree on UTC; once
/// mounted in the browser it switches to the local date.
pub(crate) fn use_today(fixed: Option<Date>) -> Signal<Date> {
    let today = RwSignal::new(fixed.unwrap_or_else(|| Date::from_days(oxide_headless::due::today())));
    if fixed.is_none() {
        Effect::new(move |_| today.set(h::today()));
    }
    today.into()
}

/// A month grid for picking one date.
#[component]
pub fn Calendar(
    /// Controlled value. Pair with `on_change`.
    #[prop(optional, into)]
    value: MaybeProp<Date>,
    /// Initial value when uncontrolled.
    #[prop(optional)]
    default_value: Option<Date>,
    /// Earliest selectable date.
    #[prop(optional, into)]
    min: MaybeProp<Date>,
    /// Latest selectable date.
    #[prop(optional, into)]
    max: MaybeProp<Date>,
    /// Days that can be focused but not picked.
    #[prop(optional, into)]
    unavailable: MaybeProp<Vec<Date>>,
    /// First day of the week, 0 = Sunday … 6 = Saturday. Default Monday.
    #[prop(default = 1)]
    week_start: u8,
    /// Today's date. Default: the browser's local date.
    #[prop(optional)]
    today: Option<Date>,
    /// Block every interaction.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Accessible name for the whole calendar. Default "Calendar".
    #[prop(optional, into)]
    label: Option<String>,
    /// Called with the picked date.
    #[prop(optional, into)]
    on_change: Option<Callback<Date>>,
) -> impl IntoView {
    let today = use_today(today);
    let input = Memo::new(move |_| CalendarInput {
        value: value.get(),
        default_value,
        today: today.get(),
        week_start,
        min: min.get(),
        max: max.get(),
        unavailable: unavailable.get().unwrap_or_default(),
        disabled: disabled.get().unwrap_or(false),
    });
    calendar_view(input, false, label.unwrap_or_else(|| "Calendar".into()), on_change)
}

/// The calendar body. `controlled` forces controlled mode even when the value
/// starts empty, for owners (the date picker) that always hold the value.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub(crate) fn calendar_view(
    input: Memo<CalendarInput>,
    controlled: bool,
    label: String,
    on_change: Option<Callback<Date>>,
) -> impl IntoView {
    let mut initial = input.with_untracked(CalendarMachine::initial);
    initial.controlled |= controlled;
    let state = RwSignal::new(initial);
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| CalendarMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });

    let root = crate::id::next_id(h::SCOPE);
    let id = StoredValue::new(root.clone());
    let send = move |event: CalendarEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| CalendarMachine::transition(s, event, i)))
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
                CalendarCommand::ValueChange(date) => {
                    if let Some(cb) = on_change {
                        cb.run(date);
                    }
                }
                // Wait a frame so a new month's cells have their ids.
                CalendarCommand::FocusDay(date) => {
                    let target = h::day_id(&id.get_value(), date);
                    request_animation_frame(move || focus_id(&target));
                }
            }
        }
    };

    let days = Memo::new(move |_| input.with(|i| state.with(|s| h::days(&id.get_value(), i, s))));
    let can = move |months: i32| input.with(|i| state.with(|s| h::can_shift(i, s, months)));
    let label_id = h::label_id(&root);
    let week_start = input.with_untracked(|i| i.week_start);

    let cell = move |index: usize| {
        let day = move || days.with(|d| d[index].clone());
        view! {
            <td
                role="gridcell"
                data-scope=h::SCOPE
                data-part="cell"
                aria-selected=move || day().selected.then_some("true")
            >
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="day"
                    id=move || day().id
                    tabindex=move || day().tabindex
                    aria-label=move || day().label
                    aria-current=move || day().today.then_some("date")
                    aria-disabled=move || day().unavailable.then_some("true")
                    aria-hidden=move || day().outside.then_some("true")
                    disabled=move || day().disabled
                    data-selected=move || day().selected.then_some("")
                    data-today=move || day().today.then_some("")
                    data-unavailable=move || day().unavailable.then_some("")
                    data-outside=move || day().outside.then_some("")
                    on:click=move |_| send(CalendarEvent::Select(day().date))
                    on:focus=move |_| send(CalendarEvent::Focus(day().date))
                    on:keydown=move |ev| {
                        if let Some(event) = h::day_keydown(&ev.key(), ev.shift_key()) {
                            ev.prevent_default();
                            send(event);
                        }
                    }
                >
                    {move || day().date.day()}
                </button>
            </td>
        }
    };

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            id=root
            role="group"
            aria-label=label
            data-disabled=move || input.with(|i| i.disabled).then_some("")
        >
            <div data-scope=h::SCOPE data-part="header">
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="previous"
                    aria-label="Previous month"
                    disabled=move || !can(-1)
                    on:click=move |_| send(CalendarEvent::ShiftMonth(-1))
                >
                    <Icon name=IconName::ChevronRight />
                </button>
                <div data-scope=h::SCOPE data-part="label" id=label_id.clone() aria-live="polite">
                    {move || state.with(|s| h::month_label(s.focused))}
                </div>
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="next"
                    aria-label="Next month"
                    disabled=move || !can(1)
                    on:click=move |_| send(CalendarEvent::ShiftMonth(1))
                >
                    <Icon name=IconName::ChevronRight />
                </button>
            </div>
            <table role="grid" data-scope=h::SCOPE data-part="grid" aria-labelledby=label_id>
                <thead>
                    <tr>
                        {h::weekdays(week_start)
                            .into_iter()
                            .map(|(short, long)| {
                                view! {
                                    <th scope="col" abbr=long data-scope=h::SCOPE data-part="weekday">
                                        {short}
                                    </th>
                                }
                            })
                            .collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {(0..6)
                        .map(|week| {
                            view! {
                                <tr data-scope=h::SCOPE data-part="week">
                                    {(0..7).map(|d| cell(week * 7 + d)).collect_view()}
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}
