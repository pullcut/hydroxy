//! Smith chart: impedance samples as reflection coefficients inside the unit disc,
//! with the constant-resistance and constant-reactance grid.

use std::fmt::Write as _;

/// Normalised grid values, as on a printed chart.
const FAMILY: [f64; 5] = [0.2, 0.5, 1.0, 2.0, 5.0];
const ARC_STEPS: usize = 48;

/// One impedance sample.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// Trace name. Every sample sharing a name is one swept trace.
    pub trace: String,
    /// Resistance (normalised, or ohms when `z0` is given).
    pub real: f64,
    /// Reactance, same units.
    pub imaginary: f64,
    /// Optional label, such as a frequency. First and last labels name the sweep ends.
    pub label: Option<String>,
}

impl Sample {
    /// A sample on `trace`.
    #[must_use]
    pub fn new(trace: impl Into<String>, real: f64, imaginary: f64) -> Self {
        Self { trace: trace.into(), real, imaginary, label: None }
    }

    /// The same sample with a label.
    #[must_use]
    pub fn labelled(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// A swept trace.
#[derive(Debug, Clone, PartialEq)]
pub struct Trace {
    /// Trace name.
    pub name: String,
    /// Sample positions in drawing coordinates.
    pub points: Vec<(f64, f64)>,
    /// Source samples, parallel to `points`.
    pub samples: Vec<Sample>,
    /// SVG `d` joining the points; empty for a single sample.
    pub path: String,
}

/// Everything a Smith chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct Smith {
    /// viewBox width.
    pub width: f64,
    /// viewBox height.
    pub height: f64,
    /// Centre of the disc.
    pub centre: (f64, f64),
    /// Radius of the boundary circle.
    pub radius: f64,
    /// Constant-resistance circles as `(cx, cy, r)`.
    pub resistance: Vec<(f64, f64, f64)>,
    /// Constant-reactance arcs as SVG `d`.
    pub reactance: Vec<String>,
    /// Traces in first-seen order.
    pub traces: Vec<Trace>,
}

/// Reflection coefficient Γ = (Z − Z₀)/(Z + Z₀). |Γ| ≤ 1 for any passive impedance.
#[must_use]
pub fn gamma(real: f64, imaginary: f64, z0: f64) -> (f64, f64) {
    let den = (real + z0).powi(2) + imaginary.powi(2);
    let den = if den == 0.0 { 1.0 } else { den };
    ((real * real + imaginary * imaginary - z0 * z0) / den, 2.0 * z0 * imaginary / den)
}

fn polyline(points: &[(f64, f64)]) -> String {
    let mut d = String::new();
    for (i, (x, y)) in points.iter().enumerate() {
        let _ = write!(d, "{}{x:.2} {y:.2}", if i == 0 { "M" } else { "L" });
    }
    d
}

/// Lay out a Smith chart. `z0` is the reference impedance the samples are measured
/// against (1 means already normalised). Hecto's rule: a wrong `z0` collapses samples onto Γ = −1.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "step counts are small")]
pub fn layout(samples: &[Sample], z0: f64) -> Smith {
    let (width, height): (f64, f64) = (460.0, 380.0);
    let z0 = if z0.is_finite() && z0 > 0.0 { z0 } else { 1.0 };
    let radius = (width / 2.0 - 34.0).min(height / 2.0 - 30.0).max(20.0);
    let (cx, cy) = (width / 2.0, height / 2.0);
    let at = |(gx, gy): (f64, f64)| (cx + gx * radius, cy - gy * radius);
    let resistance = FAMILY.iter().map(|&r| (at((r / (r + 1.0), 0.0)).0, cy, radius / (r + 1.0))).collect();
    let mut reactance = Vec::new();
    for x in FAMILY {
        for sign in [-1.0, 1.0] {
            // Sweep r from 0 to ∞ so every point comes back through `gamma` and the arc ends on the boundary.
            let pts: Vec<(f64, f64)> = (0..=ARC_STEPS)
                .map(|i| {
                    let r = (i as f64 / ARC_STEPS as f64 * std::f64::consts::FRAC_PI_2).tan();
                    at(if r.is_finite() && r < 1e12 { gamma(r, sign * x, 1.0) } else { (1.0, 0.0) })
                })
                .collect();
            reactance.push(polyline(&pts));
        }
    }
    let mut traces: Vec<Trace> = Vec::new();
    for s in samples {
        let p = at(gamma(s.real, s.imaginary, z0));
        match traces.iter_mut().find(|t| t.name == s.trace) {
            Some(t) => {
                t.points.push(p);
                t.samples.push(s.clone());
            }
            None => traces.push(Trace { name: s.trace.clone(), points: vec![p], samples: vec![s.clone()], path: String::new() }),
        }
    }
    for t in &mut traces {
        if t.points.len() >= 2 {
            t.path = polyline(&t.points);
        }
    }
    Smith { width, height, centre: (cx, cy), radius, resistance, reactance, traces }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matched_load_is_the_centre_and_open_circuit_the_right_edge() {
        let (x, y) = gamma(1.0, 0.0, 1.0);
        assert!(x.abs() < 1e-12 && y.abs() < 1e-12);
        assert!((gamma(1e9, 0.0, 1.0).0 - 1.0).abs() < 1e-6);
        assert!((gamma(0.0, 0.0, 1.0).0 + 1.0).abs() < 1e-12);
    }

    #[test]
    fn z0_normalises_raw_ohms() {
        assert_eq!(gamma(50.0, 25.0, 50.0), gamma(1.0, 0.5, 1.0));
    }

    #[test]
    fn samples_group_into_traces_and_stay_inside_the_disc() {
        let s = layout(&[Sample::new("a", 0.5, 0.5), Sample::new("b", 2.0, -1.0), Sample::new("a", 1.0, 2.0)], 1.0);
        assert_eq!(s.traces.len(), 2);
        assert_eq!(s.traces[0].points.len(), 2);
        assert_eq!(s.traces[1].path, "");
        for t in &s.traces {
            for &(x, y) in &t.points {
                assert!((x - s.centre.0).hypot(y - s.centre.1) <= s.radius + 1e-9);
            }
        }
        assert_eq!(s.reactance.len(), 10);
    }
}
