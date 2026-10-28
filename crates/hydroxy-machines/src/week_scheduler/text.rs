//! Display text for dates, times and events.

use super::{Placement, WeekEvent, placed_text};
use crate::date::Date;

/// `"Monday"`.
#[must_use]
pub fn weekday_name(d: Date) -> &'static str {
    crate::date::WEEKDAY_NAMES[usize::from(d.weekday()) % 7]
}

/// `"Mon"`.
#[must_use]
pub fn weekday_short(d: Date) -> &'static str {
    &weekday_name(d)[..3]
}

/// `"November"`.
#[must_use]
pub fn month_name(d: Date) -> &'static str {
    crate::date::MONTH_NAMES[usize::from(d.month().clamp(1, 12) - 1)]
}

/// `"Tuesday 3 November"`.
#[must_use]
pub fn date_text(d: Date) -> String {
    format!("{} {} {}", weekday_name(d), d.day(), month_name(d))
}

/// `"09:00"`. Midnight at the end of a day is `"24:00"`.
#[must_use]
pub fn clock(minute: u16) -> String {
    crate::date::hhmm(u32::from(minute / 60), u32::from(minute % 60))
}

/// `"09:00 to 10:30"`.
#[must_use]
pub fn time_range(start: u16, end: u16) -> String {
    format!("{} to {}", clock(start), clock(end))
}

/// `"Tuesday 3 November, 09:00 to 10:00"`, or `"Tuesday 3 November, all day"`.
#[must_use]
pub fn placement_text(p: Placement) -> String {
    if p.all_day {
        format!("{}, all day", date_text(p.date))
    } else {
        format!("{}, {}", date_text(p.date), time_range(p.start, p.end))
    }
}

/// An event's accessible name: `"Design review, Tuesday 3 November, 10:00 to 11:00"`.
#[must_use]
pub fn event_label(e: &WeekEvent) -> String {
    placed_text(&e.title, e.placement())
}

/// The toolbar title for the visible days: `"Tuesday 3 November 2026"`,
/// `"2 – 8 November 2026"`, `"30 November – 6 December 2026"` or
/// `"28 December 2026 – 3 January 2027"`.
#[must_use]
pub fn range_title(days: &[Date]) -> String {
    let (Some(&a), Some(&b)) = (days.first(), days.last()) else { return String::new() };
    if a == b {
        format!("{} {}", date_text(a), a.year())
    } else if a.year() != b.year() {
        format!("{} {} {} – {} {} {}", a.day(), month_name(a), a.year(), b.day(), month_name(b), b.year())
    } else if a.month() != b.month() {
        format!("{} {} – {} {} {}", a.day(), month_name(a), b.day(), month_name(b), b.year())
    } else {
        format!("{} – {} {} {}", a.day(), b.day(), month_name(b), b.year())
    }
}
