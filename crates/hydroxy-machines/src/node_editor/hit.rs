//! Hit-testing and wire geometry.

use super::graph::{Graph, Port, PortRef, Side};
use crate::canvas::Point;

/// Control points for a wire leaving `a` rightwards and entering `b` from the left.
fn wire_controls(a: Point, b: Point) -> (Point, Point) {
    let pull = ((b.x - a.x).abs() / 2.0).max(48.0).min(160.0 + (b.y - a.y).abs() / 4.0);
    (Point::new(a.x + pull, a.y), Point::new(b.x - pull, b.y))
}

/// SVG `d` for a wire from output point `a` to input point `b`.
#[must_use]
pub fn wire_path(a: Point, b: Point) -> String {
    let (c1, c2) = wire_controls(a, b);
    format!("M{:.1} {:.1}C{:.1} {:.1} {:.1} {:.1} {:.1} {:.1}", a.x, a.y, c1.x, c1.y, c2.x, c2.y, b.x, b.y)
}

fn wire_point(a: Point, b: Point, t: f64) -> Point {
    let (c1, c2) = wire_controls(a, b);
    let u = 1.0 - t;
    a.scale(u * u * u).plus(c1.scale(3.0 * u * u * t)).plus(c2.scale(3.0 * u * t * t)).plus(b.scale(t * t * t))
}

/// Distance from `p` to the wire `a`–`b`, sampled.
#[must_use]
pub fn wire_distance(p: Point, a: Point, b: Point) -> f64 {
    const STEPS: u32 = 24;
    let mut prev = a;
    let mut best = f64::INFINITY;
    for i in 1..=STEPS {
        let q = wire_point(a, b, f64::from(i) / f64::from(STEPS));
        best = best.min(crate::canvas::segment_distance(p, prev, q));
        prev = q;
    }
    best
}

/// What is under a world point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hit {
    /// A port.
    Port(PortRef),
    /// A node's body.
    Node(String),
    /// A wire.
    Edge(String),
    /// Empty canvas.
    Background,
}

/// The port nearest `p` within `radius` world units, topmost node first.
#[must_use]
pub fn port_at(graph: &Graph, p: Point, radius: f64, filter: impl Fn(&PortRef, &Port) -> bool) -> Option<PortRef> {
    let mut best: Option<(f64, PortRef)> = None;
    for n in graph.nodes.iter().rev() {
        if !n.rect().inflate(radius).contains(p) {
            continue;
        }
        for side in [Side::Input, Side::Output] {
            for (i, port) in n.ports(side).iter().enumerate() {
                let d = n.port_point(side, i).dist(p);
                let r = PortRef::new(&n.id, side, &port.id);
                if d <= radius && best.as_ref().is_none_or(|b| d < b.0) && filter(&r, port) {
                    best = Some((d, r));
                }
            }
        }
    }
    best.map(|b| b.1)
}

/// What is under world point `p`. `tolerance` is in world units.
#[must_use]
pub fn hit(graph: &Graph, p: Point, tolerance: f64) -> Hit {
    if let Some(port) = port_at(graph, p, tolerance, |_, _| true) {
        return Hit::Port(port);
    }
    if let Some(n) = graph.nodes.iter().rev().find(|n| n.rect().contains(p)) {
        return Hit::Node(n.id.clone());
    }
    graph
        .edges
        .iter()
        .filter_map(|e| graph.edge_points(e).map(|(a, b)| (wire_distance(p, a, b), e)))
        .filter(|(d, _)| *d <= tolerance)
        .min_by(|x, y| x.0.total_cmp(&y.0))
        .map_or(Hit::Background, |(_, e)| Hit::Edge(e.id.clone()))
}
