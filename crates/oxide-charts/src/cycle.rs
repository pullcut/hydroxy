//! Cycle plot layout: the x axis is one band per period of a cycle (months of the year,
//! days of the week). Inside each band a short line runs across the cycles (years, weeks)
//! and a flat line marks that period's mean, so both the seasonal shape and each
//! period's trend read at once.

use crate::annotate::Annotation;
use crate::line::{Curve, Series};
use crate::plot::{self, ValueAxis};
use crate::scale::ScaleKind;

/// Share of each band the mini line spans, centred.
const SPAN: f64 = 0.7;

/// One period's mean line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mean {
    /// The mean value.
    pub value: f64,
    /// Left end, 0..=1 across the plot.
    pub from: f64,
    /// Right end, 0..=1.
    pub to: f64,
    /// Height, 0..=1 from the bottom.
    pub at: f64,
}

/// Everything a cycle plot needs.
#[derive(Debug, Clone, PartialEq)]
pub struct CycleLayout {
    /// The value axis.
    pub axis: ValueAxis,
    /// SVG `d` for every period's mini line, in a 100×100 box (y down).
    pub lines: String,
    /// One per period; `None` when the period has no values.
    pub means: Vec<Option<Mean>>,
    /// Each point as (x, y) fractions of the plot (y from the bottom), by period then cycle.
    pub points: Vec<Vec<Option<(f64, f64)>>>,
}

/// Where cycle `c` of `cycles` sits across period `p` of `periods`, 0..=1.
#[allow(clippy::cast_precision_loss, reason = "period counts are small")]
#[must_use]
pub fn x(p: usize, c: usize, periods: usize, cycles: usize) -> f64 {
    let within = if cycles > 1 { (1.0 - SPAN) / 2.0 + SPAN * c as f64 / (cycles - 1) as f64 } else { 0.5 };
    (p as f64 + within) / periods.max(1) as f64
}

/// Lay out `cycles` (one series per cycle, one value per period) over `periods` bands.
#[allow(clippy::cast_precision_loss, reason = "cycle counts are small")]
#[must_use]
pub fn layout(cycles: &[Series], periods: usize, kind: ScaleKind, annotations: &[Annotation]) -> CycleLayout {
    let axis = ValueAxis::new(kind, cycles.iter().flat_map(|s| s.values.iter().copied()), annotations, false);
    let n = cycles.len();
    let points: Vec<Vec<Option<(f64, f64)>>> = (0..periods)
        .map(|p| {
            cycles
                .iter()
                .enumerate()
                .map(|(c, s)| s.values.get(p).copied().filter(|v| v.is_finite()).map(|v| (x(p, c, periods, n), axis.at(v))))
                .collect()
        })
        .collect();
    let lines = points
        .iter()
        .map(|row| plot::line_path(&row.iter().map(|pt| pt.map(|(x, y)| (x * 100.0, (1.0 - y) * 100.0))).collect::<Vec<_>>(), Curve::Linear))
        .collect();
    let means = (0..periods)
        .map(|p| {
            let vals: Vec<f64> = cycles.iter().filter_map(|s| s.values.get(p).copied().filter(|v| v.is_finite())).collect();
            (!vals.is_empty()).then(|| {
                let value = vals.iter().sum::<f64>() / vals.len() as f64;
                Mean { value, from: x(p, 0, periods, n.max(2)) - 0.1 / periods as f64, to: x(p, n.max(2) - 1, periods, n.max(2)) + 0.1 / periods as f64, at: axis.at(value) }
            })
        })
        .collect();
    CycleLayout { axis, lines, means, points }
}

/// The (period, cycle) of the point nearest pointer position (`x`, `y`), fractions of the plot.
#[must_use]
pub fn nearest(layout: &CycleLayout, x: f64, y: f64) -> Option<(usize, usize)> {
    let p = plot::band_at(x, layout.points.len())?;
    layout.points[p]
        .iter()
        .enumerate()
        .filter_map(|(c, pt)| pt.map(|(px, py)| (c, (px - x).powi(2) * 4.0 + (py - y).powi(2))))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(c, _)| (p, c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mini_lines_sit_inside_their_band() {
        let s = [Series::new("2024", [10.0, 20.0]), Series::new("2025", [12.0, 26.0]), Series::new("2026", [14.0, f64::NAN])];
        let l = layout(&s, 2, ScaleKind::Linear, &[]);
        assert_eq!(l.lines.matches('M').count(), 2);
        let (first, last) = (l.points[0][0].unwrap().0, l.points[0][2].unwrap().0);
        assert!((first - 0.075).abs() < 1e-9 && (last - 0.425).abs() < 1e-9);
        assert_eq!(l.points[1][2], None);
        let m = l.means[1].unwrap();
        assert!((m.value - 23.0).abs() < 1e-9);
        assert!(m.from > 0.5 && m.to < 1.0);
    }

    #[test]
    fn nearest_picks_the_band_then_the_point() {
        let s = [Series::new("a", [1.0, 5.0]), Series::new("b", [2.0, 6.0])];
        let l = layout(&s, 2, ScaleKind::Linear, &[]);
        assert_eq!(nearest(&l, 0.9, 0.9), Some((1, 1)));
        assert_eq!(nearest(&l, 0.05, 0.0), Some((0, 0)));
        assert_eq!(x(0, 0, 4, 1), 0.125);
    }
}
