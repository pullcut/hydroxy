//! `CronBuilder` keeps a five-field cron expression (`minute hour day-of-month month
//! day-of-week`) and the plain-language controls that build it in step. Editing the
//! controls rewrites the expression; typing an expression sets the controls when it fits one
//! of the presets, and "Custom" when it doesn't. [`Cron`] parses expressions (ranges, steps,
//! lists, month and weekday names, `@daily` style macros), finds upcoming runs in UTC and
//! describes them in words.

mod expr;

use hydroxy_statechart::{Machine, Transition};

pub use crate::date::{MONTH_NAMES, WEEKDAY_NAMES};
use expr::step_of;
pub use expr::{Cron, CronError, FieldError, format_utc, iso_utc, values};

/// The presets the controls offer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Frequency {
    /// `*/n * * * *`.
    Minutes,
    /// `m */n * * *`.
    Hours,
    /// `m h * * *`.
    #[default]
    Daily,
    /// `m h * * 1-5`.
    Weekdays,
    /// `m h * * <days>`.
    Weekly,
    /// `m h d * *`.
    Monthly,
    /// Anything else: edit the expression directly.
    Custom,
}

impl Frequency {
    /// Every preset, in menu order.
    pub const ALL: [Frequency; 7] = [
        Frequency::Minutes,
        Frequency::Hours,
        Frequency::Daily,
        Frequency::Weekdays,
        Frequency::Weekly,
        Frequency::Monthly,
        Frequency::Custom,
    ];

    /// Menu text.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Frequency::Minutes => "Every few minutes",
            Frequency::Hours => "Every few hours",
            Frequency::Daily => "Every day",
            Frequency::Weekdays => "Every weekday",
            Frequency::Weekly => "Every week",
            Frequency::Monthly => "Every month",
            Frequency::Custom => "Custom",
        }
    }

    /// The `<option value>` and `data-frequency`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Frequency::Minutes => "minutes",
            Frequency::Hours => "hours",
            Frequency::Daily => "daily",
            Frequency::Weekdays => "weekdays",
            Frequency::Weekly => "weekly",
            Frequency::Monthly => "monthly",
            Frequency::Custom => "custom",
        }
    }
}

/// The preset for an `as_str` value.
impl std::str::FromStr for Frequency {
    type Err = crate::UnknownName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|f| f.as_str() == s).ok_or(crate::UnknownName)
    }
}

/// Minute intervals offered (they divide an hour evenly).
pub const MINUTE_STEPS: [u32; 11] = [1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 30];
/// Hour intervals offered (they divide a day evenly).
pub const HOUR_STEPS: [u32; 7] = [1, 2, 3, 4, 6, 8, 12];

/// The plain-language controls. Fields a preset doesn't use keep their value, so switching
/// back and forth loses nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Controls {
    /// The chosen preset.
    pub frequency: Frequency,
    /// For [`Frequency::Minutes`]: one of [`MINUTE_STEPS`].
    pub every_minutes: u32,
    /// For [`Frequency::Hours`]: one of [`HOUR_STEPS`].
    pub every_hours: u32,
    /// Hour of day, 0–23.
    pub hour: u32,
    /// Minute, 0–59. Also the minute past the hour for [`Frequency::Hours`].
    pub minute: u32,
    /// For [`Frequency::Weekly`]: bit `n` set for weekday `n` (0 = Sunday).
    pub weekdays: u8,
    /// For [`Frequency::Monthly`]: day of the month, 1–31.
    pub month_day: u32,
}

impl Default for Controls {
    /// Every day at 09:00.
    fn default() -> Self {
        Self {
            frequency: Frequency::Daily,
            every_minutes: 15,
            every_hours: 2,
            hour: 9,
            minute: 0,
            weekdays: 1 << 1,
            month_day: 1,
        }
    }
}

fn weekday_list(mask: u8) -> String {
    let days = values(u64::from(mask), 0, 6);
    let mut parts = Vec::new();
    let mut i = 0;
    while i < days.len() {
        let mut j = i;
        while j + 1 < days.len() && days[j + 1] == days[j] + 1 {
            j += 1;
        }
        if j - i >= 2 {
            parts.push(format!("{}-{}", days[i], days[j]));
        } else {
            parts.extend(days[i..=j].iter().map(ToString::to_string));
        }
        i = j + 1;
    }
    parts.join(",")
}

impl Controls {
    /// The expression these controls build, or `None` for [`Frequency::Custom`].
    #[must_use]
    pub fn to_cron(&self) -> Option<String> {
        let (m, h) = (self.minute, self.hour);
        Some(match self.frequency {
            Frequency::Minutes if self.every_minutes <= 1 => "* * * * *".into(),
            Frequency::Minutes => format!("*/{} * * * *", self.every_minutes),
            Frequency::Hours if self.every_hours <= 1 => format!("{m} * * * *"),
            Frequency::Hours => format!("{m} */{} * * *", self.every_hours),
            Frequency::Daily => format!("{m} {h} * * *"),
            Frequency::Weekdays => format!("{m} {h} * * 1-5"),
            Frequency::Weekly => format!("{m} {h} * * {}", weekday_list(self.weekdays)),
            Frequency::Monthly => format!("{m} {h} {} * *", self.month_day),
            Frequency::Custom => return None,
        })
    }

    /// Controls matching `cron`, keeping `base`'s unused fields. [`Frequency::Custom`] when
    /// no preset fits.
    #[must_use]
    pub fn recognise(cron: &Cron, base: &Controls) -> Controls {
        let mut c = Controls { frequency: Frequency::Custom, ..*base };
        let mins = values(cron.minutes, 0, 59);
        let hrs = values(cron.hours, 0, 23);
        let month_days = values(cron.days, 1, 31);
        let week_days = values(cron.weekdays, 0, 6);
        let any_day = month_days.len() == 31 && week_days.len() == 7;
        if values(cron.months, 1, 12).len() != 12 {
            return c;
        }
        if any_day
            && hrs.len() == 24
            && let Some(n) = step_of(&mins, 0, 59).filter(|n| MINUTE_STEPS.contains(n))
        {
            c.frequency = Frequency::Minutes;
            c.every_minutes = n;
            return c;
        }
        let [m] = mins[..] else { return c };
        c.minute = m;
        if any_day && let Some(n) = step_of(&hrs, 0, 23).filter(|n| HOUR_STEPS.contains(n)) {
            c.frequency = Frequency::Hours;
            c.every_hours = n;
            return c;
        }
        let [h] = hrs[..] else { return c };
        c.hour = h;
        let every_date = month_days.len() == 31;
        let every_weekday = week_days.len() == 7;
        match (every_date, every_weekday, &month_days[..]) {
            (true, true, _) => c.frequency = Frequency::Daily,
            (true, false, _) if week_days == [1, 2, 3, 4, 5] => c.frequency = Frequency::Weekdays,
            (true, false, _) => {
                c.frequency = Frequency::Weekly;
                c.weekdays = week_days.iter().fold(0, |acc, d| acc | 1 << d);
            }
            (false, true, [d]) => {
                c.frequency = Frequency::Monthly;
                c.month_day = *d;
            }
            _ => {}
        }
        c
    }
}

/// Parse "09:30" from a time input into (hour, minute).
#[must_use]
pub fn parse_time(text: &str) -> Option<(u32, u32)> {
    let (h, rest) = text.split_once(':')?;
    let m = rest.get(..2)?;
    let (h, m) = (h.parse::<u32>().ok()?, m.parse::<u32>().ok()?);
    (h < 24 && m < 60).then_some((h, m))
}

/// Up to `count` runs of `expr` after `now` (Unix seconds), or nothing when it doesn't parse.
#[must_use]
pub fn upcoming(expr: &str, now: i64, count: usize) -> Vec<i64> {
    expr.parse::<Cron>().map(|c| c.upcoming(now, count)).unwrap_or_default()
}

/// Props for [`CronBuilderMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CronBuilderInput {
    /// Controlled expression. `Some` at start fixes controlled mode.
    pub value: Option<String>,
    /// Uncontrolled initial expression. Blank means every day at 09:00.
    pub default_value: String,
    /// Block editing.
    pub disabled: bool,
}

/// What [`CronBuilderMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CronBuilderState {
    /// The expression as shown in its text field.
    pub expr: String,
    /// The controls.
    pub controls: Controls,
    /// Why `expr` doesn't parse.
    pub error: Option<String>,
    /// Fixed at start: the owner holds the value.
    pub controlled: bool,
}

/// Inputs to [`CronBuilderMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CronBuilderEvent {
    /// The expression field was edited.
    SetExpr(String),
    /// A preset was picked.
    SetFrequency(Frequency),
    /// The minute interval was picked.
    SetEveryMinutes(u32),
    /// The hour interval was picked.
    SetEveryHours(u32),
    /// The time of day changed.
    SetTime {
        /// 0–23.
        hour: u32,
        /// 0–59.
        minute: u32,
    },
    /// The minute past the hour changed (for [`Frequency::Hours`]).
    SetMinute(u32),
    /// A weekday was toggled (0 = Sunday). The last selected day can't be turned off.
    ToggleWeekday(u8),
    /// The day of the month changed.
    SetMonthDay(u32),
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CronBuilderCommand {
    /// A new valid expression (or the owner should set it, when controlled).
    Change(String),
}

/// Expression and controls kept in step.
#[derive(Debug, Clone, Copy)]
pub struct CronBuilderMachine;

fn load(expr: &str, base: &Controls) -> (String, Controls, Option<String>) {
    let expr = if expr.trim().is_empty() { base.to_cron().unwrap_or_default() } else { expr.to_string() };
    match expr.parse::<Cron>() {
        Ok(cron) => {
            let controls = Controls::recognise(&cron, base);
            (expr, controls, None)
        }
        Err(e) => (expr, Controls { frequency: Frequency::Custom, ..*base }, Some(e.to_string())),
    }
}

impl Machine for CronBuilderMachine {
    type Input = CronBuilderInput;
    type State = CronBuilderState;
    type Event = CronBuilderEvent;
    type Command = CronBuilderCommand;

    fn initial(input: &CronBuilderInput) -> CronBuilderState {
        let source = input.value.as_deref().unwrap_or(&input.default_value);
        let (expr, controls, error) = load(source, &Controls::default());
        CronBuilderState { expr, controls, error, controlled: input.value.is_some() }
    }

    fn transition(
        state: &CronBuilderState,
        event: CronBuilderEvent,
        input: &CronBuilderInput,
    ) -> Transition<CronBuilderState, CronBuilderCommand> {
        use CronBuilderEvent as E;
        let mut next = state.clone();
        if input.disabled {
            return Transition::to(next);
        }
        let c = &mut next.controls;
        match event {
            E::SetExpr(text) => {
                let (_, controls, error) = load(&text, &state.controls);
                next.expr = text;
                next.controls = controls;
                next.error = error;
                let commands = if next.error.is_none() && next.expr.trim() != state.expr.trim() {
                    vec![CronBuilderCommand::Change(next.expr.trim().to_string())]
                } else {
                    Vec::new()
                };
                return Transition::with(next, commands);
            }
            E::SetFrequency(f) => c.frequency = f,
            E::SetEveryMinutes(n) => c.every_minutes = n.clamp(1, 30),
            E::SetEveryHours(n) => c.every_hours = n.clamp(1, 12),
            E::SetTime { hour, minute } => {
                c.hour = hour.min(23);
                c.minute = minute.min(59);
            }
            E::SetMinute(m) => c.minute = m.min(59),
            E::ToggleWeekday(d) if d < 7 => {
                let toggled = c.weekdays ^ (1 << d);
                if toggled != 0 {
                    c.weekdays = toggled;
                }
            }
            E::ToggleWeekday(_) => {}
            E::SetMonthDay(d) => c.month_day = d.clamp(1, 31),
        }
        let mut commands = Vec::new();
        if let Some(expr) = next.controls.to_cron() {
            if expr != state.expr.trim() || state.error.is_some() {
                commands.push(CronBuilderCommand::Change(expr.clone()));
            }
            next.expr = expr;
            next.error = None;
        }
        Transition::with(next, commands)
    }

    fn sync(state: &CronBuilderState, input: &CronBuilderInput) -> Option<CronBuilderState> {
        Some(match (&input.value, state.controlled) {
            (Some(v), true) if v.trim() != state.expr.trim() => {
                let (expr, controls, error) = load(v, &state.controls);
                CronBuilderState { expr, controls, error, controlled: true }
            }
            _ => state.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use CronBuilderEvent as E;
    use hydroxy_statechart::step as advance;

    fn cron(s: &str) -> Cron {
        s.parse().unwrap()
    }

    /// 2026-10-28 (a Wednesday) at 12:34:56 UTC.
    const NOW: i64 = 1_793_190_896;

    #[test]
    fn now_constant_is_what_it_says() {
        assert_eq!(format_utc(NOW), "Wed 28 Oct 2026, 12:34");
        assert_eq!(iso_utc(NOW), "2026-10-28T12:34Z");
    }

    #[test]
    fn parses_ranges_steps_lists_and_names() {
        let c = cron("0-10/5 9,17 * JAN-mar mon-FRI");
        assert_eq!(values(c.minutes, 0, 59), vec![0, 5, 10]);
        assert_eq!(values(c.hours, 0, 23), vec![9, 17]);
        assert_eq!(values(c.months, 1, 12), vec![1, 2, 3]);
        assert_eq!(values(c.weekdays, 0, 6), vec![1, 2, 3, 4, 5]);
        assert_eq!(values(cron("* * * * 7").weekdays, 0, 6), vec![0]);
        assert_eq!(values(cron("5/20 * * * *").minutes, 0, 59), vec![5, 25, 45]);
        assert_eq!(cron("@daily"), cron("0 0 * * *"));
    }

    #[test]
    fn rejects_bad_expressions() {
        assert!("* * * *".parse::<Cron>().is_err());
        assert!("60 * * * *".parse::<Cron>().is_err());
        assert!("* * * * */0".parse::<Cron>().is_err());
        assert!("* * 0 * *".parse::<Cron>().is_err());
        let e = "* 25 * * *".parse::<Cron>().unwrap_err();
        assert_eq!(e, CronError::Field("hour", FieldError::Range { range: "25".into(), min: 0, max: 23 }));
        assert_eq!(e.to_string(), "hour field: 25 is outside 0–23");
    }

    #[test]
    fn next_runs_on_weekdays() {
        let runs: Vec<String> = cron("0 9 * * 1-5").upcoming(NOW, 5).into_iter().map(format_utc).collect();
        assert_eq!(
            runs,
            vec![
                "Thu 29 Oct 2026, 09:00",
                "Fri 30 Oct 2026, 09:00",
                "Mon 2 Nov 2026, 09:00",
                "Tue 3 Nov 2026, 09:00",
                "Wed 4 Nov 2026, 09:00"
            ]
        );
    }

    #[test]
    fn next_runs_every_fifteen_minutes() {
        let runs: Vec<String> = cron("*/15 * * * *").upcoming(NOW, 3).into_iter().map(format_utc).collect();
        assert_eq!(runs, vec!["Wed 28 Oct 2026, 12:45", "Wed 28 Oct 2026, 13:00", "Wed 28 Oct 2026, 13:15"]);
    }

    #[test]
    fn day_fields_or_together_when_both_set() {
        let runs: Vec<String> = cron("0 0 1 * 5").upcoming(NOW, 3).into_iter().map(format_utc).collect();
        assert_eq!(runs, vec!["Fri 30 Oct 2026, 00:00", "Sun 1 Nov 2026, 00:00", "Fri 6 Nov 2026, 00:00"]);
    }

    #[test]
    fn leap_day_and_impossible_dates() {
        let runs = cron("0 12 29 2 *").upcoming(NOW, 1);
        assert_eq!(format_utc(runs[0]), "Tue 29 Feb 2028, 12:00");
        assert!(cron("0 0 30 2 *").upcoming(NOW, 1).is_empty());
    }

    #[test]
    fn describes_in_words() {
        assert_eq!(cron("* * * * *").describe(), "Every minute");
        assert_eq!(cron("*/15 * * * *").describe(), "Every 15 minutes");
        assert_eq!(cron("0 * * * *").describe(), "Every hour");
        assert_eq!(cron("30 */2 * * *").describe(), "Every 2 hours at 30 minutes past");
        assert_eq!(cron("0 9 * * 1-5").describe(), "At 09:00 on weekdays");
        assert_eq!(cron("0 9,17 * * 0,6").describe(), "At 09:00 and 17:00 at weekends");
        assert_eq!(cron("30 8 1,15 * *").describe(), "At 08:30 on the 1st and 15th of the month");
        assert_eq!(cron("0 0 1 1 *").describe(), "At 00:00 on the 1st of the month in January");
        assert_eq!(cron("*/10 9-17 * * *").describe(), "Every 10 minutes from 09:00 to 17:59");
        assert_eq!(cron("0 6 * * 1,3,5").describe(), "At 06:00 on Monday, Wednesday and Friday");
        assert_eq!(cron("0 0 1 * 5").describe(), "At 00:00 on the 1st of the month or on Friday");
    }

    #[test]
    fn controls_round_trip_through_cron() {
        let base = Controls::default();
        for f in Frequency::ALL.into_iter().filter(|f| *f != Frequency::Custom) {
            let c = Controls { frequency: f, weekdays: 0b0010_1010, minute: 30, ..base };
            let expr = c.to_cron().unwrap();
            let back = Controls::recognise(&cron(&expr), &base);
            assert_eq!(back.to_cron(), Some(expr), "{f:?}");
            assert_eq!(back.frequency, f);
        }
        assert_eq!(Controls::recognise(&cron("0 9 * * MON-FRI"), &base).frequency, Frequency::Weekdays);
        assert_eq!(Controls::recognise(&cron("0 9-17 * * *"), &base).frequency, Frequency::Custom);
        assert_eq!(Controls::recognise(&cron("0 9 * 6 *"), &base).frequency, Frequency::Custom);
    }

    #[test]
    fn machine_keeps_text_and_controls_in_step() {
        let input = CronBuilderInput::default();
        let s = CronBuilderMachine::initial(&input);
        assert_eq!(s.expr, "0 9 * * *");
        let t = advance::<CronBuilderMachine>(&s, E::SetFrequency(Frequency::Weekdays), &input);
        assert_eq!(t.commands, vec![CronBuilderCommand::Change("0 9 * * 1-5".into())]);
        let t = advance::<CronBuilderMachine>(&t.state, E::SetTime { hour: 7, minute: 45 }, &input);
        assert_eq!(t.state.expr, "45 7 * * 1-5");
        let t = advance::<CronBuilderMachine>(&t.state, E::SetExpr("0 9-17 * * *".into()), &input);
        assert_eq!(t.state.controls.frequency, Frequency::Custom);
        assert_eq!(t.commands.len(), 1);
        let t = advance::<CronBuilderMachine>(&t.state, E::SetExpr("0 9-17 * *".into()), &input);
        assert!(t.state.error.is_some());
        assert!(t.commands.is_empty());
        let t = advance::<CronBuilderMachine>(&t.state, E::SetExpr("@hourly".into()), &input);
        assert_eq!(t.state.controls.frequency, Frequency::Hours);
        assert_eq!(t.state.controls.every_hours, 1);
    }

    #[test]
    fn last_weekday_stays_selected() {
        let input = CronBuilderInput { default_value: "0 9 * * 1".into(), ..Default::default() };
        let s = CronBuilderMachine::initial(&input);
        assert_eq!(s.controls.frequency, Frequency::Weekly);
        let t = advance::<CronBuilderMachine>(&s, E::ToggleWeekday(1), &input);
        assert_eq!(t.state.expr, "0 9 * * 1");
        let t = advance::<CronBuilderMachine>(&s, E::ToggleWeekday(3), &input);
        assert_eq!(t.state.expr, "0 9 * * 1,3");
    }

    #[test]
    fn parses_time_inputs() {
        assert_eq!(parse_time("09:30"), Some((9, 30)));
        assert_eq!(parse_time("23:59:00"), Some((23, 59)));
        assert_eq!(parse_time("24:00"), None);
    }
}
