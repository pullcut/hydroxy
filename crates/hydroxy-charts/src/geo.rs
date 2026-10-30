//! Map geometry: project polygons and points, fit them to a viewBox and shade by value.
//! Shared by the geo map (choropleth and polygon map), bubble map and hexbin map.

use std::f64::consts::PI;
use std::fmt::Write as _;

use crate::scale;

/// A region: one or more rings of `(longitude, latitude)` in degrees. Rings are filled
/// with the even-odd rule, so a ring inside another is a hole.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    /// Stable id.
    pub id: String,
    /// Shown in the tooltip, table and accessible names.
    pub name: String,
    /// The measure. Non-finite values draw unshaded.
    pub value: f64,
    /// Polygon rings.
    pub rings: Vec<Vec<(f64, f64)>>,
    /// A categorical colour slot (0 to 5) instead of the value shade: a polygon map.
    pub colour: Option<usize>,
}

impl Region {
    /// A region with a single ring.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, value: f64, ring: impl Into<Vec<(f64, f64)>>) -> Self {
        Self { id: id.into(), name: name.into(), value, rings: vec![ring.into()], colour: None }
    }

    /// Fill with chart colour `slot` (0 to 5) instead of shading by value.
    #[must_use]
    pub fn with_colour(mut self, slot: usize) -> Self {
        self.colour = Some(slot.min(5));
        self
    }
}

/// A located value: a store, a city, an event.
#[derive(Debug, Clone, PartialEq)]
pub struct GeoPoint {
    /// Shown in the tooltip, table and accessible names.
    pub name: String,
    /// Longitude in degrees.
    pub lon: f64,
    /// Latitude in degrees.
    pub lat: f64,
    /// The measure: bubble area on a bubble map, a summed total on a hexbin map.
    pub value: f64,
    /// Series index (0 to 5) for the colour and legend.
    pub group: usize,
}

impl GeoPoint {
    /// A point in group 0.
    #[must_use]
    pub fn new(name: impl Into<String>, (lon, lat): (f64, f64), value: f64) -> Self {
        Self { name: name.into(), lon, lat, value, group: 0 }
    }

    /// Put the point in series `group` (0 to 5).
    #[must_use]
    pub fn in_group(mut self, group: usize) -> Self {
        self.group = group.min(5);
        self
    }
}

/// How longitude and latitude become x and y.
#[derive(Debug, Clone, Copy, Default)]
pub enum Projection {
    /// Plate carrée: x and y are the degrees themselves.
    #[default]
    Equirectangular,
    /// Web Mercator: north-up and conformal; latitude is clamped to ±85°.
    Mercator,
    /// Your own projection: `(longitude, latitude)` in, planar `(x, y)` out with y pointing
    /// north. The result is fitted to the viewBox, so its units don't matter. Non-finite
    /// output drops the point.
    Custom(fn((f64, f64)) -> (f64, f64)),
}

impl PartialEq for Projection {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Custom(a), Self::Custom(b)) => std::ptr::fn_addr_eq(*a, *b),
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

impl Eq for Projection {}

/// Unscaled planar position of a point, y pointing down.
#[must_use]
pub fn project((lon, lat): (f64, f64), projection: Projection) -> (f64, f64) {
    match projection {
        Projection::Equirectangular => (lon, -lat),
        Projection::Mercator => {
            let phi = lat.clamp(-85.0, 85.0).to_radians();
            (lon, -(PI / 4.0 + phi / 2.0).tan().ln().to_degrees())
        }
        Projection::Custom(f) => {
            let (x, y) = f((lon, lat));
            (x, -y)
        }
    }
}

/// One region, ready to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct GeoShape {
    /// SVG `d` for all rings.
    pub path: String,
    /// Position of the value within the data range (0..=1); `None` when there is no value.
    pub t: Option<f64>,
    /// Centre of the region's bounding box in viewBox units, where its tooltip points.
    pub centre: (f64, f64),
}

/// Everything a map needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct Geo {
    /// viewBox width.
    pub width: f64,
    /// viewBox height, from the map's aspect ratio.
    pub height: f64,
    /// One per input region.
    pub shapes: Vec<GeoShape>,
    /// One per input point, in viewBox units; `None` when the projection can't place it.
    pub points: Vec<Option<(f64, f64)>>,
    /// Smallest and largest region value.
    pub range: (f64, f64),
}

/// Fit `regions` and `points` together into a viewBox 640 wide.
#[must_use]
pub fn layout(regions: &[Region], points: &[GeoPoint], projection: Projection) -> Geo {
    let (width, pad) = (640.0, 6.0);
    let finite_pt = |p: (f64, f64)| p.0.is_finite() && p.1.is_finite();
    let projected: Vec<Vec<Vec<(f64, f64)>>> = regions
        .iter()
        .map(|r| {
            r.rings
                .iter()
                .map(|ring| ring.iter().map(|&p| project(p, projection)).filter(|&p| finite_pt(p)).collect())
                .collect()
        })
        .collect();
    let dots: Vec<Option<(f64, f64)>> =
        points.iter().map(|p| Some(project((p.lon, p.lat), projection)).filter(|&q| finite_pt(q))).collect();
    let pts = || projected.iter().flatten().flatten().copied().chain(dots.iter().flatten().copied());
    let (x0, x1) = pts().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| (a.min(p.0), b.max(p.0)));
    let (y0, y1) = pts().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| (a.min(p.1), b.max(p.1)));
    let (bw, bh) = ((x1 - x0).max(f64::EPSILON), (y1 - y0).max(f64::EPSILON));
    let bounded = x0.is_finite() && y0.is_finite();
    // Fit the width, but never let a tall map grow past a square.
    let scale = if bounded { ((width - 2.0 * pad) / bw).min((width - 2.0 * pad) / bh) } else { 1.0 };
    let height = if bounded { bh * scale + 2.0 * pad } else { width / 2.0 };
    let dx = if bounded { (width - bw * scale) / 2.0 } else { pad };
    let place = |(x, y): (f64, f64)| (dx + (x - x0) * scale, pad + (y - y0) * scale);
    let range = scale::extent(regions.iter().map(|r| r.value)).unwrap_or((0.0, 0.0));
    let (lo, hi) = range;
    let shapes = regions
        .iter()
        .zip(&projected)
        .map(|(r, rings)| {
            let mut path = String::new();
            let (mut a, mut b) = ((f64::INFINITY, f64::INFINITY), (f64::NEG_INFINITY, f64::NEG_INFINITY));
            for ring in rings.iter().filter(|g| g.len() >= 3) {
                for (i, &p) in ring.iter().enumerate() {
                    let (x, y) = place(p);
                    (a, b) = ((a.0.min(x), a.1.min(y)), (b.0.max(x), b.1.max(y)));
                    let _ = write!(path, "{}{x:.2} {y:.2}", if i == 0 { "M" } else { "L" });
                }
                path.push('Z');
            }
            let centre = if a.0.is_finite() {
                (f64::midpoint(a.0, b.0), f64::midpoint(a.1, b.1))
            } else {
                (width / 2.0, height / 2.0)
            };
            let t = r.value.is_finite().then(|| if hi > lo { (r.value - lo) / (hi - lo) } else { 0.5 });
            GeoShape { path, t, centre }
        })
        .collect();
    Geo { width, height, shapes, points: dots.into_iter().map(|d| d.map(place)).collect(), range }
}

/// Bubble radius with area proportional to `value`: `largest` at `max`, zero at zero.
#[must_use]
pub fn bubble_radius(value: f64, max: f64, largest: f64) -> f64 {
    if max > 0.0 && value.is_finite() { largest * (value.max(0.0) / max).sqrt() } else { 0.0 }
}

/// Point indices largest value first, so small bubbles draw on top of big ones.
#[must_use]
pub fn draw_order(points: &[GeoPoint]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..points.len()).collect();
    order.sort_by(|&a, &b| points[b].value.total_cmp(&points[a].value));
    order
}

/// Inline style for a region: its own chart colour on a polygon map, its value shade `t`,
/// and its entry stagger index `i` of `n` regions.
#[must_use]
pub fn region_style(r: &Region, t: Option<f64>, i: usize, n: usize) -> String {
    let mut s = String::new();
    if let Some(c) = r.colour {
        let _ = write!(s, "fill: {};", crate::plot::colour(c));
    }
    if let Some(t) = t {
        let _ = write!(s, "--t: {t:.3}; ");
    }
    let _ = write!(s, "--i: {i}; --ox-n: {n};");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(id: &str, v: f64, x: f64) -> Region {
        Region::new(id, id, v, [(x, 0.0), (x + 10.0, 0.0), (x + 10.0, 10.0), (x, 10.0)])
    }

    #[test]
    fn shades_by_position_in_range_and_fits_the_box() {
        let g = layout(&[square("a", 10.0, 0.0), square("b", 30.0, 10.0)], &[], Projection::Equirectangular);
        assert_eq!(g.shapes[0].t, Some(0.0));
        assert_eq!(g.shapes[1].t, Some(1.0));
        assert!((g.height - 326.0).abs() < 1e-9);
        assert!(g.shapes[0].path.starts_with("M6.00 "));
        assert!((g.shapes[0].centre.0 - 163.0).abs() < 1e-9);
    }

    #[test]
    fn tall_maps_fit_a_square_and_centre() {
        let tall = Region::new("t", "t", 1.0, [(0.0, 0.0), (1.0, 0.0), (1.0, 10.0), (0.0, 10.0)]);
        let g = layout(&[tall], &[], Projection::Equirectangular);
        assert!((g.height - 640.0).abs() < 1e-9);
        assert!((g.shapes[0].centre.0 - 320.0).abs() < 1e-9);
    }

    #[test]
    fn mercator_stretches_high_latitudes() {
        let (_, y45) = project((0.0, 45.0), Projection::Mercator);
        assert!(-y45 > 45.0);
        assert!((project((0.0, 0.0), Projection::Mercator).1).abs() < 1e-9);
    }

    #[test]
    fn custom_projection_runs_with_y_up() {
        fn swap((lon, lat): (f64, f64)) -> (f64, f64) {
            (lat, lon)
        }
        assert_eq!(project((3.0, 7.0), Projection::Custom(swap)), (7.0, -3.0));
        assert_eq!(Projection::Custom(swap), Projection::Custom(swap));
        assert_ne!(Projection::Custom(swap), Projection::Mercator);
    }

    #[test]
    fn points_share_the_fit_and_unplaceable_ones_drop() {
        fn nowhere(_: (f64, f64)) -> (f64, f64) {
            (f64::NAN, 0.0)
        }
        let pts = [GeoPoint::new("p", (20.0, 10.0), 5.0)];
        let g = layout(&[square("a", 1.0, 0.0)], &pts, Projection::Equirectangular);
        let (x, y) = g.points[0].unwrap();
        assert!((x - 634.0).abs() < 1e-9 && (y - 6.0).abs() < 1e-9, "the point widens the bounds");
        assert_eq!(layout(&[], &pts, Projection::Custom(nowhere)).points[0], None);
    }

    #[test]
    fn bubbles_scale_by_area_and_draw_big_first() {
        assert!((bubble_radius(25.0, 100.0, 20.0) - 10.0).abs() < 1e-9);
        assert_eq!(bubble_radius(f64::NAN, 100.0, 20.0), 0.0);
        let pts = [GeoPoint::new("a", (0.0, 0.0), 1.0), GeoPoint::new("b", (0.0, 0.0), 9.0)];
        assert_eq!(draw_order(&pts), vec![1, 0]);
    }

    #[test]
    fn missing_values_have_no_shade() {
        assert_eq!(layout(&[square("a", f64::NAN, 0.0)], &[], Projection::Equirectangular).shapes[0].t, None);
    }
}
