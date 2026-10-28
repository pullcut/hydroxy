//! Hexagonal binning: group planar points into pointy-top hexagons of a given radius,
//! using axial coordinates and cube rounding.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::geo::GeoPoint;

const SQRT3: f64 = 1.732_050_807_568_877_2;

/// One occupied hexagon.
#[derive(Debug, Clone, PartialEq)]
pub struct Bin {
    /// Centre x.
    pub x: f64,
    /// Centre y.
    pub y: f64,
    /// Indices of the points inside, in input order.
    pub members: Vec<usize>,
}

/// The axial cell `(q, r)` that holds `(x, y)`.
#[allow(clippy::cast_possible_truncation, reason = "cell indices are small")]
#[allow(clippy::many_single_char_names, reason = "the usual axial and cube names")]
fn cell(x: f64, y: f64, radius: f64) -> (i64, i64) {
    let q = (SQRT3 / 3.0 * x - y / 3.0) / radius;
    let r = (2.0 / 3.0 * y) / radius;
    let s = -q - r;
    let (mut rq, mut rr, rs) = (q.round(), r.round(), s.round());
    let (dq, dr, ds) = ((rq - q).abs(), (rr - r).abs(), (rs - s).abs());
    if dq > dr && dq > ds {
        rq = -rr - rs;
    } else if dr > ds {
        rr = -rq - rs;
    }
    (rq as i64, rr as i64)
}

/// Bin `points` (skipping `None`) into hexagons of `radius` (centre to corner). Bins come
/// back in reading order: top row first, left to right.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "cell indices are small")]
pub fn bin(points: &[Option<(f64, f64)>], radius: f64) -> Vec<Bin> {
    if radius <= 0.0 || !radius.is_finite() {
        return Vec::new();
    }
    let mut cells: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (i, p) in points.iter().enumerate() {
        if let Some((x, y)) = *p {
            cells.entry(cell(x, y, radius)).or_default().push(i);
        }
    }
    let mut keys: Vec<(i64, i64)> = cells.keys().copied().collect();
    keys.sort_by_key(|&(q, r)| (r, 2 * q + r));
    keys.into_iter()
        .map(|(q, r)| Bin {
            x: radius * SQRT3 * (q as f64 + r as f64 / 2.0),
            y: radius * 1.5 * r as f64,
            members: cells.remove(&(q, r)).unwrap_or_default(),
        })
        .collect()
}

/// SVG `d` for a pointy-top hexagon centred on `(x, y)`.
#[must_use]
pub fn hex_path(x: f64, y: f64, radius: f64) -> String {
    let mut d = String::new();
    for k in 0..6 {
        let a = (60.0 * f64::from(k) - 30.0).to_radians();
        let _ = write!(d, "{}{:.2} {:.2}", if k == 0 { "M" } else { "L" }, x + radius * a.cos(), y + radius * a.sin());
    }
    d.push('Z');
    d
}

/// A bin ready to draw and describe.
#[derive(Debug, Clone, PartialEq)]
pub struct HexCell {
    /// SVG `d` for the hexagon.
    pub path: String,
    /// Centre x.
    pub x: f64,
    /// Centre y.
    pub y: f64,
    /// Number of points inside.
    pub count: usize,
    /// Sum of the points' finite values.
    pub total: f64,
    /// Shade: count over the fullest bin's count (0..=1).
    pub t: f64,
    /// "First name" or "First name and 3 more".
    pub title: String,
    /// Every point's name, comma separated, for the table.
    pub names: String,
}

/// Bin `points`, already placed at `placed` (from `geo::layout`), into hexagons and work
/// out each one's shade, total and names.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "counts are small")]
pub fn cells(points: &[GeoPoint], placed: &[Option<(f64, f64)>], radius: f64) -> Vec<HexCell> {
    let bins = bin(placed, radius);
    let most = bins.iter().map(|b| b.members.len()).max().unwrap_or(1).max(1);
    bins.into_iter()
        .map(|b| {
            let names: Vec<&str> = b.members.iter().map(|&m| points[m].name.as_str()).collect();
            let count = names.len();
            let title = if count == 1 { names[0].to_string() } else { format!("{} and {} more", names[0], count - 1) };
            HexCell {
                path: hex_path(b.x, b.y, radius),
                x: b.x,
                y: b.y,
                count,
                total: b.members.iter().map(|&m| points[m].value).filter(|v| v.is_finite()).sum(),
                t: count as f64 / most as f64,
                title,
                names: names.join(", "),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearby_points_share_a_bin_and_far_ones_do_not() {
        let pts = [Some((0.0, 0.0)), Some((1.0, 1.0)), None, Some((40.0, 0.0))];
        let bins = bin(&pts, 10.0);
        assert_eq!(bins.len(), 2);
        assert_eq!(bins[0].members, vec![0, 1]);
        assert_eq!(bins[1].members, vec![3]);
    }

    #[test]
    fn every_point_lands_in_the_nearest_centre() {
        let pts: Vec<Option<(f64, f64)>> =
            (0..200).map(|i| Some((f64::from(i * 37 % 97), f64::from(i * 53 % 89)))).collect();
        let bins = bin(&pts, 8.0);
        assert_eq!(bins.iter().map(|b| b.members.len()).sum::<usize>(), 200);
        for b in &bins {
            for &m in &b.members {
                let (x, y) = pts[m].unwrap();
                let d = (x - b.x).hypot(y - b.y);
                assert!(
                    bins.iter().all(|o| (x - o.x).hypot(y - o.y) >= d - 1e-9),
                    "point {m} is nearest its own centre"
                );
            }
        }
    }

    #[test]
    fn bins_read_top_to_bottom_left_to_right() {
        let bins = bin(&[Some((50.0, 50.0)), Some((0.0, 50.0)), Some((0.0, 0.0))], 5.0);
        let order: Vec<usize> = bins.iter().map(|b| b.members[0]).collect();
        assert_eq!(order, vec![2, 1, 0]);
    }

    #[test]
    fn cells_count_total_and_shade() {
        let pts = [
            GeoPoint::new("Leeds", (0.0, 0.0), 3.0),
            GeoPoint::new("York", (0.0, 0.0), f64::NAN),
            GeoPoint::new("Hull", (0.0, 0.0), 2.0),
        ];
        let c = cells(&pts, &[Some((0.0, 0.0)), Some((1.0, 0.0)), Some((90.0, 0.0))], 10.0);
        assert_eq!((c[0].count, c[0].total, c[0].t), (2, 3.0, 1.0));
        assert_eq!(c[0].title, "Leeds and 1 more");
        assert_eq!(c[1].names, "Hull");
        assert!((c[1].t - 0.5).abs() < 1e-9);
    }

    #[test]
    fn hexagon_has_six_corners() {
        let d = hex_path(0.0, 0.0, 10.0);
        assert_eq!(d.matches('L').count(), 5);
        assert!(d.starts_with("M8.66 -5.00"));
    }
}
