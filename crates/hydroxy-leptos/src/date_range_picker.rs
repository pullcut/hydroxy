//! `<DateRangePicker>`: a button showing the chosen span that opens two months
//! in a native `popover="auto"` anchored under it. Click a start day, then an
//! end day; the days between light up as the pointer or arrow keys move.

use leptos::either::Either;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use hydroxy_headless::calendar::CalendarInput;
use hydroxy_headless::date_range_picker::{
    self as h, DateRangePickerCommand, DateRangePickerEvent, DateRangePickerInput, DateRangePickerMachine,
};
use hydroxy_statechart::Machine;

use crate::calendar::use_today;
use crate::focus::focus_id;
use crate::icon::{Icon, IconName};
use crate::overlay::{hide_popover, show_popover};

pub use hydroxy_headless::calendar::Date;
pub use hydroxy_headless::date_range_picker::DateRange;

/// A range of dates picked from a two-month calendar popover.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn DateRangePicker(
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Controlled value. Pair with `on_change`. Must be `Some` at start to control.
    #[prop(optional, into)]
    value: MaybeProp<DateRange>,
    /// Initial value when uncontrolled.
    #[prop(optional)]
    default_value: Option<DateRange>,
    /// Earliest selectable date.
    #[prop(optional, into)]
    min: MaybeProp<Date>,
    /// Latest selectable date.
    #[prop(optional, into)]
    max: MaybeProp<Date>,
    /// Days that can't be picked.
    #[prop(optional, into)]
    unavailable: MaybeProp<Vec<Date>>,
    /// First day of the week, 0 = Sunday … 6 = Saturday. Default Monday.
    #[prop(default = 1)]
    week_start: u8,
    /// Today's date. Default: the browser's local date.
    #[prop(optional)]
    today: Option<Date>,
    /// Text on the button while nothing is chosen. Default "Select dates".
    #[prop(optional, into)]
    placeholder: Option<String>,
    /// Block opening the calendar.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Form field name: submits `name-start` and `name-end` as `YYYY-MM-DD` through hidden inputs.
    #[prop(optional, into)]
    name: Option<String>,
    /// Called with the completed range.
    #[prop(optional, into)]
    on_change: Option<Callback<DateRange>>,
) -> impl IntoView {
    let today = use_today(today);
    let input = Memo::new(move |_| DateRangePickerInput {
        value: value.get(),
        default_value,
        rules: CalendarInput {
            today: today.get(),
            week_start,
            min: min.get(),
            max: max.get(),
            unavailable: unavailable.get().unwrap_or_default(),
            disabled: disabled.get().unwrap_or(false),
            ..CalendarInput::default()
        },
    });
    let state = RwSignal::new(input.with_untracked(DateRangePickerMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| DateRangePickerMachine::sync(s, i)));
        if let Some(synced) = synced
            && state.with_untracked(|s| *s != synced)
        {
            state.set(synced);
        }
    });

    let root = crate::id::next_id(h::SCOPE);
    let ids = StoredValue::new(h::ids(&root));
    let root_id = StoredValue::new(root.clone());
    let send = move |event: DateRangePickerEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| DateRangePickerMachine::transition(s, event, i)))
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
                DateRangePickerCommand::ValueChange(range) => {
                    if let Some(cb) = on_change {
                        cb.run(range);
                    }
                }
                // Wait a frame so a new month's cells have their ids.
                DateRangePickerCommand::FocusDay(date) => {
                    let target = h::day_id(&root_id.get_value(), date);
                    request_animation_frame(move || focus_id(&target));
                }
                DateRangePickerCommand::FocusTrigger => focus_id(&ids.with_value(|i| i.trigger.clone())),
            }
        }
    };

    let open = Memo::new(move |_| state.with(|s| s.open));
    // The native popover follows the state; on open, focus the focused day.
    Effect::new(move |_| {
        let content = ids.with_value(|i| i.content.clone());
        if open.get() {
            show_popover(&content);
            request_animation_frame(move || {
                let target = document().get_element_by_id(&content).and_then(|c| c.query_selector(h::FOCUS_ON_OPEN).ok().flatten());
                if let Some(el) = target {
                    let _ = el.unchecked_into::<web_sys::HtmlElement>().focus();
                }
            });
        } else {
            hide_popover(&content);
        }
    });

    let months = Memo::new(move |_| input.with(|i| state.with(|s| h::months(&root_id.get_value(), i, s))));
    let can = move |direction: i32| input.with(|i| state.with(|s| h::can_shift(i, s, direction)));
    let is_disabled = move || input.with(|i| i.rules.disabled);
    let data_state = move || state.with(h::data_state);
    let ids_now = ids.get_value();
    let week_start = input.with_untracked(|i| i.rules.week_start);
    let placeholder = placeholder.unwrap_or_else(|| h::PLACEHOLDER.into());
    let content_label = label.clone();

    let cell = move |month: usize, index: usize| {
        let day = move || months.with(|m| m[month].days[index].clone());
        view! {
            <td
                role="gridcell"
                data-scope=h::SCOPE
                data-part="cell"
                aria-selected=move || day().selected.then_some("true")
                data-range-start=move || day().start.then_some("")
                data-range-end=move || day().end.then_some("")
                data-in-range=move || day().between.then_some("")
                data-preview=move || day().preview.then_some("")
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
                    on:click=move |_| send(DateRangePickerEvent::Select(day().date))
                    on:pointerenter=move |_| send(DateRangePickerEvent::Hover(day().date))
                    on:focus=move |_| send(DateRangePickerEvent::Focus(day().date))
                    on:keydown=move |ev| {
                        if let Some(event) = h::day_keydown(&ev.key(), ev.shift_key()) {
                            ev.prevent_default();
                            send(event);
                        }
                    }
                >
                    {move || (!day().outside).then(|| day().date.day())}
                </button>
            </td>
        }
    };

    let month_view = move |month: usize| {
        let (first, last) = (month == 0, month == 1);
        let shift = move |direction: i32| {
            let (part, text) = if direction < 0 { ("previous", "Previous month") } else { ("next", "Next month") };
            view! {
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part=part
                    aria-label=text
                    disabled=move || !can(direction)
                    on:click=move |_| send(DateRangePickerEvent::ShiftMonth(direction))
                >
                    <Icon name=IconName::ChevronRight />
                </button>
            }
        };
        view! {
            <div data-scope=h::SCOPE data-part="month">
                <div data-scope=h::SCOPE data-part="header">
                    {if first { Either::Left(shift(-1)) } else { Either::Right(view! { <span data-part="spacer"></span> }) }}
                    <div
                        data-scope=h::SCOPE
                        data-part="label"
                        id=move || months.with(|m| m[month].label_id.clone())
                        aria-live="polite"
                    >
                        {move || months.with(|m| m[month].label.clone())}
                    </div>
                    {if last { Either::Left(shift(1)) } else { Either::Right(view! { <span data-part="spacer"></span> }) }}
                </div>
                <table
                    role="grid"
                    data-scope=h::SCOPE
                    data-part="grid"
                    aria-labelledby=move || months.with(|m| m[month].label_id.clone())
                >
                    <thead>
                        <tr>
                            {hydroxy_headless::calendar::weekdays(week_start)
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
                                        {(0..7).map(|d| cell(month, week * 7 + d)).collect_view()}
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        }
    };

    view! {
        <div data-scope=h::SCOPE data-part="root" data-disabled=move || is_disabled().then_some("")>
            <label data-scope=h::SCOPE data-part="label-text" for=ids_now.trigger.clone()>
                {label}
            </label>
            <div data-scope=h::SCOPE data-part="field" style=h::field_style(&root)>
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="trigger"
                    id=ids_now.trigger
                    aria-haspopup="dialog"
                    aria-expanded=move || open.get().to_string()
                    aria-controls=ids_now.content.clone()
                    popovertarget=ids_now.content.clone()
                    data-state=data_state
                    data-empty=move || state.with(|s| s.value.is_none()).then_some("")
                    disabled=is_disabled
                    on:click=move |ev| {
                        // Keep `popovertarget` for its light-dismiss exemption, but let the machine toggle.
                        ev.prevent_default();
                        send(DateRangePickerEvent::Toggle);
                    }
                >
                    <span data-scope=h::SCOPE data-part="value">
                        {move || state.with(|s| h::range_text(s.value)).unwrap_or_else(|| placeholder.clone())}
                    </span>
                    <Icon name=IconName::ChevronDown />
                </button>
            </div>
            {name.map(|name| {
                let (start, end) = (format!("{name}-start"), format!("{name}-end"));
                view! {
                    <input type="hidden" name=start prop:value=move || state.with(|s| h::form_values(s)[0].clone()) />
                    <input type="hidden" name=end prop:value=move || state.with(|s| h::form_values(s)[1].clone()) />
                }
            })}
            <div
                id=ids_now.content
                popover="auto"
                role="dialog"
                aria-label=content_label
                style=h::content_style(&root)
                data-overlay=""
                data-scope=h::SCOPE
                data-part="content"
                data-state=data_state
                on:keydown=move |ev| {
                    if let Some(event) = h::content_keydown(&ev.key()) {
                        ev.prevent_default();
                        send(event);
                    }
                }
                on:toggle=move |ev| {
                    // The browser closed it (outside click): catch the machine up.
                    let showing = event_target::<web_sys::Element>(&ev).matches(":popover-open").unwrap_or(false);
                    if !showing {
                        send(DateRangePickerEvent::Close { restore_focus: false });
                    }
                }
            >
                // Mounted only while open, so each opening starts on the value (or today).
                <Show when=move || open.get()>
                    <div data-scope=h::SCOPE data-part="months">{month_view(0)} {month_view(1)}</div>
                </Show>
            </div>
        </div>
    }
}
