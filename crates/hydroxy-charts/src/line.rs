//! Line and area charts: several series over one shared x axis.
//!
//! Paths are drawn in a 100×100 box (y down) meant for an SVG with
//! `preserveAspectRatio="none"` and `vector-effect: non-scaling-stroke`, so the
//! chart fills its container without measuring. Everything else (gridlines, dots,
//! crosshair) is a fraction of the plot area, for plain CSS positioning.

use std::fmt::Write as _;

use crate::bar::Gridline;
use crate::plot;
use crate::scale::{self, Linear, ScaleKind, ValueScale};

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

/// A confidence band around one series: a low and a high value per x position.
/// A position where either end is not finite leaves a gap.
#[derive(Debug, Clone, PartialEq)]
pub struct Band {
    /// Index of the series the band belongs to (sets its colour and name).
    pub series: usize,
    /// Lower bound per x position.
    pub low: Vec<f64>,
    /// Upper bound per x position.
    pub high: Vec<f64>,
}

impl Band {
    /// A band around series `series`.
    #[must_use]
    pub fn new(series: usize, low: impl Into<Vec<f64>>, high: impl Into<Vec<f64>>) -> Self {
        Self { series, low: low.into(), high: high.into() }
    }
}

/// SVG `d` for the filled region between `high` (left to right) and `low` (back again).
/// Runs of fewer than two points draw nothing.
#[must_use]
pub fn band_path(high: &[Option<(f64, f64)>], low: &[Option<(f64, f64)>], curve: Curve) -> String {
    type Pair = ((f64, f64), (f64, f64));
    let both: Vec<Option<Pair>> = high.iter().zip(low).map(|(h, l)| h.zip(*l)).collect();
    let mut d = String::new();
    for run in both.split(Option::is_none).filter(|r| r.len() >= 2) {
        let top: Vec<(f64, f64)> = run.iter().flatten().map(|p| p.0).collect();
        let bottom: Vec<(f64, f64)> = run.iter().flatten().rev().map(|p| p.1).collect();
        // The return path continues from the end of the top edge, so its "M" becomes "L".
        let _ = write!(d, "{}L{}Z", path(&top, curve), &path(&bottom, curve)[1..]);
    }
    d
}

/// How points are joined.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
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
    let mut d = String::with_capacity(points.len() * 16);
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
    points.split(Option::is_none).filter(|r| !r.is_empty()).map(|r| r.iter().flatten().copied().collect()).collect()
}

/// Thin a run of points (ascending x) to at most four per bucket: the first, lowest,
/// highest and last. Peaks and dips survive, so the drawn line looks the same at
/// `buckets` pixels wide. Runs short enough to draw as they are come back unchanged.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "bucket maths on clamped fractions"
)]
#[must_use]
pub fn decimate(points: &[(f64, f64)], buckets: usize) -> Vec<(f64, f64)> {
    let (Some(first), Some(last)) = (points.first(), points.last()) else { return Vec::new() };
    let span = last.0 - first.0;
    if points.len() <= buckets.max(1) * 4 || span <= 0.0 {
        return points.to_vec();
    }
    let bucket = |x: f64| (((x - first.0) / span * buckets as f64) as usize).min(buckets - 1);
    let mut out = Vec::with_capacity(buckets * 4);
    let mut start = 0;
    while start < points.len() {
        let b = bucket(points[start].0);
        let end = start + points[start..].iter().position(|p| bucket(p.0) != b).unwrap_or(points.len() - start);
        let slice = &points[start..end];
        let lowest = (0..slice.len()).min_by(|&i, &j| slice[i].1.total_cmp(&slice[j].1)).unwrap_or(0);
        let highest = (0..slice.len()).max_by(|&i, &j| slice[i].1.total_cmp(&slice[j].1)).unwrap_or(0);
        let mut keep = [0, lowest, highest, slice.len() - 1];
        keep.sort_unstable();
        let mut prev = None;
        for k in keep {
            if prev != Some(k) {
                out.push(slice[k]);
                prev = Some(k);
            }
        }
        start = end;
    }
    out
}

#[allow(clippy::cast_precision_loss, reason = "point counts are small")]
/// Horizontal position (0..=1) of each of `len` points. Categories are evenly spaced;
/// numeric `x` values (ascending) are placed proportionally.
#[must_use]
pub fn x_positions(len: usize, x: Option<&[f64]>) -> Vec<f64> {
    if let Some(x) = x.filter(|x| x.len() >= len && len > 1) {
        // Non-finite x values stay non-finite, which the layout draws as gaps.
        let mut finite = x[..len].iter().copied().filter(|v| v.is_finite());
        if let Some(first) = finite.next() {
            let scale = Linear::new((first, finite.next_back().unwrap_or(first)), (0.0, 1.0));
            return x[..len].iter().map(|&v| if v.is_finite() { scale.map(v) } else { f64::NAN }).collect();
        }
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
    /// SVG `d` for each band passed to [`layout_with_bands`], in order.
    pub bands: Vec<String>,
    /// Gridlines from the bottom up, at nice ticks.
    pub gridlines: Vec<Gridline>,
    /// Bottom of the value axis.
    pub min: f64,
    /// Top of the value axis.
    pub max: f64,
    /// The value axis, for placing annotations.
    pub scale: ValueScale,
}

/// Lay out `series` at horizontal positions `xs` (see [`x_positions`]). The value axis
/// always includes zero. Non-finite or missing values leave a gap in their line.
#[must_use]
pub fn layout(series: &[Vec<f64>], xs: &[f64], curve: Curve, tick_count: usize) -> LineLayout {
    layout_with_bands(series, &[], xs, curve, tick_count)
}

/// [`layout`] with confidence bands. The value axis grows to include the bands.
#[must_use]
pub fn layout_with_bands(
    series: &[Vec<f64>],
    bands: &[Band],
    xs: &[f64],
    curve: Curve,
    tick_count: usize,
) -> LineLayout {
    layout_scaled(series, bands, xs, curve, tick_count, ScaleKind::Linear, None)
}

/// [`layout_with_bands`] on a value axis of any [`ScaleKind`], widened to cover `extent`
/// (such as [`crate::annotate::y_extent`]). Linear and symlog axes include zero; log axes don't.
#[must_use]
pub fn layout_scaled(
    series: &[Vec<f64>],
    bands: &[Band],
    xs: &[f64],
    curve: Curve,
    tick_count: usize,
    kind: ScaleKind,
    extent: Option<(f64, f64)>,
) -> LineLayout {
    let finite = || {
        let band_values = bands.iter().flat_map(|b| b.low.iter().chain(&b.high).take(xs.len() * 2));
        let extra = extent.into_iter().flat_map(|(a, b)| [a, b]);
        series
            .iter()
            .flat_map(|s| s.iter().take(xs.len()))
            .chain(band_values)
            .copied()
            .chain(extra)
            .filter(|v| v.is_finite())
    };
    let (lo, mut hi) = if kind == ScaleKind::Log {
        let positive = || finite().filter(|v| *v > 0.0);
        let lo = positive().fold(f64::INFINITY, f64::min);
        if lo.is_finite() { (lo, positive().fold(0.0, f64::max)) } else { (1.0, 10.0) }
    } else {
        (finite().fold(0.0, f64::min), finite().fold(0.0, f64::max))
    };
    if hi <= lo {
        hi = lo + 1.0;
    }
    let scale = ValueScale::new(kind, lo, hi, tick_count);
    let (min, max) = scale.domain;
    let to_y = |f: f64| (1.0 - f) * 100.0;
    let baseline = to_y(scale.map(0.0).clamp(0.0, 1.0));

    let series = series
        .iter()
        .map(|values| {
            let points: Vec<Option<f64>> =
                (0..xs.len()).map(|i| values.get(i).copied().filter(|v| v.is_finite()).map(|v| scale.map(v))).collect();
            let drawn: Vec<Option<(f64, f64)>> = points
                .iter()
                .zip(xs)
                .map(|(p, &x)| p.filter(|_| x.is_finite()).map(|f| (x * 100.0, to_y(f))))
                .collect();
            // A line wider than the screen in points draws the same thinned.
            let runs: Vec<Vec<(f64, f64)>> = runs(&drawn).iter().map(|r| decimate(r, 1_000)).collect();
            LinePath {
                line: runs.iter().map(|r| path(r, curve)).collect(),
                area: runs.iter().map(|r| area(r, baseline, curve)).collect(),
                points,
            }
        })
        .collect();

    let drawn = |values: &[f64]| -> Vec<Option<(f64, f64)>> {
        xs.iter()
            .enumerate()
            .map(|(i, &x)| {
                values
                    .get(i)
                    .copied()
                    .filter(|v| v.is_finite() && x.is_finite())
                    .map(|v| (x * 100.0, to_y(scale.map(v))))
            })
            .collect()
    };
    let bands = bands.iter().map(|b| band_path(&drawn(&b.high), &drawn(&b.low), curve)).collect();

    LineLayout {
        xs: xs.to_vec(),
        series,
        bands,
        gridlines: scale
            .ticks(tick_count)
            .into_iter()
            .map(|value| Gridline { value, fraction: scale.map(value) })
            .collect(),
        min,
        max,
        scale,
    }
}

/// Where x value `at` sits across a plot of `len` points, 0..=1: a numeric position
/// when `x` is given (see [`x_positions`]), otherwise a point index.
#[allow(clippy::cast_precision_loss, reason = "point counts are small")]
#[must_use]
pub fn x_fraction(at: f64, len: usize, x: Option<&[f64]>) -> f64 {
    match x.filter(|x| x.len() >= len && len > 1) {
        Some(x) => Linear::new((x[0], x[len - 1]), (0.0, 1.0)).map(at),
        None if len > 1 => at / (len - 1) as f64,
        None => 0.5,
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

/// The part of `v` inside `range`, clamped to its length: how a zoomed chart slices its
/// series, labels, x positions and bands to the visible window.
#[must_use]
pub fn windowed<T: Clone>(v: &[T], range: &std::ops::Range<usize>) -> Vec<T> {
    v.get(range.start.min(v.len())..range.end.min(v.len())).unwrap_or_default().to_vec()
}

/// The chart as text rows for CSV export, header first: a label column, one per series, then
/// a low and a high column per band. Gaps read "–" and values go through `format`.
#[must_use]
pub fn export_table(
    labels: &[String],
    series: &[Series],
    bands: &[Band],
    format: &dyn Fn(f64) -> String,
) -> Vec<Vec<String>> {
    let band_name = |b: &Band| series.get(b.series).map(|s| s.name.clone()).unwrap_or_default();
    let head = std::iter::once(String::new())
        .chain(series.iter().map(|s| s.name.clone()))
        .chain(bands.iter().flat_map(|b| {
            let n = band_name(b);
            [format!("{n} low"), format!("{n} high")]
        }))
        .collect();
    let at = |v: &[f64], i: usize| plot::cell(v.get(i).copied(), &format);
    let body = labels.iter().enumerate().map(|(i, l)| {
        std::iter::once(l.clone())
            .chain(series.iter().map(|s| at(&s.values, i)))
            .chain(bands.iter().flat_map(|b| [at(&b.low, i), at(&b.high, i)]))
            .collect()
    });
    std::iter::once(head).chain(body).collect()
}

/// The live-region text for the crosshair at point `i`: "Label: Series 1 value, Series 2 value".
/// A missing label reads empty and gaps read "–".
#[must_use]
pub fn announce(labels: &[String], series: &[Series], i: usize, format: &dyn Fn(f64) -> String) -> String {
    let parts: Vec<String> =
        series.iter().map(|s| format!("{} {}", s.name, plot::cell(s.values.get(i).copied(), &format))).collect();
    format!("{}: {}", labels.get(i).map_or("", String::as_str), parts.join(", "))
}

/// Calendar tick labels for a time axis, as `(fraction along the plot, label)`. Empty unless
/// `time_axis` is set and `x` (Unix seconds) has at least two points.
#[must_use]
pub fn time_labels(x: Option<&[f64]>, time_axis: bool) -> Vec<(f64, String)> {
    match x {
        Some(x) if time_axis && x.len() > 1 => scale::time_ticks(x[0], x[x.len() - 1], 6)
            .into_iter()
            .map(|t| (x_fraction(t.at, x.len(), Some(x)), t.label))
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimation_keeps_extremes_and_ends() {
        let points: Vec<(f64, f64)> =
            (0..100_000).map(|i| (f64::from(i), if i == 54_321 { 1e6 } else { f64::from(i % 7) })).collect();
        let thin = decimate(&points, 100);
        assert!(thin.len() <= 400);
        assert_eq!(thin.first(), points.first());
        assert_eq!(thin.last(), points.last());
        assert!(thin.contains(&(54_321.0, 1e6)));
        assert!(thin.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(decimate(&points[..10], 100), points[..10].to_vec());
    }

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
    fn non_finite_x_values_are_gaps() {
        let xs = x_positions(4, Some(&[f64::NAN, 1.0, f64::INFINITY, 3.0]));
        let l = layout(&[vec![1.0, 2.0, 3.0, 4.0]], &xs, Curve::Linear, 5);
        let d = format!("{}{}", l.series[0].line, l.series[0].area);
        assert!(!d.contains("NaN") && !d.contains("inf"), "{d}");
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
    fn bands_close_between_high_and_low_and_widen_the_axis() {
        let xs = x_positions(4, None);
        let band = Band::new(0, [8.0, 9.0, f64::NAN, 11.0], [12.0, 14.0, 15.0, 30.0]);
        let l = layout_with_bands(&[vec![10.0, 11.0, 12.0, 13.0]], &[band], &xs, Curve::Linear, 5);
        assert!(l.max >= 30.0, "axis includes the band");
        // Points 0 and 1 form one run (a gap at 2 and a lone point at 3 draw nothing).
        assert_eq!(l.bands[0].matches('M').count(), 1);
        assert!(l.bands[0].ends_with('Z') && l.bands[0].contains('L'));
        assert_eq!(layout(&[vec![1.0]], &x_positions(1, None), Curve::Linear, 5).bands.len(), 0);
    }

    #[test]
    fn log_axes_skip_zero_and_extents_widen_the_axis() {
        let xs = x_positions(3, None);
        let l = layout_scaled(&[vec![3.0, 40.0, 900.0]], &[], &xs, Curve::Linear, 5, ScaleKind::Log, None);
        assert_eq!((l.min, l.max), (1.0, 1000.0));
        assert!(l.gridlines.iter().any(|g| (g.value - 100.0).abs() < 1e-9));
        let l = layout_scaled(&[vec![3.0, 4.0]], &[], &xs, Curve::Linear, 5, ScaleKind::Linear, Some((0.0, 120.0)));
        assert!(l.max >= 120.0);
        assert!((x_fraction(1.0, 3, None) - 0.5).abs() < 1e-12);
        assert!((x_fraction(5.0, 3, Some(&[0.0, 2.0, 10.0])) - 0.5).abs() < 1e-12);
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

    #[test]
    fn windows_clamp_to_the_data() {
        let v = [1, 2, 3, 4];
        assert_eq!(windowed(&v, &(1..3)), vec![2, 3]);
        assert_eq!(windowed(&v, &(2..9)), vec![3, 4]);
        assert_eq!(windowed(&v, &(7..9)), Vec::<i32>::new());
        assert_eq!(windowed(&v, &std::ops::Range { start: 3, end: 1 }), Vec::<i32>::new());
    }

    #[test]
    fn export_lists_series_then_bands_with_gaps() {
        let labels = vec!["Jan".to_string(), "Feb".to_string()];
        let series = [Series::new("Users", [1.0, f64::NAN])];
        let bands = [Band::new(0, [0.5, 1.5], [1.5])];
        let table = export_table(&labels, &series, &bands, &|v| format!("{v:.1}"));
        assert_eq!(table[0], vec!["", "Users", "Users low", "Users high"]);
        assert_eq!(table[1], vec!["Jan", "1.0", "0.5", "1.5"]);
        assert_eq!(table[2], vec!["Feb", "–", "1.5", "–"]);
    }

    #[test]
    fn announcements_name_each_series() {
        let labels = vec!["Jan".to_string()];
        let series = [Series::new("A", [2.0]), Series::new("B", [f64::NAN])];
        assert_eq!(announce(&labels, &series, 0, &|v| format!("{v}")), "Jan: A 2, B –");
        assert_eq!(announce(&labels, &series, 5, &|v| format!("{v}")), ": A –, B –");
    }

    #[test]
    fn time_labels_need_a_time_axis_and_two_points() {
        let day = 86_400.0;
        let x: &[f64] = &[0.0, 10.0 * day];
        let labels = time_labels(Some(x), true);
        assert!(labels.len() >= 2 && labels.iter().all(|(f, _)| (0.0..=1.0).contains(f)));
        assert_eq!(labels[0].1, "1 Jan");
        assert_eq!(time_labels(Some(x), false), vec![]);
        assert_eq!(time_labels(Some(&x[..1]), true), vec![]);
        assert_eq!(time_labels(None, true), vec![]);
    }
}
