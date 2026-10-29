//! Hive plot layout: two to four axes radiating from the centre. A node sits on the
//! axis for its type, as far out as its value, and links between axes are curves that
//! bend through the angle between them. Coordinates are px in a `SIZE`×`SIZE` box;
//! angles run clockwise from twelve o'clock.

#![allow(clippy::many_single_char_names, reason = "polar geometry reads best in a, b, r, d")]

use std::collections::BTreeMap;
use std::f64::consts::{PI, TAU};

pub use crate::network::Link;

/// Width and height of the drawing.
pub const SIZE: f64 = 440.0;
const INNER: f64 = 34.0;
const OUTER: f64 = 178.0;
const LABEL: f64 = 194.0;

/// A node to place.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Unique id, also its label.
    pub id: String,
    /// Which axis it sits on, from 0.
    pub axis: usize,
    /// Where along the axis: low values near the centre, high values at the tip.
    pub value: f64,
}

impl Node {
    /// A node on `axis` at `value`.
    #[must_use]
    pub fn new(id: impl Into<String>, axis: usize, value: f64) -> Self {
        Self { id: id.into(), axis, value }
    }
}

/// One axis line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Axis {
    /// Angle in radians, clockwise from twelve o'clock.
    pub angle: f64,
    /// Inner end.
    pub from: (f64, f64),
    /// Outer end.
    pub to: (f64, f64),
    /// Where the axis name sits, past the tip.
    pub label: (f64, f64),
    /// `text-anchor` for the name.
    pub anchor: &'static str,
}

/// A placed node.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    /// The node's id.
    pub id: String,
    /// Its axis.
    pub axis: usize,
    /// Its value.
    pub value: f64,
    /// Centre x.
    pub x: f64,
    /// Centre y.
    pub y: f64,
    /// Number of links drawn at this node.
    pub degree: usize,
}

/// A placed link, by node index.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    /// Index of the source node.
    pub source: usize,
    /// Index of the target node.
    pub target: usize,
    /// The link's value, cleaned to a positive number.
    pub value: f64,
    /// SVG `d`: one cubic Bézier from source to target.
    pub path: String,
}

/// The laid-out hive.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Hive {
    /// One per axis.
    pub axes: Vec<Axis>,
    /// Nodes on an existing axis with a finite value, in input order.
    pub nodes: Vec<Placed>,
    /// Links between nodes on different axes.
    pub edges: Vec<Edge>,
    /// The value range mapped onto every axis, centre to tip.
    pub domain: (f64, f64),
}

fn point(r: f64, a: f64) -> (f64, f64) {
    crate::geom::polar(SIZE / 2.0, r, a)
}

/// Lay out `nodes` on `axes` axes (clamped to 2..=4). Nodes on a missing axis or with a
/// non-finite value are dropped, as are links to unknown ids and links that stay on one
/// axis. Every axis shares one value scale, so positions compare across axes.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "axis counts are tiny")]
pub fn layout(nodes: &[Node], links: &[Link], axes: usize) -> Hive {
    let count = axes.clamp(2, 4);
    // Three or four axes share the circle from twelve o'clock; two sit 120° apart,
    // either side of the top.
    let spacing = TAU / count.max(3) as f64;
    let start = if count == 2 { -spacing / 2.0 } else { 0.0 };
    let axes: Vec<Axis> = (0..count)
        .map(|k| {
            let angle = start + k as f64 * spacing;
            let s = angle.sin();
            Axis {
                angle,
                from: point(INNER, angle),
                to: point(OUTER, angle),
                label: point(LABEL, angle),
                anchor: if s > 0.3 {
                    "start"
                } else if s < -0.3 {
                    "end"
                } else {
                    "middle"
                },
            }
        })
        .collect();
    let kept: Vec<&Node> = nodes.iter().filter(|n| n.axis < count && n.value.is_finite()).collect();
    let (lo, hi) = crate::scale::extent(kept.iter().map(|n| n.value)).unwrap_or((0.0, 0.0));
    let radius =
        |v: f64| if hi > lo { INNER + (v - lo) / (hi - lo) * (OUTER - INNER) } else { f64::midpoint(INNER, OUTER) };
    let mut placed: Vec<Placed> = kept
        .iter()
        .map(|n| {
            let (x, y) = point(radius(n.value), axes[n.axis].angle);
            Placed { id: n.id.clone(), axis: n.axis, value: n.value, x, y, degree: 0 }
        })
        .collect();
    let index: BTreeMap<&str, usize> = placed.iter().enumerate().map(|(i, n)| (n.id.as_str(), i)).collect();
    let mut edges = Vec::new();
    for l in links {
        let (Some(&s), Some(&t)) = (index.get(l.source.as_str()), index.get(l.target.as_str())) else { continue };
        let (a, b) = (&placed[s], &placed[t]);
        if a.axis == b.axis {
            continue;
        }
        let (aa, ab) = (axes[a.axis].angle, axes[b.axis].angle);
        // Bend through the bisector of the smaller angle between the two axes.
        let d = (ab - aa + PI).rem_euclid(TAU) - PI;
        let mid = aa + d / 2.0;
        let (ra, rb) = (radius(a.value), radius(b.value));
        let (c1, c2) = (point(ra, mid), point(rb, mid));
        let path =
            format!("M{:.2} {:.2}C{:.2} {:.2} {:.2} {:.2} {:.2} {:.2}", a.x, a.y, c1.0, c1.1, c2.0, c2.1, b.x, b.y);
        let value = if l.value.is_finite() && l.value > 0.0 { l.value } else { 1.0 };
        edges.push(Edge { source: s, target: t, value, path });
    }
    for e in &edges {
        placed[e.source].degree += 1;
        placed[e.target].degree += 1;
    }
    Hive { axes, nodes: placed, edges, domain: (lo, hi) }
}

/// Which nodes are `i` or linked to it.
#[must_use]
pub fn neighbours(h: &Hive, i: usize) -> Vec<bool> {
    let mut on = vec![false; h.nodes.len()];
    if let Some(slot) = on.get_mut(i) {
        *slot = true;
    }
    for e in &h.edges {
        if e.source == i {
            on[e.target] = true;
        } else if e.target == i {
            on[e.source] = true;
        }
    }
    on
}

/// Node indices axis by axis, centre outwards, for keyboard stepping.
#[must_use]
pub fn order(h: &Hive) -> Vec<usize> {
    let mut o: Vec<usize> = (0..h.nodes.len()).collect();
    o.sort_by(|&a, &b| h.nodes[a].axis.cmp(&h.nodes[b].axis).then(h.nodes[a].value.total_cmp(&h.nodes[b].value)));
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> (Vec<Node>, Vec<Link>) {
        let nodes = vec![
            Node::new("a", 0, 0.0),
            Node::new("b", 1, 10.0),
            Node::new("c", 2, 5.0),
            Node::new("d", 0, 10.0),
            Node::new("x", 7, 1.0),
            Node::new("y", 1, f64::NAN),
        ];
        let links = vec![
            Link::new("a", "b", 1.0),
            Link::new("a", "d", 1.0),
            Link::new("c", "d", 2.0),
            Link::new("a", "x", 1.0),
        ];
        (nodes, links)
    }

    #[test]
    fn nodes_sit_on_their_axis_by_value() {
        let (nodes, links) = sample();
        let h = layout(&nodes, &links, 3);
        assert_eq!(h.nodes.len(), 4, "missing axis and NaN dropped");
        assert_eq!(h.domain, (0.0, 10.0));
        let dist = |p: &Placed| (p.x - SIZE / 2.0).hypot(p.y - SIZE / 2.0);
        assert!((dist(&h.nodes[0]) - INNER).abs() < 1e-9);
        assert!((dist(&h.nodes[1]) - OUTER).abs() < 1e-9);
        // Axis 0 points straight up.
        assert!((h.nodes[3].x - SIZE / 2.0).abs() < 1e-9 && h.nodes[3].y < SIZE / 2.0);
    }

    #[test]
    fn links_cross_axes_only() {
        let (nodes, links) = sample();
        let h = layout(&nodes, &links, 3);
        let pairs: Vec<(usize, usize)> = h.edges.iter().map(|e| (e.source, e.target)).collect();
        assert_eq!(pairs, vec![(0, 1), (2, 3)]);
        assert_eq!(h.nodes[0].degree, 1);
        assert!(h.edges[0].path.starts_with('M') && h.edges[0].path.contains('C'));
        assert_eq!(neighbours(&h, 3), vec![false, false, true, true]);
        assert_eq!(order(&h), vec![0, 3, 1, 2]);
    }

    #[test]
    fn axis_counts_clamp_and_two_axes_are_symmetric() {
        let h = layout(&[], &[], 1);
        assert_eq!(h.axes.len(), 2);
        assert!((h.axes[0].angle + h.axes[1].angle).abs() < 1e-9);
        assert_eq!(layout(&[], &[], 9).axes.len(), 4);
        assert_eq!(h.domain, (0.0, 0.0));
    }
}
