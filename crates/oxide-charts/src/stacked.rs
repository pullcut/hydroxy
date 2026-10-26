//! Stacked vertical bars. Positive values stack up from zero and negative values
//! stack down from it (a diverging stack), so every segment keeps its true length.
//! Positions are fractions of the plot area, for plain CSS layout.

use crate::bar::Gridline;
use crate::scale::{Linear, ticks};

/// One series' piece of a column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    /// Distance of the segment's lower edge from the bottom of the plot, 0..=1.
    pub bottom: f64,
    /// Segment height, 0..=1. Zero for zero or non-finite values.
    pub height: f64,
}

/// One category's stack.
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    /// One per series, in series order.
    pub segments: Vec<Segment>,
    /// Net total: positives plus negatives.
    pub total: f64,
    /// Top of the positive stack (or the zero line), where the total label sits.
    pub top: f64,
}

/// Everything a stacked bar chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct StackLayout {
    /// One per category.
    pub columns: Vec<Column>,
    /// Gridlines from the bottom up, at nice ticks.
    pub gridlines: Vec<Gridline>,
    /// Height of the zero line, 0..=1.
    pub baseline: f64,
    /// Bottom of the value axis.
    pub min: f64,
    /// Top of the value axis.
    pub max: f64,
}

/// Stack `series` (each one value per category) across `categories` columns.
/// Missing or non-finite values count as zero.
#[must_use]
pub fn layout(series: &[Vec<f64>], categories: usize, tick_count: usize) -> StackLayout {
    let value = |s: &[f64], c: usize| s.get(c).copied().filter(|v| v.is_finite()).unwrap_or(0.0);
    let sums: Vec<(f64, f64)> = (0..categories)
        .map(|c| {
            series.iter().fold((0.0, 0.0), |(pos, neg), s| {
                let v = value(s, c);
                if v >= 0.0 { (pos + v, neg) } else { (pos, neg + v) }
            })
        })
        .collect();
    let lo = sums.iter().map(|s| s.1).fold(0.0, f64::min);
    let mut hi = sums.iter().map(|s| s.0).fold(0.0, f64::max);
    if hi <= lo {
        hi = lo + 1.0;
    }
    let tick_values = ticks(lo, hi, tick_count);
    let (min, max) = (tick_values.first().copied().unwrap_or(lo), tick_values.last().copied().unwrap_or(hi));
    let scale = Linear::new((min, max), (0.0, 1.0));

    let columns = sums
        .iter()
        .enumerate()
        .map(|(c, &(pos_total, neg_total))| {
            let (mut pos, mut neg) = (0.0, 0.0);
            let segments = series
                .iter()
                .map(|s| {
                    let v = value(s, c);
                    let (from, to) = if v >= 0.0 {
                        pos += v;
                        (pos - v, pos)
                    } else {
                        neg += v;
                        (neg, neg - v)
                    };
                    Segment { bottom: scale.map(from), height: scale.map(to) - scale.map(from) }
                })
                .collect();
            Column { segments, total: pos_total + neg_total, top: scale.map(pos_total) }
        })
        .collect();

    StackLayout {
        columns,
        gridlines: tick_values.iter().map(|&value| Gridline { value, fraction: scale.map(value) }).collect(),
        baseline: scale.map(0.0),
        min,
        max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn positives_stack_upwards_from_zero() {
        let l = layout(&[vec![20.0, 10.0], vec![30.0, 0.0]], 2, 5);
        assert!(close(l.max, 50.0) && close(l.min, 0.0));
        let c = &l.columns[0];
        assert!(close(c.segments[0].bottom, 0.0) && close(c.segments[0].height, 0.4));
        assert!(close(c.segments[1].bottom, 0.4) && close(c.segments[1].height, 0.6));
        assert!(close(c.total, 50.0) && close(c.top, 1.0));
        assert!(close(l.columns[1].segments[1].height, 0.0));
    }

    #[test]
    fn negatives_stack_downwards_and_totals_are_net() {
        let l = layout(&[vec![40.0], vec![-20.0], vec![-20.0]], 1, 5);
        assert!(l.min < 0.0 && l.baseline > 0.0);
        let c = &l.columns[0];
        assert!(close(c.segments[0].bottom, l.baseline));
        assert!(close(c.segments[1].bottom + c.segments[1].height, l.baseline));
        assert!(close(c.segments[2].bottom + c.segments[2].height, c.segments[1].bottom));
        assert!(close(c.total, 0.0));
        assert!(close(c.top, c.segments[0].bottom + c.segments[0].height));
    }

    #[test]
    fn missing_and_non_finite_values_are_zero() {
        let l = layout(&[vec![f64::NAN], vec![]], 1, 5);
        assert!(l.columns[0].segments.iter().all(|s| s.height == 0.0));
        assert!(l.max > l.min);
    }
}
