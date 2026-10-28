//! Scalar fields on a grid: marching-squares contour lines and a normalised heat field.
#![allow(clippy::cast_precision_loss, reason = "grid sizes are small")]

use std::fmt::Write as _;

use crate::scale::{self, ticks};

type Pt = (f64, f64);

/// A contour at one level.
#[derive(Debug, Clone, PartialEq)]
pub struct Contour {
    /// The value the line follows.
    pub level: f64,
    /// SVG `d`: one `M…L…` segment per crossed cell.
    pub path: String,
}

/// Everything a contour or heat chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// viewBox width.
    pub width: f64,
    /// viewBox height.
    pub height: f64,
    /// Size of one (square) cell.
    pub cell: f64,
    /// Smallest and largest finite value.
    pub range: (f64, f64),
    /// Position of each value within the range (0..=1), row-major; `None` for gaps.
    pub heat: Vec<Option<f64>>,
    /// One per level, lowest first.
    pub contours: Vec<Contour>,
}

/// Contour levels strictly inside `(min, max)`: about `count` nice values.
#[must_use]
pub fn levels(min: f64, max: f64, count: usize) -> Vec<f64> {
    if max.partial_cmp(&min) != Some(std::cmp::Ordering::Greater) {
        return Vec::new();
    }
    ticks(min, max, count + 1).into_iter().filter(|&t| t > min && t < max).collect()
}

/// Marching squares for one `level` over `values` (row-major, `cols` × rows).
/// Values sit at cell centres; coordinates are scaled by `cell`. Cells with a gap are skipped.
#[must_use]
pub fn contour(values: &[f64], cols: usize, level: f64, cell: f64) -> String {
    let mut d = String::new();
    if cols < 2 {
        return d;
    }
    let rows = values.len() / cols;
    for r in 0..rows.saturating_sub(1) {
        for c in 0..cols - 1 {
            let (tl, tr) = (values[r * cols + c], values[r * cols + c + 1]);
            let (bl, br) = (values[(r + 1) * cols + c], values[(r + 1) * cols + c + 1]);
            if ![tl, tr, bl, br].iter().all(|v| v.is_finite()) {
                continue;
            }
            let case = usize::from(tl >= level)
                | usize::from(tr >= level) << 1
                | usize::from(br >= level) << 2
                | usize::from(bl >= level) << 3;
            let (x, y) = (c as f64, r as f64);
            let f = |a: f64, b: f64| {
                if (b - a).abs() < f64::EPSILON { 0.5 } else { ((level - a) / (b - a)).clamp(0.0, 1.0) }
            };
            let top = (x + f(tl, tr), y);
            let right = (x + 1.0, y + f(tr, br));
            let bottom = (x + f(bl, br), y + 1.0);
            let left = (x, y + f(tl, bl));
            let centre_high = (tl + tr + bl + br) / 4.0 >= level;
            let segs: &[(Pt, Pt)] = match case {
                1 | 14 => &[(left, top)],
                2 | 13 => &[(top, right)],
                3 | 12 => &[(left, right)],
                4 | 11 => &[(right, bottom)],
                5 if centre_high => &[(left, bottom), (top, right)],
                5 => &[(left, top), (right, bottom)],
                6 | 9 => &[(top, bottom)],
                7 | 8 => &[(left, bottom)],
                10 if centre_high => &[(left, top), (right, bottom)],
                10 => &[(top, right), (left, bottom)],
                _ => &[],
            };
            for &((x1, y1), (x2, y2)) in segs {
                let _ = write!(
                    d,
                    "M{:.2} {:.2}L{:.2} {:.2}",
                    (x1 + 0.5) * cell,
                    (y1 + 0.5) * cell,
                    (x2 + 0.5) * cell,
                    (y2 + 0.5) * cell
                );
            }
        }
    }
    d
}

/// Lay out a `cols`-wide grid of values with about `level_count` contour levels.
#[must_use]
pub fn layout(values: &[f64], cols: usize, level_count: usize) -> Field {
    let cols = cols.max(1);
    let rows = values.len().div_ceil(cols);
    let width = 600.0;
    let cell = width / cols as f64;
    let range = scale::extent(values.iter().copied()).unwrap_or((0.0, 0.0));
    let span = range.1 - range.0;
    let heat =
        values.iter().map(|&v| v.is_finite().then(|| if span > 0.0 { (v - range.0) / span } else { 0.5 })).collect();
    let mut padded = values.to_vec();
    padded.resize(rows * cols, f64::NAN);
    let contours = levels(range.0, range.1, level_count)
        .into_iter()
        .map(|level| Contour { level, path: contour(&padded, cols, level, cell) })
        .collect();
    Field { width, height: cell * rows as f64, cell, range, heat, contours }
}

/// The `(row, column)` under a pointer at `fx`, `fy` (0..=1 fractions across and down the
/// plot) on a `rows`×`cols` grid. Edges clamp into the grid.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "grid sizes are small"
)]
#[must_use]
pub fn cell_at(fx: f64, fy: f64, rows: usize, cols: usize) -> Option<(usize, usize)> {
    if rows == 0 || cols == 0 || !fx.is_finite() || !fy.is_finite() {
        return None;
    }
    let pick = |f: f64, n: usize| ((f.clamp(0.0, 1.0) * n as f64) as usize).min(n - 1);
    Some((pick(fy, rows), pick(fx, cols)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_fractions_pick_a_cell() {
        assert_eq!(cell_at(0.0, 0.0, 2, 4), Some((0, 0)));
        assert_eq!(cell_at(0.6, 0.6, 2, 4), Some((1, 2)));
        assert_eq!(cell_at(1.0, 1.2, 2, 4), Some((1, 3)));
        assert_eq!(cell_at(0.5, 0.5, 0, 4), None);
    }

    #[test]
    fn one_high_corner_cuts_a_single_segment() {
        let d = contour(&[1.0, 0.0, 0.0, 0.0], 2, 0.5, 10.0);
        assert_eq!(d.matches('M').count(), 1);
        assert_eq!(d, "M5.00 10.00L10.00 5.00");
    }

    #[test]
    fn flat_fields_have_no_contours() {
        let f = layout(&[3.0; 6], 3, 4);
        assert_eq!(f.contours, []);
        assert!((f.height - 400.0).abs() < 1e-9);
    }

    #[test]
    fn levels_stay_inside_the_range_and_heat_is_normalised() {
        let f = layout(&[0.0, 10.0, 20.0, 30.0], 2, 3);
        assert!(f.contours.iter().all(|c| c.level > 0.0 && c.level < 30.0 && !c.path.is_empty()));
        assert_eq!(f.heat, [Some(0.0), Some(1.0 / 3.0), Some(2.0 / 3.0), Some(1.0)]);
    }
}
