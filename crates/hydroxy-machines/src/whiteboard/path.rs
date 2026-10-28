//! Freehand ink: simplification, smoothing and storing points on a pen shape.

use std::fmt::Write as _;

use crate::canvas::segment_distance;

use super::{Point, Rect, Shape};

/// Ramer–Douglas–Peucker: drop points within `epsilon` of the line they sit on.
#[must_use]
pub fn simplify(points: &[Point], epsilon: f64) -> Vec<Point> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let (first, last) = (points[0], points[points.len() - 1]);
    let (mut index, mut far) = (0, 0.0);
    for (i, p) in points.iter().enumerate().take(points.len() - 1).skip(1) {
        let d = segment_distance(*p, first, last);
        if d > far {
            (index, far) = (i, d);
        }
    }
    if far <= epsilon {
        return vec![first, last];
    }
    let mut left = simplify(&points[..=index], epsilon);
    left.pop();
    left.extend(simplify(&points[index..], epsilon));
    left
}

/// A smooth SVG path through `points`: quadratic curves between midpoints.
#[must_use]
pub fn smooth_path(points: &[Point]) -> String {
    let mut d = String::new();
    match points {
        [] => {}
        [p] => {
            let _ = write!(d, "M{:.1} {:.1}l0.1 0", p.x, p.y);
        }
        [a, b] => {
            let _ = write!(d, "M{:.1} {:.1}L{:.1} {:.1}", a.x, a.y, b.x, b.y);
        }
        _ => {
            let _ = write!(d, "M{:.1} {:.1}", points[0].x, points[0].y);
            for w in points.windows(2).skip(1) {
                let m = w[0].mid(w[1]);
                let _ = write!(d, "Q{:.1} {:.1} {:.1} {:.1}", w[0].x, w[0].y, m.x, m.y);
            }
            let l = points[points.len() - 1];
            let _ = write!(d, "L{:.1} {:.1}", l.x, l.y);
        }
    }
    d
}

/// Store world-unit `ink` as a pen shape's box and fractional points.
pub(super) fn set_ink(s: &mut Shape, ink: &[Point]) {
    let Some(r) = Rect::around(ink.iter().copied()) else { return };
    let (w, h) = (r.w.max(1.0), r.h.max(1.0));
    (s.x, s.y, s.w, s.h) = (r.x, r.y, w, h);
    s.points = ink.iter().map(|p| Point::new((p.x - r.x) / w, (p.y - r.y) / h)).collect();
}
