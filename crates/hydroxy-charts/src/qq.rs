//! Normal quantile–quantile plots: each sorted sample value against the normal quantile
//! it would have if the sample were normal, with the line those points would follow.

use std::fmt::Write as _;

use crate::annotate::Annotation;
use crate::bar::Gridline;
use crate::plot::ValueAxis;
use crate::scale::{Linear, ScaleKind, ticks};

/// The standard normal quantile (inverse CDF) of `p` in (0, 1), by Acklam's rational
/// approximation (relative error below 1.2e-9). Infinite at 0 and 1, NaN outside.
#[must_use]
pub fn normal_quantile(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    const D: [f64; 4] =
        [7.784_695_709_041_462e-3, 3.224_671_290_700_398e-1, 2.445_134_137_142_996, 3.754_408_661_907_416];
    const LOW: f64 = 0.024_25;
    if !(0.0..=1.0).contains(&p) {
        return f64::NAN;
    }
    if p == 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    let tail = |q: f64| {
        let q = (-2.0 * q.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    if p < LOW {
        tail(p)
    } else if p > 1.0 - LOW {
        -tail(1.0 - p)
    } else {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    }
}

/// One sample value against its theoretical quantile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QqPoint {
    /// The normal quantile, in standard deviations.
    pub theoretical: f64,
    /// The sample value.
    pub sample: f64,
    /// Horizontal position, 0..=1.
    pub x: f64,
    /// Vertical position, 0..=1.
    pub y: f64,
}

/// Everything a Q–Q chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct QqLayout {
    /// Sorted ascending.
    pub points: Vec<QqPoint>,
    /// Ticks along the theoretical (x) axis.
    pub x_ticks: Vec<Gridline>,
    /// The theoretical axis domain, in standard deviations.
    pub x_domain: (f64, f64),
    /// SVG `d` of the reference line (sample mean + standard deviation × quantile), in the 100×100 box.
    pub reference: String,
    /// The sample (y) axis.
    pub axis: ValueAxis,
}

#[allow(clippy::cast_precision_loss, reason = "sample sizes are far below 2^52")]
/// Pair each finite sample value, sorted, with the normal quantile at (i + 0.5) / n.
#[must_use]
pub fn layout(sample: &[f64], annotations: &[Annotation]) -> QqLayout {
    let mut sorted: Vec<f64> = sample.iter().copied().filter(|v| v.is_finite()).collect();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len() as f64;
    let z: Vec<f64> = (0..sorted.len()).map(|i| normal_quantile((i as f64 + 0.5) / n)).collect();
    let mean = sorted.iter().sum::<f64>() / n.max(1.0);
    let sd = (sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0)).sqrt();
    let zmax = z.iter().fold(1.0_f64, |m, v| m.max(v.abs()));
    let tick_values = ticks(-zmax, zmax, 7);
    let x_domain = (tick_values.first().copied().unwrap_or(-zmax), tick_values.last().copied().unwrap_or(zmax));
    let sx = Linear::new(x_domain, (0.0, 1.0));
    let axis = ValueAxis::new(ScaleKind::Linear, sorted.iter().copied(), annotations, false);
    let points = sorted
        .iter()
        .zip(&z)
        .map(|(&s, &t)| QqPoint { theoretical: t, sample: s, x: sx.map(t), y: axis.scale.map(s) })
        .collect();
    let mut reference = String::new();
    if !sorted.is_empty() {
        let end = |t: f64| (sx.map(t) * 100.0, (1.0 - axis.scale.map(mean + sd * t)) * 100.0);
        let ((x0, y0), (x1, y1)) = (end(x_domain.0), end(x_domain.1));
        let _ = write!(reference, "M{x0:.3} {y0:.3}L{x1:.3} {y1:.3}");
    }
    QqLayout {
        points,
        x_ticks: tick_values.iter().map(|&value| Gridline { value, fraction: sx.map(value) }).collect(),
        x_domain,
        reference,
        axis,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantiles_match_known_values() {
        assert!(normal_quantile(0.5).abs() < 1e-12);
        assert!((normal_quantile(0.975) - 1.959_963_985).abs() < 1e-8);
        assert!((normal_quantile(0.01) + 2.326_347_874).abs() < 1e-8);
        assert!(normal_quantile(0.0).is_infinite() && normal_quantile(1.5).is_nan());
    }

    #[test]
    fn points_pair_sorted_samples_with_quantiles() {
        let l = layout(&[3.0, 1.0, f64::NAN, 2.0], &[]);
        assert_eq!(l.points.len(), 3);
        assert_eq!(l.points[0].sample, 1.0);
        assert!(l.points[1].theoretical.abs() < 1e-12, "the median sits at z = 0");
        assert!((l.points[1].x - 0.5).abs() < 1e-12);
        assert!(l.reference.starts_with('M'));
        assert_eq!(layout(&[], &[]).reference, "");
    }
}
