//! Forest plots: one row per study with its estimate and confidence interval on a shared
//! axis, a line of no effect, and the pooled estimate as a diamond. Ratios read on a log axis.

use crate::scale::ticks;

/// One study's result, or the pooled result.
#[derive(Debug, Clone, PartialEq)]
pub struct Study {
    /// Study name, shown in the first column.
    pub label: String,
    /// Point estimate.
    pub estimate: f64,
    /// Lower confidence limit.
    pub low: f64,
    /// Upper confidence limit.
    pub high: f64,
    /// Weight in the pooled analysis, any units; shown as a share of the total.
    pub weight: f64,
}

impl Study {
    /// A study with its estimate, interval and weight.
    #[must_use]
    pub fn new(label: impl Into<String>, estimate: f64, (low, high): (f64, f64), weight: f64) -> Self {
        Self { label: label.into(), estimate, low, high, weight }
    }
}

/// One row, placed. Positions are fractions across the plot column (0 = left).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mark {
    /// The estimate.
    pub at: f64,
    /// Lower limit.
    pub low: f64,
    /// Upper limit.
    pub high: f64,
    /// Square side as a share of the largest (area tracks weight), 0..=1.
    pub size: f64,
    /// Weight as a share of all study weights, 0..=1.
    pub share: f64,
}

/// An axis tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisTick {
    /// The value.
    pub value: f64,
    /// Fraction across the plot column.
    pub at: f64,
}

/// Everything a forest plot needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct ForestLayout {
    /// One per study; `None` when its numbers can't be placed (missing, or not positive on a log axis).
    pub rows: Vec<Option<Mark>>,
    /// The pooled diamond.
    pub pooled: Option<Mark>,
    /// Where the line of no effect sits: 1 on a log axis, else 0.
    pub null: f64,
    /// Axis ticks.
    pub ticks: Vec<AxisTick>,
}

/// Lay out `studies` and the `pooled` result on one axis. `log` puts ratios on a log axis
/// with the line of no effect at 1; otherwise it sits at 0. The axis always shows it.
#[must_use]
pub fn layout(studies: &[Study], pooled: Option<&Study>, log: bool) -> ForestLayout {
    let ok = |s: &Study| [s.estimate, s.low, s.high].iter().all(|v| v.is_finite() && (!log || *v > 0.0));
    let t = |v: f64| if log { v.ln() } else { v };
    let null_value: f64 = if log { 1.0 } else { 0.0 };
    let (lo, hi) = studies
        .iter()
        .chain(pooled)
        .filter(|s| ok(s))
        .flat_map(|s| [s.low, s.high, s.estimate])
        .fold((null_value, null_value), |(a, b), v| (a.min(v), b.max(v)));
    let ticks = axis(lo, hi, log);
    let (d0, d1) = (ticks.first().map_or(lo, |v| t(*v)), ticks.last().map_or(hi, |v| t(*v)));
    let span = if d1 > d0 { d1 - d0 } else { 1.0 };
    let map = |v: f64| (t(v) - d0) / span;
    let weight = |s: &Study| if s.weight.is_finite() { s.weight.max(0.0) } else { 0.0 };
    let total: f64 = studies.iter().map(weight).sum();
    let heaviest = studies.iter().map(weight).fold(0.0, f64::max);
    let mark = |s: &Study| {
        ok(s).then(|| Mark {
            at: map(s.estimate),
            low: map(s.low.min(s.high)),
            high: map(s.low.max(s.high)),
            size: if heaviest > 0.0 { (weight(s) / heaviest).sqrt() } else { 1.0 },
            share: if total > 0.0 { weight(s) / total } else { 0.0 },
        })
    };
    ForestLayout {
        rows: studies.iter().map(mark).collect(),
        pooled: pooled.and_then(mark).map(|m| Mark { size: 1.0, share: 1.0, ..m }),
        null: map(null_value),
        ticks: ticks.into_iter().map(|value| AxisTick { value, at: map(value) }).collect(),
    }
}

/// Nice tick values covering `lo..=hi`. Log axes step through 1, 2, 5 per decade (or
/// whole decades when the range is wide), so ratios read as ½, 1, 2.
#[allow(clippy::cast_possible_truncation, reason = "decade exponents are small")]
fn axis(lo: f64, hi: f64, log: bool) -> Vec<f64> {
    if !log {
        return ticks(lo, if hi > lo { hi } else { lo + 1.0 }, 5);
    }
    let (a, b) = (lo.log10().floor() as i32, hi.log10().ceil() as i32);
    let steps: &[f64] = match hi / lo {
        r if r <= 12.0 => &[1.0, 2.0, 5.0],
        r if r <= 150.0 => &[1.0, 3.0],
        _ => &[1.0],
    };
    let all: Vec<f64> = (a..=b).flat_map(|p| steps.iter().map(move |s| s * 10f64.powi(p))).collect();
    // Keep the ticks inside the data plus one tick either side.
    let first = all.iter().rposition(|&v| v <= lo * (1.0 + 1e-9)).unwrap_or(0);
    let last = all.iter().position(|&v| v >= hi * (1.0 - 1e-9)).unwrap_or(all.len() - 1);
    all[first..=last].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn studies() -> Vec<Study> {
        vec![Study::new("A", 0.8, (0.6, 1.1), 30.0), Study::new("B", 0.5, (0.3, 0.9), 10.0), Study::new("C", f64::NAN, (0.1, 1.0), 5.0)]
    }

    #[test]
    fn log_axis_centres_ratios_on_one() {
        let l = layout(&studies(), Some(&Study::new("Pooled", 0.7, (0.55, 0.9), f64::NAN)), true);
        let values: Vec<f64> = l.ticks.iter().map(|t| t.value).collect();
        assert_eq!(values, vec![0.2, 0.5, 1.0, 2.0]);
        let (half, two) = (l.ticks[1].at, l.ticks[3].at);
        assert!(((l.null - half) - (two - l.null)).abs() < 1e-9, "½ and 2 sit evenly about 1");
        assert!(l.rows[2].is_none());
        assert!(l.pooled.is_some_and(|p| p.low < p.at && p.at < p.high));
    }

    #[test]
    fn squares_scale_by_area_and_shares_sum_to_one() {
        let l = layout(&studies(), None, true);
        let (a, b) = (l.rows[0].unwrap_or_else(|| unreachable!()), l.rows[1].unwrap_or_else(|| unreachable!()));
        assert!((a.size - 1.0).abs() < 1e-9 && (b.size - (1.0f64 / 3.0).sqrt()).abs() < 1e-9);
        assert!((a.share + b.share - 40.0 / 45.0).abs() < 1e-9);
    }

    #[test]
    fn linear_axis_includes_zero() {
        let l = layout(&[Study::new("A", 2.0, (1.0, 3.0), 1.0)], None, false);
        assert_eq!(l.ticks.first().map(|t| t.value), Some(0.0));
        assert!(l.null.abs() < 1e-9);
        let m = l.rows[0].unwrap_or_else(|| unreachable!());
        assert!(m.low < m.at && m.at < m.high && m.high <= 1.0);
    }

    #[test]
    fn non_positive_values_drop_on_a_log_axis() {
        let l = layout(&[Study::new("A", 0.0, (-1.0, 1.0), 1.0)], None, true);
        assert!(l.rows[0].is_none());
        assert_ne!(l.ticks.len(), 0);
    }
}
