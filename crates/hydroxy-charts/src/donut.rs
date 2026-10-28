//! Donut segments as SVG arcs along the ring's centre line, drawn with a stroke as
//! wide as the ring. A stroke (rather than a filled shape) is what lets a segment
//! sweep in from its own start angle: animate `stroke-dashoffset` with
//! `pathLength="1"`. Angles start at 12 o'clock and run clockwise.

use std::f64::consts::{PI, TAU};

use crate::geom::{arc_d, polar};

/// One ring segment.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// SVG `d` for the arc along the ring's centre line. Stroke it with
    /// [`Ring::stroke_width`], no fill, butt caps.
    pub path: String,
    /// Share of the total, 0..=1.
    pub fraction: f64,
}

/// The ring the segments sit on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ring {
    /// Radius of the centre line.
    pub radius: f64,
    /// Stroke width: the ring's thickness.
    pub stroke_width: f64,
}

/// The ring for a `size`×`size` box with a ring `thickness` px wide.
#[must_use]
pub fn ring(size: f64, thickness: f64) -> Ring {
    let thickness = thickness.min(size / 2.0).max(0.0);
    Ring { radius: (size - thickness) / 2.0, stroke_width: thickness }
}

/// Segments for `values` on [`ring`]`(size, thickness)`, with a `gap` px space between
/// neighbouring segments (the surface shows through).
#[must_use]
pub fn segments(values: &[f64], size: f64, thickness: f64, gap: f64) -> Vec<Segment> {
    let clean: Vec<f64> = values.iter().map(|v| if v.is_finite() { v.max(0.0) } else { 0.0 }).collect();
    let total: f64 = clean.iter().sum();
    let c = size / 2.0;
    let r = ring(size, thickness).radius;
    let nonzero = clean.iter().filter(|v| **v > 0.0).count();
    // Half the gap, as an angle on the centre line.
    let half_gap = if nonzero > 1 { (gap / 2.0) / r.max(1.0) } else { 0.0 };

    let mut start = 0.0;
    clean
        .iter()
        .map(|&v| {
            let fraction = if total > 0.0 { v / total } else { 0.0 };
            let sweep = fraction * TAU;
            let path = if fraction <= 0.0 {
                String::new()
            } else if nonzero == 1 {
                // A full ring: two half arcs in one path, since one arc can't close on itself.
                let (x, y) = polar(c, r, TAU);
                format!("{}A{r:.3} {r:.3} 0 0 1 {x:.3} {y:.3}", arc_d(c, r, 0.0, PI))
            } else {
                let (a, b) = (start + half_gap, start + sweep - half_gap);
                if b > a { arc_d(c, r, a, b) } else { String::new() }
            };
            start += sweep;
            Segment { path, fraction }
        })
        .collect()
}

/// Where segment `index` sits for a tooltip: the middle of its arc on the ring's centre
/// line, as 0..=1 fractions of the box from the left and top. `None` for a missing or
/// empty segment.
#[must_use]
pub fn anchor(values: &[f64], index: usize, size: f64, thickness: f64) -> Option<(f64, f64)> {
    let clean: Vec<f64> = values.iter().map(|v| if v.is_finite() { v.max(0.0) } else { 0.0 }).collect();
    let total: f64 = clean.iter().sum();
    let v = *clean.get(index)?;
    if total <= 0.0 || v <= 0.0 || size <= 0.0 {
        return None;
    }
    let before: f64 = clean[..index].iter().sum();
    let mid = (before + v / 2.0) / total * TAU;
    Some(polar(0.5, ring(size, thickness).radius / size, mid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors_sit_mid_arc() {
        // Two halves: the first is centred at 3 o'clock, the second at 9 o'clock.
        let (x, y) = anchor(&[1.0, 1.0], 0, 100.0, 10.0).unwrap();
        assert!((x - 0.95).abs() < 1e-9 && (y - 0.5).abs() < 1e-9);
        let (x, _) = anchor(&[1.0, 1.0], 1, 100.0, 10.0).unwrap();
        assert!((x - 0.05).abs() < 1e-9);
        assert_eq!(anchor(&[0.0, 1.0], 0, 100.0, 10.0), None);
        assert_eq!(anchor(&[1.0], 3, 100.0, 10.0), None);
    }

    #[test]
    fn fractions_sum_to_one_and_paths_are_open_arcs() {
        let s = segments(&[1.0, 1.0, 2.0], 180.0, 22.0, 2.0);
        let sum: f64 = s.iter().map(|x| x.fraction).sum();
        assert!((sum - 1.0).abs() < 1e-9);
        assert!(s.iter().all(|x| x.path.starts_with('M') && x.path.contains('A') && !x.path.contains('Z')));
    }

    #[test]
    fn large_arc_flag_only_past_half() {
        let big = segments(&[3.0, 1.0], 180.0, 22.0, 2.0);
        assert!(big[0].path.contains(" 0 1 1 "));
        assert!(big[1].path.contains(" 0 0 1 "));
    }

    #[test]
    fn first_segment_starts_at_twelve_oclock_on_the_centre_line() {
        // size 100, thickness 10: centre line radius 45, so 12 o'clock is (50, 5).
        let s = segments(&[1.0, 1.0], 100.0, 10.0, 0.0);
        assert!(s[0].path.starts_with("M50.000 5.000"));
        assert_eq!(ring(100.0, 10.0), Ring { radius: 45.0, stroke_width: 10.0 });
    }

    #[test]
    fn a_single_value_draws_a_full_ring_and_zeros_draw_nothing() {
        let s = segments(&[0.0, 5.0], 100.0, 10.0, 2.0);
        assert_eq!(s[0].path, String::new());
        assert_eq!(s[1].path.matches('A').count(), 2);
    }
}
