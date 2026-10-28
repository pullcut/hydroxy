//! Kernel density estimates (Gaussian kernel, Silverman's bandwidth) and the density,
//! ridgeline and violin layouts built on them. Paths are in a 0..100 viewBox, y down.

use std::f64::consts::PI;

use crate::annotate::{self, Annotation};
use crate::boxplot::{self, BoxGeom, Group};
use crate::line::{self, Curve};
use crate::probe::{self, Cell, Pick, Readout, Row, Stop, Tick};
use crate::scale::{Linear, ScaleKind, ValueScale};

/// Points sampled along each curve.
const SAMPLES: usize = 64;

fn finite(values: &[f64]) -> Vec<f64> {
    let mut v: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    v.sort_by(f64::total_cmp);
    v
}

#[allow(clippy::cast_precision_loss, reason = "sample counts are small")]
/// Silverman's rule of thumb: `0.9 · min(sd, IQR / 1.34) · n^(-1/5)`. Falls back to the
/// standard deviation, then 1, when the spread is zero.
#[must_use]
pub fn silverman(values: &[f64]) -> f64 {
    let v = finite(values);
    let n = v.len() as f64;
    if v.len() < 2 {
        return 1.0;
    }
    let mean = v.iter().sum::<f64>() / n;
    let sd = (v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    let iqr = boxplot::quantile(&v, 0.75) - boxplot::quantile(&v, 0.25);
    let spread = if iqr > 0.0 { sd.min(iqr / 1.34) } else { sd };
    if spread > 0.0 { 0.9 * spread * n.powf(-0.2) } else { 1.0 }
}

#[allow(clippy::cast_precision_loss, reason = "sample counts are small")]
/// The Gaussian kernel density of `values` with `bandwidth`, evaluated at each of `at`.
/// The curve integrates to 1. Empty input gives zeros.
#[must_use]
pub fn kde(values: &[f64], bandwidth: f64, at: &[f64]) -> Vec<f64> {
    let v = finite(values);
    let h = if bandwidth.is_finite() && bandwidth > 0.0 { bandwidth } else { 1.0 };
    let norm = 1.0 / (v.len().max(1) as f64 * h * (2.0 * PI).sqrt());
    at.iter().map(|&x| v.iter().map(|&xi| (-0.5 * ((x - xi) / h).powi(2)).exp()).sum::<f64>() * norm).collect()
}

#[allow(clippy::cast_precision_loss, reason = "sample counts are small")]
/// `count` evenly spaced points from `lo` to `hi`, both included.
#[must_use]
pub fn grid(lo: f64, hi: f64, count: usize) -> Vec<f64> {
    let last = count.max(2) - 1;
    (0..=last).map(|i| lo + (hi - lo) * i as f64 / last as f64).collect()
}

/// One curve, ready to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// Group index.
    pub group: usize,
    /// Colour slot.
    pub colour: usize,
    /// Outline path.
    pub line: String,
    /// Filled path.
    pub area: String,
}

/// Curves plus the shared readout.
#[derive(Debug, Clone, PartialEq)]
pub struct DensityLayout {
    /// One per group.
    pub shapes: Vec<Shape>,
    /// Inner box plots (violins only), in plot fractions on the value axis.
    pub boxes: Vec<Option<BoxGeom>>,
    /// Ticks, stops, legend, notes and table.
    pub readout: Readout,
}

fn bandwidths(groups: &[Group], bandwidth: Option<f64>) -> Vec<f64> {
    groups
        .iter()
        .map(|g| bandwidth.filter(|b| b.is_finite() && *b > 0.0).unwrap_or_else(|| silverman(&g.values)))
        .collect()
}

/// The value domain: the data, padded by two bandwidths so tails can fall away.
fn domain(groups: &[Group], hs: &[f64], extra: Option<(f64, f64)>) -> ValueScale {
    let pad = 2.0 * hs.iter().copied().fold(0.0, f64::max);
    let (lo, hi) = probe::extent(groups).map_or((0.0, 1.0), |(lo, hi)| (lo - pad, hi + pad));
    let (lo, hi) = probe::widen((lo, hi), extra);
    ValueScale::new(ScaleKind::Linear, lo, if hi > lo { hi } else { lo + 1.0 }, 6)
}

/// Keep five groups and pool the rest into "Other", so overlaid curves never share a colour.
#[must_use]
pub fn fold(groups: &[Group]) -> Vec<Group> {
    if groups.len() <= 6 {
        return groups.to_vec();
    }
    let mut out = groups[..5].to_vec();
    out.push(Group::new("Other", groups[5..].iter().flat_map(|g| g.values.iter().copied()).collect::<Vec<_>>()));
    out
}

fn summary_rows(values: &[f64], h: f64) -> Vec<Row> {
    let v = finite(values);
    vec![
        Row::new("Count", Cell::Count(v.len())),
        Row::new("Median", if v.is_empty() { Cell::Text("–".into()) } else { Cell::Value(boxplot::quantile(&v, 0.5)) }),
        Row::new("Bandwidth", Cell::Value(h)),
    ]
}

/// Overlaid density curves on one value axis. More than six groups fold into "Other".
/// `bandwidth` overrides Silverman's rule for every group.
#[must_use]
pub fn density(groups: &[Group], bandwidth: Option<f64>, annotations: &[Annotation]) -> DensityLayout {
    let shown = fold(groups);
    let hs = bandwidths(&shown, bandwidth);
    let sx = domain(&shown, &hs, probe::x_extent(annotations));
    let xs = grid(sx.domain.0, sx.domain.1, SAMPLES);
    let curves: Vec<Vec<f64>> = shown.iter().zip(&hs).map(|(g, &h)| kde(&g.values, h, &xs)).collect();
    let top = curves.iter().flatten().copied().fold(0.0, f64::max);
    let (top, _) = probe::widen((top, top), annotate::y_extent(annotations));
    let sy = ValueScale::new(ScaleKind::Linear, 0.0, if top > 0.0 { top } else { 1.0 }, 5);
    let fx: Vec<f64> = xs.iter().map(|&x| sx.map(x)).collect();
    let shapes = curves
        .iter()
        .enumerate()
        .map(|(g, ys)| {
            let pts: Vec<(f64, f64)> =
                fx.iter().zip(ys).map(|(&x, &y)| (x * 100.0, 100.0 - sy.map(y) * 100.0)).collect();
            Shape {
                group: g,
                colour: g,
                line: line::path(&pts, Curve::Monotone),
                area: line::area(&pts, 100.0, Curve::Monotone),
            }
        })
        .collect();
    let stops = xs
        .iter()
        .enumerate()
        .map(|(i, &x)| Stop {
            x: fx[i],
            y: sy.map(curves.iter().map(|c| c[i]).fold(0.0, f64::max)),
            title: Cell::Value(x),
            colour: None,
            rows: shown
                .iter()
                .enumerate()
                .map(|(g, grp)| Row { name: grp.label.clone(), colour: Some(g), value: Cell::Small(curves[g][i]) })
                .collect(),
        })
        .collect();
    let (head, rows) = probe::observations(groups);
    DensityLayout {
        shapes,
        boxes: Vec::new(),
        readout: Readout {
            x_ticks: probe::value_ticks(sx, 6, Cell::Value),
            y_ticks: probe::value_ticks(sy, 5, Cell::Small),
            stops,
            pick: Pick::X,
            legend: probe::legend(shown.iter().map(|g| g.label.clone())),
            notes: annotate::place(annotations, |v| sx.map(v), |v| sy.map(v)),
            head,
            rows,
            ..Readout::default()
        },
    }
}

#[allow(clippy::cast_precision_loss, reason = "group counts are small")]
/// Stacked density curves, one row per group from the top. Each ridge is scaled to its
/// own peak, which reaches `1 + overlap` rows high, so ridges overlap the row above.
#[must_use]
pub fn ridgeline(groups: &[Group], bandwidth: Option<f64>, overlap: f64, annotations: &[Annotation]) -> DensityLayout {
    let n = groups.len().max(1);
    let overlap = if overlap.is_finite() { overlap.clamp(0.0, 4.0) } else { 1.0 };
    let unit = 1.0 / (n as f64 + overlap);
    let base = |g: usize| (n - 1 - g.min(n - 1)) as f64 * unit;
    let hs = bandwidths(groups, bandwidth);
    let sx = domain(groups, &hs, probe::x_extent(annotations));
    let xs = grid(sx.domain.0, sx.domain.1, SAMPLES);
    let fx: Vec<f64> = xs.iter().map(|&x| sx.map(x)).collect();
    let mut shapes = Vec::new();
    let mut stops = Vec::new();
    for (g, group) in groups.iter().enumerate() {
        let ys = kde(&group.values, hs[g], &xs);
        let peak = ys.iter().copied().fold(0.0, f64::max);
        let scale = if peak > 0.0 { (1.0 + overlap) * unit / peak } else { 0.0 };
        let pts: Vec<(f64, f64)> =
            fx.iter().zip(&ys).map(|(&x, &y)| (x * 100.0, 100.0 - (base(g) + y * scale) * 100.0)).collect();
        let colour = probe::slot(g, groups.len());
        shapes.push(Shape {
            group: g,
            colour,
            line: line::path(&pts, Curve::Monotone),
            area: line::area(&pts, 100.0 - base(g) * 100.0, Curve::Monotone),
        });
        let at = ys.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).map_or(0, |(i, _)| i);
        let mut rows = summary_rows(&group.values, hs[g]);
        rows.insert(2, Row::new("Peak at", Cell::Value(xs[at])));
        stops.push(Stop { x: fx[at], y: base(g), title: Cell::Text(group.label.clone()), colour: Some(colour), rows });
    }
    let (head, rows) = probe::observations(groups);
    DensityLayout {
        shapes,
        boxes: Vec::new(),
        readout: Readout {
            x_ticks: probe::value_ticks(sx, 6, Cell::Value),
            y_ticks: groups
                .iter()
                .enumerate()
                .map(|(g, grp)| Tick { fraction: base(g), label: Cell::Text(grp.label.clone()) })
                .collect(),
            stops,
            pick: Pick::Row,
            legend: probe::legend(groups.iter().map(|g| g.label.clone())),
            notes: annotate::place(annotations, |v| sx.map(v), |r| base(0) - r * unit),
            head,
            rows,
            ..Readout::default()
        },
    }
}

#[allow(clippy::cast_precision_loss, reason = "group counts are small")]
/// Mirrored density per group, side by side, on a vertical value axis. Each violin is
/// trimmed to its data and scaled to its own peak. `boxes` carries the inner box plots.
#[must_use]
pub fn violin(groups: &[Group], bandwidth: Option<f64>, annotations: &[Annotation]) -> DensityLayout {
    let n = groups.len().max(1);
    let hs = bandwidths(groups, bandwidth);
    let (lo, hi) = probe::widen(probe::extent(groups).unwrap_or((0.0, 1.0)), annotate::y_extent(annotations));
    let sy = ValueScale::new(ScaleKind::Linear, lo, if hi > lo { hi } else { lo + 1.0 }, 5);
    let f = |v: f64| sy.map(v).clamp(0.0, 1.0);
    let half = Linear::new((0.0, 1.0), (0.0, 0.42 * 100.0 / n as f64));
    let mut shapes = Vec::new();
    let mut stops = Vec::new();
    let mut boxes = Vec::new();
    for (g, group) in groups.iter().enumerate() {
        let v = finite(&group.values);
        let colour = probe::slot(g, groups.len());
        let cx = probe::band(g, n) * 100.0;
        let Some(stats) = boxplot::summarize(&v) else {
            boxes.push(None);
            stops.push(Stop {
                x: probe::band(g, n),
                y: 0.0,
                title: Cell::Text(group.label.clone()),
                colour: Some(colour),
                rows: vec![Row::new("Count", Cell::Count(0))],
            });
            continue;
        };
        let ys = grid(v[0], v[v.len() - 1], SAMPLES);
        let ds = kde(&v, hs[g], &ys);
        let peak = ds.iter().copied().fold(0.0, f64::max);
        let w = |d: f64| half.map(if peak > 0.0 { d / peak } else { 0.0 });
        let right: Vec<(f64, f64)> = ys.iter().zip(&ds).map(|(&y, &d)| (cx + w(d), 100.0 - f(y) * 100.0)).collect();
        let left: Vec<(f64, f64)> = right.iter().rev().map(|&(x, y)| (2.0 * cx - x, y)).collect();
        let outline = format!("{}L{}Z", line::path(&right, Curve::Linear), &line::path(&left, Curve::Linear)[1..]);
        shapes.push(Shape { group: g, colour, line: outline.clone(), area: outline });
        let mut rows = summary_rows(&v, hs[g]);
        rows.splice(2..2, [Row::new("Q1", Cell::Value(stats.q1)), Row::new("Q3", Cell::Value(stats.q3))]);
        stops.push(Stop {
            x: probe::band(g, n),
            y: f(stats.median),
            title: Cell::Text(group.label.clone()),
            colour: Some(colour),
            rows,
        });
        boxes.push(Some(BoxGeom {
            low: f(stats.low),
            q1: f(stats.q1),
            median: f(stats.median),
            q3: f(stats.q3),
            high: f(stats.high),
            outliers: stats.outliers.iter().map(|&o| f(o)).collect(),
            stats,
        }));
    }
    let (head, rows) = probe::observations(groups);
    DensityLayout {
        shapes,
        boxes,
        readout: Readout {
            x_ticks: groups
                .iter()
                .enumerate()
                .map(|(g, grp)| Tick { fraction: probe::band(g, n), label: Cell::Text(grp.label.clone()) })
                .collect(),
            y_ticks: probe::value_ticks(sy, 5, Cell::Value),
            x_bands: true,
            stops,
            pick: Pick::X,
            legend: probe::legend(groups.iter().map(|g| g.label.clone())),
            notes: annotate::place(annotations, |i| probe::band(0, n) + i / n as f64, |v| sy.map(v)),
            head,
            rows,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: [f64; 10] = [2.0, 3.0, 3.5, 4.0, 4.2, 4.8, 5.0, 5.5, 6.0, 9.0];

    #[test]
    fn silverman_matches_the_rule() {
        // iqr 5.375 - 3.625 = 1.75, and 1.75 / 1.34 is below the sd, so it wins.
        let h = silverman(&SAMPLE);
        assert!((h - 0.9 * (1.75 / 1.34) * 10f64.powf(-0.2)).abs() < 1e-3, "{h}");
        assert!((silverman(&[5.0, 5.0, 5.0]) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn kde_integrates_to_one() {
        let xs = grid(-10.0, 25.0, 2001);
        let ys = kde(&SAMPLE, 0.8, &xs);
        let step = xs[1] - xs[0];
        let area: f64 = ys.iter().sum::<f64>() * step;
        assert!((area - 1.0).abs() < 1e-3, "{area}");
        let peak = ys.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).map_or(0.0, |(i, _)| xs[i]);
        assert!((3.0..6.0).contains(&peak));
    }

    #[test]
    fn density_shares_axes_and_folds_extra_groups() {
        let groups: Vec<Group> =
            (0..8).map(|i| Group::new(format!("g{i}"), SAMPLE.map(|v| v + f64::from(i)))).collect();
        let l = density(&groups, None, &[]);
        assert_eq!(l.shapes.len(), 6);
        assert_eq!(l.readout.legend.last().map(|l| l.0.as_str()), Some("Other"));
        assert_eq!(l.readout.stops.len(), SAMPLES);
        assert_eq!(l.readout.rows.len(), 80, "the table keeps every observation");
        assert!(l.shapes[0].area.ends_with('Z'));
    }

    #[test]
    fn ridges_stack_from_the_top() {
        let groups = [Group::new("a", SAMPLE), Group::new("b", SAMPLE)];
        let l = ridgeline(&groups, Some(0.5), 1.0, &[]);
        assert!(l.readout.stops[0].y > l.readout.stops[1].y);
        assert!((l.readout.y_ticks[0].fraction - 1.0 / 3.0).abs() < 1e-9);
        assert_eq!(nearest_row(&l, 0.9), Some(0));
        assert_eq!(nearest_row(&l, 0.1), Some(1));
    }

    fn nearest_row(l: &DensityLayout, y: f64) -> Option<usize> {
        probe::nearest(&l.readout.stops, 0.5, y, l.readout.pick)
    }

    #[test]
    fn violins_carry_box_stats_and_skip_empty_groups() {
        let l =
            violin(&[Group::new("a", SAMPLE), Group::new("b", [f64::NAN])], None, &[Annotation::y_line(20.0, "Cap")]);
        assert_eq!(l.shapes.len(), 1);
        let b = l.boxes[0].as_ref().map(|b| (b.q1, b.median, b.q3));
        assert!(b.is_some_and(|(q1, m, q3)| q1 < m && m < q3));
        assert!(l.boxes[1].is_none());
        assert_eq!(l.readout.notes.len(), 1, "the axis widens to fit the annotation");
    }
}
