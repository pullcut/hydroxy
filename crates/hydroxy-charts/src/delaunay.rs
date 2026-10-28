//! Delaunay triangulation (Bowyer-Watson) and the Voronoi cells it implies. A scatter
//! uses the cells to show which dot each part of the plot picks on hover.

#![allow(clippy::many_single_char_names, reason = "geometry follows the published formulas")]

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

type P = (f64, f64);

#[derive(Clone, Copy)]
struct Tri {
    v: [usize; 3],
    /// Circumcentre and squared radius.
    c: (f64, f64, f64),
}

fn circle(a: P, b: P, c: P) -> (f64, f64, f64) {
    let d = 2.0 * (a.0 * (b.1 - c.1) + b.0 * (c.1 - a.1) + c.0 * (a.1 - b.1));
    if d.abs() < 1e-12 {
        return (0.0, 0.0, f64::INFINITY);
    }
    let (a2, b2, c2) = (a.0 * a.0 + a.1 * a.1, b.0 * b.0 + b.1 * b.1, c.0 * c.0 + c.1 * c.1);
    let x = (a2 * (b.1 - c.1) + b2 * (c.1 - a.1) + c2 * (a.1 - b.1)) / d;
    let y = (a2 * (c.0 - b.0) + b2 * (a.0 - c.0) + c2 * (b.0 - a.0)) / d;
    (x, y, (a.0 - x).powi(2) + (a.1 - y).powi(2))
}

/// Every triangle, including those touching the three super-triangle corners at
/// indices `n..n + 3`. Exact duplicate points are skipped.
// O(n²) Bowyer-Watson; fine for the few thousand points a scatter draws as
// elements. Swap for a sweep-hull if it ever runs on canvas-sized clouds.
fn all_triangles(points: &[P]) -> Vec<Tri> {
    let n = points.len();
    let (mut lo, mut hi) = ((f64::INFINITY, f64::INFINITY), (f64::NEG_INFINITY, f64::NEG_INFINITY));
    for p in points {
        lo = (lo.0.min(p.0), lo.1.min(p.1));
        hi = (hi.0.max(p.0), hi.1.max(p.1));
    }
    if n == 0 || !lo.0.is_finite() {
        return Vec::new();
    }
    let span = (hi.0 - lo.0).max(hi.1 - lo.1).max(1.0) * 1000.0;
    let mid = (f64::midpoint(lo.0, hi.0), f64::midpoint(lo.1, hi.1));
    let mut pts: Vec<P> = points.to_vec();
    pts.extend([(mid.0 - 2.0 * span, mid.1 - span), (mid.0 + 2.0 * span, mid.1 - span), (mid.0, mid.1 + 2.0 * span)]);
    let make = |v: [usize; 3], pts: &[P]| Tri { v, c: circle(pts[v[0]], pts[v[1]], pts[v[2]]) };
    let mut tris = vec![make([n, n + 1, n + 2], &pts)];
    let mut seen = HashSet::new();
    for i in 0..n {
        let p = pts[i];
        if !seen.insert((p.0.to_bits(), p.1.to_bits())) {
            continue;
        }
        let (bad, keep): (Vec<Tri>, Vec<Tri>) = tris.into_iter().partition(|t| (p.0 - t.c.0).powi(2) + (p.1 - t.c.1).powi(2) < t.c.2);
        tris = keep;
        // The hole's boundary: edges of exactly one bad triangle.
        let mut count: HashMap<(usize, usize), (usize, usize, u8)> = HashMap::new();
        for t in &bad {
            for k in 0..3 {
                let (a, b) = (t.v[k], t.v[(k + 1) % 3]);
                count.entry((a.min(b), a.max(b))).and_modify(|e| e.2 += 1).or_insert((a, b, 1));
            }
        }
        for &(a, b, c) in count.values() {
            if c == 1 {
                tris.push(make([a, b, i], &pts));
            }
        }
    }
    tris
}

/// Delaunay triangles over `points`, as index triples. Exact duplicates are skipped and
/// all-collinear input has no triangles.
#[must_use]
pub fn triangulate(points: &[P]) -> Vec<[usize; 3]> {
    let n = points.len();
    all_triangles(points).into_iter().filter(|t| t.v.iter().all(|&v| v < n)).map(|t| t.v).collect()
}

/// Clip `poly` to the side of the bisector of `a`–`b` nearer `a`.
fn clip(poly: &[P], a: P, b: P) -> Vec<P> {
    let (m, d) = ((f64::midpoint(a.0, b.0), f64::midpoint(a.1, b.1)), (b.0 - a.0, b.1 - a.1));
    let side = |q: P| (q.0 - m.0) * d.0 + (q.1 - m.1) * d.1;
    let mut out = Vec::with_capacity(poly.len() + 1);
    for k in 0..poly.len() {
        let (p, q) = (poly[k], poly[(k + 1) % poly.len()]);
        let (sp, sq) = (side(p), side(q));
        if sp <= 0.0 {
            out.push(p);
        }
        if (sp < 0.0 && sq > 0.0) || (sp > 0.0 && sq < 0.0) {
            let t = sp / (sp - sq);
            out.push((p.0 + t * (q.0 - p.0), p.1 + t * (q.1 - p.1)));
        }
    }
    out
}

/// The Voronoi cell of each point inside the `width`×`height` box from the origin: the
/// region nearer that point than any other. A skipped duplicate gets an empty cell.
#[must_use]
pub fn voronoi(points: &[P], width: f64, height: f64) -> Vec<Vec<P>> {
    let n = points.len();
    let mut near: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for t in all_triangles(points) {
        for k in 0..3 {
            let (a, b) = (t.v[k], t.v[(k + 1) % 3]);
            if a < n && b < n {
                near[a].insert(b);
                near[b].insert(a);
            }
        }
    }
    let mut first = HashMap::new();
    (0..n)
        .map(|i| {
            let p = points[i];
            if *first.entry((p.0.to_bits(), p.1.to_bits())).or_insert(i) != i {
                return Vec::new();
            }
            // A lone point, or one in an all-collinear row, still splits the box with its neighbours.
            let others: Vec<usize> = if near[i].is_empty() { (0..n).filter(|&j| points[j] != p).collect() } else { near[i].iter().copied().collect() };
            others.iter().fold(vec![(0.0, 0.0), (width, 0.0), (width, height), (0.0, height)], |cell, &j| clip(&cell, p, points[j]))
        })
        .collect()
}

/// SVG `d` for a polygon; empty for an empty one.
#[must_use]
pub fn path(poly: &[P]) -> String {
    let mut d = String::new();
    for (k, p) in poly.iter().enumerate() {
        let _ = write!(d, "{}{:.1} {:.1}", if k == 0 { 'M' } else { 'L' }, p.0, p.1);
    }
    if !d.is_empty() {
        d.push('Z');
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cloud() -> Vec<P> {
        let mut s = 12_345u64;
        (0..60)
            .map(|_| {
                let mut r = || {
                    s ^= s << 13;
                    s ^= s >> 7;
                    s ^= s << 17;
                    #[allow(clippy::cast_precision_loss, reason = "test noise")]
                    let v = (s % 10_000) as f64 / 100.0;
                    v
                };
                (r(), r())
            })
            .collect()
    }

    #[test]
    fn no_point_inside_any_circumcircle() {
        let pts = cloud();
        let tris = triangulate(&pts);
        assert!(tris.len() > 50, "{} triangles", tris.len());
        for t in &tris {
            let (cx, cy, r2) = circle(pts[t[0]], pts[t[1]], pts[t[2]]);
            for (i, p) in pts.iter().enumerate() {
                if !t.contains(&i) {
                    assert!((p.0 - cx).powi(2) + (p.1 - cy).powi(2) >= r2 - 1e-6, "point {i} inside {t:?}");
                }
            }
        }
    }

    #[test]
    fn grid_triangle_count_matches_euler() {
        let pts: Vec<P> = (0..3).flat_map(|y| (0..3).map(move |x| (f64::from(x), f64::from(y)))).collect();
        // 2n - h - 2 triangles for n points with h on the hull.
        assert_eq!(triangulate(&pts).len(), 2 * 9 - 8 - 2);
        assert_eq!(triangulate(&[(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)]), Vec::<[usize; 3]>::new());
        assert_eq!(triangulate(&[]), Vec::<[usize; 3]>::new());
    }

    #[test]
    fn cells_tile_the_box_and_hold_their_point() {
        let pts = cloud();
        let cells = voronoi(&pts, 100.0, 100.0);
        let area = |c: &Vec<P>| (0..c.len()).map(|k| c[k].0 * c[(k + 1) % c.len()].1 - c[(k + 1) % c.len()].0 * c[k].1).sum::<f64>().abs() / 2.0;
        let total: f64 = cells.iter().map(area).sum();
        assert!((total - 10_000.0).abs() < 1e-6, "{total}");
        for (p, c) in pts.iter().zip(&cells) {
            let inside = (0..c.len()).all(|k| {
                let (a, b) = (c[k], c[(k + 1) % c.len()]);
                (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0) >= -1e-9
            });
            assert!(inside, "{p:?}");
        }
    }

    #[test]
    fn duplicates_and_collinear_rows_still_split_the_box() {
        let cells = voronoi(&[(10.0, 50.0), (10.0, 50.0), (50.0, 50.0), (90.0, 50.0)], 100.0, 100.0);
        assert_eq!(cells[1], Vec::<(f64, f64)>::new());
        assert_eq!(path(&cells[0]), "M0.0 0.0L30.0 0.0L30.0 100.0L0.0 100.0Z");
        assert_eq!(path(&[]), "");
    }
}
