//! Parallel coordinates: one vertical axis per dimension, each with its own scale, and one
//! polyline per row crossing them all. Brushes on the axes filter rows; a row stays in
//! only when it is inside every brush.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "pixel maths on small, clamped sizes"
)]

use crate::plot::ValueAxis;
use crate::raster::{Raster, Rgba};
use crate::scale::ScaleKind;

/// One axis: a name and a value per row.
#[derive(Debug, Clone, PartialEq)]
pub struct Dimension {
    /// The axis title, such as "Price (£)".
    pub name: String,
    /// One value per row. Non-finite values leave a gap in that row's line.
    pub values: Vec<f64>,
}

impl Dimension {
    /// A named axis.
    #[must_use]
    pub fn new(name: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        Self { name: name.into(), values: values.into_iter().collect() }
    }
}

/// A brush on one axis: the kept range as fractions of the axis, bottom to top.
pub type Range = (f64, f64);

/// Everything parallel coordinates need to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct ParallelLayout {
    /// Each axis's position across the plot, 0..=1.
    pub xs: Vec<f64>,
    /// Each axis's own scale and ticks, fitted to its data (not to zero).
    pub axes: Vec<ValueAxis>,
    /// Each row's height on each axis, 0..=1 from the bottom; `None` for a gap.
    pub ys: Vec<Vec<Option<f64>>>,
}

/// Lay out `dims`; the row count is the longest dimension.
#[must_use]
pub fn layout(dims: &[Dimension]) -> ParallelLayout {
    let n = dims.len();
    let xs = (0..n).map(|i| if n > 1 { i as f64 / (n - 1) as f64 } else { 0.5 }).collect();
    let axes: Vec<ValueAxis> =
        dims.iter().map(|d| ValueAxis::new(ScaleKind::Linear, d.values.iter().copied(), &[], false)).collect();
    let rows = dims.iter().map(|d| d.values.len()).max().unwrap_or(0);
    let ys = (0..rows)
        .map(|r| {
            dims.iter()
                .zip(&axes)
                .map(|(d, a)| d.values.get(r).copied().filter(|v| v.is_finite()).map(|v| a.at(v)))
                .collect()
        })
        .collect();
    ParallelLayout { xs, axes, ys }
}

/// Which rows are inside every brush. `brushes` has one entry per axis; `None` keeps all.
/// A row with a gap on a brushed axis is filtered out.
#[must_use]
pub fn selected(layout: &ParallelLayout, brushes: &[Option<Range>]) -> Vec<bool> {
    layout
        .ys
        .iter()
        .map(|row| {
            brushes.iter().take(layout.xs.len()).enumerate().all(|(a, b)| {
                b.is_none_or(|(lo, hi)| row.get(a).copied().flatten().is_some_and(|y| (lo..=hi).contains(&y)))
            })
        })
        .collect()
}

/// SVG `d` for one row in the 100×100 box, breaking at gaps.
#[must_use]
pub fn path(xs: &[f64], row: &[Option<f64>]) -> String {
    let points: Vec<Option<(f64, f64)>> =
        xs.iter().zip(row).map(|(&x, y)| y.map(|y| (x * 100.0, (1.0 - y) * 100.0))).collect();
    crate::line::runs(&points).iter().map(|r| crate::line::path(r, crate::line::Curve::Linear)).collect()
}

/// The axis within `tolerance` of `x` (plot fractions), nearest first.
#[must_use]
pub fn axis_near(xs: &[f64], x: f64, tolerance: f64) -> Option<usize> {
    xs.iter()
        .enumerate()
        .map(|(i, &a)| (i, (a - x).abs()))
        .filter(|&(_, d)| d <= tolerance)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// A row's height at `x` (plot fractions), between the axes either side. Tooltips anchor here.
#[must_use]
pub fn height_at(xs: &[f64], row: &[Option<f64>], x: f64) -> Option<f64> {
    let last = xs.len().checked_sub(1)?;
    if last == 0 {
        return row.first().copied().flatten();
    }
    let x = x.clamp(xs[0], xs[last]);
    let a = xs.windows(2).position(|w| x <= w[1]).unwrap_or(last - 1);
    let (y0, y1) = (row.get(a).copied().flatten()?, row.get(a + 1).copied().flatten()?);
    let t = (x - xs[a]) / (xs[a + 1] - xs[a]);
    Some(y0 + (y1 - y0) * t)
}

/// The kept row whose line passes nearest the pointer at (`x`, `y`), within `reach`.
#[must_use]
pub fn nearest(layout: &ParallelLayout, keep: &[bool], x: f64, y: f64, reach: f64) -> Option<usize> {
    layout
        .ys
        .iter()
        .enumerate()
        .filter(|(i, _)| keep.get(*i).copied().unwrap_or(true))
        .filter_map(|(i, row)| height_at(&layout.xs, row, x).map(|h| (i, (h - y).abs())))
        .filter(|&(_, d)| d <= reach)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// Arrow-key movement through the kept rows only (see [`crate::nav::list_key`]).
#[must_use]
pub fn step(key: &str, keep: &[bool], current: Option<usize>) -> Option<Option<usize>> {
    let kept: Vec<usize> = (0..keep.len()).filter(|&i| keep[i]).collect();
    let at = current.and_then(|c| kept.iter().position(|&i| i == c));
    crate::nav::list_key(key, at, kept.len()).map(|next| next.map(|p| kept[p]))
}

/// The brush a drag from `start` to `y` (axis fractions) draws, or `None` for a click,
/// which clears the axis.
#[must_use]
pub fn drag(start: f64, y: f64) -> Option<Range> {
    let (a, b) = (start.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
    ((a - b).abs() >= 0.01).then(|| (a.min(b), a.max(b)))
}

/// Distinct category names in first-seen order, and each row's index into them.
#[must_use]
pub fn groups(labels: &[String]) -> (Vec<String>, Vec<usize>) {
    let mut names: Vec<String> = Vec::new();
    let index = labels
        .iter()
        .map(|l| {
            names.iter().position(|n| n == l).unwrap_or_else(|| {
                names.push(l.clone());
                names.len() - 1
            })
        })
        .collect();
    (names, index)
}

/// Paint a segment `thick` px wide, antialiased across its width.
fn segment(out: &mut Raster, (x0, y0): (f64, f64), (x1, y1): (f64, f64), thick: f64, colour: Rgba) {
    let (dx, dy) = (x1 - x0, y1 - y0);
    // Step along the longer direction, covering the shorter one with the line's width.
    let steep = dy.abs() > dx.abs();
    let (a0, a1, b0, db) = if steep { (y0, y1, x0, dx) } else { (x0, x1, y0, dy) };
    let slope = if (a1 - a0).abs() < f64::EPSILON { 0.0 } else { db / (a1 - a0) };
    let half = thick * slope.hypot(1.0) / 2.0;
    let (lo, hi) = (a0.min(a1).floor().max(0.0) as usize, a1.max(a0).ceil() as usize);
    let (limit_a, limit_b) = if steep { (out.height, out.width) } else { (out.width, out.height) };
    for along in lo..hi.min(limit_a) {
        let mid = b0 + (along as f64 + 0.5 - a0) * slope;
        let (top, bottom) = (mid - half, mid + half);
        for across in top.floor().max(0.0) as usize..(bottom.ceil() as usize).min(limit_b) {
            let cover = (bottom.min(across as f64 + 1.0) - top.max(across as f64)).clamp(0.0, 1.0);
            if cover > 0.0 {
                let mut paint = colour;
                paint[3] = (f64::from(paint[3]) * cover).round() as u8;
                let (px, py) = if steep { (across, along) } else { (along, across) };
                out.rect((px, py), (px + 1, py + 1), paint);
            }
        }
    }
}

/// Draw every row at `width` × `height` device pixels, `dpr` per CSS pixel. Filtered rows go first in `muted`;
/// kept rows go on top in `colours[group[row]]` (the last colour for groups past the end).
/// Lines thin out and fade as the count grows so dense bundles still show their shape.
#[must_use]
pub fn raster(
    layout: &ParallelLayout,
    keep: &[bool],
    group: &[usize],
    (colours, muted): (&[Rgba], Rgba),
    (width, height): (usize, usize),
    dpr: f64,
) -> Raster {
    let mut out = Raster::new(width, height);
    let (w, h) = (width as f64, height as f64);
    let kept = keep.iter().filter(|k| **k).count();
    let alpha = if kept > 2_000 {
        0.3
    } else if kept > 500 {
        0.5
    } else {
        0.8
    };
    let thick = dpr.max(1.0);
    let mut draw = |row: &[Option<f64>], colour: Rgba| {
        let pts: Vec<Option<(f64, f64)>> =
            layout.xs.iter().zip(row).map(|(&x, y)| y.map(|y| (x * w, (1.0 - y) * h))).collect();
        for pair in pts.windows(2) {
            if let [Some(a), Some(b)] = pair {
                segment(&mut out, *a, *b, thick, colour);
            }
        }
    };
    let mut faded = muted;
    faded[3] = (f64::from(muted[3]) * 0.35).round() as u8;
    for (i, row) in layout.ys.iter().enumerate() {
        if !keep.get(i).copied().unwrap_or(true) {
            draw(row, faded);
        }
    }
    if let Some(&fallback) = colours.last() {
        for (i, row) in layout.ys.iter().enumerate() {
            if keep.get(i).copied().unwrap_or(true) {
                let mut c = group.get(i).and_then(|&g| colours.get(g)).copied().unwrap_or(fallback);
                c[3] = (f64::from(c[3]) * alpha).round() as u8;
                draw(row, c);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dims() -> Vec<Dimension> {
        vec![Dimension::new("a", [0.0, 10.0, 5.0]), Dimension::new("b", [100.0, 0.0, f64::NAN])]
    }

    #[test]
    fn each_axis_gets_its_own_scale() {
        let l = layout(&dims());
        assert_eq!(l.xs, vec![0.0, 1.0]);
        assert_eq!(l.axes[0].scale.domain, (0.0, 10.0));
        assert_eq!(l.axes[1].scale.domain, (0.0, 100.0));
        assert_eq!(l.ys[1], vec![Some(1.0), Some(0.0)]);
        assert_eq!(l.ys[2][1], None);
    }

    #[test]
    fn brushes_and_together() {
        let l = layout(&dims());
        assert_eq!(selected(&l, &[None, None]), vec![true, true, true]);
        assert_eq!(selected(&l, &[Some((0.4, 1.0)), None]), vec![false, true, true]);
        // Row 2 has a gap on the second axis, so a brush there drops it.
        assert_eq!(selected(&l, &[Some((0.4, 1.0)), Some((0.0, 0.5))]), vec![false, true, false]);
    }

    #[test]
    fn paths_hover_and_axes() {
        let l = layout(&dims());
        assert_eq!(path(&l.xs, &l.ys[0]), "M0.000 100.000L100.000 0.000");
        assert_eq!(path(&l.xs, &l.ys[2]), "M0.000 50.000h0");
        // Halfway across, row 0 is at 0.5 and row 1 at 0.5 too; at x = 0.1, row 1 is near the top.
        assert_eq!(nearest(&l, &[true, true, true], 0.1, 0.9, 0.1), Some(1));
        assert_eq!(nearest(&l, &[true, false, true], 0.1, 0.9, 0.1), None);
        assert_eq!(axis_near(&l.xs, 0.97, 0.05), Some(1));
        assert_eq!(axis_near(&l.xs, 0.5, 0.05), None);
    }

    #[test]
    fn keys_skip_filtered_rows_and_drags_make_brushes() {
        assert_eq!(step("ArrowRight", &[false, true, true], None), Some(Some(1)));
        assert_eq!(step("ArrowRight", &[true, false, true], Some(0)), Some(Some(2)));
        assert_eq!(step("ArrowRight", &[false, false], None), None);
        assert_eq!(drag(0.8, 0.2), Some((0.2, 0.8)));
        assert_eq!(drag(0.5, 1.4), Some((0.5, 1.0)));
        assert_eq!(drag(0.5, 0.505), None);
    }

    #[test]
    fn groups_keep_first_seen_order() {
        let labels: Vec<String> = ["Petrol", "Diesel", "Petrol", "Electric"].map(String::from).to_vec();
        assert_eq!(groups(&labels), (vec!["Petrol".into(), "Diesel".into(), "Electric".into()], vec![0, 1, 0, 2]));
    }

    #[test]
    fn raster_draws_kept_lines_over_faded_ones() {
        let l = layout(&dims());
        let r = raster(&l, &[true, false, true], &[0, 0, 0], (&[[255, 0, 0, 255]], [0, 0, 0, 255]), (20, 20), 1.0);
        // Row 0 runs from bottom left to top right, so it paints (4, 15) red.
        let i = (15 * 20 + 4) * 4;
        assert!(r.rgba[i] > 100 && r.rgba[i + 3] > 0);
        // The top left corner only has row 1 (filtered), so it is faint and dark.
        let i = (20 + 1) * 4;
        assert!(r.rgba[i + 3] > 0 && r.rgba[i + 3] < 128);
    }
}
