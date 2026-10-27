//! Waterfall and Pareto charts: bars that build on each other.

use std::fmt::Write as _;

use crate::annotate::Annotation;
use crate::plot::{ValueAxis, band_centre};
use crate::scale::ScaleKind;

/// One waterfall step: a change, or a bar showing the running total so far.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    /// Shown on the axis, tooltip and table.
    pub label: String,
    /// The change. Ignored for totals. Non-finite values count as zero.
    pub value: f64,
    /// Draw the running total from zero instead of a change.
    pub total: bool,
}

impl Step {
    /// A change from the running total.
    #[must_use]
    pub fn change(label: impl Into<String>, value: f64) -> Self {
        Self { label: label.into(), value, total: false }
    }

    /// A bar showing the running total so far.
    #[must_use]
    pub fn total(label: impl Into<String>) -> Self {
        Self { label: label.into(), value: 0.0, total: true }
    }
}

/// How a step is coloured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepKind {
    /// The running total went up (or stayed level).
    Increase,
    /// The running total went down.
    Decrease,
    /// A total bar.
    Total,
}

impl StepKind {
    /// The `data-kind` value.
    #[must_use]
    pub const fn attr(self) -> &'static str {
        match self {
            StepKind::Increase => "increase",
            StepKind::Decrease => "decrease",
            StepKind::Total => "total",
        }
    }
}

/// One drawn step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterfallBar {
    /// Colouring.
    pub kind: StepKind,
    /// Running total before the step, in data units (zero for totals).
    pub start: f64,
    /// Running total after the step, in data units.
    pub end: f64,
    /// `start` as a plot fraction.
    pub from: f64,
    /// `end` as a plot fraction.
    pub to: f64,
}

/// Everything a waterfall chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct WaterfallLayout {
    /// One per step.
    pub bars: Vec<WaterfallBar>,
    /// The value axis, always including zero.
    pub axis: ValueAxis,
}

/// Lay out the running totals of `steps`.
#[must_use]
pub fn waterfall(steps: &[Step], annotations: &[Annotation]) -> WaterfallLayout {
    let mut cursor = 0.0;
    let raw: Vec<(StepKind, f64, f64)> = steps
        .iter()
        .map(|s| {
            if s.total {
                return (StepKind::Total, 0.0, cursor);
            }
            let v = if s.value.is_finite() { s.value } else { 0.0 };
            let start = cursor;
            cursor += v;
            (if v < 0.0 { StepKind::Decrease } else { StepKind::Increase }, start, cursor)
        })
        .collect();
    let axis = ValueAxis::new(ScaleKind::Linear, raw.iter().flat_map(|r| [r.1, r.2]), annotations, true);
    let bars = raw.into_iter().map(|(kind, start, end)| WaterfallBar { kind, start, end, from: axis.at(start), to: axis.at(end) }).collect();
    WaterfallLayout { bars, axis }
}

/// Everything a Pareto chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct ParetoLayout {
    /// Input indices, largest value first.
    pub order: Vec<usize>,
    /// Bar heights in sorted order, 0..=1.
    pub bars: Vec<f64>,
    /// Running share of the total in sorted order, 0..=1.
    pub cumulative: Vec<f64>,
    /// SVG `d` of the cumulative line through the bar centres, in the 100×100 box.
    pub line: String,
    /// The bars' value axis, from zero.
    pub axis: ValueAxis,
}

/// Sort `values` largest first and accumulate their share. Negative or non-finite values count as zero.
#[must_use]
pub fn pareto(values: &[f64], annotations: &[Annotation]) -> ParetoLayout {
    let clean = |v: f64| if v.is_finite() { v.max(0.0) } else { 0.0 };
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|&a, &b| clean(values[b]).total_cmp(&clean(values[a])));
    let total: f64 = values.iter().map(|&v| clean(v)).sum();
    let axis = ValueAxis::new(ScaleKind::Linear, values.iter().map(|&v| clean(v)), annotations, true);
    let mut running = 0.0;
    let cumulative: Vec<f64> = order
        .iter()
        .map(|&i| {
            running += clean(values[i]);
            if total > 0.0 { running / total } else { 0.0 }
        })
        .collect();
    let mut line = String::new();
    for (i, c) in cumulative.iter().enumerate() {
        let _ = write!(line, "{}{:.3} {:.3}", if i == 0 { 'M' } else { 'L' }, band_centre(i, order.len()) * 100.0, (1.0 - c) * 100.0);
    }
    let bars = order.iter().map(|&i| axis.at(clean(values[i]))).collect();
    ParetoLayout { order, bars, cumulative, line, axis }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_run_and_totals_reset_to_zero() {
        let steps = [Step::change("Start", 100.0), Step::change("Refunds", -30.0), Step::change("Fees", f64::NAN), Step::total("Net")];
        let l = waterfall(&steps, &[]);
        assert_eq!(l.bars[1].kind, StepKind::Decrease);
        assert_eq!((l.bars[1].start, l.bars[1].end), (100.0, 70.0));
        assert_eq!(l.bars[2].kind, StepKind::Increase, "NaN counts as a level step");
        assert_eq!((l.bars[3].kind, l.bars[3].start, l.bars[3].end), (StepKind::Total, 0.0, 70.0));
        assert_eq!(l.bars[3].from, 0.0);
    }

    #[test]
    fn negative_running_totals_widen_the_axis() {
        let l = waterfall(&[Step::change("Loss", -40.0)], &[]);
        assert!(l.axis.scale.domain.0 <= -40.0);
        assert!(l.bars[0].to < l.bars[0].from);
    }

    #[test]
    fn pareto_sorts_and_accumulates() {
        let l = pareto(&[10.0, 50.0, 40.0, -5.0], &[]);
        assert_eq!(l.order, vec![1, 2, 0, 3]);
        assert_eq!(l.cumulative, vec![0.5, 0.9, 1.0, 1.0]);
        assert!(l.line.starts_with("M12.500 50.000L37.500 10.000"));
        assert_eq!(pareto(&[0.0], &[]).cumulative, vec![0.0]);
    }
}
