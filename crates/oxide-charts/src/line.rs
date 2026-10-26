//! Line and area charts: several series over one shared x axis.
//!
//! Paths are drawn in a 100×100 box (y down) meant for an SVG with
//! `preserveAspectRatio="none"` and `vector-effect: non-scaling-stroke`, so the
//! chart fills its container without measuring. Everything else (gridlines, dots,
//! crosshair) is a fraction of the plot area, for plain CSS positioning.

use std::fmt::Write as _;

use crate::bar::Gridline;
use crate::scale::{Linear, ticks};

/// One named series: a value per x position. Non-finite values are gaps.
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    /// Shown in the legend, tooltip and table.
    pub name: String,
    /// One value per x position, in order.
    pub values: Vec<f64>,
}

impl Series {
    /// A named series.
    #[must_use]
    pub fn new(name: impl Into<String>, values: impl Into<Vec<f64>>) -> Self {
        Self { name: name.into(), values: values.into() }
    }
}

/// How points are joined.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Curve {
    /// Straight segments.
    #[default]
    Linear,
    /// Monotone cubic: smooth, but never overshoots a data point.
    Monotone,
}

/// SVG `d` through `points` (already in drawing coordinates). One point draws a
/// zero-length segment, which round line caps render as a dot.
#[must_use]
pub fn path(points: &[(f64, f64)], curve: Curve) -> String {
    let mut d = String::new();
    let Some(&(x0, y0)) = points.first() else { return d };
    let _ = write!(d, "M{x0:.3} {y0:.3}");
    if points.len() == 1 {
        d.push_str("h0");
        return d;
    }
    match curve {
        Curve::Linear => {
            for &(x, y) in &points[1..] {
                let _ = write!(d, "L{x:.3} {y:.3}");
            }
        }
        Curve::Monotone => {
            let t = tangents(points);
            for i in 1..points.len() {
                let ((xa, ya), (xb, yb)) = (points[i - 1], points[i]);
                let dx = (xb - xa) / 3.0;
                let (c1x, c1y, c2x, c2y) = (xa + dx, ya + dx * t[i - 1], xb - dx, yb - dx * t[i]);
                let _ = write!(d, "C{c1x:.3} {c1y:.3} {c2x:.3} {c2y:.3} {xb:.3} {yb:.3}");
            }
        }
    }
    d
}

/// [`path`] closed down to `baseline` (a y in drawing coordinates), for area fills.
#[must_use]
pub fn area(points: &[(f64, f64)], baseline: f64, curve: Curve) -> String {
    let (Some(&(first, _)), Some(&(last, _))) = (points.first(), points.last()) else { return String::new() };
    let mut d = path(points, curve);
    let _ = write!(d, "L{last:.3} {baseline:.3}L{first:.3} {baseline:.3}Z");
    d
}

/// Fritsch–Carlson tangents (as d3's monotoneX), with one-sided secants at the ends.
fn tangents(p: &[(f64, f64)]) -> Vec<f64> {
    let n = p.len();
    let secant = |i: usize| {
        let h = p[i + 1].0 - p[i].0;
        if h == 0.0 { 0.0 } else { (p[i + 1].1 - p[i].1) / h }
    };
    (0..n)
        .map(|i| {
            if i == 0 {
                return secant(0);
            }
            if i == n - 1 {
                return secant(n - 2);
            }
            let (s0, s1) = (secant(i - 1), secant(i));
            if s0 * s1 <= 0.0 {
                return 0.0;
            }
            let (h0, h1) = (p[i].0 - p[i - 1].0, p[i + 1].0 - p[i].0);
            let mid = (s0 * h1 + s1 * h0) / (h0 + h1);
            2.0 * s0.signum() * s0.abs().min(s1.abs()).min(0.5 * mid.abs())
        })
        .collect()
}

/// Split points into runs of consecutive present values; a `None` breaks the line.
#[must_use]
pub fn runs(points: &[Option<(f64, f64)>]) -> Vec<Vec<(f64, f64)>> {
    points
        .split(Option::is_none)
        .filter(|r| !r.is_empty())
        .map(|r| r.iter().flatten().copied().collect())
        .collect()
}

#[allow(clippy::cast_precision_loss, reason = "point counts are small")]
/// Horizontal position (0..=1) of each of `len` points. Categories are evenly spaced;
/// numeric `x` values (ascending) are placed proportionally.
#[must_use]
pub fn x_positions(len: usize, x: Option<&[f64]>) -> Vec<f64> {
    if let Some(x) = x.filter(|x| x.len() >= len && len > 1) {
        let scale = Linear::new((x[0], x[len - 1]), (0.0, 1.0));
        return x[..len].iter().map(|&v| scale.map(v)).collect();
    }
    match len {
        0 => Vec::new(),
        1 => vec![0.5],
        _ => (0..len).map(|i| i as f64 / (len - 1) as f64).collect(),
    }
}

/// One series, ready to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct LinePath {
    /// SVG `d` for the line, in the 100×100 box.
    pub line: String,
    /// SVG `d` for the area under the line, down to zero (or the nearest domain edge).
    pub area: String,
    /// Height of each point as a fraction of the plot (0 = bottom); `None` for gaps.
    pub points: Vec<Option<f64>>,
}

/// Everything a line or area chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct LineLayout {
    /// Horizontal position of each index, 0..=1.
    pub xs: Vec<f64>,
    /// One per series, in order.
    pub series: Vec<LinePath>,
    /// Gridlines from the bottom up, at nice ticks.
    pub gridlines: Vec<Gridline>,
    /// Bottom of the value axis.
    pub min: f64,
    /// Top of the value axis.
    pub max: f64,
}

/// Lay out `series` at horizontal positions `xs` (see [`x_positions`]). The value axis
/// always includes zero. Non-finite or missing values leave a gap in their line.
#[must_use]
pub fn layout(series: &[Vec<f64>], xs: &[f64], curve: Curve, tick_count: usize) -> LineLayout {
    let finite = || series.iter().flat_map(|s| s.iter().take(xs.len())).copied().filter(|v| v.is_finite());
    let lo = finite().fold(0.0, f64::min);
    let mut hi = finite().fold(0.0, f64::max);
    if hi <= lo {
        hi = lo + 1.0;
    }
    let tick_values = ticks(lo, hi, tick_count);
    let (min, max) = (tick_values.first().copied().unwrap_or(lo), tick_values.last().copied().unwrap_or(hi));
    let scale = Linear::new((min, max), (0.0, 1.0));
    let to_y = |f: f64| (1.0 - f) * 100.0;
    let baseline = to_y(scale.map(0.0).clamp(0.0, 1.0));

    let series = series
        .iter()
        .map(|values| {
            let points: Vec<Option<f64>> = (0..xs.len())
                .map(|i| values.get(i).copied().filter(|v| v.is_finite()).map(|v| scale.map(v)))
                .collect();
            let drawn: Vec<Option<(f64, f64)>> = points.iter().zip(xs).map(|(p, &x)| p.map(|f| (x * 100.0, to_y(f)))).collect();
            let runs = runs(&drawn);
            LinePath {
                line: runs.iter().map(|r| path(r, curve)).collect(),
                area: runs.iter().map(|r| area(r, baseline, curve)).collect(),
                points,
            }
        })
        .collect();

    LineLayout {
        xs: xs.to_vec(),
        series,
        gridlines: tick_values.iter().map(|&value| Gridline { value, fraction: scale.map(value) }).collect(),
        min,
        max,
    }
}

/// The index whose position in `xs` (ascending) is nearest to `target`.
/// Ties go to the left. `None` when there are no points.
#[must_use]
pub fn nearest_index(xs: &[f64], target: f64) -> Option<usize> {
    let i = xs.partition_point(|&x| x < target);
    match (i.checked_sub(1), xs.get(i)) {
        (Some(a), Some(&b)) => Some(if target - xs[a] <= b - target { a } else { i }),
        (Some(a), None) => Some(a),
        (None, Some(_)) => Some(0),
        (None, None) => None,
    }
}

/// Crosshair keyboard handling. Returns the new index (or `None` to hide) when `key`
/// is one the crosshair handles, so the caller knows to prevent the default.
/// Left/Right step (starting from the end nearest the key), Home/End jump, Escape hides.
#[must_use]
pub fn crosshair_key(key: &str, current: Option<usize>, len: usize) -> Option<Option<usize>> {
    let last = len.checked_sub(1)?;
    match key {
        "ArrowLeft" => Some(Some(current.map_or(last, |i| i.saturating_sub(1).min(last)))),
        "ArrowRight" => Some(Some(current.map_or(0, |i| (i + 1).min(last)))),
        "Home" => Some(Some(0)),
        "End" => Some(Some(last)),
        "Escape" if current.is_some() => Some(None),
        _ => None,
    }
}

/// Which of `len` axis labels to show so at most `max` appear, evenly spaced from the first.
#[must_use]
pub fn label_indices(len: usize, max: usize) -> Vec<usize> {
    if len <= max.max(1) {
        return (0..len).collect();
    }
    let step = (len - 1).div_ceil(max.max(2) - 1);
    (0..len).step_by(step).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_paths_and_closed_areas() {
        let pts = [(0.0, 10.0), (50.0, 0.0), (100.0, 5.0)];
        assert_eq!(path(&pts, Curve::Linear), "M0.000 10.000L50.000 0.000L100.000 5.000");
        assert!(area(&pts, 100.0, Curve::Linear).ends_with("L100.000 100.000L0.000 100.000Z"));
        assert_eq!(path(&[(1.0, 2.0)], Curve::Linear), "M1.000 2.000h0");
        assert_eq!(path(&[], Curve::Linear), "");
    }

    #[test]
    fn monotone_curves_flatten_at_peaks_and_stay_straight_on_lines() {
        // A peak at the middle point gets a flat tangent.
        let t = tangents(&[(0.0, 0.0), (1.0, 5.0), (2.0, 0.0)]);
        assert!(t[1].abs() < 1e-12);
        // Collinear points keep the line's slope everywhere.
        let t = tangents(&[(0.0, 0.0), (1.0, 2.0), (2.0, 4.0)]);
        assert!(t.iter().all(|s| (s - 2.0).abs() < 1e-12));
        assert!(path(&[(0.0, 0.0), (1.0, 1.0)], Curve::Monotone).contains('C'));
    }

    #[test]
    fn gaps_split_runs() {
        let r = runs(&[Some((0.0, 1.0)), None, Some((2.0, 1.0)), Some((3.0, 1.0)), None]);
        assert_eq!(r, vec![vec![(0.0, 1.0)], vec![(2.0, 1.0), (3.0, 1.0)]]);
    }

    #[test]
    fn positions_for_categories_and_numbers() {
        assert_eq!(x_positions(3, None), vec![0.0, 0.5, 1.0]);
        assert_eq!(x_positions(1, None), vec![0.5]);
        assert_eq!(x_positions(3, Some(&[0.0, 1.0, 4.0])), vec![0.0, 0.25, 1.0]);
    }

    #[test]
    fn layout_includes_zero_and_handles_negatives_and_gaps() {
        let xs = x_positions(3, None);
        let l = layout(&[vec![10.0, f64::NAN, 87.0], vec![-12.0, 4.0]], &xs, Curve::Linear, 5);
        assert!(l.min < 0.0 && l.max >= 87.0);
        assert_eq!(l.series[0].points[1], None);
        assert_eq!(l.series[1].points[2], None, "missing values are gaps");
        assert_eq!(l.series[0].line.matches('M').count(), 2, "a gap starts a new subpath");
        let zero = l.gridlines.iter().find(|g| g.value == 0.0).map(|g| g.fraction);
        assert!(zero.is_some_and(|f| f > 0.0));
        // The area closes at the zero line, not the bottom of the plot.
        let base = (1.0 - zero.unwrap_or(0.0)) * 100.0;
        assert!(l.series[0].area.contains(&format!("{base:.3}Z")));
    }

    #[test]
    fn flat_data_still_gets_an_axis() {
        let l = layout(&[vec![0.0, 0.0]], &x_positions(2, None), Curve::Linear, 5);
        assert!(l.max > l.min);
    }

    #[test]
    fn nearest_index_picks_the_closest_point() {
        let xs = [0.0, 0.25, 0.5, 1.0];
        assert_eq!(nearest_index(&xs, -1.0), Some(0));
        assert_eq!(nearest_index(&xs, 0.2), Some(1));
        assert_eq!(nearest_index(&xs, 0.125), Some(0), "ties go left");
        assert_eq!(nearest_index(&xs, 0.8), Some(3));
        assert_eq!(nearest_index(&xs, 2.0), Some(3));
        assert_eq!(nearest_index(&[], 0.5), None);
    }

    #[test]
    fn crosshair_keys_step_jump_and_hide() {
        assert_eq!(crosshair_key("ArrowRight", None, 4), Some(Some(0)));
        assert_eq!(crosshair_key("ArrowLeft", None, 4), Some(Some(3)));
        assert_eq!(crosshair_key("ArrowRight", Some(3), 4), Some(Some(3)));
        assert_eq!(crosshair_key("ArrowLeft", Some(0), 4), Some(Some(0)));
        assert_eq!(crosshair_key("End", Some(1), 4), Some(Some(3)));
        assert_eq!(crosshair_key("Escape", Some(1), 4), Some(None));
        assert_eq!(crosshair_key("Escape", None, 4), None, "let Escape through when nothing is shown");
        assert_eq!(crosshair_key("a", Some(1), 4), None);
        assert_eq!(crosshair_key("ArrowRight", None, 0), None);
    }

    #[test]
    fn labels_thin_out_evenly() {
        assert_eq!(label_indices(4, 6), vec![0, 1, 2, 3]);
        assert_eq!(label_indices(12, 6), vec![0, 3, 6, 9]);
        assert_eq!(label_indices(11, 6), vec![0, 2, 4, 6, 8, 10]);
    }
}
