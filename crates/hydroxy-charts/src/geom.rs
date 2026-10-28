//! Polar geometry shared by the round charts: a point on a circle and the SVG paths of an
//! arc and an annular sector. Angles are radians clockwise from twelve o'clock.

use std::f64::consts::{PI, TAU};
use std::fmt::Write as _;

/// The point at radius `r` and `angle` around the centre `(c, c)`, with y down.
#[must_use]
pub fn polar(c: f64, r: f64, angle: f64) -> (f64, f64) {
    (c + r * angle.sin(), c - r * angle.cos())
}

/// SVG `d` for the open arc of radius `r` around `(c, c)` from `start` to `end`.
/// Sweeps of a full turn or more need two arcs; see [`crate::donut`].
#[must_use]
pub fn arc_d(c: f64, r: f64, start: f64, end: f64) -> String {
    let large = i32::from(end - start > PI);
    let (x0, y0) = polar(c, r, start);
    let (x1, y1) = polar(c, r, end);
    let mut d = String::new();
    let _ = write!(d, "M{x0:.3} {y0:.3}A{r:.3} {r:.3} 0 {large} 1 {x1:.3} {y1:.3}");
    d
}

/// SVG `d` for the annular sector between radii `r0` and `r1` and angles `a0` and `a1`
/// around `(c, c)`. One arc can't close on itself, so a full turn is drawn just short of it.
#[must_use]
pub fn sector_d(c: f64, r0: f64, r1: f64, a0: f64, a1: f64) -> String {
    let a1 = a1.min(a0 + TAU - 1e-4);
    let large = i32::from(a1 - a0 > PI);
    let (ox0, oy0) = polar(c, r1, a0);
    let (ox1, oy1) = polar(c, r1, a1);
    let (ix1, iy1) = polar(c, r0, a1);
    let (ix0, iy0) = polar(c, r0, a0);
    let mut d = String::new();
    let _ = write!(
        d,
        "M{ox0:.3} {oy0:.3}A{r1:.3} {r1:.3} 0 {large} 1 {ox1:.3} {oy1:.3}L{ix1:.3} {iy1:.3}A{r0:.3} {r0:.3} 0 {large} 0 {ix0:.3} {iy0:.3}Z"
    );
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polar_starts_at_twelve_and_runs_clockwise() {
        let (x, y) = polar(50.0, 10.0, 0.0);
        assert!((x - 50.0).abs() < 1e-9 && (y - 40.0).abs() < 1e-9);
        let (x, y) = polar(50.0, 10.0, PI / 2.0);
        assert!((x - 60.0).abs() < 1e-9 && (y - 50.0).abs() < 1e-9);
    }

    #[test]
    fn arcs_and_sectors_are_closed_correctly() {
        assert_eq!(arc_d(50.0, 10.0, 0.0, PI / 2.0), "M50.000 40.000A10.000 10.000 0 0 1 60.000 50.000");
        assert!(arc_d(50.0, 10.0, 0.0, 4.0).contains(" 0 1 1 "), "over half a turn takes the large arc");
        let d = sector_d(50.0, 5.0, 10.0, 0.0, PI / 2.0);
        assert!(d.starts_with(
            "M50.000 40.000A10.000 10.000 0 0 1 60.000 50.000L55.000 50.000A5.000 5.000 0 0 0 50.000 45.000"
        ));
        assert!(d.ends_with('Z'));
        let full = sector_d(50.0, 5.0, 10.0, 0.0, TAU);
        assert!(full.contains("M50.000 40.000"), "a full turn still draws");
        assert!(!full.contains("NaN"));
    }
}
