//! Histogram binning: equal-width bins by Sturges' rule or an explicit count.
//! Bar heights reuse [`crate::bar::layout`], so a histogram scales like a bar chart.

use crate::bar::{self, BarLayout};

/// One bin: the half-open interval `[lo, hi)` (the last bin also includes `hi`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bin {
    /// Lower edge.
    pub lo: f64,
    /// Upper edge.
    pub hi: f64,
    /// How many values fall in the bin.
    pub count: usize,
}

/// Sturges' rule: `ceil(log2(n)) + 1` bins, at least one.
#[must_use]
pub fn sturges(n: usize) -> usize {
    if n < 2 { 1 } else { (n - 1).ilog2() as usize + 2 }
}

/// Equal-width bins over the finite `values`. `count` is the bin count; `None` uses
/// [`sturges`]. Empty when there are no finite values. All-equal values get one
/// unit-wide bin.
#[must_use]
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "bin indices are small and non-negative")]
pub fn bins(values: &[f64], count: Option<usize>) -> Vec<Bin> {
    let clean: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if clean.is_empty() {
        return Vec::new();
    }
    let lo = clean.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = clean.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let n = count.unwrap_or_else(|| sturges(clean.len())).max(1);
    let (n, hi) = if hi > lo { (n, hi) } else { (1, lo + 1.0) };
    let width = (hi - lo) / n as f64;
    let mut out: Vec<Bin> = (0..n).map(|i| Bin { lo: lo + width * i as f64, hi: lo + width * (i + 1) as f64, count: 0 }).collect();
    for v in clean {
        let i = (((v - lo) / width) as usize).min(n - 1);
        out[i].count += 1;
    }
    out
}

/// Bar heights and gridlines for `bins`.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "counts are small")]
pub fn layout(bins: &[Bin], tick_count: usize) -> BarLayout {
    let counts: Vec<f64> = bins.iter().map(|b| b.count as f64).collect();
    bar::layout(&counts, None, tick_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sturges_matches_the_rule() {
        assert_eq!(sturges(0), 1);
        assert_eq!(sturges(1), 1);
        assert_eq!(sturges(2), 2);
        assert_eq!(sturges(8), 4);
        assert_eq!(sturges(9), 5);
        assert_eq!(sturges(100), 8);
    }

    #[test]
    fn values_land_in_bins_and_the_maximum_in_the_last() {
        let b = bins(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0], Some(5));
        assert_eq!(b.len(), 5);
        assert_eq!(b.iter().map(|b| b.count).collect::<Vec<_>>(), vec![2, 2, 2, 2, 2]);
        assert!((b[4].hi - 10.0).abs() < 1e-9);
        assert_eq!(b.iter().map(|b| b.count).sum::<usize>(), 10);
    }

    #[test]
    fn default_count_uses_sturges_and_ignores_non_finite() {
        let v: Vec<f64> = (0..100).map(f64::from).chain([f64::NAN, f64::INFINITY]).collect();
        assert_eq!(bins(&v, None).len(), 8);
        assert_eq!(bins(&[f64::NAN], None).len(), 0);
        assert_eq!(bins(&[], Some(4)).len(), 0);
    }

    #[test]
    fn identical_values_make_one_bin() {
        let b = bins(&[3.0, 3.0, 3.0], Some(6));
        assert_eq!(b, vec![Bin { lo: 3.0, hi: 4.0, count: 3 }]);
        assert!((layout(&b, 5).bars[0].fraction - 0.6).abs() < 0.41);
    }
}
