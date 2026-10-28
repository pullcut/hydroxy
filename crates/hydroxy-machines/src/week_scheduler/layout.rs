//! Lane packing and pixel geometry for the week grid.

use std::cmp::Reverse;

use super::{GridPoint, HOUR_HEIGHT, WeekEvent};
use crate::date::Date;

/// Lanes for overlapping spans. Returns `(lane, lanes)` for each span, in input
/// order: spans that overlap, directly or through others, form a cluster that
/// splits its width into `lanes`; each span takes the leftmost lane free at its
/// start. Earlier starts go first, and longer spans first on a tie.
#[must_use]
pub fn pack(spans: &[(u16, u16)]) -> Vec<(usize, usize)> {
    let mut order: Vec<usize> = (0..spans.len()).collect();
    order.sort_by_key(|&i| (spans[i].0, Reverse(spans[i].1)));
    let mut out = vec![(0, 1); spans.len()];
    let mut cluster: Vec<usize> = Vec::new();
    let mut lane_ends: Vec<u16> = Vec::new();
    let mut cluster_end = 0;
    let finish = |cluster: &mut Vec<usize>, lanes: usize, out: &mut [(usize, usize)]| {
        for &i in &*cluster {
            out[i].1 = lanes;
        }
        cluster.clear();
    };
    for i in order {
        let (start, end) = spans[i];
        let end = end.max(start + 1);
        if !cluster.is_empty() && start >= cluster_end {
            finish(&mut cluster, lane_ends.len(), &mut out);
            lane_ends.clear();
        }
        let lane = if let Some(free) = lane_ends.iter().position(|&e| e <= start) {
            free
        } else {
            lane_ends.push(0);
            lane_ends.len() - 1
        };
        lane_ends[lane] = end;
        out[i].0 = lane;
        cluster_end = if cluster.is_empty() { end } else { cluster_end.max(end) };
        cluster.push(i);
    }
    finish(&mut cluster, lane_ends.len(), &mut out);
    out
}

/// A timed event placed on the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    /// Index into the events given to [`layout`].
    pub index: usize,
    /// Index into the visible days.
    pub day: usize,
    /// Its lane within the day.
    pub lane: usize,
    /// Lanes its cluster splits into.
    pub lanes: usize,
}

/// Place every timed event that falls on a visible day.
#[must_use]
pub fn layout(events: &[WeekEvent], days: &[Date]) -> Vec<Placed> {
    let mut out = Vec::new();
    for (day, date) in days.iter().enumerate() {
        let on: Vec<usize> = (0..events.len()).filter(|&i| !events[i].all_day && events[i].date == *date).collect();
        let spans: Vec<(u16, u16)> = on.iter().map(|&i| (events[i].start, events[i].end)).collect();
        for (&index, (lane, lanes)) in on.iter().zip(pack(&spans)) {
            out.push(Placed { index, day, lane, lanes });
        }
    }
    out
}

/// Rows for the all-day events on each visible day: `lane` is the row, `lanes`
/// the number of all-day events that day. Order follows `events`.
#[must_use]
pub fn all_day_rows(events: &[WeekEvent], days: &[Date]) -> Vec<Placed> {
    let mut out = Vec::new();
    for (day, date) in days.iter().enumerate() {
        let on: Vec<usize> = (0..events.len()).filter(|&i| events[i].all_day && events[i].date == *date).collect();
        let lanes = on.len();
        out.extend(on.into_iter().enumerate().map(|(lane, index)| Placed { index, day, lane, lanes }));
    }
    out
}

/// Px from the top of the grid for a minute of the day.
#[must_use]
pub fn minute_y(minute: u16) -> f64 {
    f64::from(minute) * HOUR_HEIGHT / 60.0
}

/// The grid point under `(x, y)` px, measured from the grid's top-left, for a grid
/// `width` px wide showing `days` days.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "clamped to the day range first"
)]
pub fn point_at(x: f64, y: f64, width: f64, days: usize) -> GridPoint {
    let n = days.max(1);
    let column = if width > 0.0 { width / n as f64 } else { 1.0 };
    let day = (x / column).floor().clamp(0.0, (n - 1) as f64) as usize;
    let minute = (y * 60.0 / HOUR_HEIGHT).round().clamp(-1.0e6, 1.0e6) as i32;
    GridPoint { day, minute }
}

/// `left`, `width`, `top` and `height` for a block in lane `lane` of `lanes`, on
/// visible day `day` of `days`, from `start` to `end`.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "a handful of days and lanes")]
pub fn block_style(day: usize, lane: usize, lanes: usize, days: usize, start: u16, end: u16) -> String {
    let (n, lanes) = (days.max(1) as f64, lanes.max(1) as f64);
    let left = (day as f64 + lane as f64 / lanes) / n * 100.0;
    let width = 100.0 / n / lanes;
    let (top, height) = (minute_y(start), minute_y(end.max(start)) - minute_y(start));
    format!("left: {left:.4}%; width: {width:.4}%; top: {top}px; height: {height}px;")
}

/// `left` and `width` for visible day `day` of `days`.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "a handful of days")]
pub fn column_style(day: usize, days: usize) -> String {
    let n = days.max(1) as f64;
    format!("left: {:.4}%; width: {:.4}%;", day as f64 / n * 100.0, 100.0 / n)
}

/// `left`, `width` and `top` for an all-day event in row `row` of day `day`. The
/// row height comes from the stylesheet's `--ws-chip`.
#[must_use]
pub fn all_day_style(day: usize, row: usize, days: usize) -> String {
    format!("{} top: calc({row} * var(--ws-chip));", column_style(day, days))
}
