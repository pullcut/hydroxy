//! Vector fields on a regular grid: an arrow per grid point showing direction and
//! magnitude, plus streamlines traced through the field.
#![allow(
    clippy::many_single_char_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "short names follow the maths; grid sizes are small; fractions are clamped"
)]

use std::fmt::Write as _;

use crate::bar::Gridline;
use crate::plot::ValueAxis;
use crate::raster::{Raster, Rgba};
use crate::scale::{ScaleKind, ValueScale, ticks};

/// A vector field sampled on a regular grid.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VectorField {
    /// Grid x positions, ascending.
    pub xs: Vec<f64>,
    /// Grid y positions, ascending.
    pub ys: Vec<f64>,
    /// x components, row by row from the lowest y: `u[row * xs.len() + col]`.
    pub u: Vec<f64>,
    /// y components, laid out like `u`.
    pub v: Vec<f64>,
}

impl VectorField {
    /// Sample `f(x, y) -> (u, v)` at every grid point.
    #[must_use]
    pub fn from_fn(xs: impl Into<Vec<f64>>, ys: impl Into<Vec<f64>>, f: impl Fn(f64, f64) -> (f64, f64)) -> Self {
        let (xs, ys) = (xs.into(), ys.into());
        let (u, v) = ys.iter().flat_map(|&y| xs.iter().map(move |&x| (x, y))).map(|(x, y)| f(x, y)).unzip();
        Self { xs, ys, u, v }
    }

    /// `count` evenly spaced values from `lo` to `hi`, for grid axes.
    #[must_use]
    pub fn steps(lo: f64, hi: f64, count: usize) -> Vec<f64> {
        let n = count.max(2) - 1;
        (0..=n).map(|i| lo + (hi - lo) * i as f64 / n as f64).collect()
    }

    /// The vector at grid point (`row`, `col`), `None` if missing or non-finite.
    #[must_use]
    pub fn at(&self, row: usize, col: usize) -> Option<(f64, f64)> {
        let i = row * self.xs.len() + col;
        let (u, v) = (*self.u.get(i)?, *self.v.get(i)?);
        (u.is_finite() && v.is_finite()).then_some((u, v))
    }

    /// The field at (`x`, `y`), bilinearly interpolated; `None` outside the grid.
    #[must_use]
    pub fn sample(&self, x: f64, y: f64) -> Option<(f64, f64)> {
        let (c, fx) = cell(&self.xs, x)?;
        let (r, fy) = cell(&self.ys, y)?;
        let (a, b, c2, d) = (self.at(r, c)?, self.at(r, c + 1)?, self.at(r + 1, c)?, self.at(r + 1, c + 1)?);
        let mix =
            |p: f64, q: f64, s: f64, t: f64| (p * (1.0 - fx) + q * fx) * (1.0 - fy) + (s * (1.0 - fx) + t * fx) * fy;
        Some((mix(a.0, b.0, c2.0, d.0), mix(a.1, b.1, c2.1, d.1)))
    }
}

/// Which grid interval holds `p`, and how far along it, for ascending `axis`.
fn cell(axis: &[f64], p: f64) -> Option<(usize, f64)> {
    if axis.len() < 2 || !(axis[0]..=axis[axis.len() - 1]).contains(&p) {
        return None;
    }
    let i = axis.partition_point(|&a| a <= p).clamp(1, axis.len() - 1) - 1;
    let span = axis[i + 1] - axis[i];
    Some((i, if span > 0.0 { (p - axis[i]) / span } else { 0.0 }))
}

/// One arrow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arrow {
    /// Grid row, from the lowest y.
    pub row: usize,
    /// Grid column.
    pub col: usize,
    /// The grid point.
    pub x: f64,
    /// The grid point.
    pub y: f64,
    /// x component.
    pub u: f64,
    /// y component.
    pub v: f64,
    /// Vector length.
    pub magnitude: f64,
    /// Magnitude as a share of the largest, 0..=1, for colour.
    pub level: f64,
    /// Tail position, 0..=1 from the left. Arrows are centred on their grid point.
    pub px: f64,
    /// Tail position, 0..=1 from the bottom.
    pub py: f64,
    /// Arrow extent across, as a share of the plot width (may be negative).
    pub dx: f64,
    /// Arrow extent up, as a share of the plot height (may be negative).
    pub dy: f64,
}

impl Arrow {
    /// The grid point, as plot fractions (y from the bottom): the arrow's middle.
    #[must_use]
    pub fn centre(&self) -> (f64, f64) {
        (self.px + self.dx / 2.0, self.py + self.dy / 2.0)
    }

    /// Direction in degrees, counter-clockwise from +x, 0..360.
    #[must_use]
    pub fn degrees(&self) -> f64 {
        self.v.atan2(self.u).to_degrees().rem_euclid(360.0)
    }
}

/// Everything a quiver chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct QuiverLayout {
    /// Arrows row by row from the lowest y; missing vectors are skipped.
    pub arrows: Vec<Arrow>,
    /// Grid size: rows (ys), columns (xs).
    pub shape: (usize, usize),
    /// The x axis.
    pub x_axis: ValueAxis,
    /// The y axis.
    pub axis: ValueAxis,
    /// Smallest and largest magnitude.
    pub range: (f64, f64),
    /// Streamlines, SVG `d` in the 100×100 box (y down). Empty unless asked for.
    pub streamlines: Vec<String>,
}

impl QuiverLayout {
    /// The arrow at grid point (`row`, `col`).
    #[must_use]
    pub fn arrow(&self, row: usize, col: usize) -> Option<&Arrow> {
        self.arrows.iter().find(|a| a.row == row && a.col == col)
    }
}

/// Smallest gap between neighbouring grid positions.
fn spacing(axis: &[f64]) -> f64 {
    axis.windows(2).map(|w| (w[1] - w[0]).abs()).filter(|d| *d > 0.0).fold(f64::INFINITY, f64::min)
}

/// Lay out `field`. With `scaled`, arrow length follows magnitude (the longest fills about
/// 90% of a grid cell); otherwise every arrow is the same length and only colour shows
/// magnitude. `streamlines` traces that many lines (roughly) from seeds spread over the grid.
#[must_use]
pub fn layout(field: &VectorField, scaled: bool, streamlines: usize) -> QuiverLayout {
    let (cols, rows) = (field.xs.len(), field.ys.len());
    let gap = spacing(&field.xs).min(spacing(&field.ys));
    let gap = if gap.is_finite() {
        gap
    } else {
        (last(&field.xs) - first(&field.xs)).min(last(&field.ys) - first(&field.ys)) / 2.0
    };
    // Half a cell of room around the grid, so edge arrows stay inside the plot.
    let x_axis = exact(first(&field.xs) - gap / 2.0, last(&field.xs) + gap / 2.0);
    let axis = exact(first(&field.ys) - gap / 2.0, last(&field.ys) + gap / 2.0);
    let (xd, yd) = (x_axis.scale.domain, axis.scale.domain);
    let (wx, wy) = ((xd.1 - xd.0).max(f64::EPSILON), (yd.1 - yd.0).max(f64::EPSILON));
    let mags = (0..rows)
        .flat_map(|r| (0..cols).map(move |c| (r, c)))
        .filter_map(|(r, c)| field.at(r, c))
        .map(|(u, v)| u.hypot(v));
    let (lo, hi) = mags.fold((f64::INFINITY, 0.0_f64), |(lo, hi), m| (lo.min(m), hi.max(m)));
    let lo = if lo.is_finite() { lo } else { 0.0 };
    let mut arrows = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let Some((u, v)) = field.at(r, c) else { continue };
            let magnitude = u.hypot(v);
            let level = if hi > 0.0 { magnitude / hi } else { 0.0 };
            // Length in data units: proportional to magnitude, or fixed.
            let len = gap * 0.9 * if scaled { level } else { 1.0 };
            let (ux, uy) = if magnitude > 0.0 { (u / magnitude, v / magnitude) } else { (0.0, 0.0) };
            let (x, y) = (field.xs[c], field.ys[r]);
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            let (dx, dy) = (ux * len / wx, uy * len / wy);
            // Arrows are centred on their grid point, so with half a cell of padding none leave the plot.
            arrows.push(Arrow {
                row: r,
                col: c,
                x,
                y,
                u,
                v,
                magnitude,
                level,
                px: x_axis.scale.map(x) - dx / 2.0,
                py: axis.scale.map(y) - dy / 2.0,
                dx,
                dy,
            });
        }
    }
    let lines = if streamlines > 0 {
        trace(field, streamlines, gap).iter().map(|l| path(l, &x_axis, &axis)).collect()
    } else {
        Vec::new()
    };
    QuiverLayout { arrows, shape: (rows, cols), x_axis, axis, range: (lo, hi), streamlines: lines }
}

/// A linear axis over exactly `lo..=hi` (not widened), with nice ticks inside it.
fn exact(lo: f64, hi: f64) -> ValueAxis {
    let scale = ValueScale { kind: ScaleKind::Linear, domain: (lo, hi) };
    let gridlines = ticks(lo, hi, 5)
        .into_iter()
        .filter(|v| (lo..=hi).contains(v))
        .map(|value| Gridline { value, fraction: scale.map(value) })
        .collect();
    ValueAxis { scale, gridlines }
}

fn first(v: &[f64]) -> f64 {
    v.iter().copied().find(|f| f.is_finite()).unwrap_or(0.0)
}

fn last(v: &[f64]) -> f64 {
    v.iter().rev().copied().find(|l| l.is_finite()).filter(|l| *l > first(v)).unwrap_or(first(v) + 1.0)
}

fn path(points: &[(f64, f64)], x: &ValueAxis, y: &ValueAxis) -> String {
    let mut d = String::new();
    for (i, &(px, py)) in points.iter().enumerate() {
        let _ = write!(
            d,
            "{}{:.2} {:.2}",
            if i == 0 { 'M' } else { 'L' },
            x.scale.map(px) * 100.0,
            (1.0 - y.scale.map(py)) * 100.0
        );
    }
    d
}

/// Trace streamlines with fourth-order Runge–Kutta through the normalised field, forwards
/// and backwards from seeds on a coarse grid, stopping at the edge, at stagnation points,
/// or where another line already passed.
fn trace(field: &VectorField, count: usize, gap: f64) -> Vec<Vec<(f64, f64)>> {
    let (Some(&x0), Some(&x1), Some(&y0), Some(&y1)) =
        (field.xs.first(), field.xs.last(), field.ys.first(), field.ys.last())
    else {
        return Vec::new();
    };
    let (w, h) = (x1 - x0, y1 - y0);
    if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
        return Vec::new();
    }
    // Occupancy grid so lines keep apart; about `count` seeds spread evenly.
    let side = (count as f64).sqrt().ceil().clamp(1.0, 100.0) as usize;
    let res = side * 4;
    let mut taken = vec![false; res * res];
    let slot = |x: f64, y: f64| {
        let (i, j) = (((x - x0) / w * res as f64) as usize, ((y - y0) / h * res as f64) as usize);
        i.min(res - 1) + j.min(res - 1) * res
    };
    let dir = |x: f64, y: f64| {
        let (u, v) = field.sample(x, y)?;
        let m = u.hypot(v);
        (m > 1e-12).then(|| (u / m, v / m))
    };
    let step = gap * 0.25;
    let walk = |start: (f64, f64), sign: f64, taken: &[bool], own: &mut Vec<usize>| {
        let mut out = Vec::new();
        let mut p = start;
        for _ in 0..400 {
            let k1 = dir(p.0, p.1);
            let Some(k1) = k1 else { break };
            let mid = |k: (f64, f64), f: f64| (p.0 + sign * k.0 * step * f, p.1 + sign * k.1 * step * f);
            let Some(k2) = dir(mid(k1, 0.5).0, mid(k1, 0.5).1) else { break };
            let Some(k3) = dir(mid(k2, 0.5).0, mid(k2, 0.5).1) else { break };
            let Some(k4) = dir(mid(k3, 1.0).0, mid(k3, 1.0).1) else { break };
            let next = (
                p.0 + sign * step * (k1.0 + 2.0 * k2.0 + 2.0 * k3.0 + k4.0) / 6.0,
                p.1 + sign * step * (k1.1 + 2.0 * k2.1 + 2.0 * k3.1 + k4.1) / 6.0,
            );
            if !(x0..=x1).contains(&next.0) || !(y0..=y1).contains(&next.1) {
                break;
            }
            let s = slot(next.0, next.1);
            if taken[s] && !own.contains(&s) {
                break;
            }
            if own.last() != Some(&s) {
                own.push(s);
            }
            out.push(next);
            p = next;
        }
        out
    };
    let mut lines = Vec::new();
    for j in 0..side {
        for i in 0..side {
            let seed = (x0 + w * (i as f64 + 0.5) / side as f64, y0 + h * (j as f64 + 0.5) / side as f64);
            if taken[slot(seed.0, seed.1)] {
                continue;
            }
            let mut own = vec![slot(seed.0, seed.1)];
            let back = walk(seed, -1.0, &taken, &mut own);
            let ahead = walk(seed, 1.0, &taken, &mut own);
            let line: Vec<(f64, f64)> = back.into_iter().rev().chain([seed]).chain(ahead).collect();
            if line.len() >= 4 {
                for &s in &own {
                    taken[s] = true;
                }
                lines.push(line);
            }
        }
    }
    lines
}

/// Draw `layout`'s arrows at `width` × `height` device pixels for grids too large for the
/// DOM. `ramp` colours magnitude from lowest to highest (at least one entry).
#[must_use]
pub fn paint(layout: &QuiverLayout, ramp: &[Rgba], width: usize, height: usize, dpr: f64) -> Raster {
    let mut out = Raster::new(width, height);
    if ramp.is_empty() {
        return out;
    }
    let (w, h) = (width as f64, height as f64);
    let r = 0.7 * dpr;
    for a in &layout.arrows {
        let colour = ramp[((a.level * (ramp.len() - 1) as f64).round() as usize).min(ramp.len() - 1)];
        let (x0, y0) = (a.px * w, (1.0 - a.py) * h);
        let (vx, vy) = (a.dx * w, -a.dy * h);
        let len = vx.hypot(vy);
        if len < 0.5 {
            out.disc(x0, y0, r, colour);
            continue;
        }
        let (ux, uy) = (vx / len, vy / len);
        let head = (len * 0.35).min(5.0 * dpr);
        stroke(&mut out, (x0, y0), (x0 + vx, y0 + vy), r, colour);
        // The head: two short barbs back from the tip, about 28° either side.
        let (tx, ty) = (x0 + vx, y0 + vy);
        let (c, s) = (0.88, 0.47);
        stroke(&mut out, (tx, ty), (tx - head * (ux * c - uy * s), ty - head * (uy * c + ux * s)), r, colour);
        stroke(&mut out, (tx, ty), (tx - head * (ux * c + uy * s), ty - head * (uy * c - ux * s)), r, colour);
    }
    out
}

/// An antialiased line from `a` to `b`, `r` pixels either side of centre, painted over
/// what is there. (`Raster`'s own blend is private to it, so this mixes in place.)
fn stroke(out: &mut Raster, a: (f64, f64), b: (f64, f64), r: f64, colour: Rgba) {
    let (w, h) = (out.width as f64, out.height as f64);
    let x0 = (a.0.min(b.0) - r - 1.0).floor().clamp(0.0, w) as usize;
    let x1 = (a.0.max(b.0) + r + 1.0).ceil().clamp(0.0, w) as usize;
    let y0 = (a.1.min(b.1) - r - 1.0).floor().clamp(0.0, h) as usize;
    let y1 = (a.1.max(b.1) + r + 1.0).ceil().clamp(0.0, h) as usize;
    let (vx, vy) = (b.0 - a.0, b.1 - a.1);
    let len2 = (vx * vx + vy * vy).max(f64::EPSILON);
    for y in y0..y1 {
        for x in x0..x1 {
            let (px, py) = (x as f64 + 0.5 - a.0, y as f64 + 0.5 - a.1);
            let t = ((px * vx + py * vy) / len2).clamp(0.0, 1.0);
            let d = (px - t * vx).hypot(py - t * vy);
            let cover = (r + 0.5 - d).clamp(0.0, 1.0);
            if cover <= 0.0 {
                continue;
            }
            let i = (y * out.width + x) * 4;
            let src = f64::from(colour[3]) / 255.0 * cover;
            let px = &mut out.rgba[i..i + 4];
            let dst = f64::from(px[3]) / 255.0;
            let alpha = src + dst * (1.0 - src);
            for c in 0..3 {
                px[c] = ((f64::from(colour[c]) * src + f64::from(px[c]) * dst * (1.0 - src)) / alpha)
                    .round()
                    .clamp(0.0, 255.0) as u8;
            }
            px[3] = (alpha * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn swirl() -> VectorField {
        let s = VectorField::steps(-1.0, 1.0, 5);
        VectorField::from_fn(s.clone(), s, |x, y| (-y, x))
    }

    #[test]
    fn grid_samples_and_interpolates() {
        let f = swirl();
        assert_eq!(f.u.len(), 25);
        assert_eq!(f.at(0, 0), Some((1.0, -1.0)));
        let (u, v) = f.sample(0.25, 0.0).unwrap();
        assert!(u.abs() < 1e-12 && (v - 0.25).abs() < 1e-12, "a linear field interpolates exactly");
        assert_eq!(f.sample(2.0, 0.0), None);
    }

    #[test]
    fn arrows_scale_and_point_the_right_way() {
        let l = layout(&swirl(), true, 0);
        assert_eq!(l.shape, (5, 5));
        assert_eq!(l.arrows.len(), 25);
        let centre = l.arrow(2, 2).unwrap();
        assert_eq!(centre.magnitude, 0.0);
        assert_eq!((centre.dx, centre.dy), (0.0, 0.0));
        let right = l.arrow(2, 4).unwrap();
        assert!((right.degrees() - 90.0).abs() < 1e-9, "at (1, 0) the swirl points up");
        assert!(right.dy > 0.0 && right.dx.abs() < 1e-12);
        let corner = l.arrow(4, 4).unwrap();
        assert!((corner.level - 1.0).abs() < 1e-12);
        // The axes get half a cell of room each side: -1.25..1.25.
        assert_eq!(l.x_axis.scale.domain, (-1.25, 1.25));
        assert_eq!(l.x_axis.gridlines.first().map(|g| g.value), Some(-1.0));
        assert!(
            l.arrows.iter().all(|a| (0.0..=1.0).contains(&(a.px + a.dx)) && (0.0..=1.0).contains(&(a.py + a.dy))),
            "arrows stay inside"
        );
        // The longest arrow spans 90% of a 0.5-wide cell on a 2.5-wide axis.
        assert!((corner.dx.hypot(corner.dy) - 0.18).abs() < 1e-9);
        assert!(right.dy < corner.dx.hypot(corner.dy));
        let fixed = layout(&swirl(), false, 0);
        assert!((fixed.arrow(2, 4).unwrap().dy - 0.18).abs() < 1e-9);
    }

    #[test]
    fn non_finite_grid_and_huge_streamline_counts_stay_sane() {
        let mut f = swirl();
        f.xs[0] = f64::NAN;
        f.ys[1] = f64::INFINITY;
        let l = layout(&f, true, usize::MAX);
        let a = l.arrows.iter().flat_map(|a| [a.px, a.py, a.dx, a.dy]);
        assert!(a.chain([l.x_axis.scale.domain.0, l.axis.scale.domain.0]).all(f64::is_finite));
        assert!(l.streamlines.iter().all(|d| !d.contains("NaN") && !d.contains("inf")));
        assert_ne!(layout(&swirl(), true, usize::MAX).streamlines.len(), 0);
    }

    #[test]
    fn streamlines_circle_the_swirl() {
        let l = layout(&swirl(), false, 9);
        assert_ne!(l.streamlines.len(), 0);
        assert!(l.streamlines.iter().all(|d| d.starts_with('M') && d.contains('L')));
    }

    #[test]
    fn paint_draws_near_the_arrows() {
        let l = layout(&swirl(), true, 0);
        let r = paint(&l, &[[0, 0, 0, 255]], 40, 40, 1.0);
        assert!(r.rgba.chunks(4).any(|p| p[3] > 0));
    }
}
