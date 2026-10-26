//! Box plots: quartiles, whiskers at 1.5 IQR and outliers, laid out on a shared value axis.

use crate::bar::Gridline;
use crate::scale::{Linear, ticks};

/// A named group of raw values.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    /// Shown under the box and in the table.
    pub label: String,
    /// The observations. Non-finite values are ignored.
    pub values: Vec<f64>,
}

impl Group {
    /// A named group.
    #[must_use]
    pub fn new(label: impl Into<String>, values: impl Into<Vec<f64>>) -> Self {
        Self { label: label.into(), values: values.into() }
    }
}

/// Five-number summary of one group, with outliers.
#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    /// Count of finite values.
    pub n: usize,
    /// Lower whisker: the smallest value within 1.5 IQR below the first quartile.
    pub low: f64,
    /// First quartile.
    pub q1: f64,
    /// Median.
    pub median: f64,
    /// Third quartile.
    pub q3: f64,
    /// Upper whisker: the largest value within 1.5 IQR above the third quartile.
    pub high: f64,
    /// Values beyond the whiskers, ascending.
    pub outliers: Vec<f64>,
}

/// The `p` quantile (0..=1) of ascending `sorted`, interpolating linearly between ranks.
#[must_use]
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "ranks are small")]
pub fn quantile(sorted: &[f64], p: f64) -> f64 {
    let Some(&last) = sorted.last() else { return f64::NAN };
    let rank = p.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let i = rank.floor() as usize;
    let next = sorted.get(i + 1).copied().unwrap_or(last);
    sorted[i] + (next - sorted[i]) * (rank - i as f64)
}

/// Summarise the finite `values`; `None` when there are none.
#[must_use]
pub fn summarize(values: &[f64]) -> Option<Stats> {
    let mut v: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let (q1, median, q3) = (quantile(&v, 0.25), quantile(&v, 0.5), quantile(&v, 0.75));
    let fence = 1.5 * (q3 - q1);
    let inside = |x: &&f64| **x >= q1 - fence && **x <= q3 + fence;
    let low = v.iter().find(inside).copied().unwrap_or(q1);
    let high = v.iter().rev().find(inside).copied().unwrap_or(q3);
    let outliers = v.iter().copied().filter(|x| *x < low || *x > high).collect();
    Some(Stats { n: v.len(), low, q1, median, q3, high, outliers })
}

/// One box in axis fractions (0 = bottom of the plot).
#[derive(Debug, Clone, PartialEq)]
pub struct BoxGeom {
    /// The numbers behind it.
    pub stats: Stats,
    /// Lower whisker end.
    pub low: f64,
    /// First quartile.
    pub q1: f64,
    /// Median.
    pub median: f64,
    /// Third quartile.
    pub q3: f64,
    /// Upper whisker end.
    pub high: f64,
    /// Outlier heights.
    pub outliers: Vec<f64>,
}

/// Everything a box plot needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxLayout {
    /// One per group; `None` for a group with no finite values.
    pub boxes: Vec<Option<BoxGeom>>,
    /// Gridlines from the bottom up at nice ticks.
    pub gridlines: Vec<Gridline>,
    /// Bottom of the value axis.
    pub min: f64,
    /// Top of the value axis.
    pub max: f64,
}

/// Lay out one box per group on a shared axis. The axis spans all data, outliers
/// included, and does not have to include zero.
#[must_use]
pub fn layout(groups: &[Vec<f64>], tick_count: usize) -> BoxLayout {
    let stats: Vec<Option<Stats>> = groups.iter().map(|g| summarize(g)).collect();
    let all = || stats.iter().flatten().flat_map(|s| [s.low, s.high].into_iter().chain(s.outliers.iter().copied()));
    let lo = all().fold(f64::INFINITY, f64::min);
    let hi = all().fold(f64::NEG_INFINITY, f64::max);
    let (lo, hi) = if lo.is_finite() { (lo, hi) } else { (0.0, 1.0) };
    let tick_values = ticks(lo, if hi > lo { hi } else { lo + 1.0 }, tick_count);
    let (min, max) = (tick_values.first().copied().unwrap_or(lo), tick_values.last().copied().unwrap_or(hi));
    let scale = Linear::new((min, max), (0.0, 1.0));
    let f = |v: f64| scale.map(v).clamp(0.0, 1.0);
    BoxLayout {
        boxes: stats
            .into_iter()
            .map(|s| {
                s.map(|s| BoxGeom {
                    low: f(s.low),
                    q1: f(s.q1),
                    median: f(s.median),
                    q3: f(s.q3),
                    high: f(s.high),
                    outliers: s.outliers.iter().map(|&o| f(o)).collect(),
                    stats: s,
                })
            })
            .collect(),
        gridlines: tick_values.iter().map(|&value| Gridline { value, fraction: scale.map(value) }).collect(),
        min,
        max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantiles_interpolate() {
        let v = [1.0, 2.0, 3.0, 4.0];
        assert!((quantile(&v, 0.5) - 2.5).abs() < 1e-12);
        assert!((quantile(&v, 0.25) - 1.75).abs() < 1e-12);
        assert!((quantile(&v, 1.0) - 4.0).abs() < 1e-12);
        assert!(quantile(&[], 0.5).is_nan());
    }

    #[test]
    fn whiskers_stop_at_1_5_iqr_and_the_rest_are_outliers() {
        // q1 = 3, q3 = 7 (iqr 4), fences -3 and 13.
        let s = summarize(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 40.0, f64::NAN]).unwrap_or_else(|| unreachable!());
        assert_eq!(s.n, 10);
        assert!((s.q1 - 3.25).abs() < 1e-9 && (s.q3 - 7.75).abs() < 1e-9);
        assert!((s.low - 1.0).abs() < 1e-12 && (s.high - 9.0).abs() < 1e-12);
        assert_eq!(s.outliers, vec![40.0]);
    }

    #[test]
    fn low_outliers_and_empty_groups() {
        let s = summarize(&[-50.0, 10.0, 11.0, 12.0, 13.0, 14.0]).unwrap_or_else(|| unreachable!());
        assert_eq!(s.outliers, vec![-50.0]);
        assert!((s.low - 10.0).abs() < 1e-12);
        assert!(summarize(&[f64::NAN]).is_none());
    }

    #[test]
    fn layout_shares_one_axis_and_keeps_order() {
        let l = layout(&[vec![10.0, 20.0, 30.0], vec![], vec![50.0, 60.0, 70.0, 200.0]], 5);
        assert!(l.boxes[1].is_none());
        let (a, c) = (l.boxes[0].as_ref().unwrap_or_else(|| unreachable!()), l.boxes[2].as_ref().unwrap_or_else(|| unreachable!()));
        assert!(a.q1 < a.median && a.median < a.q3 && a.q3 < c.q1);
        assert_eq!(c.outliers.len(), 1);
        assert!(l.min <= 10.0 && l.max >= 200.0);
    }
}
