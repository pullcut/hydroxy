//! Radial marks: a gauge arc and concentric progress rings. Arcs are stroked paths with
//! `pathLength="1"`, so a value is just a dash length and can animate. Angles are degrees
//! clockwise from 12 o'clock.

use std::fmt::Write as _;

fn polar(c: f64, r: f64, deg: f64) -> (f64, f64) {
    let a = deg.to_radians();
    (c + r * a.sin(), c - r * a.cos())
}

/// SVG `d` for an arc of radius `r` around `(c, c)` from `start` to `end` degrees.
/// A sweep of a full turn or more is drawn as two halves.
#[must_use]
pub fn arc_path(c: f64, r: f64, start: f64, end: f64) -> String {
    let sweep = (end - start).clamp(0.0, 360.0);
    let (x0, y0) = polar(c, r, start);
    let mut d = String::new();
    if sweep >= 360.0 {
        let (xm, ym) = polar(c, r, start + 180.0);
        let _ = write!(d, "M{x0:.3} {y0:.3}A{r:.3} {r:.3} 0 1 1 {xm:.3} {ym:.3}A{r:.3} {r:.3} 0 1 1 {x0:.3} {y0:.3}");
    } else {
        let (x1, y1) = polar(c, r, start + sweep);
        let large = i32::from(sweep > 180.0);
        let _ = write!(d, "M{x0:.3} {y0:.3}A{r:.3} {r:.3} 0 {large} 1 {x1:.3} {y1:.3}");
    }
    d
}

/// A colour that applies from `from` upwards.
#[derive(Debug, Clone, PartialEq)]
pub struct Threshold {
    /// The value at which this band starts.
    pub from: f64,
    /// Any CSS colour, usually a token such as `var(--status-danger)`.
    pub colour: String,
}

impl Threshold {
    /// A band starting at `from`.
    #[must_use]
    pub fn new(from: f64, colour: impl Into<String>) -> Self {
        Self { from, colour: colour.into() }
    }
}

/// Everything a gauge needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct GaugeLayout {
    /// Path of the whole track.
    pub track: String,
    /// Share of the track the value fills, 0..=1.
    pub fraction: f64,
    /// Colour of the band the value is in, if any threshold applies.
    pub colour: Option<String>,
    /// Where each threshold sits on the track as `(x1, y1, x2, y2)` tick marks across the arc.
    pub marks: Vec<(f64, f64, f64, f64)>,
    /// Radius of the track's centre line.
    pub radius: f64,
}

/// Total sweep of the gauge arc in degrees: three quarters of a circle, open at the bottom.
pub const GAUGE_SWEEP: f64 = 270.0;

/// A gauge in a `size`×`size` box with a track `thickness` px wide, showing `value`
/// on a `min..=max` scale. `thresholds` need not be sorted.
#[must_use]
pub fn gauge(value: f64, min: f64, max: f64, thickness: f64, size: f64, thresholds: &[Threshold]) -> GaugeLayout {
    let c = size / 2.0;
    let radius = ((size - thickness) / 2.0).max(1.0);
    let start = -GAUGE_SWEEP / 2.0;
    let span = max - min;
    let at = |v: f64| if span > 0.0 { ((v - min) / span).clamp(0.0, 1.0) } else { 0.0 };
    let value = if value.is_finite() { value } else { min };
    let colour = thresholds
        .iter()
        .filter(|t| t.from <= value)
        .max_by(|a, b| a.from.total_cmp(&b.from))
        .map(|t| t.colour.clone());
    let half = thickness / 2.0;
    let marks = thresholds
        .iter()
        .filter(|t| t.from > min && t.from < max)
        .map(|t| {
            let deg = start + at(t.from) * GAUGE_SWEEP;
            let (x1, y1) = polar(c, radius - half, deg);
            let (x2, y2) = polar(c, radius + half, deg);
            (x1, y1, x2, y2)
        })
        .collect();
    GaugeLayout { track: arc_path(c, radius, start, start + GAUGE_SWEEP), fraction: at(value), colour, marks, radius }
}

/// Where a tooltip pins to a radial mark: the point at `deg` degrees on a circle of
/// `radius` around the centre of a `size`×`size` box, as 0..=1 fractions from the left and top.
#[must_use]
pub fn anchor(size: f64, radius: f64, deg: f64) -> (f64, f64) {
    if size <= 0.0 {
        return (0.5, 0.5);
    }
    polar(0.5, radius / size, deg)
}

/// The angle of the tip of a gauge's value arc filled to `fraction`, for [`anchor`].
#[must_use]
pub fn gauge_angle(fraction: f64) -> f64 {
    -GAUGE_SWEEP / 2.0 + fraction.clamp(0.0, 1.0) * GAUGE_SWEEP
}

/// The band `value` falls in on a `min..=max` scale cut by `thresholds`: from the highest
/// threshold reached (or `min`) up to the next one (or `max`).
#[must_use]
pub fn band(value: f64, min: f64, max: f64, thresholds: &[Threshold]) -> (f64, f64) {
    let mut cuts: Vec<f64> = thresholds.iter().map(|t| t.from).filter(|f| *f > min && *f < max).collect();
    cuts.sort_by(f64::total_cmp);
    let lo = cuts.iter().copied().filter(|c| *c <= value).fold(min, f64::max);
    let hi = cuts.iter().copied().find(|c| *c > value).unwrap_or(max);
    (lo, hi)
}

/// One progress ring.
#[derive(Debug, Clone, PartialEq)]
pub struct RingMark {
    /// Full circle path, starting at 12 o'clock.
    pub path: String,
    /// Share filled, 0..=1.
    pub fraction: f64,
    /// Radius of the ring's centre line.
    pub radius: f64,
}

/// Concentric rings for `values`, the first outermost, each filled against `max`.
/// Rings are `thickness` px wide with `gap` px between them; rings that would not fit are dropped.
#[must_use]
pub fn rings(values: &[f64], max: f64, size: f64, thickness: f64, gap: f64) -> Vec<RingMark> {
    let c = size / 2.0;
    values
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| {
            #[allow(clippy::cast_precision_loss, reason = "ring counts are tiny")]
            let radius = c - thickness / 2.0 - i as f64 * (thickness + gap);
            (radius > thickness / 2.0).then(|| RingMark {
                path: arc_path(c, radius, 0.0, 360.0),
                fraction: if max > 0.0 && v.is_finite() { (v / max).clamp(0.0, 1.0) } else { 0.0 },
                radius,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arcs_start_at_the_requested_angle() {
        assert!(arc_path(50.0, 40.0, 0.0, 90.0).starts_with("M50.000 10.000A40.000 40.000 0 0 1 90.000 50.000"));
        assert_eq!(arc_path(50.0, 40.0, 0.0, 360.0).matches('A').count(), 2);
    }

    #[test]
    fn gauge_picks_the_highest_band_reached() {
        let t = [Threshold::new(0.0, "a"), Threshold::new(60.0, "b"), Threshold::new(90.0, "c")];
        let g = gauge(75.0, 0.0, 100.0, 12.0, 160.0, &t);
        assert_eq!(g.colour.as_deref(), Some("b"));
        assert!((g.fraction - 0.75).abs() < 1e-9);
        assert_eq!(g.marks.len(), 2);
        assert_eq!(gauge(f64::NAN, 0.0, 100.0, 12.0, 160.0, &[]).colour, None);
    }

    #[test]
    fn gauge_clamps_out_of_range_values() {
        assert!((gauge(500.0, 0.0, 100.0, 12.0, 160.0, &[]).fraction - 1.0).abs() < 1e-9);
        assert!(gauge(5.0, 3.0, 3.0, 12.0, 160.0, &[]).fraction.abs() < 1e-9);
    }

    #[test]
    fn anchors_and_bands() {
        let (x, y) = anchor(100.0, 40.0, 90.0);
        assert!((x - 0.9).abs() < 1e-9 && (y - 0.5).abs() < 1e-9);
        assert!(gauge_angle(0.5).abs() < 1e-9);
        assert!((gauge_angle(2.0) - 135.0).abs() < 1e-9);
        let t = [Threshold::new(60.0, "b"), Threshold::new(90.0, "c")];
        assert_eq!(band(75.0, 0.0, 100.0, &t), (60.0, 90.0));
        assert_eq!(band(10.0, 0.0, 100.0, &t), (0.0, 60.0));
        assert_eq!(band(95.0, 0.0, 100.0, &t), (90.0, 100.0));
    }

    #[test]
    fn rings_step_inwards_and_drop_when_out_of_room() {
        let r = rings(&[50.0, 100.0], 100.0, 120.0, 12.0, 4.0);
        assert!((r[0].radius - 54.0).abs() < 1e-9 && (r[1].radius - 38.0).abs() < 1e-9);
        assert!((r[0].fraction - 0.5).abs() < 1e-9);
        assert_eq!(rings(&[1.0; 10], 1.0, 60.0, 12.0, 4.0).len(), 2);
    }
}
