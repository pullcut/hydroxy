//! Empirical cumulative distribution functions: for each group, the share of values at or
//! below each x, drawn as a step line, with an optional fitted normal CDF to compare against.
#![allow(
    clippy::many_single_char_names,
    clippy::cast_precision_loss,
    reason = "short names follow the maths; sample sizes are far below 2^52"
)]

use std::fmt::Write as _;

use crate::annotate::Annotation;
use crate::plot::ValueAxis;
use crate::scale::ScaleKind;

/// A named sample.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// The group's name, for the legend.
    pub name: String,
    /// The values, in any order. Non-finite values are dropped.
    pub values: Vec<f64>,
}

impl Sample {
    /// A named sample.
    #[must_use]
    pub fn new(name: impl Into<String>, values: impl Into<Vec<f64>>) -> Self {
        Self { name: name.into(), values: values.into() }
    }
}

/// The standard normal CDF, via the complementary error function (Numerical Recipes'
/// `erfcc`, absolute error below 1.2e-7).
#[must_use]
pub fn normal_cdf(z: f64) -> f64 {
    let x = z.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.5 * x);
    let poly = -x * x - 1.265_512_23
        + t * (1.000_023_68
            + t * (0.374_091_96
                + t * (0.096_784_18
                    + t * (-0.186_288_06
                        + t * (0.278_868_07
                            + t * (-1.135_203_98 + t * (1.488_515_87 + t * (-0.822_152_23 + t * 0.170_872_77))))))));
    let erfc = t * poly.exp();
    if z >= 0.0 { 1.0 - erfc / 2.0 } else { erfc / 2.0 }
}

/// The share of `sorted` (ascending) at or below `x`.
#[must_use]
pub fn ecdf_at(sorted: &[f64], x: f64) -> f64 {
    if sorted.is_empty() { 0.0 } else { sorted.partition_point(|&v| v <= x) as f64 / sorted.len() as f64 }
}

/// One group's curve, ready to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct EcdfCurve {
    /// The group's name.
    pub name: String,
    /// The finite values, ascending.
    pub sorted: Vec<f64>,
    /// Sample mean.
    pub mean: f64,
    /// Sample standard deviation.
    pub sd: f64,
    /// SVG `d` of the step line, in the 100×100 box.
    pub line: String,
    /// SVG `d` of the fitted normal CDF, in the 100×100 box. Empty for fewer than two values.
    pub reference: String,
}

/// Everything an ECDF chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct EcdfLayout {
    /// One curve per group, in input order.
    pub curves: Vec<EcdfCurve>,
    /// The value (x) axis.
    pub x_axis: ValueAxis,
    /// The cumulative share (y) axis, 0..=1.
    pub axis: ValueAxis,
    /// Every distinct value across groups, ascending, for the crosshair.
    pub xs: Vec<f64>,
}

/// Lay out a step line per group (the first six) over a shared value axis.
#[must_use]
pub fn layout(samples: &[Sample], annotations: &[Annotation]) -> EcdfLayout {
    let finite = |s: &Sample| s.values.iter().copied().filter(|v| v.is_finite()).collect::<Vec<f64>>();
    let all: Vec<f64> = samples.iter().flat_map(finite).collect();
    let xs_extent = annotations.iter().filter_map(|a| match *a {
        Annotation::XLine { at, .. } => Some(at),
        _ => None,
    });
    let x_axis = ValueAxis::new(ScaleKind::Linear, all.iter().copied().chain(xs_extent), &[], false);
    let axis = ValueAxis::over(ScaleKind::Linear, 0.0, 1.0);
    let (lo, hi) = x_axis.scale.domain;
    let px = |v: f64| x_axis.scale.map(v) * 100.0;
    let py = |p: f64| (1.0 - p) * 100.0;
    let curves = samples
        .iter()
        .take(6)
        .map(|s| {
            let mut sorted = finite(s);
            sorted.sort_by(f64::total_cmp);
            let n = sorted.len() as f64;
            let mean = sorted.iter().sum::<f64>() / n.max(1.0);
            let sd = (sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0)).sqrt();
            let mut line = String::new();
            if !sorted.is_empty() {
                let _ = write!(line, "M{:.3} {:.3}", px(lo), py(0.0));
                let mut i = 0;
                while i < sorted.len() {
                    let v = sorted[i];
                    i += sorted[i..].iter().take_while(|w| w.total_cmp(&v).is_eq()).count();
                    let _ = write!(line, "H{:.3}V{:.3}", px(v), py(i as f64 / n));
                }
                let _ = write!(line, "H{:.3}", px(hi));
            }
            let mut reference = String::new();
            if sorted.len() > 1 && sd > 0.0 {
                for k in 0..=96 {
                    let v = lo + (hi - lo) * f64::from(k) / 96.0;
                    let _ = write!(
                        reference,
                        "{}{:.3} {:.3}",
                        if k == 0 { 'M' } else { 'L' },
                        px(v),
                        py(normal_cdf((v - mean) / sd))
                    );
                }
            }
            EcdfCurve { name: s.name.clone(), sorted, mean, sd, line, reference }
        })
        .collect();
    let mut xs = all;
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    EcdfLayout { curves, x_axis, axis, xs }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_cdf_matches_known_values() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-7);
        assert!((normal_cdf(1.959_964) - 0.975).abs() < 1e-6);
        assert!((normal_cdf(-1.0) - 0.158_655_25).abs() < 1e-6);
    }

    #[test]
    fn ecdf_counts_ties_and_bounds() {
        let s = [1.0, 2.0, 2.0, 4.0];
        assert_eq!(ecdf_at(&s, 0.5), 0.0);
        assert_eq!(ecdf_at(&s, 2.0), 0.75);
        assert_eq!(ecdf_at(&s, 3.0), 0.75);
        assert_eq!(ecdf_at(&s, 9.0), 1.0);
        assert_eq!(ecdf_at(&[], 1.0), 0.0);
    }

    #[test]
    fn layout_steps_once_per_distinct_value() {
        let l = layout(&[Sample::new("a", [2.0, 1.0, 2.0, f64::NAN, 4.0]), Sample::new("empty", [])], &[]);
        let c = &l.curves[0];
        assert_eq!(c.sorted, vec![1.0, 2.0, 2.0, 4.0]);
        assert_eq!(c.line.matches('V').count(), 3);
        assert!(c.line.ends_with("V0.000H100.000"));
        assert!(c.reference.starts_with('M'));
        assert_eq!(l.curves[1].line, "");
        assert_eq!(l.xs, vec![1.0, 2.0, 4.0]);
    }
}
