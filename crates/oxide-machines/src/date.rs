//! A plain calendar date (proleptic Gregorian, no time zone) and the arithmetic
//! the calendar needs. Day counts use Howard Hinnant's civil-from-days algorithms.

use std::fmt;
use std::str::FromStr;

/// A calendar date. Ordered chronologically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

impl Default for Date {
    /// 1970-01-01.
    fn default() -> Self {
        Self { year: 1970, month: 1, day: 1 }
    }
}

/// Whether `year` has a 29 February.
#[must_use]
pub const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Days in `month` (1–12) of `year`.
#[must_use]
pub const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl Date {
    /// A date, or `None` if the month or day is out of range.
    #[must_use]
    pub const fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if month < 1 || month > 12 || day < 1 || day > days_in_month(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// The year.
    #[must_use]
    pub const fn year(self) -> i32 {
        self.year
    }

    /// The month, 1–12.
    #[must_use]
    pub const fn month(self) -> u8 {
        self.month
    }

    /// The day of the month, from 1.
    #[must_use]
    pub const fn day(self) -> u8 {
        self.day
    }

    /// Days since 1970-01-01 (negative before).
    #[must_use]
    pub fn to_days(self) -> i64 {
        let m = i64::from(self.month);
        let y = i64::from(self.year) - i64::from(m <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// The date `days` after 1970-01-01.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "month and day are in range by construction")]
    pub fn from_days(days: i64) -> Self {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u8;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
        Self { year, month, day }
    }

    /// Day of the week, 0 = Sunday … 6 = Saturday.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "rem_euclid(7) is 0..7")]
    pub fn weekday(self) -> u8 {
        // 1970-01-01 was a Thursday.
        (self.to_days() + 4).rem_euclid(7) as u8
    }

    /// The date `n` days later (earlier when negative).
    #[must_use]
    pub fn add_days(self, n: i64) -> Self {
        Self::from_days(self.to_days() + n)
    }

    /// The same day `n` months later, clamped to the end of a shorter month
    /// (31 January + 1 month is 28 or 29 February).
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "rem_euclid(12) is 0..12")]
    pub fn add_months(self, n: i32) -> Self {
        let index = self.year * 12 + i32::from(self.month) - 1 + n;
        let year = index.div_euclid(12);
        let month = index.rem_euclid(12) as u8 + 1;
        Self { year, month, day: self.day.min(days_in_month(year, month)) }
    }

    /// The same day `n` years later, clamped (29 February becomes 28 February).
    #[must_use]
    pub fn add_years(self, n: i32) -> Self {
        self.add_months(n * 12)
    }

    /// The first day of this date's month.
    #[must_use]
    pub const fn first_of_month(self) -> Self {
        Self { day: 1, ..self }
    }

    /// The last day of this date's month.
    #[must_use]
    pub const fn last_of_month(self) -> Self {
        Self { day: days_in_month(self.year, self.month), ..self }
    }

    /// Days back to the start of this date's week, given the week's first day (0 = Sunday).
    #[must_use]
    pub fn days_into_week(self, week_start: u8) -> i64 {
        i64::from((self.weekday() + 7 - week_start % 7) % 7)
    }

    /// Six full weeks covering this date's month, starting on `week_start`
    /// (0 = Sunday). Days from the months either side fill the gaps.
    #[must_use]
    pub fn month_grid(self, week_start: u8) -> [Date; 42] {
        let first = self.first_of_month();
        let start = first.add_days(-first.days_into_week(week_start)).to_days();
        std::array::from_fn(|i| Self::from_days(start + i64::try_from(i).unwrap_or_default()))
    }
}

/// `YYYY-MM-DD`.
impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Why a string isn't a `YYYY-MM-DD` date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseDateError;

impl fmt::Display for ParseDateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expected a date as YYYY-MM-DD")
    }
}

impl std::error::Error for ParseDateError {}

/// Parses ISO `YYYY-MM-DD`, strictly: four-digit year, two-digit month and day.
impl FromStr for Date {
    type Err = ParseDateError;

    fn from_str(s: &str) -> Result<Self, ParseDateError> {
        let b = s.as_bytes();
        let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' || !digits(0..4) || !digits(5..7) || !digits(8..10) {
            return Err(ParseDateError);
        }
        let num = |r: std::ops::Range<usize>| s[r].parse::<u16>().map_err(|_| ParseDateError);
        let (year, month, day) = (num(0..4)?, num(5..7)?, num(8..10)?);
        #[allow(clippy::cast_possible_truncation, reason = "two digits fit in u8")]
        Date::new(i32::from(year), month as u8, day as u8).ok_or(ParseDateError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Date {
        s.parse().unwrap()
    }

    #[test]
    fn days_round_trip() {
        assert_eq!(d("1970-01-01").to_days(), 0);
        assert_eq!(d("2000-03-01").to_days(), 11_017);
        assert_eq!(d("2024-02-29").to_days(), 19_782);
        assert_eq!(d("1969-12-31").to_days(), -1);
        for n in -800_000..800_000 {
            assert_eq!(Date::from_days(n).to_days(), n);
        }
    }

    #[test]
    fn leap_years() {
        assert!(is_leap_year(2024));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2026));
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2100, 2), 28);
        assert!(Date::new(2023, 2, 29).is_none());
        assert!(Date::new(2024, 2, 29).is_some());
        assert!(Date::new(2026, 4, 31).is_none());
        assert!(Date::new(2026, 13, 1).is_none());
        assert!(Date::new(2026, 1, 0).is_none());
    }

    #[test]
    fn weekdays() {
        assert_eq!(d("1970-01-01").weekday(), 4);
        assert_eq!(d("2026-10-26").weekday(), 1);
        assert_eq!(d("2000-01-01").weekday(), 6);
        assert_eq!(d("1969-12-28").weekday(), 0);
    }

    #[test]
    fn adding_days_crosses_months_and_years() {
        assert_eq!(d("2026-12-31").add_days(1), d("2027-01-01"));
        assert_eq!(d("2024-02-28").add_days(1), d("2024-02-29"));
        assert_eq!(d("2023-02-28").add_days(1), d("2023-03-01"));
        assert_eq!(d("2026-03-01").add_days(-1), d("2026-02-28"));
        assert_eq!(d("2026-10-26").add_days(-7), d("2026-10-19"));
    }

    #[test]
    fn adding_months_clamps_the_day() {
        assert_eq!(d("2026-01-31").add_months(1), d("2026-02-28"));
        assert_eq!(d("2024-01-31").add_months(1), d("2024-02-29"));
        assert_eq!(d("2026-03-31").add_months(-1), d("2026-02-28"));
        assert_eq!(d("2026-12-15").add_months(1), d("2027-01-15"));
        assert_eq!(d("2026-01-15").add_months(-1), d("2025-12-15"));
        assert_eq!(d("2026-05-31").add_months(-25), d("2024-04-30"));
        assert_eq!(d("2024-02-29").add_years(1), d("2025-02-28"));
        assert_eq!(d("2024-02-29").add_years(4), d("2028-02-29"));
    }

    #[test]
    fn month_ends() {
        assert_eq!(d("2024-02-10").last_of_month(), d("2024-02-29"));
        assert_eq!(d("2026-09-10").last_of_month(), d("2026-09-30"));
        assert_eq!(d("2026-09-10").first_of_month(), d("2026-09-01"));
    }

    #[test]
    fn grid_starts_on_the_week_start() {
        // October 2026 starts on a Thursday.
        let monday = d("2026-10-15").month_grid(1);
        assert_eq!(monday[0], d("2026-09-28"));
        assert_eq!(monday[3], d("2026-10-01"));
        assert_eq!(monday[41], d("2026-11-08"));
        let sunday = d("2026-10-15").month_grid(0);
        assert_eq!(sunday[0], d("2026-09-27"));
        assert!(sunday.iter().all(|x| x.to_days() - sunday[0].to_days() < 42));
        // A month starting on the week start begins in the first cell.
        assert_eq!(d("2026-06-20").month_grid(1)[0], d("2026-06-01"));
    }

    #[test]
    fn parse_and_format() {
        assert_eq!(d("2026-10-26").to_string(), "2026-10-26");
        assert_eq!(Date::new(33, 1, 2).unwrap().to_string(), "0033-01-02");
        for bad in ["", "2026-1-26", "2026/10/26", "26-10-2026", "2026-02-30", "2026-00-10", "+026-10-26", "2026-10-26x"] {
            assert!(bad.parse::<Date>().is_err(), "{bad}");
        }
    }

    #[test]
    fn orders_chronologically() {
        assert!(d("2026-01-31") < d("2026-02-01"));
        assert!(d("2025-12-31") < d("2026-01-01"));
    }
}
