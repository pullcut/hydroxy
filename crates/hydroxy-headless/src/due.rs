//! Due dates: whether a date is overdue, due soon or later, and its short label.
//! Dates are plain `YYYY-MM-DD` strings and whole days in UTC, so the server and the
//! browser agree during hydration.

/// How close a due date is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DueState {
    /// More than `soon_days` away.
    Later,
    /// Today, or within `soon_days`.
    Soon,
    /// Before today.
    Overdue,
}

impl DueState {
    /// The `data-state` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            DueState::Later => "later",
            DueState::Soon => "soon",
            DueState::Overdue => "overdue",
        }
    }
}

/// A due date, ready to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Due {
    /// See [`DueState`].
    pub state: DueState,
    /// Day and short month, such as "26 Oct".
    pub label: String,
    /// Tooltip and accessible name, such as "Due 26 Oct" or "Overdue since 26 Oct".
    pub title: String,
}

/// Days since 1970-01-01 for a `YYYY-MM-DD` date, or `None` if it isn't one.
#[must_use]
pub fn parse_ymd(date: &str) -> Option<i64> {
    let mut parts = date.trim().splitn(3, '-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.get(..2)?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=days_in_month(y, m)).contains(&d) {
        return None;
    }
    // Howard Hinnant's days_from_civil.
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        2 if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// "26 Oct" for a `YYYY-MM-DD` date.
fn short_label(date: &str) -> Option<String> {
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let mut parts = date.trim().splitn(3, '-').skip(1);
    let m: usize = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.get(..2)?.parse().ok()?;
    Some(format!("{d} {}", MONTHS.get(m.checked_sub(1)?)?))
}

/// Classify `date` against `today` (days since 1970-01-01). `None` if `date` isn't a date.
#[must_use]
pub fn due(date: &str, today: i64, soon_days: i64) -> Option<Due> {
    let day = parse_ymd(date)?;
    let label = short_label(date)?;
    let state = if day < today {
        DueState::Overdue
    } else if day - today <= soon_days {
        DueState::Soon
    } else {
        DueState::Later
    };
    let title = match state {
        DueState::Overdue => format!("Overdue since {label}"),
        _ => format!("Due {label}"),
    };
    Some(Due { state, label, title })
}

/// Today's date as days since 1970-01-01, in UTC.
#[must_use]
pub fn today() -> i64 {
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    let ms = js_sys::Date::now();
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    #[allow(clippy::cast_precision_loss, reason = "milliseconds since 1970 fit in f64 for millions of years")]
    let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0.0, |d| d.as_millis() as f64);
    #[allow(clippy::cast_possible_truncation, reason = "whole days since 1970 fit in i64")]
    let days = (ms / 86_400_000.0).floor() as i64;
    days
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dates_to_days_since_1970() {
        assert_eq!(parse_ymd("1970-01-01"), Some(0));
        assert_eq!(parse_ymd("2000-03-01"), Some(11_017));
        assert_eq!(parse_ymd("2024-02-29"), Some(19_782));
        assert_eq!(parse_ymd("2023-02-29"), None);
        assert_eq!(parse_ymd("26 Oct"), None);
    }

    #[test]
    fn classifies_against_today() {
        let today = parse_ymd("2026-10-26").unwrap();
        assert_eq!(due("2026-10-25", today, 3).unwrap().state, DueState::Overdue);
        assert_eq!(due("2026-10-26", today, 3).unwrap().state, DueState::Soon);
        assert_eq!(due("2026-10-29", today, 3).unwrap().state, DueState::Soon);
        assert_eq!(due("2026-10-30", today, 3).unwrap().state, DueState::Later);
    }

    #[test]
    fn labels_are_day_and_short_month() {
        let today = parse_ymd("2026-10-26").unwrap();
        let d = due("2026-11-03", today, 3).unwrap();
        assert_eq!((d.label.as_str(), d.title.as_str()), ("3 Nov", "Due 3 Nov"));
        assert_eq!(due("2026-10-01", today, 3).unwrap().title, "Overdue since 1 Oct");
    }
}
