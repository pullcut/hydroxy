//! `CronBuilder` keeps a five-field cron expression (`minute hour day-of-month month
//! day-of-week`) and the plain-language controls that build it in step. Editing the
//! controls rewrites the expression; typing an expression sets the controls when it fits one
//! of the presets, and "Custom" when it doesn't. [`Cron`] parses expressions (ranges, steps,
//! lists, month and weekday names, `@daily` style macros), finds upcoming runs in UTC and
//! describes them in words.

use std::fmt::{self, Write as _};
use std::str::FromStr;

use oxide_statechart::{Machine, Transition};

use crate::date::Date;

/// Full month names, January first.
pub const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November",
    "December",
];

/// Full weekday names, Sunday (cron's 0) first.
pub const WEEKDAY_NAMES: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

const FIELD_NAMES: [&str; 5] = ["minute", "hour", "day of month", "month", "day of week"];

/// Why an expression didn't parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronError(pub String);

impl fmt::Display for CronError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CronError {}

/// A parsed cron expression. Each field is a bit set: bit `n` means value `n` matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cron {
    /// Minutes, 0–59.
    pub minutes: u64,
    /// Hours, 0–23.
    pub hours: u64,
    /// Days of the month, 1–31.
    pub days: u64,
    /// Months, 1–12.
    pub months: u64,
    /// Days of the week, 0 (Sunday) – 6.
    pub weekdays: u64,
    /// The day-of-month field starts with `*`. With both day fields restricted, a day
    /// matches when either does (classic cron behaviour).
    pub dom_star: bool,
    /// The day-of-week field starts with `*`.
    pub dow_star: bool,
}

fn has(mask: u64, n: u32) -> bool {
    n < 64 && mask & (1 << n) != 0
}

/// The smallest set bit at or after `from`.
fn next_bit(mask: u64, from: u32) -> Option<u32> {
    if from >= 64 {
        return None;
    }
    let rest = mask >> from;
    (rest != 0).then(|| from + rest.trailing_zeros())
}

/// The values in `mask` between `min` and `max`, ascending.
#[must_use]
pub fn values(mask: u64, min: u32, max: u32) -> Vec<u32> {
    (min..=max).filter(|&n| has(mask, n)).collect()
}

fn parse_value(text: &str, names: &[&str], offset: u32) -> Option<u32> {
    if let Ok(n) = text.parse::<u32>() {
        return Some(n);
    }
    let lower = text.to_ascii_lowercase();
    names
        .iter()
        .position(|n| lower.len() == 3 && n.to_ascii_lowercase().starts_with(&lower))
        .and_then(|i| u32::try_from(i).ok())
        .map(|i| i + offset)
}

/// Parse one field into a bit set. `names` are matched by their first three letters.
fn parse_field(text: &str, min: u32, max: u32, names: &[&str], offset: u32) -> Result<u64, String> {
    let mut mask = 0;
    for part in text.split(',') {
        let (range, step) = match part.split_once('/') {
            Some((r, s)) => {
                let step = s.parse::<u32>().ok().filter(|s| *s > 0).ok_or_else(|| format!("bad step \"{s}\""))?;
                (r, Some(step))
            }
            None => (part, None),
        };
        let value = |t: &str| parse_value(t, names, offset).ok_or_else(|| format!("\"{t}\" is not a value"));
        let (lo, hi) = match range {
            "*" | "?" => (min, max),
            r => {
                if let Some((a, b)) = r.split_once('-') {
                    (value(a)?, value(b)?)
                } else {
                    let v = value(r)?;
                    (v, if step.is_some() { max } else { v })
                }
            }
        };
        if lo < min || hi > max || lo > hi {
            return Err(format!("{range} is outside {min}–{max}"));
        }
        let step = step.unwrap_or(1) as usize;
        for n in (lo..=hi).step_by(step) {
            mask |= 1 << n;
        }
    }
    Ok(mask)
}

impl FromStr for Cron {
    type Err = CronError;

    fn from_str(text: &str) -> Result<Self, CronError> {
        let text = text.trim();
        let expanded = match text.to_ascii_lowercase().as_str() {
            "@yearly" | "@annually" => "0 0 1 1 *",
            "@monthly" => "0 0 1 * *",
            "@weekly" => "0 0 * * 0",
            "@daily" | "@midnight" => "0 0 * * *",
            "@hourly" => "0 * * * *",
            _ => text,
        };
        let fields: Vec<&str> = expanded.split_whitespace().collect();
        if fields.len() != 5 {
            return Err(CronError(format!("Expected 5 fields, found {}", fields.len())));
        }
        let err = |i: usize| move |m: String| CronError(format!("{} field: {m}", capitalise(FIELD_NAMES[i])));
        let minutes = parse_field(fields[0], 0, 59, &[], 0).map_err(err(0))?;
        let hours = parse_field(fields[1], 0, 23, &[], 0).map_err(err(1))?;
        let days = parse_field(fields[2], 1, 31, &[], 0).map_err(err(2))?;
        let months = parse_field(fields[3], 1, 12, &MONTH_NAMES, 1).map_err(err(3))?;
        let mut weekdays = parse_field(fields[4], 0, 7, &WEEKDAY_NAMES, 0).map_err(err(4))?;
        // 7 is another name for Sunday.
        if has(weekdays, 7) {
            weekdays = (weekdays & !(1 << 7)) | 1;
        }
        Ok(Cron {
            minutes,
            hours,
            days,
            months,
            weekdays,
            dom_star: fields[2].starts_with('*') || fields[2] == "?",
            dow_star: fields[4].starts_with('*') || fields[4] == "?",
        })
    }
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
}

impl Cron {
    fn day_matches(&self, date: Date) -> bool {
        let dom = has(self.days, u32::from(date.day()));
        let dow = has(self.weekdays, u32::from(date.weekday()));
        match (self.dom_star, self.dow_star) {
            (true, true) => dom && dow,
            (true, false) => dow,
            (false, true) => dom,
            (false, false) => dom || dow,
        }
    }

    /// The first run strictly after `after` (Unix seconds, UTC), or `None` if there is none
    /// in the next ~30 years (such as 30 February).
    #[must_use]
    pub fn next_after(&self, after: i64) -> Option<i64> {
        // Work in whole minutes since the epoch.
        let mut t = after.div_euclid(60) + 1;
        let limit = t + 30 * 366 * 1440;
        while t < limit {
            let days = t.div_euclid(1440);
            let in_day = u32::try_from(t.rem_euclid(1440)).unwrap_or(0);
            let date = Date::from_days(days);
            if !has(self.months, u32::from(date.month())) {
                t = date.first_of_month().add_months(1).to_days() * 1440;
                continue;
            }
            if !self.day_matches(date) {
                t = (days + 1) * 1440;
                continue;
            }
            let (hour, minute) = (in_day / 60, in_day % 60);
            if !has(self.hours, hour) {
                t = match next_bit(self.hours, hour) {
                    Some(h) if h < 24 => days * 1440 + i64::from(h) * 60,
                    _ => (days + 1) * 1440,
                };
                continue;
            }
            match next_bit(self.minutes, minute) {
                Some(m) if m < 60 => return Some((days * 1440 + i64::from(hour * 60 + m)) * 60),
                _ => t = days * 1440 + i64::from(hour + 1) * 60,
            }
        }
        None
    }

    /// Up to `count` runs after `after`, in order.
    #[must_use]
    pub fn upcoming(&self, after: i64, count: usize) -> Vec<i64> {
        let mut out = Vec::with_capacity(count);
        let mut t = after;
        while out.len() < count {
            let Some(next) = self.next_after(t) else { break };
            out.push(next);
            t = next;
        }
        out
    }

    /// The schedule in words, such as "At 09:00 on weekdays".
    #[must_use]
    pub fn describe(&self) -> String {
        let mut out = describe_time(&values(self.minutes, 0, 59), &values(self.hours, 0, 23));
        let month_days = values(self.days, 1, 31);
        let week_days = values(self.weekdays, 0, 6);
        let by_date = (!self.dom_star && month_days.len() < 31).then(|| describe_days(&month_days));
        let by_weekday = (!self.dow_star && week_days.len() < 7).then(|| describe_weekdays(&week_days));
        match (by_date, by_weekday) {
            (Some(a), Some(b)) => {
                let _ = write!(out, " {a} or {b}");
            }
            (Some(a), None) | (None, Some(a)) => {
                let _ = write!(out, " {a}");
            }
            (None, None) => {}
        }
        let months = values(self.months, 1, 12);
        if months.len() < 12 {
            let names = runs(&months, |m| MONTH_NAMES[m as usize - 1].to_string());
            let _ = write!(out, " in {}", phrase(&names));
        }
        out
    }
}

/// `a`, `a and b`, `a, b and c`.
fn phrase(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// Group sorted values into "X to Y" for runs of three or more.
fn runs(values: &[u32], name: impl Fn(u32) -> String) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < values.len() {
        let mut j = i;
        while j + 1 < values.len() && values[j + 1] == values[j] + 1 {
            j += 1;
        }
        if j - i >= 2 {
            out.push(format!("{} to {}", name(values[i]), name(values[j])));
        } else {
            out.extend(values[i..=j].iter().map(|&v| name(v)));
        }
        i = j + 1;
    }
    out
}

/// The step when `values` is every `n`th value from `min` up to `max` (n ≥ 1). `None` for a
/// single value.
fn step_of(values: &[u32], min: u32, max: u32) -> Option<u32> {
    if values.len() < 2 || values[0] != min {
        return None;
    }
    let n = values[1] - values[0];
    let expected = (max - min) / n + 1;
    (values.windows(2).all(|w| w[1] - w[0] == n) && values.len() == expected as usize).then_some(n)
}

fn hhmm(h: u32, m: u32) -> String {
    format!("{h:02}:{m:02}")
}

fn describe_time(mins: &[u32], hrs: &[u32]) -> String {
    let all_hours = hrs.len() == 24;
    let minute_step = step_of(mins, 0, 59);
    let contiguous = hrs.windows(2).all(|w| w[1] == w[0] + 1);
    match (mins, minute_step) {
        (_, Some(1)) if all_hours => "Every minute".into(),
        (_, Some(n)) if all_hours => format!("Every {n} minutes"),
        ([0], _) if all_hours => "Every hour".into(),
        ([m], _) if all_hours => format!("Every hour at {m} minutes past"),
        ([m], _) => match step_of(hrs, 0, 23) {
            Some(n) if *m == 0 => format!("Every {n} hours"),
            Some(n) => format!("Every {n} hours at {m} minutes past"),
            None if hrs.len() <= 6 => {
                format!("At {}", phrase(&hrs.iter().map(|&h| hhmm(h, *m)).collect::<Vec<_>>()))
            }
            None => format!("At {m} minutes past {}", hour_list(hrs)),
        },
        (_, Some(n)) if contiguous => {
            let every = if n == 1 { "Every minute".to_string() } else { format!("Every {n} minutes") };
            format!("{every} from {} to {}", hhmm(hrs[0], 0), hhmm(hrs[hrs.len() - 1], 59))
        }
        _ => format!(
            "At minute {} past {}",
            phrase(&runs(mins, |m| m.to_string())),
            if all_hours { "every hour".to_string() } else { hour_list(hrs) }
        ),
    }
}

fn hour_list(hrs: &[u32]) -> String {
    let word = if hrs.len() == 1 { "hour" } else { "hours" };
    format!("{word} {}", phrase(&runs(hrs, |h| h.to_string())))
}

fn ordinal(n: u32) -> String {
    let suffix = match (n % 10, n % 100) {
        (1, 11) | (2, 12) | (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

fn describe_days(doms: &[u32]) -> String {
    if let Some(n) = step_of(doms, 1, 31) {
        return format!("on every {} day of the month", ordinal(n));
    }
    format!("on the {} of the month", phrase(&runs(doms, ordinal)))
}

fn describe_weekdays(dows: &[u32]) -> String {
    match dows {
        [1, 2, 3, 4, 5] => "on weekdays".into(),
        [0, 6] => "at weekends".into(),
        _ => format!("on {}", phrase(&runs(dows, |d| WEEKDAY_NAMES[d as usize].to_string()))),
    }
}

/// "Thu 29 Oct 2026, 09:00" for a Unix time in UTC.
#[must_use]
pub fn format_utc(secs: i64) -> String {
    let date = Date::from_days(secs.div_euclid(86_400));
    let in_day = secs.rem_euclid(86_400);
    format!(
        "{} {} {} {}, {:02}:{:02}",
        &WEEKDAY_NAMES[usize::from(date.weekday())][..3],
        date.day(),
        &MONTH_NAMES[usize::from(date.month()) - 1][..3],
        date.year(),
        in_day / 3600,
        in_day % 3600 / 60
    )
}

/// "2026-10-29T09:00Z", for a `<time datetime>`.
#[must_use]
pub fn iso_utc(secs: i64) -> String {
    let date = Date::from_days(secs.div_euclid(86_400));
    let in_day = secs.rem_euclid(86_400);
    format!("{date}T{:02}:{:02}Z", in_day / 3600, in_day % 3600 / 60)
}

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

    /// The preset for an `as_str` value.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.as_str() == s)
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
        assert!(e.0.starts_with("Hour field"), "{e}");
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
        let t = CronBuilderMachine::transition(&s, E::SetFrequency(Frequency::Weekdays), &input).resolve(&s);
        assert_eq!(t.commands, vec![CronBuilderCommand::Change("0 9 * * 1-5".into())]);
        let t = CronBuilderMachine::transition(&t.state, E::SetTime { hour: 7, minute: 45 }, &input).resolve(&t.state);
        assert_eq!(t.state.expr, "45 7 * * 1-5");
        let t = CronBuilderMachine::transition(&t.state, E::SetExpr("0 9-17 * * *".into()), &input).resolve(&t.state);
        assert_eq!(t.state.controls.frequency, Frequency::Custom);
        assert_eq!(t.commands.len(), 1);
        let t = CronBuilderMachine::transition(&t.state, E::SetExpr("0 9-17 * *".into()), &input).resolve(&t.state);
        assert!(t.state.error.is_some());
        assert!(t.commands.is_empty());
        let t = CronBuilderMachine::transition(&t.state, E::SetExpr("@hourly".into()), &input).resolve(&t.state);
        assert_eq!(t.state.controls.frequency, Frequency::Hours);
        assert_eq!(t.state.controls.every_hours, 1);
    }

    #[test]
    fn last_weekday_stays_selected() {
        let input = CronBuilderInput { default_value: "0 9 * * 1".into(), ..Default::default() };
        let s = CronBuilderMachine::initial(&input);
        assert_eq!(s.controls.frequency, Frequency::Weekly);
        let t = CronBuilderMachine::transition(&s, E::ToggleWeekday(1), &input).resolve(&s);
        assert_eq!(t.state.expr, "0 9 * * 1");
        let t = CronBuilderMachine::transition(&s, E::ToggleWeekday(3), &input).resolve(&s);
        assert_eq!(t.state.expr, "0 9 * * 1,3");
    }

    #[test]
    fn parses_time_inputs() {
        assert_eq!(parse_time("09:30"), Some((9, 30)));
        assert_eq!(parse_time("23:59:00"), Some((23, 59)));
        assert_eq!(parse_time("24:00"), None);
    }
}
