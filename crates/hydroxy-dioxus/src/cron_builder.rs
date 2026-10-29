//! `CronBuilder`: build a schedule in plain language or type a cron expression. Both stay in
//! step, with the schedule read back in words and the next five runs listed in UTC.

use dioxus::prelude::*;
use hydroxy_headless::cron_builder::{
    self as h, CronBuilderCommand, CronBuilderEvent, CronBuilderInput, CronBuilderMachine, Frequency,
};
use hydroxy_statechart::Machine;

use crate::drive::drive;

/// A step picker: the allowed steps, the current one, and its label.
type Interval = (&'static [u32], u32, fn(u32) -> String);

/// A cron schedule editor.
#[component]
pub fn CronBuilder(
    /// Controlled expression. Pair with `on_change`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Initial expression when uncontrolled. Blank means every day at 09:00.
    #[props(default)]
    default_value: String,
    /// Called with each new valid expression.
    on_change: Option<EventHandler<String>>,
    /// Unix seconds to list runs after. Defaults to the browser's clock.
    #[props(default)]
    now: ReadSignal<Option<i64>>,
    /// Accessible name for the whole builder.
    #[props(default = "Schedule".to_string())]
    label: String,
    /// Block editing.
    #[props(default)]
    disabled: ReadSignal<bool>,
) -> Element {
    let input = use_memo(move || CronBuilderInput {
        value: value(),
        default_value: default_value.clone(),
        disabled: disabled(),
    });
    let mut state = use_signal(|| CronBuilderMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = CronBuilderMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let mut browser_now = use_signal(|| 0_i64);
    use_future(move || async move {
        if let Ok(ms) = document::eval("return Date.now();").join::<f64>().await {
            #[allow(clippy::cast_possible_truncation, reason = "milliseconds since 1970 fit in i64")]
            browser_now.set((ms / 1000.0) as i64);
        }
    });
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(h::SCOPE)));

    let send = move |event: CronBuilderEvent| {
        let commands = drive::<CronBuilderMachine>(state, input, event);
        for CronBuilderCommand::Change(expr) in commands {
            if let Some(cb) = on_change {
                cb.call(expr);
            }
        }
    };

    let s = state.read();
    let c = s.controls;
    let f = c.frequency;
    let off = input.read().disabled;
    let clock = now().unwrap_or_else(&*browser_now);
    let runs = if clock == 0 { Vec::new() } else { h::upcoming(&s.expr, clock, h::RUN_COUNT) };
    let error = s.error.clone();
    let described = h::expression_described_by(&ids, error.is_some());
    let interval: Option<Interval> = match f {
        Frequency::Minutes => Some((&h::MINUTE_STEPS, c.every_minutes, h::minutes_label)),
        Frequency::Hours => Some((&h::HOUR_STEPS, c.every_hours, h::hours_label)),
        _ => None,
    };

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            role: "group",
            "aria-label": "{label}",
            "data-frequency": f.as_str(),
            "data-disabled": off.then_some(""),
            div { "data-scope": h::SCOPE, "data-part": "controls",
                div { "data-scope": h::SCOPE, "data-part": "control",
                    label { "data-scope": h::SCOPE, "data-part": "control-label", r#for: ids.frequency.clone(), "Repeat" }
                    select {
                        id: ids.frequency.clone(),
                        "data-scope": h::SCOPE,
                        "data-part": "frequency",
                        disabled: off,
                        onchange: move |e| {
                            if let Ok(f) = e.value().parse::<Frequency>() {
                                send(CronBuilderEvent::SetFrequency(f));
                            }
                        },
                        for option in Frequency::ALL {
                            option { key: "{option.as_str()}", value: option.as_str(), selected: option == f, {option.label()} }
                        }
                    }
                }
                if let Some((steps, current, text)) = interval {
                    div { "data-scope": h::SCOPE, "data-part": "control",
                        label { "data-scope": h::SCOPE, "data-part": "control-label", r#for: ids.interval.clone(), "Interval" }
                        select {
                            id: ids.interval.clone(),
                            "data-scope": h::SCOPE,
                            "data-part": "interval",
                            disabled: off,
                            onchange: move |e| {
                                let n = e.value().parse().unwrap_or(1);
                                send(if f == Frequency::Minutes { CronBuilderEvent::SetEveryMinutes(n) } else { CronBuilderEvent::SetEveryHours(n) });
                            },
                            for &n in steps {
                                option { key: "{n}", value: "{n}", selected: n == current, {text(n)} }
                            }
                        }
                    }
                }
                if f == Frequency::Hours {
                    div { "data-scope": h::SCOPE, "data-part": "control",
                        label { "data-scope": h::SCOPE, "data-part": "control-label", r#for: ids.time.clone(), "Minutes past the hour" }
                        input {
                            id: ids.time.clone(),
                            r#type: "number",
                            min: "0",
                            max: "59",
                            "data-scope": h::SCOPE,
                            "data-part": "minute",
                            disabled: off,
                            value: "{c.minute}",
                            onchange: move |e| {
                                if let Ok(m) = e.value().parse() {
                                    send(CronBuilderEvent::SetMinute(m));
                                }
                            },
                        }
                    }
                }
                if matches!(f, Frequency::Daily | Frequency::Weekdays | Frequency::Weekly | Frequency::Monthly) {
                    div { "data-scope": h::SCOPE, "data-part": "control",
                        label { "data-scope": h::SCOPE, "data-part": "control-label", r#for: ids.time.clone(), "At (UTC)" }
                        input {
                            id: ids.time.clone(),
                            r#type: "time",
                            step: "60",
                            required: true,
                            "data-scope": h::SCOPE,
                            "data-part": "time",
                            disabled: off,
                            value: h::time_value(&c),
                            onchange: move |e| {
                                if let Some((hour, minute)) = h::parse_time(&e.value()) {
                                    send(CronBuilderEvent::SetTime { hour, minute });
                                }
                            },
                        }
                    }
                }
                if f == Frequency::Weekly {
                    div { "data-scope": h::SCOPE, "data-part": "control",
                        span { id: ids.weekdays_label.clone(), "data-scope": h::SCOPE, "data-part": "control-label", "On" }
                        div {
                            "data-scope": h::SCOPE,
                            "data-part": "weekdays",
                            role: "group",
                            "aria-labelledby": ids.weekdays_label.clone(),
                            for (d, short, full) in h::WEEKDAY_TOGGLES {
                                button {
                                    key: "{d}",
                                    r#type: "button",
                                    "data-scope": h::SCOPE,
                                    "data-part": "weekday",
                                    "aria-label": full,
                                    "aria-pressed": "{h::weekday_on(&c, d)}",
                                    disabled: off,
                                    onclick: move |_| send(CronBuilderEvent::ToggleWeekday(d)),
                                    "{short}"
                                }
                            }
                        }
                    }
                }
                if f == Frequency::Monthly {
                    div { "data-scope": h::SCOPE, "data-part": "control",
                        label { "data-scope": h::SCOPE, "data-part": "control-label", r#for: ids.month_day.clone(), "Day of the month" }
                        select {
                            id: ids.month_day.clone(),
                            "data-scope": h::SCOPE,
                            "data-part": "month-day",
                            disabled: off,
                            onchange: move |e| {
                                if let Ok(d) = e.value().parse() {
                                    send(CronBuilderEvent::SetMonthDay(d));
                                }
                            },
                            for d in 1..=31u32 {
                                option { key: "{d}", value: "{d}", selected: d == c.month_day, {h::month_day_label(d)} }
                            }
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "expression-field",
                label { "data-scope": h::SCOPE, "data-part": "control-label", r#for: ids.expression.clone(), "Cron expression" }
                input {
                    id: ids.expression.clone(),
                    r#type: "text",
                    "data-scope": h::SCOPE,
                    "data-part": "expression",
                    spellcheck: "false",
                    autocomplete: "off",
                    autocapitalize: "off",
                    disabled: off,
                    "aria-invalid": error.is_some().then_some("true"),
                    "aria-describedby": described,
                    value: s.expr.clone(),
                    oninput: move |e| send(CronBuilderEvent::SetExpr(e.value())),
                }
                p { id: ids.error.clone(), "data-scope": h::SCOPE, "data-part": "error", hidden: error.is_none(),
                    {error.clone().unwrap_or_default()}
                }
            }
            p { id: ids.description.clone(), "data-scope": h::SCOPE, "data-part": "description", "aria-live": "polite",
                {h::description(&s)}
            }
            p { id: ids.runs_label.clone(), "data-scope": h::SCOPE, "data-part": "runs-label", "Next runs (UTC)" }
            ol { "data-scope": h::SCOPE, "data-part": "runs", "aria-labelledby": ids.runs_label,
                if runs.is_empty() && clock != 0 {
                    li { "data-scope": h::SCOPE, "data-part": "run", "data-empty": "", "No upcoming runs" }
                }
                for t in runs {
                    li { key: "{t}", "data-scope": h::SCOPE, "data-part": "run",
                        time { datetime: h::iso_utc(t), {h::format_utc(t)} }
                    }
                }
            }
        }
    }
}
