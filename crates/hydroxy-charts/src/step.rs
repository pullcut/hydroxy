//! Step charts: values that hold until they jump, such as prices, rates and headcounts.

use std::fmt::Write as _;

use crate::annotate::Annotation;
use crate::line::{self, Series};
use crate::plot::{self, ValueAxis};
use crate::scale::ScaleKind;

/// Where the jump between two values happens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum StepMode {
    /// A value holds from its point until the next one (a rate set on a date).
    #[default]
    After,
    /// A value holds from the previous point up to its own (a total at period end).
    Before,
    /// The jump falls halfway between points.
    Middle,
}

impl StepMode {
    /// The `data-variant` value.
    #[must_use]
    pub const fn attr(self) -> &'static str {
        match self {
            StepMode::After => "after",
            StepMode::Before => "before",
            StepMode::Middle => "middle",
        }
    }
}

/// A labelled moment on the x axis, such as "Rate cut".
#[derive(Debug, Clone, PartialEq)]
pub struct StepEvent {
    /// Point index the event sits at.
    pub at: usize,
    /// Short label drawn above the plot.
    pub label: String,
}

impl StepEvent {
    /// An event at point `at`.
    #[must_use]
    pub fn new(at: usize, label: impl Into<String>) -> Self {
        Self { at, label: label.into() }
    }
}

/// SVG `d` through `points` (drawing coordinates, ascending x) as steps.
#[must_use]
pub fn step_path(points: &[(f64, f64)], mode: StepMode) -> String {
    let mut d = String::new();
    let Some(&(x0, y0)) = points.first() else { return d };
    let _ = write!(d, "M{x0:.3} {y0:.3}");
    if points.len() == 1 {
        d.push_str("h0");
    }
    for w in points.windows(2) {
        let ((xa, ya), (xb, yb)) = (w[0], w[1]);
        let _ = match mode {
            StepMode::After => write!(d, "L{xb:.3} {ya:.3}L{xb:.3} {yb:.3}"),
            StepMode::Before => write!(d, "L{xa:.3} {yb:.3}L{xb:.3} {yb:.3}"),
            StepMode::Middle => {
                let m = f64::midpoint(xa, xb);
                write!(d, "L{m:.3} {ya:.3}L{m:.3} {yb:.3}L{xb:.3} {yb:.3}")
            }
        };
    }
    d
}

/// Everything a step chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct StepLayout {
    /// Horizontal position of each point, 0..=1.
    pub xs: Vec<f64>,
    /// SVG `d` per series, breaking at gaps.
    pub lines: Vec<String>,
    /// Height of each series' value at each point, 0..=1; `None` for gaps.
    pub points: Vec<Vec<Option<f64>>>,
    /// The value axis.
    pub axis: ValueAxis,
}

/// Lay out `len` evenly spaced points of each series as steps.
#[must_use]
pub fn layout(
    series: &[Series],
    len: usize,
    mode: StepMode,
    kind: ScaleKind,
    annotations: &[Annotation],
) -> StepLayout {
    let xs = line::x_positions(len, None);
    let axis = ValueAxis::new(kind, series.iter().flat_map(|s| s.values.iter().take(len).copied()), annotations, false);
    let lines = series
        .iter()
        .map(|s| line::runs(&plot::drawn(&s.values, &xs, &axis)).iter().map(|r| step_path(r, mode)).collect())
        .collect();
    let points = series.iter().map(|s| plot::heights(&s.values, len, &axis)).collect();
    StepLayout { xs, lines, points, axis }
}

/// The point whose value is showing at `x` (0..=1): the last point at or before `x` for
/// [`StepMode::After`], the first at or after it for [`StepMode::Before`], else the nearest.
#[must_use]
pub fn held_index(xs: &[f64], x: f64, mode: StepMode) -> Option<usize> {
    if xs.is_empty() {
        return None;
    }
    let last = xs.len() - 1;
    Some(match mode {
        StepMode::After => xs.partition_point(|&p| p <= x).saturating_sub(1),
        StepMode::Before => xs.partition_point(|&p| p < x).min(last),
        StepMode::Middle => line::nearest_index(xs, x)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_jump_where_the_mode_says() {
        let p = [(0.0, 10.0), (50.0, 20.0)];
        assert_eq!(step_path(&p, StepMode::After), "M0.000 10.000L50.000 10.000L50.000 20.000");
        assert_eq!(step_path(&p, StepMode::Before), "M0.000 10.000L0.000 20.000L50.000 20.000");
        assert_eq!(step_path(&p, StepMode::Middle), "M0.000 10.000L25.000 10.000L25.000 20.000L50.000 20.000");
        assert_eq!(step_path(&p[..1], StepMode::After), "M0.000 10.000h0");
        assert_eq!(step_path(&[], StepMode::After), "");
    }

    #[test]
    fn layout_breaks_at_gaps() {
        let s = Series::new("Rate", [5.0, 5.25, f64::NAN, 4.5, 4.0]);
        let l = layout(&[s], 5, StepMode::After, ScaleKind::Linear, &[]);
        assert_eq!(l.lines[0].matches('M').count(), 2);
        assert_eq!(l.points[0][2], None);
        assert_eq!(l.xs.len(), 5);
    }

    #[test]
    fn held_index_follows_the_mode() {
        let xs = [0.0, 0.5, 1.0];
        assert_eq!(held_index(&xs, 0.49, StepMode::After), Some(0));
        assert_eq!(held_index(&xs, 0.5, StepMode::After), Some(1));
        assert_eq!(held_index(&xs, 0.01, StepMode::Before), Some(1));
        assert_eq!(held_index(&xs, 0.0, StepMode::Before), Some(0));
        assert_eq!(held_index(&xs, 0.7, StepMode::Middle), Some(1));
        assert_eq!(held_index(&[], 0.5, StepMode::After), None);
    }
}
