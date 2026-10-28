//! Cron expression parsing, upcoming runs and plain-language descriptions.

use std::fmt::{self, Write as _};
use std::str::FromStr;

use crate::date::{Date, MONTH_NAMES, WEEKDAY_NAMES, hhmm};

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
pub(super) fn step_of(values: &[u32], min: u32, max: u32) -> Option<u32> {
    if values.len() < 2 || values[0] != min {
        return None;
    }
    let n = values[1] - values[0];
    let expected = (max - min) / n + 1;
    (values.windows(2).all(|w| w[1] - w[0] == n) && values.len() == expected as usize).then_some(n)
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
    let (h, m) = (u32::try_from(in_day / 3600).unwrap_or(0), u32::try_from(in_day % 3600 / 60).unwrap_or(0));
    format!(
        "{} {} {} {}, {}",
        &WEEKDAY_NAMES[usize::from(date.weekday())][..3],
        date.day(),
        &MONTH_NAMES[usize::from(date.month()) - 1][..3],
        date.year(),
        hhmm(h, m)
    )
}

/// "2026-10-29T09:00Z", for a `<time datetime>`.
#[must_use]
pub fn iso_utc(secs: i64) -> String {
    let date = Date::from_days(secs.div_euclid(86_400));
    let in_day = secs.rem_euclid(86_400);
    let (h, m) = (u32::try_from(in_day / 3600).unwrap_or(0), u32::try_from(in_day % 3600 / 60).unwrap_or(0));
    format!("{date}T{}Z", hhmm(h, m))
}
