//! Rank charts: a bump chart (rank over time) and a slope chart (two points in time).
//! Geometry is in an SVG viewBox of `width` × `height`; text stays in the adapter.
#![allow(
    clippy::cast_precision_loss,
    clippy::needless_range_loop,
    reason = "row and column counts are small; the loop indexes two tables"
)]

use std::fmt::Write as _;

use crate::scale::{Linear, nice_domain};

/// Rank (0 = best) of every series at every column. Non-finite values rank last;
/// ties keep series order.
#[must_use]
pub fn ranks(series: &[Vec<f64>], higher_is_better: bool) -> Vec<Vec<usize>> {
    let cols = series.iter().map(Vec::len).max().unwrap_or(0);
    let mut out = vec![vec![0; cols]; series.len()];
    for c in 0..cols {
        let value = |s: usize| series[s].get(c).copied().filter(|v| v.is_finite());
        let mut order: Vec<usize> = (0..series.len()).collect();
        order.sort_by(|&a, &b| match (value(a), value(b)) {
            (Some(x), Some(y)) => {
                if higher_is_better {
                    y.total_cmp(&x)
                } else {
                    x.total_cmp(&y)
                }
            }
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        for (rank, s) in order.into_iter().enumerate() {
            out[s][c] = rank;
        }
    }
    out
}

/// One bump line.
#[derive(Debug, Clone, PartialEq)]
pub struct BumpLine {
    /// Rank at each column (0 = top).
    pub ranks: Vec<usize>,
    /// Marker centre at each column.
    pub points: Vec<(f64, f64)>,
    /// SVG `d` joining the markers with level-ended S curves.
    pub path: String,
}

/// Everything a bump chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct Bump {
    /// viewBox width.
    pub width: f64,
    /// viewBox height.
    pub height: f64,
    /// x of each column.
    pub columns: Vec<f64>,
    /// Left and right edge of the plot, where name labels sit.
    pub edges: (f64, f64),
    /// One line per series.
    pub lines: Vec<BumpLine>,
}

/// Lay out `series` (one value list per series) as a bump chart.
#[must_use]
pub fn bump(series: &[Vec<f64>], higher_is_better: bool) -> Bump {
    let (width, pad_x, top, row_h) = (640.0, 104.0, 30.0, 40.0);
    let cols = series.iter().map(Vec::len).max().unwrap_or(0);
    let r = ranks(series, higher_is_better);
    let height = top + row_h * series.len().max(1) as f64;
    let columns: Vec<f64> = match cols {
        0 => vec![],
        1 => vec![width / 2.0],
        _ => (0..cols).map(|c| pad_x + (width - 2.0 * pad_x) * c as f64 / (cols - 1) as f64).collect(),
    };
    let lines = r
        .into_iter()
        .map(|ranks| {
            let points: Vec<(f64, f64)> =
                ranks.iter().enumerate().map(|(c, &k)| (columns[c], top + row_h * (k as f64 + 0.5))).collect();
            let mut path = String::new();
            if let Some(&(x, y)) = points.first() {
                let _ = write!(path, "M{x:.2} {y:.2}");
                if points.len() == 1 {
                    path.push_str("h0");
                }
            }
            for w in points.windows(2) {
                let ((xa, ya), (xb, yb)) = (w[0], w[1]);
                let mid = f64::midpoint(xa, xb);
                let _ = write!(path, "C{mid:.2} {ya:.2} {mid:.2} {yb:.2} {xb:.2} {yb:.2}");
            }
            BumpLine { ranks, points, path }
        })
        .collect();
    Bump { width, height, columns, edges: (pad_x, width - pad_x), lines }
}

/// Push `ys` apart so neighbours are at least `gap` away, keeping order and staying
/// inside `[lo, hi]` where there is room. Returns the adjusted values in input order.
#[must_use]
pub fn spread(ys: &[f64], gap: f64, lo: f64, hi: f64) -> Vec<f64> {
    let mut order: Vec<usize> = (0..ys.len()).collect();
    order.sort_by(|&a, &b| ys[a].total_cmp(&ys[b]));
    let mut v: Vec<f64> = order.iter().map(|&i| ys[i].clamp(lo, hi)).collect();
    for i in 1..v.len() {
        v[i] = v[i].max(v[i - 1] + gap);
    }
    if let Some(&last) = v.last() {
        let over = last - hi;
        if over > 0.0 {
            let n = v.len();
            v[n - 1] -= over;
            for i in (0..n - 1).rev() {
                v[i] = v[i].min(v[i + 1] - gap);
            }
        }
    }
    let mut out = vec![0.0; ys.len()];
    for (k, &i) in order.iter().enumerate() {
        out[i] = v[k];
    }
    out
}

/// One slope line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlopeLine {
    /// y at the left and right end.
    pub y: (f64, f64),
    /// y of the labels at each end, nudged apart so they never overlap.
    pub label_y: (f64, f64),
}

/// Everything a slope chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct Slope {
    /// viewBox width.
    pub width: f64,
    /// viewBox height.
    pub height: f64,
    /// x of the left and right axis.
    pub x: (f64, f64),
    /// y range of the plot.
    pub plot: (f64, f64),
    /// One per input pair.
    pub lines: Vec<SlopeLine>,
}

/// Lay out `(from, to)` pairs on one shared value scale.
#[must_use]
pub fn slope(pairs: &[(f64, f64)]) -> Slope {
    let (width, height, pad_x, top, bottom) = (640.0, 340.0, 150.0, 34.0, 14.0);
    let finite = |v: f64| if v.is_finite() { v } else { 0.0 };
    let all = pairs.iter().flat_map(|&(a, b)| [finite(a), finite(b)]);
    let lo = all.clone().fold(f64::INFINITY, f64::min);
    let hi = all.fold(f64::NEG_INFINITY, f64::max);
    let (lo, hi) = if lo.is_finite() { nice_domain(lo, hi, 5) } else { (0.0, 1.0) };
    let scale = Linear::new((lo, hi), (height - bottom, top));
    let ys: Vec<(f64, f64)> = pairs.iter().map(|&(a, b)| (scale.map(finite(a)), scale.map(finite(b)))).collect();
    let left = spread(&ys.iter().map(|y| y.0).collect::<Vec<_>>(), 15.0, top, height - bottom);
    let right = spread(&ys.iter().map(|y| y.1).collect::<Vec<_>>(), 15.0, top, height - bottom);
    let lines = ys.iter().enumerate().map(|(i, &y)| SlopeLine { y, label_y: (left[i], right[i]) }).collect();
    Slope { width, height, x: (pad_x, width - pad_x), plot: (top, height - bottom), lines }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_each_column_independently() {
        let r = ranks(&[vec![5.0, 1.0], vec![9.0, 2.0], vec![f64::NAN, 3.0]], true);
        assert_eq!(r, [vec![1, 2], vec![0, 1], vec![2, 0]]);
        assert_eq!(ranks(&[vec![5.0], vec![9.0]], false), [vec![0], vec![1]]);
    }

    #[test]
    fn bump_points_sit_in_rank_rows() {
        let b = bump(&[vec![1.0, 2.0], vec![2.0, 1.0]], true);
        assert_eq!(b.lines[0].ranks, [1, 0]);
        assert!(b.lines[0].points[0].1 > b.lines[0].points[1].1);
        assert!(b.lines[0].path.starts_with('M') && b.lines[0].path.contains('C'));
    }

    #[test]
    fn spread_keeps_gap_and_bounds() {
        let v = spread(&[10.0, 11.0, 12.0], 10.0, 0.0, 100.0);
        assert!(v[1] - v[0] >= 10.0 - 1e-9 && v[2] - v[1] >= 10.0 - 1e-9);
        let v = spread(&[98.0, 99.0, 100.0], 10.0, 0.0, 100.0);
        assert!((v[2] - 100.0).abs() < 1e-9 && (v[0] - 80.0).abs() < 1e-9);
    }

    #[test]
    fn slope_higher_values_sit_higher() {
        let s = slope(&[(10.0, 50.0), (40.0, 20.0)]);
        assert!(s.lines[0].y.1 < s.lines[0].y.0);
        assert!(s.lines[1].y.1 > s.lines[1].y.0);
    }
}
