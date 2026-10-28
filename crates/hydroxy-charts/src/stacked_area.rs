//! Stacked areas and streamgraphs: series stacked over a shared x axis, on one of four baselines.

use crate::annotate::Annotation;
use crate::line::{self, Curve, Series};
use crate::plot::ValueAxis;
use crate::scale::ScaleKind;

/// Where the stack sits.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StackOffset {
    /// From zero upwards.
    #[default]
    Zero,
    /// Each x position scaled to 100%, so the chart shows shares.
    Expand,
    /// The streamgraph baseline that keeps ribbons as flat as possible (Byron and Wattenberg).
    Wiggle,
    /// Centred on zero.
    Silhouette,
}

impl StackOffset {
    /// The `data-offset` value.
    #[must_use]
    pub const fn attr(self) -> &'static str {
        match self {
            StackOffset::Zero => "zero",
            StackOffset::Expand => "expand",
            StackOffset::Wiggle => "wiggle",
            StackOffset::Silhouette => "silhouette",
        }
    }
}

/// Everything a stacked area chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct StackedAreaLayout {
    /// Horizontal position of each x, 0..=1.
    pub xs: Vec<f64>,
    /// SVG `d` per series for its ribbon, in the 100×100 box.
    pub areas: Vec<String>,
    /// Height of each series' top edge per x, 0..=1.
    pub tops: Vec<Vec<f64>>,
    /// Total of every series per x.
    pub totals: Vec<f64>,
    /// The value axis. Streamgraph offsets have no gridlines, since their values are relative.
    pub axis: ValueAxis,
}

/// The per-x baseline for `offset`, given clean non-negative values `v[series][x]`.
fn baseline(v: &[Vec<f64>], totals: &[f64], offset: StackOffset) -> Vec<f64> {
    match offset {
        StackOffset::Zero | StackOffset::Expand => vec![0.0; totals.len()],
        StackOffset::Silhouette => totals.iter().map(|t| -t / 2.0).collect(),
        StackOffset::Wiggle => {
            // d3's stackOffsetWiggle: shift each column to minimise every ribbon's slope.
            let mut base = vec![0.0; totals.len()];
            let mut y = 0.0;
            for j in 1..totals.len() {
                let (mut s1, mut s2) = (0.0, 0.0);
                for i in 0..v.len() {
                    let mut s3 = (v[i][j] - v[i][j - 1]) / 2.0;
                    for k in &v[..i] {
                        s3 += k[j] - k[j - 1];
                    }
                    s1 += v[i][j];
                    s2 += s3 * v[i][j];
                }
                base[j - 1] = y;
                if s1 != 0.0 {
                    y -= s2 / s1;
                }
            }
            if let Some(last) = base.last_mut() {
                *last = y;
            }
            base
        }
    }
}

/// Stack `series` over `len` x positions. Missing, negative or non-finite values count as zero.
#[must_use]
pub fn layout(
    series: &[Series],
    len: usize,
    offset: StackOffset,
    curve: Curve,
    annotations: &[Annotation],
) -> StackedAreaLayout {
    let xs = line::x_positions(len, None);
    let mut v: Vec<Vec<f64>> = series
        .iter()
        .map(|s| (0..len).map(|i| s.values.get(i).copied().filter(|v| v.is_finite()).unwrap_or(0.0).max(0.0)).collect())
        .collect();
    let totals: Vec<f64> = (0..len).map(|i| v.iter().map(|s| s[i]).sum()).collect();
    if offset == StackOffset::Expand {
        for s in &mut v {
            for (x, t) in s.iter_mut().zip(&totals) {
                *x = if *t > 0.0 { *x / t } else { 0.0 };
            }
        }
    }
    let base = baseline(&v, &totals, offset);
    // lower[s][i] and upper[s][i] in data units.
    let mut lower = Vec::with_capacity(v.len());
    let mut running = base.clone();
    for s in &v {
        lower.push(running.clone());
        for (r, x) in running.iter_mut().zip(s) {
            *r += x;
        }
    }
    let upper: Vec<Vec<f64>> =
        lower.iter().zip(&v).map(|(lo, s)| lo.iter().zip(s).map(|(a, b)| a + b).collect()).collect();
    let extent = base.iter().chain(&running).copied();
    let mut axis = match offset {
        StackOffset::Expand => ValueAxis::over(ScaleKind::Linear, 0.0, 1.0),
        StackOffset::Zero => ValueAxis::new(ScaleKind::Linear, extent, annotations, true),
        StackOffset::Wiggle | StackOffset::Silhouette => ValueAxis::new(ScaleKind::Linear, extent, &[], false),
    };
    if matches!(offset, StackOffset::Wiggle | StackOffset::Silhouette) {
        axis.gridlines.clear();
    }
    let at = |val: f64| axis.scale.map(val);
    let pts = |edge: &[f64]| -> Vec<Option<(f64, f64)>> {
        edge.iter().zip(&xs).map(|(&e, &x)| Some((x * 100.0, (1.0 - at(e)) * 100.0))).collect()
    };
    let areas = lower.iter().zip(&upper).map(|(lo, hi)| line::band_path(&pts(hi), &pts(lo), curve)).collect();
    let tops = upper.iter().map(|hi| hi.iter().map(|&e| at(e)).collect()).collect();
    StackedAreaLayout { xs, areas, tops, totals, axis }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two() -> Vec<Series> {
        vec![Series::new("A", [1.0, 2.0, 3.0]), Series::new("B", [3.0, 2.0, f64::NAN])]
    }

    #[test]
    fn zero_offset_stacks_from_the_axis() {
        let l = layout(&two(), 3, StackOffset::Zero, Curve::Linear, &[]);
        assert_eq!(l.totals, vec![4.0, 4.0, 3.0]);
        assert!((l.tops[1][0] - l.axis.at(4.0)).abs() < 1e-9);
        assert_eq!(l.areas.len(), 2);
        assert!(l.areas[0].ends_with('Z'));
    }

    #[test]
    fn expand_fills_to_one() {
        let l = layout(&two(), 3, StackOffset::Expand, Curve::Linear, &[]);
        assert!(l.tops[1].iter().all(|t| (t - 1.0).abs() < 1e-9));
        assert_eq!(l.axis.scale.domain, (0.0, 1.0));
    }

    #[test]
    fn silhouette_centres_and_wiggle_flattens() {
        let s = two();
        let v: Vec<Vec<f64>> = vec![vec![1.0, 2.0, 3.0], vec![3.0, 2.0, 0.0]];
        let base = baseline(&v, &[4.0, 4.0, 3.0], StackOffset::Silhouette);
        assert_eq!(base, vec![-2.0, -2.0, -1.5]);
        // One series that grows: wiggle shifts the baseline down by half its growth each step.
        let w = baseline(&[vec![0.0, 2.0, 4.0]], &[0.0, 2.0, 4.0], StackOffset::Wiggle);
        assert_eq!(w, vec![0.0, -1.0, -2.0]);
        assert_eq!(layout(&s, 3, StackOffset::Wiggle, Curve::Linear, &[]).axis.gridlines, vec![]);
    }
}
