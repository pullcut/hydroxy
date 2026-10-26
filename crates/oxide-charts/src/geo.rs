//! Choropleth geometry: project polygons, fit them to a viewBox and shade by value.

use std::f64::consts::PI;
use std::fmt::Write as _;

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
}

impl Region {
    /// A region with a single ring.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, value: f64, ring: impl Into<Vec<(f64, f64)>>) -> Self {
        Self { id: id.into(), name: name.into(), value, rings: vec![ring.into()] }
    }
}

/// How longitude and latitude become x and y.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Projection {
    /// Plate carrée: x and y are the degrees themselves.
    #[default]
    Equirectangular,
    /// Web Mercator: north-up and conformal; latitude is clamped to ±85°.
    Mercator,
}

/// Unscaled planar position of a point, y pointing down.
#[must_use]
pub fn project((lon, lat): (f64, f64), projection: Projection) -> (f64, f64) {
    match projection {
        Projection::Equirectangular => (lon, -lat),
        Projection::Mercator => {
            let phi = lat.clamp(-85.0, 85.0).to_radians();
            (lon, -(PI / 4.0 + phi / 2.0).tan().ln().to_degrees())
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
}

/// Everything a choropleth needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct Geo {
    /// viewBox width.
    pub width: f64,
    /// viewBox height, from the map's aspect ratio.
    pub height: f64,
    /// One per input region.
    pub shapes: Vec<GeoShape>,
    /// Smallest and largest value.
    pub range: (f64, f64),
}

/// Fit `regions` into a viewBox 640 wide.
#[must_use]
pub fn layout(regions: &[Region], projection: Projection) -> Geo {
    let (width, pad) = (640.0, 6.0);
    let projected: Vec<Vec<Vec<(f64, f64)>>> =
        regions.iter().map(|r| r.rings.iter().map(|ring| ring.iter().map(|&p| project(p, projection)).collect()).collect()).collect();
    let pts = || projected.iter().flatten().flatten();
    let (x0, x1) = pts().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| (a.min(p.0), b.max(p.0)));
    let (y0, y1) = pts().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| (a.min(p.1), b.max(p.1)));
    let (bw, bh) = ((x1 - x0).max(f64::EPSILON), (y1 - y0).max(f64::EPSILON));
    let bounded = x0.is_finite() && y0.is_finite();
    let scale = if bounded { (width - 2.0 * pad) / bw } else { 1.0 };
    let height = if bounded { (bh * scale + 2.0 * pad).min(width) } else { width / 2.0 };
    let finite = || regions.iter().map(|r| r.value).filter(|v| v.is_finite());
    let (lo, hi) = (finite().fold(f64::INFINITY, f64::min), finite().fold(f64::NEG_INFINITY, f64::max));
    let range = if lo.is_finite() { (lo, hi) } else { (0.0, 0.0) };
    let shapes = regions
        .iter()
        .zip(&projected)
        .map(|(r, rings)| {
            let mut path = String::new();
            for ring in rings.iter().filter(|g| g.len() >= 3) {
                for (i, &(x, y)) in ring.iter().enumerate() {
                    let _ = write!(path, "{}{:.2} {:.2}", if i == 0 { "M" } else { "L" }, pad + (x - x0) * scale, pad + (y - y0) * scale);
                }
                path.push('Z');
            }
            let t = r.value.is_finite().then(|| if hi > lo { (r.value - lo) / (hi - lo) } else { 0.5 });
            GeoShape { path, t }
        })
        .collect();
    Geo { width, height, shapes, range }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(id: &str, v: f64, x: f64) -> Region {
        Region::new(id, id, v, [(x, 0.0), (x + 10.0, 0.0), (x + 10.0, 10.0), (x, 10.0)])
    }

    #[test]
    fn shades_by_position_in_range_and_fits_the_box() {
        let g = layout(&[square("a", 10.0, 0.0), square("b", 30.0, 10.0)], Projection::Equirectangular);
        assert_eq!(g.shapes[0].t, Some(0.0));
        assert_eq!(g.shapes[1].t, Some(1.0));
        assert!((g.height - 326.0).abs() < 1e-9);
        assert!(g.shapes[0].path.starts_with("M6.00 "));
    }

    #[test]
    fn mercator_stretches_high_latitudes() {
        let (_, y45) = project((0.0, 45.0), Projection::Mercator);
        assert!(-y45 > 45.0);
        assert!((project((0.0, 0.0), Projection::Mercator).1).abs() < 1e-9);
    }

    #[test]
    fn missing_values_have_no_shade() {
        assert_eq!(layout(&[square("a", f64::NAN, 0.0)], Projection::Equirectangular).shapes[0].t, None);
    }
}
