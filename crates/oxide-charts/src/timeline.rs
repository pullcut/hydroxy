//! Timeline geometry: spans and events packed into lanes on a date axis with nice
//! calendar ticks. Times are milliseconds since the Unix epoch (UTC).

/// Milliseconds in an hour.
pub const HOUR: f64 = 3_600_000.0;
/// Milliseconds in a day.
pub const DAY: f64 = 24.0 * HOUR;

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// Days since the epoch for a civil date (month 1..=12).
#[must_use]
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The civil date `(year, month 1..=12, day)` for days since the epoch.
#[must_use]
pub fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// Epoch milliseconds for midnight UTC on a civil date.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "dates fit f64 exactly")]
pub fn ms_from_civil(y: i64, m: i64, d: i64) -> f64 {
    days_from_civil(y, m, d) as f64 * DAY
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, reason = "dates are well inside i64 and f64")]
fn split(ms: f64) -> (i64, i64, i64, i64) {
    let days = (ms / DAY).floor() as i64;
    let (y, m, d) = civil_from_days(days);
    let hour = ((ms - days as f64 * DAY) / HOUR).floor() as i64;
    (y, m, d, hour)
}

/// A span or event in a lane. An event has `end == start`.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// Shown on the item, in tooltips and in the table.
    pub label: String,
    /// Start time in epoch milliseconds.
    pub start: f64,
    /// End time in epoch milliseconds.
    pub end: f64,
}

impl Item {
    /// A time range.
    #[must_use]
    pub fn span(label: impl Into<String>, start: f64, end: f64) -> Self {
        Self { label: label.into(), start, end }
    }

    /// A single moment.
    #[must_use]
    pub fn event(label: impl Into<String>, at: f64) -> Self {
        Self::span(label, at, at)
    }

    /// Whether this is a single moment.
    #[must_use]
    pub fn is_event(&self) -> bool {
        self.start.to_bits() == self.end.to_bits()
    }
}

/// A named row group of items.
#[derive(Debug, Clone, PartialEq)]
pub struct Lane {
    /// Lane name, shown at the left.
    pub name: String,
    /// The items in this lane.
    pub items: Vec<Item>,
}

impl Lane {
    /// A lane of items.
    #[must_use]
    pub fn new(name: impl Into<String>, items: impl Into<Vec<Item>>) -> Self {
        Self { name: name.into(), items: items.into() }
    }
}

/// `layout` for lanes: items are flattened in lane order, so `Placed::index` counts
/// through all items lane by lane.
#[must_use]
pub fn layout_lanes(lanes: &[Lane], tick_target: usize) -> TimelineLayout {
    let flat: Vec<(usize, f64, f64)> = lanes.iter().enumerate().flat_map(|(l, lane)| lane.items.iter().map(move |i| (l, i.start, i.end))).collect();
    layout(&flat, lanes.len(), tick_target)
}

/// A date like "5 Oct 2026", with the time ("5 Oct 2026 14:00") when it is not midnight.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "month is 1..=12")]
pub fn format_date(ms: f64) -> String {
    if !ms.is_finite() {
        return "–".to_string();
    }
    let (y, m, d, hour) = split(ms);
    let minutes = ((ms.rem_euclid(DAY)) / 60_000.0).floor() % 60.0;
    let date = format!("{d} {} {y}", MONTHS[(m - 1) as usize]);
    if hour == 0 && minutes == 0.0 { date } else { format!("{date} {hour:02}:{minutes:02.0}") }
}

/// A labelled tick on the date axis.
#[derive(Debug, Clone, PartialEq)]
pub struct DateTick {
    /// Time of the tick.
    pub time: f64,
    /// Formatted label.
    pub label: String,
}

#[derive(Clone, Copy)]
enum Step {
    /// Fixed length in ms, aligned to multiples of itself plus an offset.
    Fixed(f64, f64),
    /// Calendar months, aligned so the month index is a multiple of this.
    Months(i64),
}

// Epoch day 0 was a Thursday; Mondays sit 4 days later.
const MONDAY: f64 = 4.0 * DAY;

fn steps() -> Vec<(f64, Step)> {
    let mut out = vec![
        (HOUR, Step::Fixed(HOUR, 0.0)),
        (3.0 * HOUR, Step::Fixed(3.0 * HOUR, 0.0)),
        (6.0 * HOUR, Step::Fixed(6.0 * HOUR, 0.0)),
        (12.0 * HOUR, Step::Fixed(12.0 * HOUR, 0.0)),
        (DAY, Step::Fixed(DAY, 0.0)),
        (2.0 * DAY, Step::Fixed(2.0 * DAY, 0.0)),
        (7.0 * DAY, Step::Fixed(7.0 * DAY, MONDAY)),
        (14.0 * DAY, Step::Fixed(14.0 * DAY, MONDAY)),
    ];
    for n in [1, 2, 3, 6, 12, 24, 60, 120, 240, 600, 1200] {
        #[allow(clippy::cast_precision_loss, reason = "tiny")]
        out.push((n as f64 * 30.4375 * DAY, Step::Months(n)));
    }
    out
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "month is 1..=12")]
fn label(step: Step, ms: f64) -> String {
    let (y, m, d, hour) = split(ms);
    let month = MONTHS[(m - 1) as usize];
    match step {
        Step::Fixed(len, _) if len < DAY => {
            if hour == 0 { format!("{d} {month}") } else { format!("{hour:02}:00") }
        }
        Step::Fixed(..) => format!("{d} {month}"),
        Step::Months(n) if n >= 12 => y.to_string(),
        Step::Months(_) if m == 1 => format!("{month} {y}"),
        Step::Months(_) => month.to_string(),
    }
}

/// Calendar-aligned ticks within `[start, end]`: hours, days, weeks (Mondays), months or
/// years, whichever gives no more than about `target` ticks. Empty for an empty range.
#[must_use]
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, reason = "dates and tick counts are small")]
pub fn date_ticks(start: f64, end: f64, target: usize) -> Vec<DateTick> {
    if end.partial_cmp(&start) != Some(std::cmp::Ordering::Greater) || !start.is_finite() || !end.is_finite() {
        return Vec::new();
    }
    let span = end - start;
    let all = steps();
    let &(_, step) = all.iter().find(|(len, _)| span / len <= target.max(1) as f64).unwrap_or_else(|| &all[all.len() - 1]);
    let mut out = Vec::new();
    match step {
        Step::Fixed(len, offset) => {
            let mut t = ((start - offset) / len).ceil() * len + offset;
            while t <= end {
                out.push(DateTick { time: t, label: label(step, t) });
                t += len;
            }
        }
        Step::Months(n) => {
            let (y, m, ..) = split(start);
            let mut index = y * 12 + (m - 1);
            index += (n - index.rem_euclid(n)) % n;
            loop {
                let t = ms_from_civil(index.div_euclid(12), index.rem_euclid(12) + 1, 1);
                if t > end {
                    break;
                }
                if t >= start {
                    out.push(DateTick { time: t, label: label(step, t) });
                }
                index += n;
            }
        }
    }
    out
}

/// Lane assignment and horizontal placement of one item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    /// Index into the input items.
    pub index: usize,
    /// Lane (row group) of the item.
    pub lane: usize,
    /// Sub-row within the lane, 0 first; overlapping items stack.
    pub row: usize,
    /// Left edge as a fraction of the axis.
    pub left: f64,
    /// Width as a fraction of the axis; 0 for an event.
    pub width: f64,
}

/// A tick placed on the axis.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedTick {
    /// Position as a fraction of the axis.
    pub fraction: f64,
    /// Formatted label.
    pub label: String,
}

/// Everything a timeline chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineLayout {
    /// One per input item, in input order.
    pub items: Vec<Placed>,
    /// Number of sub-rows each lane needs (at least 1).
    pub lane_rows: Vec<usize>,
    /// Date ticks.
    pub ticks: Vec<PlacedTick>,
    /// Start of the axis.
    pub start: f64,
    /// End of the axis.
    pub end: f64,
}

/// Place `items` (`lane`, `start`, `end`; an event has `end == start`) in `lane_count`
/// lanes. The axis covers all items with a little padding. Items that overlap in time
/// within a lane stack into sub-rows. Items with non-finite times are placed at zero width
/// on the left edge; lanes beyond `lane_count` are clamped to the last.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "tiny")]
pub fn layout(items: &[(usize, f64, f64)], lane_count: usize, tick_target: usize) -> TimelineLayout {
    let lane_count = lane_count.max(1);
    let good = || items.iter().filter(|(_, s, e)| s.is_finite() && e.is_finite());
    let lo = good().map(|&(_, s, e)| s.min(e)).fold(f64::INFINITY, f64::min);
    let hi = good().map(|&(_, s, e)| s.max(e)).fold(f64::NEG_INFINITY, f64::max);
    let (lo, hi) = if lo.is_finite() { (lo, hi) } else { (0.0, DAY) };
    let pad = if hi > lo { (hi - lo) * 0.03 } else { DAY };
    let (start, end) = (lo - pad, hi + pad);
    let span = end - start;
    let gap = span * 0.01;

    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by(|&a, &b| items[a].1.total_cmp(&items[b].1));
    let mut row_ends: Vec<Vec<f64>> = vec![Vec::new(); lane_count];
    let mut placed = vec![Placed { index: 0, lane: 0, row: 0, left: 0.0, width: 0.0 }; items.len()];
    for i in order {
        let (lane, s, e) = items[i];
        let lane = lane.min(lane_count - 1);
        let (s, e) = if s.is_finite() && e.is_finite() { (s.min(e), s.max(e)) } else { (start, start) };
        let ends = &mut row_ends[lane];
        let row = ends.iter().position(|&end| s >= end + gap).unwrap_or(ends.len());
        if row == ends.len() {
            ends.push(e);
        } else {
            ends[row] = e;
        }
        placed[i] = Placed { index: i, lane, row, left: (s - start) / span, width: (e - s) / span };
    }
    TimelineLayout {
        items: placed,
        lane_rows: row_ends.iter().map(|r| r.len().max(1)).collect(),
        ticks: date_ticks(start, end, tick_target).into_iter().map(|t| PlacedTick { fraction: (t.time - start) / span, label: t.label }).collect(),
        start,
        end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_dates_round_trip() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(days_from_civil(2024, 2, 29)), (2024, 2, 29));
        assert_eq!(civil_from_days(days_from_civil(1969, 12, 31)), (1969, 12, 31));
        assert_eq!(days_from_civil(2000, 3, 1) - days_from_civil(2000, 2, 28), 2);
    }

    #[test]
    fn month_ticks_land_on_the_first_of_the_month() {
        let t = date_ticks(ms_from_civil(2026, 1, 20), ms_from_civil(2026, 7, 10), 8);
        let labels: Vec<_> = t.iter().map(|t| t.label.as_str()).collect();
        assert_eq!(labels, ["Feb", "Mar", "Apr", "May", "Jun", "Jul"]);
        assert!((t[0].time - ms_from_civil(2026, 2, 1)).abs() < 1.0);
    }

    #[test]
    fn day_ticks_and_week_ticks_are_aligned() {
        let t = date_ticks(ms_from_civil(2026, 10, 1), ms_from_civil(2026, 10, 8), 10);
        assert_eq!(t.len(), 8);
        assert_eq!(t[0].label, "1 Oct");
        // Weekly ticks fall on Mondays: 2026-10-05 is a Monday.
        let w = date_ticks(ms_from_civil(2026, 9, 1), ms_from_civil(2026, 12, 1), 14);
        assert!(w.iter().any(|t| (t.time - ms_from_civil(2026, 10, 5)).abs() < 1.0), "{w:?}");
    }

    #[test]
    fn year_ticks_and_january_labels() {
        let t = date_ticks(ms_from_civil(2020, 6, 1), ms_from_civil(2026, 6, 1), 6);
        assert_eq!(t.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(), ["2021", "2022", "2023", "2024", "2025", "2026"]);
        let t = date_ticks(ms_from_civil(2026, 11, 1), ms_from_civil(2027, 3, 1), 6);
        assert!(t.iter().any(|t| t.label == "Jan 2027"));
    }

    #[test]
    fn hour_ticks_show_clock_times() {
        let t0 = ms_from_civil(2026, 10, 5) + 8.0 * HOUR;
        let t = date_ticks(t0, t0 + 6.0 * HOUR, 8);
        assert_eq!(t.first().map(|t| t.label.as_str()), Some("08:00"));
        assert_eq!(date_ticks(5.0, 5.0, 5).len(), 0);
    }

    #[test]
    fn items_pack_into_sub_rows_per_lane() {
        let d = |n: f64| n * DAY;
        let l = layout(&[(0, d(0.0), d(10.0)), (0, d(5.0), d(8.0)), (0, d(20.0), d(25.0)), (1, d(2.0), d(2.0))], 2, 6);
        assert_eq!(l.items[0].row, 0);
        assert_eq!(l.items[1].row, 1, "overlap stacks");
        assert_eq!(l.items[2].row, 0, "a later item reuses the first row");
        assert_eq!(l.lane_rows, vec![2, 1]);
        assert!(l.items[3].width == 0.0 && l.items[0].left > 0.0);
        assert!(l.items[2].left + l.items[2].width < 1.0);
        assert_ne!(l.ticks.len(), 0);
    }

    #[test]
    fn lanes_flatten_in_order_and_dates_format() {
        let lanes = [Lane::new("A", [Item::span("x", 0.0, DAY)]), Lane::new("B", [Item::event("y", 2.0 * DAY)])];
        let l = layout_lanes(&lanes, 5);
        assert_eq!((l.items[0].lane, l.items[1].lane), (0, 1));
        assert!(lanes[1].items[0].is_event() && !lanes[0].items[0].is_event());
        assert_eq!(format_date(ms_from_civil(2026, 10, 5)), "5 Oct 2026");
        assert_eq!(format_date(ms_from_civil(2026, 10, 5) + 14.5 * HOUR), "5 Oct 2026 14:30");
    }

    #[test]
    fn empty_and_single_event_still_get_an_axis() {
        assert_eq!(layout(&[], 1, 5).items.len(), 0);
        let l = layout(&[(0, 1000.0 * DAY, 1000.0 * DAY)], 1, 5);
        assert!(l.end > l.start && (l.items[0].left - 0.5).abs() < 1e-9);
    }
}
