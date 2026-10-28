//! Bland–Altman (difference) plots: for paired readings from two methods, each pair's
//! mean against its difference, with the mean difference (bias) and the 95% limits of
//! agreement (bias ± 1.96 SD), each with an optional confidence interval.
#![allow(
    clippy::many_single_char_names,
    clippy::cast_precision_loss,
    reason = "short names follow the maths; sample sizes are far below 2^52"
)]

use crate::annotate::Annotation;
use crate::plot::ValueAxis;
use crate::qq::normal_quantile;
use crate::scale::ScaleKind;

/// One pair of readings, placed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgreementPoint {
    /// Index of the pair in the input.
    pub index: usize,
    /// Reading from the first method.
    pub a: f64,
    /// Reading from the second method.
    pub b: f64,
    /// `(a + b) / 2`.
    pub mean: f64,
    /// `a - b`.
    pub diff: f64,
    /// Horizontal position, 0..=1.
    pub x: f64,
    /// Vertical position, 0..=1.
    pub y: f64,
}

/// A horizontal reference line: the bias or a limit of agreement, with its confidence interval.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limit {
    /// The value, in difference units.
    pub value: f64,
    /// The 95% confidence interval of the value.
    pub ci: (f64, f64),
    /// Where the line sits, 0..=1 from the bottom.
    pub at: f64,
    /// Where the interval sits, 0..=1 from the bottom, low then high.
    pub ci_at: (f64, f64),
}

/// Everything a Bland–Altman chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct AgreementLayout {
    /// The pairs with two finite readings, in input order.
    pub points: Vec<AgreementPoint>,
    /// The mean (x) axis.
    pub x_axis: ValueAxis,
    /// The difference (y) axis.
    pub axis: ValueAxis,
    /// Mean difference.
    pub bias: Limit,
    /// Bias − 1.96 SD.
    pub lower: Limit,
    /// Bias + 1.96 SD.
    pub upper: Limit,
    /// Standard deviation of the differences.
    pub sd: f64,
}

/// The two-sided Student t quantile for probability `p` with `df` degrees of freedom, by the
/// Cornish–Fisher expansion around the normal quantile (within 0.1% for df ≥ 3).
#[must_use]
pub fn t_quantile(p: f64, df: f64) -> f64 {
    let z = normal_quantile(p);
    if !df.is_finite() || df <= 0.0 {
        return z;
    }
    let (z3, z5, z7) = (z.powi(3), z.powi(5), z.powi(7));
    z + (z3 + z) / (4.0 * df)
        + (5.0 * z5 + 16.0 * z3 + 3.0 * z) / (96.0 * df * df)
        + (3.0 * z7 + 19.0 * z5 + 17.0 * z3 - 15.0 * z) / (384.0 * df.powi(3))
}

/// Pair `a[i]` with `b[i]` (extra readings on either side are ignored, as are pairs with a
/// non-finite reading) and work out the bias and limits of agreement. The difference axis
/// keeps zero, the limits and their intervals in view.
#[must_use]
pub fn layout(a: &[f64], b: &[f64], annotations: &[Annotation]) -> AgreementLayout {
    let pairs: Vec<(usize, f64, f64)> = a
        .iter()
        .zip(b)
        .enumerate()
        .filter(|(_, (x, y))| x.is_finite() && y.is_finite())
        .map(|(i, (&x, &y))| (i, x, y))
        .collect();
    let n = pairs.len() as f64;
    let diffs: Vec<f64> = pairs.iter().map(|&(_, x, y)| x - y).collect();
    let bias = diffs.iter().sum::<f64>() / n.max(1.0);
    let sd = (diffs.iter().map(|d| (d - bias).powi(2)).sum::<f64>() / (n - 1.0).max(1.0)).sqrt();
    let z = normal_quantile(0.975);
    let t = if n > 1.0 { t_quantile(0.975, n - 1.0) } else { 0.0 };
    // Bland & Altman (1999): SE(bias) = s/√n, SE(limit) ≈ s·√(3/n).
    let se_bias = sd / n.max(1.0).sqrt();
    let se_limit = sd * (3.0 / n.max(1.0)).sqrt();
    let raw = [(bias, t * se_bias), (bias - z * sd, t * se_limit), (bias + z * sd, t * se_limit)];
    let means: Vec<f64> = pairs.iter().map(|&(_, x, y)| f64::midpoint(x, y)).collect();
    let x_axis = ValueAxis::new(ScaleKind::Linear, means.iter().copied(), &[], false);
    let reach = raw.iter().flat_map(|&(v, h)| [v - h, v + h]);
    let axis = ValueAxis::new(ScaleKind::Linear, diffs.iter().copied().chain(reach), annotations, true);
    let limit = |(value, half): (f64, f64)| Limit {
        value,
        ci: (value - half, value + half),
        at: axis.scale.map(value),
        ci_at: (axis.scale.map(value - half), axis.scale.map(value + half)),
    };
    let points = pairs
        .iter()
        .zip(&means)
        .zip(&diffs)
        .map(|((&(index, a, b), &mean), &diff)| AgreementPoint {
            index,
            a,
            b,
            mean,
            diff,
            x: x_axis.scale.map(mean),
            y: axis.scale.map(diff),
        })
        .collect();
    AgreementLayout { points, bias: limit(raw[0]), lower: limit(raw[1]), upper: limit(raw[2]), sd, x_axis, axis }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_quantiles_match_tables() {
        assert!((t_quantile(0.975, 10.0) - 2.228).abs() < 2e-3);
        assert!((t_quantile(0.975, 30.0) - 2.042).abs() < 1e-3);
        assert!((t_quantile(0.975, f64::INFINITY) - 1.96).abs() < 1e-3);
    }

    #[test]
    fn bias_and_limits_follow_the_differences() {
        // Differences 1, 2, 3, 4, 5: bias 3, SD √2.5.
        let a = [11.0, 22.0, 33.0, 44.0, 55.0, f64::NAN];
        let b = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0];
        let l = layout(&a, &b, &[]);
        assert_eq!(l.points.len(), 5);
        assert!((l.bias.value - 3.0).abs() < 1e-12);
        assert!((l.sd - 2.5_f64.sqrt()).abs() < 1e-12);
        assert!((l.upper.value - (3.0 + 1.959_964 * l.sd)).abs() < 1e-5);
        assert!((l.lower.value - (3.0 - 1.959_964 * l.sd)).abs() < 1e-5);
        assert!(l.bias.ci.0 < 3.0 && l.bias.ci.1 > 3.0);
        assert!(l.upper.ci.1 - l.upper.ci.0 > l.bias.ci.1 - l.bias.ci.0, "limits are less certain than the bias");
        assert!((l.points[0].mean - 10.5).abs() < 1e-12);
        assert!(l.axis.scale.domain.0 <= l.lower.ci.0 && l.axis.scale.domain.1 >= l.upper.ci.1);
        assert!(l.points.iter().all(|p| (0.0..=1.0).contains(&p.x) && (0.0..=1.0).contains(&p.y)));
    }

    #[test]
    fn empty_input_is_safe() {
        let l = layout(&[], &[], &[]);
        assert_eq!(l.points.len(), 0);
        assert!(l.sd.is_finite() && l.bias.value.is_finite());
    }
}
