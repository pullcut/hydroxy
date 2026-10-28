//! Pure geometry: [`Timeline`] turns dates into pixels and back, plus the
//! header ticks and dependency connectors.

use super::{BAR_HEIGHT, GanttScale, GanttTask, ROW_HEIGHT, short_date};
use crate::date::Date;

/// A column mark in the header.
#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    /// Left edge in px.
    pub x: f64,
    /// Width in px.
    pub width: f64,
    /// Day number (day scale) or the week's first date (week scale).
    pub label: String,
    /// The first day or week of a month. Drawn with a heavier line.
    pub major: bool,
    /// A Saturday or Sunday, in day scale.
    pub weekend: bool,
}

/// A month in the header's upper row.
#[derive(Debug, Clone, PartialEq)]
pub struct MonthSpan {
    /// Left edge in px.
    pub x: f64,
    /// Width in px.
    pub width: f64,
    /// `"October 2026"`-style text, short enough to fit: `"Oct 2026"`.
    pub label: String,
}

/// A box in px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Left.
    pub x: f64,
    /// Top.
    pub y: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

/// A line from one bar to the bar that depends on it.
#[derive(Debug, Clone, PartialEq)]
pub struct Connector {
    /// The task it leaves.
    pub from: String,
    /// The task it enters.
    pub to: String,
    /// SVG path data for the line.
    pub line: String,
    /// SVG path data for the arrowhead at the end.
    pub arrow: String,
}

/// Dates mapped to px along a horizontal axis of whole days.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timeline {
    /// The first day on the axis.
    pub start: Date,
    /// How many days it spans.
    pub days: i64,
    /// Width of one day in px.
    pub px_per_day: f64,
    /// The scale, which decides the tick marks.
    pub scale: GanttScale,
}

/// Empty days kept before the first and after the last task, so bars can move.
const PAD_BEFORE: i64 = 7;
const PAD_AFTER: i64 = 14;
const MIN_DAYS: i64 = 28;

impl Timeline {
    /// An axis that fits `tasks` and `today` with room to move them. In week
    /// scale it starts on a Monday and ends on a Sunday.
    #[must_use]
    pub fn fit(scale: GanttScale, tasks: &[GanttTask], today: Option<Date>) -> Self {
        let mut first = tasks.iter().map(|t| t.start).chain(today).min().unwrap_or_default();
        let mut last = tasks.iter().map(|t| t.end).chain(today).max().unwrap_or_default();
        first = first.add_days(-PAD_BEFORE);
        last = last.add_days(PAD_AFTER);
        if scale == GanttScale::Week {
            first = first.add_days(-first.days_into_week(1));
            last = last.add_days(6 - last.days_into_week(1));
        }
        let days = (last.to_days() - first.to_days() + 1).max(MIN_DAYS);
        Self { start: first, days, px_per_day: scale.px_per_day(), scale }
    }

    /// The last day on the axis.
    #[must_use]
    pub fn end(&self) -> Date {
        self.start.add_days(self.days - 1)
    }

    /// Total width in px.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "a few hundred days")]
    pub fn width(&self) -> f64 {
        self.days as f64 * self.px_per_day
    }

    /// Left edge of `date`'s column.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "a few hundred days")]
    pub fn x(&self, date: Date) -> f64 {
        (date.to_days() - self.start.to_days()) as f64 * self.px_per_day
    }

    /// The box of a bar from `start` to `end` inclusive, in row `row`.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "a few hundred days and rows")]
    pub fn bar(&self, start: Date, end: Date, row: usize) -> Rect {
        let days = (end.to_days() - start.to_days() + 1).max(1);
        Rect {
            x: self.x(start),
            y: row as f64 * ROW_HEIGHT + (ROW_HEIGHT - BAR_HEIGHT) / 2.0,
            width: days as f64 * self.px_per_day,
            height: BAR_HEIGHT,
        }
    }

    /// The day under `x` px, clamped to the axis.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, reason = "a few hundred days")]
    pub fn date_at(&self, x: f64) -> Date {
        let day = (x / self.px_per_day).floor() as i64;
        self.start.add_days(day.clamp(0, self.days - 1))
    }

    /// Whole days for a pointer move of `dx` px. Snaps to the nearest day.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, reason = "a few hundred days")]
    pub fn days_for_px(&self, dx: f64) -> i64 {
        if dx.is_finite() { (dx / self.px_per_day).round() as i64 } else { 0 }
    }

    /// Limit a move of `delta` days so the bar `start..=end` stays on the axis.
    /// A resize passes the edge it moves as both ends.
    #[must_use]
    pub fn clamp_delta(&self, start: Date, end: Date, delta: i64) -> i64 {
        let lowest = self.start.to_days() - start.to_days();
        let highest = self.end().to_days() - end.to_days();
        if lowest > highest { 0 } else { delta.clamp(lowest, highest) }
    }

    /// The header's lower row: a column per day, or per week.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "a few hundred days")]
    pub fn ticks(&self) -> Vec<Tick> {
        let step_days: i64 = if self.scale == GanttScale::Week { 7 } else { 1 };
        let step = usize::try_from(step_days).unwrap_or(1);
        (0..self.days)
            .step_by(step)
            .map(|offset| {
                let date = self.start.add_days(offset);
                let width = step_days as f64 * self.px_per_day;
                let first_of_month = (0..step_days).any(|d| date.add_days(d).day() == 1);
                Tick {
                    x: offset as f64 * self.px_per_day,
                    width,
                    label: if self.scale == GanttScale::Week { short_date(date) } else { date.day().to_string() },
                    major: first_of_month,
                    weekend: self.scale == GanttScale::Day && matches!(date.weekday(), 0 | 6),
                }
            })
            .collect()
    }

    /// The header's upper row: one span per month, clipped to the axis.
    #[must_use]
    #[allow(clippy::cast_precision_loss, reason = "a few hundred days")]
    pub fn months(&self) -> Vec<MonthSpan> {
        let mut spans: Vec<MonthSpan> = Vec::new();
        let mut month_start = 0_i64;
        for offset in 0..self.days {
            let date = self.start.add_days(offset);
            let next = self.start.add_days(offset + 1);
            if offset == self.days - 1 || next.month() != date.month() {
                spans.push(MonthSpan {
                    x: month_start as f64 * self.px_per_day,
                    width: (offset - month_start + 1) as f64 * self.px_per_day,
                    label: format!("{} {}", crate::date::MONTH_SHORT[usize::from(date.month() - 1)], date.year()),
                });
                month_start = offset + 1;
            }
        }
        spans
    }

    /// Where the today line goes: the middle of the day's column. `None` when off the axis.
    #[must_use]
    pub fn today_x(&self, today: Date) -> Option<f64> {
        (today >= self.start && today <= self.end()).then(|| self.x(today) + self.px_per_day / 2.0)
    }

    /// Connectors for every dependency whose two tasks are both listed.
    /// `tasks` is in row order; bars use the tasks' own dates.
    #[must_use]
    pub fn connectors(&self, tasks: &[GanttTask]) -> Vec<Connector> {
        let bars: Vec<Rect> = tasks.iter().enumerate().map(|(row, t)| self.bar(t.start, t.end, row)).collect();
        let mut out = Vec::new();
        for (to_row, task) in tasks.iter().enumerate() {
            for dep in &task.depends_on {
                let Some(from_row) = tasks.iter().position(|t| t.id == *dep) else { continue };
                let (line, arrow) = connector_path(bars[from_row], bars[to_row]);
                out.push(Connector { from: dep.clone(), to: task.id.clone(), line, arrow });
            }
        }
        out
    }
}

/// How far a connector leaves a bar before turning, in px.
const STUB: f64 = 10.0;
const ARROW: f64 = 5.0;

/// The path from the right edge of `from` to the left edge of `to`, as an
/// elbow, and an arrowhead pointing right at its end. When `to` starts too
/// soon after `from` ends, the line runs through the gap between the rows.
#[must_use]
pub fn connector_path(from: Rect, to: Rect) -> (String, String) {
    let (x1, y1) = (from.x + from.width, from.y + from.height / 2.0);
    let (x2, y2) = (to.x, to.y + to.height / 2.0);
    let line = if x2 >= x1 + 2.0 * STUB || (y1 - y2).abs() < f64::EPSILON {
        let mid = if (y1 - y2).abs() < f64::EPSILON { x1 } else { x1 + STUB };
        format!("M{x1} {y1} H{mid} V{y2} H{x2}")
    } else {
        // Run along the row boundary between the two bars.
        let gap = if y2 > y1 { (from.y + from.height + to.y) / 2.0 } else { (to.y + to.height + from.y) / 2.0 };
        format!("M{x1} {y1} H{} V{gap} H{} V{y2} H{x2}", x1 + STUB, x2 - STUB)
    };
    let arrow = format!("M{x2} {y2} l{} {} v{} z", -ARROW, -ARROW * 0.6, ARROW * 1.2);
    (line, arrow)
}
