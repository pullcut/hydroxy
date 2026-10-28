//! `<CronBuilder>`: build a schedule in plain language or type a cron expression. Both stay in
//! step, with the schedule read back in words and the next five runs listed in UTC.

use leptos::prelude::*;
use hydroxy_headless::cron_builder::{
    self as h, CronBuilderCommand, CronBuilderEvent, CronBuilderInput, CronBuilderMachine, Frequency,
};
use hydroxy_statechart::Machine;

/// A cron schedule editor.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn CronBuilder(
    /// Controlled expression. Pair with `on_change`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial expression when uncontrolled. Blank means every day at 09:00.
    #[prop(optional, into)]
    default_value: String,
    /// Called with each new valid expression.
    #[prop(optional, into)]
    on_change: Option<Callback<String>>,
    /// Unix seconds to list runs after. Defaults to the browser's clock.
    #[prop(optional, into)]
    now: MaybeProp<i64>,
    /// Accessible name for the whole builder.
    #[prop(default = "Schedule".into(), into)]
    label: String,
    /// Block editing.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
) -> impl IntoView {
    let input = Memo::new(move |_| CronBuilderInput {
        value: value.get(),
        default_value: default_value.clone(),
        disabled: disabled.get().unwrap_or(false),
    });
    let state = RwSignal::new(input.with_untracked(CronBuilderMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| CronBuilderMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    // The clock is read after hydration, so the server and first client render agree.
    let clock = RwSignal::new(now.get_untracked().unwrap_or(0));
    Effect::new(move |_| {
        #[allow(clippy::cast_possible_truncation, reason = "milliseconds since 1970 fit in i64")]
        let t = now.get().unwrap_or_else(|| (web_sys::js_sys::Date::now() / 1000.0) as i64);
        clock.set(t);
    });

    let send = move |event: CronBuilderEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| CronBuilderMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for CronBuilderCommand::Change(expr) in step.commands {
            if let Some(cb) = on_change {
                cb.run(expr);
            }
        }
    };

    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let off = move || input.with(|i| i.disabled);
    let controls = move || state.with(|s| s.controls);
    // Only a preset change rebuilds the controls, so focus stays put while editing them.
    let freq = Memo::new(move |_| state.with(|s| s.controls.frequency));
    let freq = move || freq.get();
    let error = move || state.with(|s| s.error.clone());
    let runs = Memo::new(move |_| {
        let t = clock.get();
        if t == 0 { Vec::new() } else { state.with(|s| h::upcoming(&s.expr, t, h::RUN_COUNT)) }
    });

    let interval = {
        let id = ids.interval.clone();
        move || {
            let (steps, label): (&[u32], fn(u32) -> String) = match freq() {
                Frequency::Minutes => (&h::MINUTE_STEPS, h::minutes_label),
                Frequency::Hours => (&h::HOUR_STEPS, h::hours_label),
                _ => return None,
            };
            let minutes = freq() == Frequency::Minutes;
            let current = move || {
                let c = controls();
                if minutes { c.every_minutes } else { c.every_hours }
            };
            Some(view! {
                <div data-scope=h::SCOPE data-part="control">
                    <label data-scope=h::SCOPE data-part="control-label" for=id.clone()>"Interval"</label>
                    <select
                        id=id.clone()
                        data-scope=h::SCOPE
                        data-part="interval"
                        disabled=off
                        on:change=move |ev| {
                            let n = event_target_value(&ev).parse().unwrap_or(1);
                            send(if minutes { CronBuilderEvent::SetEveryMinutes(n) } else { CronBuilderEvent::SetEveryHours(n) });
                        }
                    >
                        {steps
                            .iter()
                            .map(|&n| view! { <option value=n.to_string() prop:selected=move || n == current()>{label(n)}</option> })
                            .collect_view()}
                    </select>
                </div>
            })
        }
    };

    let time = {
        let id = ids.time.clone();
        move || match freq() {
            Frequency::Minutes | Frequency::Custom => None,
            Frequency::Hours => Some(
                view! {
                    <div data-scope=h::SCOPE data-part="control">
                        <label data-scope=h::SCOPE data-part="control-label" for=id.clone()>"Minutes past the hour"</label>
                        <input
                            id=id.clone()
                            type="number"
                            min="0"
                            max="59"
                            data-scope=h::SCOPE
                            data-part="minute"
                            disabled=off
                            prop:value=move || controls().minute.to_string()
                            on:change=move |ev| {
                                if let Ok(m) = event_target_value(&ev).parse() {
                                    send(CronBuilderEvent::SetMinute(m));
                                }
                            }
                        />
                    </div>
                }
                .into_any(),
            ),
            _ => Some(
                view! {
                    <div data-scope=h::SCOPE data-part="control">
                        <label data-scope=h::SCOPE data-part="control-label" for=id.clone()>"At (UTC)"</label>
                        <input
                            id=id.clone()
                            type="time"
                            step="60"
                            required
                            data-scope=h::SCOPE
                            data-part="time"
                            disabled=off
                            prop:value=move || h::time_value(&controls())
                            on:change=move |ev| {
                                if let Some((hour, minute)) = h::parse_time(&event_target_value(&ev)) {
                                    send(CronBuilderEvent::SetTime { hour, minute });
                                }
                            }
                        />
                    </div>
                }
                .into_any(),
            ),
        }
    };

    let weekdays = {
        let label_id = ids.weekdays_label.clone();
        move || {
            (freq() == Frequency::Weekly).then(|| {
                view! {
                    <div data-scope=h::SCOPE data-part="control">
                        <span id=label_id.clone() data-scope=h::SCOPE data-part="control-label">"On"</span>
                        <div data-scope=h::SCOPE data-part="weekdays" role="group" aria-labelledby=label_id.clone()>
                            {h::WEEKDAY_TOGGLES
                                .iter()
                                .map(|&(d, short, full)| {
                                    let on = move || h::weekday_on(&controls(), d);
                                    view! {
                                        <button
                                            type="button"
                                            data-scope=h::SCOPE
                                            data-part="weekday"
                                            aria-label=full
                                            aria-pressed=move || on().to_string()
                                            disabled=off
                                            on:click=move |_| send(CronBuilderEvent::ToggleWeekday(d))
                                        >
                                            {short}
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </div>
                }
            })
        }
    };

    let month_day = {
        let id = ids.month_day.clone();
        move || {
            (freq() == Frequency::Monthly).then(|| {
                view! {
                    <div data-scope=h::SCOPE data-part="control">
                        <label data-scope=h::SCOPE data-part="control-label" for=id.clone()>"Day of the month"</label>
                        <select
                            id=id.clone()
                            data-scope=h::SCOPE
                            data-part="month-day"
                            disabled=off
                            on:change=move |ev| {
                                if let Ok(d) = event_target_value(&ev).parse() {
                                    send(CronBuilderEvent::SetMonthDay(d));
                                }
                            }
                        >
                            {(1..=31)
                                .map(|d| view! { <option value=d.to_string() prop:selected=move || d == controls().month_day>{h::month_day_label(d)}</option> })
                                .collect_view()}
                        </select>
                    </div>
                }
            })
        }
    };

    let expr_described = {
        let ids = ids.clone();
        move || h::expression_described_by(&ids, error().is_some())
    };

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            role="group"
            aria-label=label
            data-frequency=move || freq().as_str()
            data-disabled=move || off().then_some("")
        >
            <div data-scope=h::SCOPE data-part="controls">
                <div data-scope=h::SCOPE data-part="control">
                    <label data-scope=h::SCOPE data-part="control-label" for=ids.frequency.clone()>"Repeat"</label>
                    <select
                        id=ids.frequency.clone()
                        data-scope=h::SCOPE
                        data-part="frequency"
                        disabled=off
                        on:change=move |ev| {
                            if let Some(f) = Frequency::parse(&event_target_value(&ev)) {
                                send(CronBuilderEvent::SetFrequency(f));
                            }
                        }
                    >
                        {Frequency::ALL
                            .into_iter()
                            .map(|f| view! { <option value=f.as_str() prop:selected=move || freq() == f>{f.label()}</option> })
                            .collect_view()}
                    </select>
                </div>
                {interval}
                {time}
                {weekdays}
                {month_day}
            </div>
            <div data-scope=h::SCOPE data-part="expression-field">
                <label data-scope=h::SCOPE data-part="control-label" for=ids.expression.clone()>"Cron expression"</label>
                <input
                    id=ids.expression.clone()
                    type="text"
                    data-scope=h::SCOPE
                    data-part="expression"
                    spellcheck="false"
                    autocomplete="off"
                    autocapitalize="off"
                    disabled=off
                    aria-invalid=move || error().is_some().then_some("true")
                    aria-describedby=expr_described
                    prop:value=move || state.with(|s| s.expr.clone())
                    on:input=move |ev| send(CronBuilderEvent::SetExpr(event_target_value(&ev)))
                />
                <p id=ids.error.clone() data-scope=h::SCOPE data-part="error" hidden=move || error().is_none()>
                    {error}
                </p>
            </div>
            <p id=ids.description.clone() data-scope=h::SCOPE data-part="description" aria-live="polite">
                {move || state.with(h::description)}
            </p>
            <p id=ids.runs_label.clone() data-scope=h::SCOPE data-part="runs-label">"Next runs (UTC)"</p>
            <ol data-scope=h::SCOPE data-part="runs" aria-labelledby=ids.runs_label.clone()>
                {move || {
                    let list = runs.get();
                    if list.is_empty() && clock.get() != 0 {
                        view! { <li data-scope=h::SCOPE data-part="run" data-empty="">"No upcoming runs"</li> }.into_any()
                    } else {
                        list.into_iter()
                            .map(|t| {
                                view! {
                                    <li data-scope=h::SCOPE data-part="run">
                                        <time datetime=h::iso_utc(t)>{h::format_utc(t)}</time>
                                    </li>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </ol>
        </div>
    }
}
