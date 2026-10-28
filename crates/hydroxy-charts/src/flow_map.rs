//! Flow maps: curved arrows from origin to destination over a base map, as wide as their
//! volume. Places and the base map are fitted by [`crate::geo`].

use std::fmt::Write as _;

use crate::geo::{self, Geo, GeoPoint, Projection, Region};

/// A movement from one place to another.
#[derive(Debug, Clone, PartialEq)]
pub struct Flow {
    /// Index of the origin in the places.
    pub from: usize,
    /// Index of the destination in the places.
    pub to: usize,
    /// The volume. Non-finite or negative volumes draw at the thinnest width.
    pub value: f64,
}

impl Flow {
    /// A flow from place `from` to place `to`.
    #[must_use]
    pub fn new(from: usize, to: usize, value: f64) -> Self {
        Self { from, to, value }
    }
}

/// One flow, ready to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct Arc {
    /// SVG `d` of the curve, from just outside the origin to the base of the arrowhead.
    pub path: String,
    /// SVG `d` of the arrowhead, its tip just outside the destination.
    pub head: String,
    /// Stroke width in viewBox units.
    pub width: f64,
    /// The curve's midpoint, where its tooltip points.
    pub mid: (f64, f64),
}

/// Everything a flow map needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLayout {
    /// The fitted base map and places.
    pub geo: Geo,
    /// One per input flow; `None` when an end can't be placed or both ends coincide.
    pub arcs: Vec<Option<Arc>>,
    /// Flow indices widest first, so thin flows draw on top.
    pub order: Vec<usize>,
}

/// Radius of a place's dot, in viewBox units.
pub const PLACE_RADIUS: f64 = 4.0;
/// Width of the thinnest flow, in viewBox units.
const MIN_WIDTH: f64 = 1.5;
/// How far the curve bows out, as a share of its length.
const BOW: f64 = 0.2;

/// Lay out `flows` between `places` over `regions`. The widest flow is `max_width`
/// viewBox units (the map is 640 wide); widths are linear in volume. Every curve bows to
/// the same side of its direction, so a flow and its return run side by side.
#[must_use]
pub fn layout(
    places: &[GeoPoint],
    flows: &[Flow],
    regions: &[Region],
    projection: Projection,
    max_width: f64,
) -> FlowLayout {
    let geo = geo::layout(regions, places, projection);
    let volume = |f: &Flow| if f.value.is_finite() { f.value.max(0.0) } else { 0.0 };
    let max = flows.iter().map(volume).fold(0.0, f64::max);
    let max_width = max_width.max(MIN_WIDTH);
    let arcs = flows
        .iter()
        .map(|f| {
            let a = geo.points.get(f.from).copied().flatten()?;
            let b = geo.points.get(f.to).copied().flatten()?;
            let width = MIN_WIDTH + if max > 0.0 { (max_width - MIN_WIDTH) * volume(f) / max } else { 0.0 };
            arc(a, b, width)
        })
        .collect();
    let mut order: Vec<usize> = (0..flows.len()).collect();
    order.sort_by(|&a, &b| volume(&flows[b]).total_cmp(&volume(&flows[a])));
    FlowLayout { geo, arcs, order }
}

fn unit((x, y): (f64, f64)) -> (f64, f64) {
    let d = x.hypot(y);
    if d > 1e-12 { (x / d, y / d) } else { (0.0, 0.0) }
}

/// A bowed curve from `a` to `b` of `width`, with an arrowhead at `b`.
fn arc(a: (f64, f64), b: (f64, f64), width: f64) -> Option<Arc> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = dx.hypot(dy);
    let head_len = 5.0 + 1.6 * width;
    let half = 3.0 + 0.5 * width;
    let gap = PLACE_RADIUS + 1.5;
    if len < 2.0 * gap + head_len + 4.0 {
        return None;
    }
    // Bow to the left of travel (y down, so this is (dy, -dx)).
    let c = (a.0 + dx / 2.0 + dy * BOW, a.1 + dy / 2.0 - dx * BOW);
    let u_end = unit((b.0 - c.0, b.1 - c.1));
    let u_start = unit((c.0 - a.0, c.1 - a.1));
    let tip = (b.0 - u_end.0 * gap, b.1 - u_end.1 * gap);
    let base = (tip.0 - u_end.0 * head_len, tip.1 - u_end.1 * head_len);
    let start = (a.0 + u_start.0 * gap, a.1 + u_start.1 * gap);
    // `base` lies on the line from the control point to `b`, so the curve meets the head
    // along its axis.
    let mut path = String::new();
    let _ = write!(path, "M{:.2} {:.2}Q{:.2} {:.2} {:.2} {:.2}", start.0, start.1, c.0, c.1, base.0, base.1);
    let (nx, ny) = (-u_end.1 * half, u_end.0 * half);
    // The head starts half a unit back so no seam shows where it meets the curve.
    let back = (base.0 - u_end.0 * 0.5, base.1 - u_end.1 * 0.5);
    let head = format!(
        "M{:.2} {:.2}L{:.2} {:.2}L{:.2} {:.2}Z",
        tip.0,
        tip.1,
        back.0 + nx,
        back.1 + ny,
        back.0 - nx,
        back.1 - ny
    );
    let mid = (0.25 * start.0 + 0.5 * c.0 + 0.25 * base.0, 0.25 * start.1 + 0.5 * c.1 + 0.25 * base.1);
    Some(Arc { path, head, width, mid })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn places() -> Vec<GeoPoint> {
        vec![
            GeoPoint::new("a", (0.0, 0.0), 0.0),
            GeoPoint::new("b", (10.0, 0.0), 0.0),
            GeoPoint::new("c", (10.0, 0.05), 0.0),
        ]
    }

    #[test]
    fn widths_scale_with_volume_and_widest_draws_first() {
        let flows = [Flow::new(0, 1, 10.0), Flow::new(1, 0, 40.0)];
        let l = layout(&places(), &flows, &[], Projection::Equirectangular, 12.0);
        let w: Vec<f64> = l.arcs.iter().map(|a| a.as_ref().map_or(0.0, |a| a.width)).collect();
        assert!((w[1] - 12.0).abs() < 1e-9);
        assert!((w[0] - (1.5 + 10.5 / 4.0)).abs() < 1e-9);
        assert_eq!(l.order, vec![1, 0]);
    }

    #[test]
    fn a_flow_and_its_return_bow_to_opposite_sides() {
        let l = layout(&places(), &[Flow::new(0, 1, 1.0), Flow::new(1, 0, 1.0)], &[], Projection::Equirectangular, 8.0);
        let (there, back) = (l.arcs[0].as_ref().map(|a| a.mid), l.arcs[1].as_ref().map(|a| a.mid));
        let (Some(there), Some(back)) = (there, back) else { unreachable!() };
        let y = l.geo.points[0].map_or(0.0, |p| p.1);
        assert!((there.1 - y) * (back.1 - y) < 0.0, "{there:?} {back:?}");
    }

    #[test]
    fn the_head_tip_stops_short_of_the_destination() {
        let l = layout(&places(), &[Flow::new(0, 1, 1.0)], &[], Projection::Equirectangular, 8.0);
        let head = l.arcs[0].as_ref().map(|a| a.head.clone()).unwrap_or_default();
        let tip: Vec<f64> =
            head[1..].split('L').next().unwrap_or_default().split(' ').filter_map(|s| s.parse().ok()).collect();
        let b = l.geo.points[1].unwrap_or_default();
        let d = (tip[0] - b.0).hypot(tip[1] - b.1);
        assert!((d - (PLACE_RADIUS + 1.5)).abs() < 1e-2, "{d}");
    }

    #[test]
    fn unplaceable_and_tiny_flows_drop() {
        let l = layout(
            &places(),
            &[Flow::new(0, 9, 1.0), Flow::new(1, 2, 1.0), Flow::new(0, 0, 1.0)],
            &[],
            Projection::Equirectangular,
            8.0,
        );
        assert!(l.arcs.iter().all(Option::is_none));
    }
}
