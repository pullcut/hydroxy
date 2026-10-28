//! Cartograms: regions resized by value. A Dorling cartogram swaps each region for a
//! circle whose area is its value, nudged apart from its neighbours; a non-contiguous
//! cartogram shrinks each region about its centroid so its area tracks its value density.
//! Projection and fitting come from [`crate::geo`].

#![allow(clippy::many_single_char_names, reason = "geometry reads best as x, y, r, p")]

use std::f64::consts::PI;

use crate::geo::{self, Geo, Projection, Region};

/// Which kind of cartogram to draw.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CartogramKind {
    /// One circle per region, area proportional to value, relaxed so circles don't overlap.
    #[default]
    Dorling,
    /// Each region keeps its shape and shrinks about its centroid; the densest region keeps
    /// its size.
    NonContiguous,
}

/// One region, resized.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    /// Centroid of the region's largest ring, in viewBox units.
    pub centroid: (f64, f64),
    /// Where the Dorling circle settled, in viewBox units.
    pub at: (f64, f64),
    /// Dorling circle radius; zero for a missing or non-positive value.
    pub r: f64,
    /// Non-contiguous scale factor about the centroid, 0..=1.
    pub k: f64,
}

/// Everything a cartogram needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct CartogramLayout {
    /// The fitted map: shapes, value range and viewBox size.
    pub geo: Geo,
    /// One per input region.
    pub cells: Vec<Cell>,
}

/// Share of the map's land area the Dorling circles cover between them.
const FILL: f64 = 0.45;
/// Space kept between neighbouring circles, in viewBox units.
const GAP: f64 = 1.5;

/// Signed area and centroid of one ring.
fn ring_centroid(ring: &[(f64, f64)]) -> (f64, (f64, f64)) {
    let (mut a, mut cx, mut cy) = (0.0, 0.0, 0.0);
    for (i, &(x0, y0)) in ring.iter().enumerate() {
        let (x1, y1) = ring[(i + 1) % ring.len()];
        let cross = x0 * y1 - x1 * y0;
        a += cross;
        cx += (x0 + x1) * cross;
        cy += (y0 + y1) * cross;
    }
    if a.abs() < 1e-12 {
        #[allow(clippy::cast_precision_loss, reason = "ring sizes are small")]
        let n = ring.len().max(1) as f64;
        let (sx, sy) = ring.iter().fold((0.0, 0.0), |(sx, sy), p| (sx + p.0, sy + p.1));
        return (0.0, (sx / n, sy / n));
    }
    (a / 2.0, (cx / (3.0 * a), cy / (3.0 * a)))
}

/// Lay out a cartogram of `regions`. Both kinds are worked out, so switching is free.
#[must_use]
pub fn layout(regions: &[Region], projection: Projection) -> CartogramLayout {
    let finite = |p: &(f64, f64)| p.0.is_finite() && p.1.is_finite();
    let rings: Vec<Vec<Vec<(f64, f64)>>> = regions
        .iter()
        .map(|r| r.rings.iter().map(|ring| ring.iter().map(|&p| geo::project(p, projection)).filter(finite).collect::<Vec<_>>()).filter(|g| g.len() >= 3).collect())
        .collect();
    // Per region: area and centroid of its largest ring, plus its total area, unscaled.
    let shapes: Vec<(f64, (f64, f64))> = rings
        .iter()
        .map(|rs| {
            let parts: Vec<(f64, (f64, f64))> = rs.iter().map(|g| ring_centroid(g)).collect();
            let total: f64 = parts.iter().map(|p| p.0.abs()).sum();
            let main = parts.iter().max_by(|a, b| a.0.abs().total_cmp(&b.0.abs())).map_or((0.0, 0.0), |p| p.1);
            (total, main)
        })
        .collect();
    let geo = geo::layout(regions, &[], projection);
    let (scale, place) = fit(&rings, geo.width);
    let area: Vec<f64> = shapes.iter().map(|s| s.0 * scale * scale).collect();
    let centroid: Vec<(f64, f64)> = shapes.iter().zip(&geo.shapes).map(|(s, g)| if s.0 > 0.0 { place(s.1) } else { g.centre }).collect();
    let value = |r: &Region| if r.value.is_finite() { r.value.max(0.0) } else { 0.0 };
    // Dorling: circles together cover FILL of the land.
    let total_v: f64 = regions.iter().map(value).sum();
    let total_a: f64 = area.iter().sum();
    let per = if total_v > 0.0 { FILL * total_a / (PI * total_v) } else { 0.0 };
    let radii: Vec<f64> = regions.iter().map(|r| (value(r) * per).sqrt()).collect();
    let at = relax(&centroid, &radii, (geo.width, geo.height));
    // Non-contiguous: the densest region keeps its size, the rest shrink to match.
    let density: Vec<f64> = regions.iter().zip(&area).map(|(r, &a)| if a > 0.0 { value(r) / a } else { 0.0 }).collect();
    let densest = density.iter().copied().fold(0.0, f64::max);
    let cells = (0..regions.len())
        .map(|i| Cell { centroid: centroid[i], at: at[i], r: radii[i], k: if densest > 0.0 { (density[i] / densest).sqrt() } else { 0.0 } })
        .collect();
    CartogramLayout { geo, cells }
}

/// The scale and placement [`geo::layout`] uses for regions alone (640 wide, 6 padding).
fn fit(rings: &[Vec<Vec<(f64, f64)>>], width: f64) -> (f64, impl Fn((f64, f64)) -> (f64, f64)) {
    let pad = 6.0;
    let pts = || rings.iter().flatten().flatten();
    let (x0, x1) = pts().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| (a.min(p.0), b.max(p.0)));
    let (y0, y1) = pts().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| (a.min(p.1), b.max(p.1)));
    let (bw, bh) = ((x1 - x0).max(f64::EPSILON), (y1 - y0).max(f64::EPSILON));
    let bounded = x0.is_finite() && y0.is_finite();
    let scale = if bounded { ((width - 2.0 * pad) / bw).min((width - 2.0 * pad) / bh) } else { 1.0 };
    let dx = if bounded { (width - bw * scale) / 2.0 } else { pad };
    (scale, move |(x, y): (f64, f64)| (dx + (x - x0) * scale, pad + (y - y0) * scale))
}

/// Push overlapping circles apart while each is pulled gently back to its centroid and
/// kept inside the viewBox; a circle pinned at an edge makes its neighbours give way.
/// Pushing comes last in each pass, so circles end apart. Deterministic: the same input
/// always settles the same way.
// O(n²) per pass; fine for a few hundred regions, a grid index if maps get bigger.
#[allow(clippy::cast_precision_loss, reason = "region counts are small")]
fn relax(centroid: &[(f64, f64)], r: &[f64], (w, h): (f64, f64)) -> Vec<(f64, f64)> {
    let mut p = centroid.to_vec();
    let live: Vec<usize> = (0..p.len()).filter(|&i| r[i] > 0.0).collect();
    for _ in 0..300 {
        for &i in &live {
            p[i].0 += (centroid[i].0 - p[i].0) * 0.04;
            p[i].1 += (centroid[i].1 - p[i].1) * 0.04;
            p[i] = (p[i].0.clamp(r[i].min(w / 2.0), (w - r[i]).max(w / 2.0)), p[i].1.clamp(r[i].min(h / 2.0), (h - r[i]).max(h / 2.0)));
        }
        for (a, &i) in live.iter().enumerate() {
            for &j in &live[a + 1..] {
                let (dx, dy) = (p[j].0 - p[i].0, p[j].1 - p[i].1);
                let d = dx.hypot(dy);
                let overlap = r[i] + r[j] + GAP - d;
                if overlap <= 0.0 {
                    continue;
                }
                // Coincident centres part along a fixed angle per pair.
                let (ux, uy) = if d > 1e-9 { (dx / d, dy / d) } else { let t = (i * 7 + j * 13) as f64; (t.cos(), t.sin()) };
                let share = r[i] + r[j];
                let (mi, mj) = (overlap * r[j] / share, overlap * r[i] / share);
                p[i] = (p[i].0 - ux * mi, p[i].1 - uy * mi);
                p[j] = (p[j].0 + ux * mj, p[j].1 + uy * mj);
            }
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(id: &str, v: f64, x: f64, side: f64) -> Region {
        Region::new(id, id, v, [(x, 0.0), (x + side, 0.0), (x + side, side), (x, side)])
    }

    #[test]
    fn centroids_match_the_fitted_map() {
        let l = layout(&[square("a", 1.0, 0.0, 10.0), square("b", 4.0, 10.0, 10.0)], Projection::Equirectangular);
        for (c, s) in l.cells.iter().zip(&l.geo.shapes) {
            assert!((c.centroid.0 - s.centre.0).abs() < 1e-6 && (c.centroid.1 - s.centre.1).abs() < 1e-6, "{c:?} vs {s:?}");
        }
    }

    #[test]
    fn circle_areas_track_values_and_cover_the_fill_share() {
        let l = layout(&[square("a", 1.0, 0.0, 10.0), square("b", 4.0, 10.0, 10.0)], Projection::Equirectangular);
        let (a, b) = (l.cells[0].r, l.cells[1].r);
        assert!((b / a - 2.0).abs() < 1e-9);
        // Two 314-unit squares at this fit: land is 2 · 314².
        let land = 2.0 * 314.0_f64.powi(2);
        assert!((PI * (a * a + b * b) - FILL * land).abs() < 1e-6 * land);
    }

    #[test]
    fn circles_relax_apart_but_stay_near_home() {
        let l = layout(&[square("a", 5.0, 0.0, 10.0), square("b", 5.0, 10.0, 10.0), square("c", 5.0, 20.0, 10.0)], Projection::Equirectangular);
        for i in 0..3 {
            for j in i + 1..3 {
                let (p, q) = (l.cells[i], l.cells[j]);
                let d = (p.at.0 - q.at.0).hypot(p.at.1 - q.at.1);
                assert!(d >= p.r + q.r - 0.5, "circles {i} and {j} overlap: {d} < {}", p.r + q.r);
            }
        }
        assert!(l.cells[0].at.0 < l.cells[1].at.0 && l.cells[1].at.0 < l.cells[2].at.0, "order along the map is kept");
    }

    #[test]
    fn densest_region_keeps_its_size() {
        let l = layout(&[square("a", 1.0, 0.0, 10.0), square("b", 4.0, 10.0, 10.0), square("c", f64::NAN, 20.0, 10.0)], Projection::Equirectangular);
        assert!((l.cells[1].k - 1.0).abs() < 1e-9);
        assert!((l.cells[0].k - 0.5).abs() < 1e-9);
        assert_eq!((l.cells[2].k, l.cells[2].r), (0.0, 0.0));
    }

    #[test]
    fn empty_input_is_harmless() {
        assert_eq!(layout(&[], Projection::Mercator).cells.len(), 0);
    }
}
