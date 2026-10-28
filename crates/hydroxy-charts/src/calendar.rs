//! Calendar heatmap layout: one cell per day in week columns (Monday at the top), the
//! way a contribution graph reads. A year or less is one continuous block; longer spans
//! break into one block per calendar year so the years stack and line up.

use crate::scale::{self, civil, days_from_civil};

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// Short weekday names, Monday first, matching [`Day::weekday`].
pub const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// Spans up to this many days stay one continuous block.
const ONE_BLOCK: i64 = 371;

/// One day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Day {
    /// Days since 1970-01-01.
    pub date: i64,
    /// Which block (year row) it sits in.
    pub block: usize,
    /// Week column inside its block, from 0.
    pub week: usize,
    /// 0 = Monday … 6 = Sunday; the row.
    pub weekday: usize,
    /// The value, `None` when missing or not finite.
    pub value: Option<f64>,
    /// Position on the colour ramp, 0..=1. Zero for missing days.
    pub level: f64,
}

/// A month name over the week column holding its first day.
#[derive(Debug, Clone, PartialEq)]
pub struct MonthLabel {
    /// Week column inside the block.
    pub week: usize,
    /// "Jan", "Feb", …
    pub name: &'static str,
}

/// A run of weeks drawn as one grid: a calendar year, or the whole span when it fits in one.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    /// The year, or empty for a single continuous block.
    pub label: String,
    /// Month names along the top.
    pub months: Vec<MonthLabel>,
    /// Index of the block's first day in [`CalendarLayout::days`].
    pub first: usize,
    /// One past its last day.
    pub end: usize,
}

/// Everything a calendar heatmap needs.
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarLayout {
    /// Every day in order.
    pub days: Vec<Day>,
    /// The blocks, earliest first.
    pub blocks: Vec<Block>,
    /// Week columns per block; the same for every block so cells line up.
    pub weeks: usize,
    /// Bottom of the colour ramp: zero for counts, else the smallest value.
    pub min: f64,
    /// Top of the colour ramp: the largest value.
    pub max: f64,
}

/// 0 = Monday … 6 = Sunday. 1970-01-01 was a Thursday.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "rem_euclid(7) is 0..7")]
#[must_use]
pub fn weekday(date: i64) -> usize {
    (date + 3).rem_euclid(7) as usize
}

/// A date as "Tue 3 Mar 2026".
#[must_use]
pub fn day_name(date: i64) -> String {
    let (y, m, d) = civil(date);
    format!("{} {d} {} {y}", WEEKDAYS[weekday(date)], MONTHS[(m - 1) as usize])
}

/// Lay out one value per day starting on `start` (days since 1970-01-01, see
/// [`days_from_civil`]). Non-finite values are missing days.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    reason = "day counts are small and non-negative"
)]
#[must_use]
pub fn layout(start: i64, values: &[f64]) -> CalendarLayout {
    let n = values.len() as i64;
    let finite = |v: f64| Some(v).filter(|v| v.is_finite());
    let (min, max) = scale::extent(values.iter().copied()).map_or((0.0, 0.0), |(lo, hi)| (lo.min(0.0), hi));
    let span = max - min;
    let end = start + n;
    // Block starts: one block, or 1 January of each year (the first clipped to the data).
    let single = n <= ONE_BLOCK;
    let mut starts = vec![start];
    if !single {
        let (y0, _, _) = civil(start);
        let (y1, _, _) = civil(end - 1);
        starts.extend((y0 + 1..=y1).map(|y| days_from_civil(y, 1, 1)));
    }
    // Each block's grid starts on the Monday on or before its first day; a year block on
    // the Monday before 1 January, so every year's columns line up.
    let origin = |b: usize| {
        let first = if single || b > 0 { starts[b] } else { days_from_civil(civil(start).0, 1, 1) };
        first - weekday(first) as i64
    };
    let mut days = Vec::with_capacity(values.len());
    let mut blocks = Vec::new();
    let mut weeks = 0;
    for b in 0..starts.len() {
        let (from, to) = (starts[b], starts.get(b + 1).copied().unwrap_or(end));
        let o = origin(b);
        let first = days.len();
        let mut months = Vec::new();
        for date in from..to {
            let value = finite(values[(date - start) as usize]);
            let level = value.map_or(0.0, |v| if span > 0.0 { ((v - min) / span).clamp(0.0, 1.0) } else { 1.0 });
            let week = ((date - o) / 7) as usize;
            let (_, m, d) = civil(date);
            if d == 1 || date == from {
                // A label for a month that starts in the last day or two of a week would
                // crowd its neighbour, so the first partial month waits for room.
                if months.last().is_none_or(|l: &MonthLabel| week >= l.week + 3) {
                    months.push(MonthLabel { week, name: MONTHS[(m - 1) as usize] });
                } else if let Some(last) = months.last_mut().filter(|_| d == 1) {
                    *last = MonthLabel { week, name: MONTHS[(m - 1) as usize] };
                }
            }
            weeks = weeks.max(week + 1);
            days.push(Day { date, block: b, week, weekday: weekday(date), value, level });
        }
        let label = if single { String::new() } else { civil(from).0.to_string() };
        blocks.push(Block { label, months, first, end: days.len() });
    }
    CalendarLayout { days, blocks, weeks, min, max }
}

/// `data-edge` for a day in week column `week` of `weeks`: "start" or "end" near either
/// side, so its tooltip opens inwards instead of off the chart.
#[must_use]
pub fn edge(week: usize, weeks: usize) -> Option<&'static str> {
    if week < 5 {
        Some("start")
    } else if week + 6 > weeks {
        Some("end")
    } else {
        None
    }
}

/// Keyboard movement over the days: Up/Down step a day (a row in the grid),
/// Left/Right a week (a column), Home/End jump to the first or last day, Escape clears.
#[must_use]
pub fn key(key: &str, current: Option<usize>, len: usize) -> Option<Option<usize>> {
    let last = len.checked_sub(1)?;
    let Some(i) = current else {
        return matches!(key, "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | "Home" | "End")
            .then_some(Some(if matches!(key, "End" | "ArrowLeft" | "ArrowUp") { last } else { 0 }));
    };
    let next = match key {
        "ArrowDown" => (i + 1).min(last),
        "ArrowUp" => i.saturating_sub(1),
        "ArrowRight" => (i + 7).min(last),
        "ArrowLeft" => i.saturating_sub(7),
        "Home" => 0,
        "End" => last,
        "Escape" => return Some(None),
        _ => return None,
    };
    Some(Some(next))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weekdays_start_on_monday() {
        assert_eq!(weekday(0), 3, "1970-01-01 was a Thursday");
        assert_eq!(weekday(days_from_civil(2026, 10, 26)), 0);
        assert_eq!(day_name(days_from_civil(2026, 3, 3)), "Tue 3 Mar 2026");
    }

    #[test]
    fn a_year_is_one_block_in_week_columns() {
        let start = days_from_civil(2026, 1, 1); // a Thursday
        let l = layout(start, &[1.0; 365]);
        assert_eq!(l.blocks.len(), 1);
        assert_eq!(l.blocks[0].label, "");
        assert_eq!((l.days[0].week, l.days[0].weekday), (0, 3));
        assert_eq!((l.days[4].week, l.days[4].weekday), (1, 0), "Monday 5 Jan opens week 1");
        assert_eq!(l.weeks, 53);
        assert_eq!(l.blocks[0].months.len(), 12);
        assert_eq!(l.blocks[0].months[1].name, "Feb");
    }

    #[test]
    fn longer_spans_split_by_year_and_line_up() {
        let start = days_from_civil(2024, 6, 15);
        let end = days_from_civil(2026, 3, 1);
        let l = layout(start, &vec![2.0; usize::try_from(end - start).unwrap()]);
        assert_eq!(l.blocks.iter().map(|b| b.label.as_str()).collect::<Vec<_>>(), ["2024", "2025", "2026"]);
        // The partial first year keeps its place: mid-June 2024 sits 23 weeks after Monday 1 January.
        assert_eq!(l.days[0].week, 23);
        assert_eq!(l.blocks[1].first, l.blocks[0].end);
        assert_eq!(l.days[l.blocks[1].first].date, days_from_civil(2025, 1, 1));
        assert_eq!(l.blocks[2].months.iter().map(|m| m.name).collect::<Vec<_>>(), ["Jan", "Feb"]);
    }

    #[test]
    fn levels_start_at_zero_for_counts_and_skip_gaps() {
        let l = layout(0, &[2.0, 4.0, f64::NAN, 8.0]);
        assert_eq!((l.min, l.max), (0.0, 8.0));
        assert!((l.days[1].level - 0.5).abs() < 1e-9);
        assert_eq!(l.days[2].value, None);
        let neg = layout(0, &[-4.0, 4.0]);
        assert_eq!(neg.min, -4.0);
        assert_eq!(layout(0, &[]).days.len(), 0);
    }

    #[test]
    fn keys_step_days_and_weeks() {
        assert_eq!(key("ArrowRight", None, 30), Some(Some(0)));
        assert_eq!(key("ArrowRight", Some(3), 30), Some(Some(10)));
        assert_eq!(key("ArrowLeft", Some(3), 30), Some(Some(0)));
        assert_eq!(key("ArrowDown", Some(29), 30), Some(Some(29)));
        assert_eq!(key("End", Some(0), 30), Some(Some(29)));
        assert_eq!(key("Escape", Some(4), 30), Some(None));
        assert_eq!(key("Escape", None, 30), None);
        assert_eq!(key("x", Some(0), 30), None);
    }
}
