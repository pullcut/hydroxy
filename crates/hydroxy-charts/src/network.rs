//! Force-directed network layout. A fixed number of Fruchterman-Reingold iterations
//! from a seeded start, so the same input always gives the same picture and nothing
//! random happens at render time. Coordinates are px in a `width`×`height` box.

#![allow(clippy::cast_precision_loss, reason = "item counts are small")]

use std::collections::HashMap;

/// A node.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Unique id, also its label.
    pub id: String,
    /// Group number; nodes in a group share a colour.
    pub group: usize,
}

impl Node {
    /// A node.
    #[must_use]
    pub fn new(id: impl Into<String>, group: usize) -> Self {
        Self { id: id.into(), group }
    }
}

/// A link between two node ids.
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    /// One end's id.
    pub source: String,
    /// The other end's id.
    pub target: String,
    /// Strength: heavier links pull their nodes closer and draw thicker.
    pub value: f64,
}

impl Link {
    /// A link.
    #[must_use]
    pub fn new(source: impl Into<String>, target: impl Into<String>, value: f64) -> Self {
        Self { source: source.into(), target: target.into(), value }
    }
}

/// A placed node.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    /// The node's id.
    pub id: String,
    /// The node's group.
    pub group: usize,
    /// Centre x.
    pub x: f64,
    /// Centre y.
    pub y: f64,
    /// Radius, growing with the number of links.
    pub radius: f64,
    /// Number of links at this node.
    pub degree: usize,
}

/// A placed link, by node index.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    /// Index of one end.
    pub source: usize,
    /// Index of the other end.
    pub target: usize,
    /// The link's strength, cleaned to a positive number.
    pub value: f64,
}

/// The laid-out network.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    /// Nodes, in input order.
    pub nodes: Vec<Placed>,
    /// Links whose ends both exist, other links and self-links dropped.
    pub edges: Vec<Edge>,
}

/// xorshift64: tiny, seedable, identical everywhere.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Lay out `nodes` and `links` in a `width`×`height` box, running `iterations` steps from `seed`.
#[must_use]
pub fn layout(nodes: &[Node], links: &[Link], width: f64, height: f64, iterations: usize, seed: u64) -> Layout {
    let n = nodes.len();
    let index: HashMap<&str, usize> = nodes.iter().enumerate().map(|(i, n)| (n.id.as_str(), i)).collect();
    let edges: Vec<Edge> = links
        .iter()
        .filter_map(|l| {
            let (s, t) = (*index.get(l.source.as_str())?, *index.get(l.target.as_str())?);
            let value = if l.value.is_finite() && l.value > 0.0 { l.value } else { 1.0 };
            (s != t).then_some(Edge { source: s, target: t, value })
        })
        .collect();
    let mut degree = vec![0usize; n];
    for e in &edges {
        degree[e.source] += 1;
        degree[e.target] += 1;
    }
    let radius = |i: usize| (3.0 * (degree[i] as f64).sqrt() + 5.0).min(15.0);
    let pad = 34.0;

    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let mut pos: Vec<(f64, f64)> = (0..n).map(|_| (rng.next() * width, rng.next() * height)).collect();
    let k = (width * height / n.max(1) as f64).sqrt() * 0.7;
    let max_value = edges.iter().map(|e| e.value).fold(1.0, f64::max);
    let t0 = width.min(height) / 8.0;
    for it in 0..iterations {
        let temp = t0 * (1.0 - it as f64 / iterations as f64);
        let mut disp = vec![(0.0, 0.0); n];
        for i in 0..n {
            for j in (i + 1)..n {
                let (mut dx, mut dy) = (pos[i].0 - pos[j].0, pos[i].1 - pos[j].1);
                let mut d = dx.hypot(dy);
                if d < 1e-6 {
                    (dx, dy, d) = (1.0, 0.0, 1.0);
                }
                let f = k * k / d / d;
                disp[i].0 += dx * f;
                disp[i].1 += dy * f;
                disp[j].0 -= dx * f;
                disp[j].1 -= dy * f;
            }
        }
        for e in &edges {
            let (dx, dy) = (pos[e.source].0 - pos[e.target].0, pos[e.source].1 - pos[e.target].1);
            let d = dx.hypot(dy).max(1e-6);
            let f = d / k * (0.5 + 0.5 * e.value / max_value);
            disp[e.source].0 -= dx * f;
            disp[e.source].1 -= dy * f;
            disp[e.target].0 += dx * f;
            disp[e.target].1 += dy * f;
        }
        for i in 0..n {
            // A light pull to the middle keeps disconnected pieces on the page.
            disp[i].0 -= (pos[i].0 - width / 2.0) * 0.02;
            disp[i].1 -= (pos[i].1 - height / 2.0) * 0.02;
            let d = disp[i].0.hypot(disp[i].1).max(1e-9);
            let step = d.min(temp);
            pos[i].0 += disp[i].0 / d * step;
            pos[i].1 += disp[i].1 / d * step;
        }
    }

    // Fit the result into the padded box, keeping its shape.
    if n > 0 {
        let (min_x, max_x) = pos.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |a, p| (a.0.min(p.0), a.1.max(p.0)));
        let (min_y, max_y) = pos.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |a, p| (a.0.min(p.1), a.1.max(p.1)));
        let (sw, sh) = ((max_x - min_x).max(1e-6), (max_y - min_y).max(1e-6));
        let scale = ((width - 2.0 * pad) / sw).min((height - 2.0 * pad) / sh).max(0.0);
        let scale = if n == 1 { 0.0 } else { scale };
        let (cx, cy) = (f64::midpoint(min_x, max_x), f64::midpoint(min_y, max_y));
        for p in &mut pos {
            *p = ((p.0 - cx).mul_add(scale, width / 2.0), (p.1 - cy).mul_add(scale, height / 2.0));
        }
    }
    let nodes = nodes
        .iter()
        .enumerate()
        .map(|(i, nd)| Placed { id: nd.id.clone(), group: nd.group, x: pos[i].0, y: pos[i].1, radius: radius(i), degree: degree[i] })
        .collect();
    Layout { nodes, edges }
}

/// Node `i` and the nodes it links to, as one flag per node.
#[must_use]
pub fn neighbours(layout: &Layout, i: usize) -> Vec<bool> {
    let mut out = vec![false; layout.nodes.len()];
    if i < out.len() {
        out[i] = true;
        for e in &layout.edges {
            if e.source == i {
                out[e.target] = true;
            } else if e.target == i {
                out[e.source] = true;
            }
        }
    }
    out
}

/// Total value of the links touching node `i`.
#[must_use]
pub fn weight(layout: &Layout, i: usize) -> f64 {
    layout.edges.iter().filter(|e| e.source == i || e.target == i).map(|e| e.value).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_sums_touching_links() {
        let nodes = [Node::new("a", 0), Node::new("b", 0), Node::new("c", 1)];
        let lay = layout(&nodes, &[Link::new("a", "b", 2.0), Link::new("b", "c", 3.0)], 200.0, 200.0, 10, 1);
        assert!((weight(&lay, 1) - 5.0).abs() < 1e-9);
        assert!((weight(&lay, 0) - 2.0).abs() < 1e-9);
    }

    fn two_clusters() -> (Vec<Node>, Vec<Link>) {
        let nodes: Vec<Node> = ["a1", "a2", "a3", "a4", "b1", "b2", "b3", "b4"].iter().map(|s| Node::new(*s, usize::from(s.starts_with('b')))).collect();
        let mut links = Vec::new();
        for g in ["a", "b"] {
            for i in 1..=4 {
                for j in (i + 1)..=4 {
                    links.push(Link::new(format!("{g}{i}"), format!("{g}{j}"), 1.0));
                }
            }
        }
        links.push(Link::new("a1", "b1", 1.0));
        (nodes, links)
    }

    #[test]
    fn same_input_and_seed_give_the_same_layout() {
        let (n, l) = two_clusters();
        assert_eq!(layout(&n, &l, 600.0, 400.0, 200, 7), layout(&n, &l, 600.0, 400.0, 200, 7));
        assert_ne!(layout(&n, &l, 600.0, 400.0, 200, 7), layout(&n, &l, 600.0, 400.0, 200, 8));
    }

    #[test]
    fn nodes_stay_inside_the_box() {
        let (n, l) = two_clusters();
        for p in layout(&n, &l, 600.0, 400.0, 200, 1).nodes {
            assert!(p.x > 0.0 && p.x < 600.0 && p.y > 0.0 && p.y < 400.0, "{p:?}");
        }
    }

    #[test]
    fn linked_nodes_end_up_closer_than_unlinked_ones() {
        let (n, l) = two_clusters();
        let lay = layout(&n, &l, 600.0, 400.0, 300, 3);
        let d = |i: usize, j: usize| (lay.nodes[i].x - lay.nodes[j].x).hypot(lay.nodes[i].y - lay.nodes[j].y);
        let inside = (d(1, 2) + d(2, 3) + d(1, 3) + d(5, 6) + d(6, 7)) / 5.0;
        let across = (d(2, 6) + d(3, 7) + d(1, 5) + d(2, 7)) / 4.0;
        assert!(inside < across, "{inside} vs {across}");
    }

    #[test]
    fn dangling_and_self_links_are_dropped_and_degree_counts() {
        let nodes = vec![Node::new("x", 0), Node::new("y", 0)];
        let links = vec![Link::new("x", "y", 1.0), Link::new("x", "nope", 1.0), Link::new("y", "y", 1.0)];
        let lay = layout(&nodes, &links, 200.0, 200.0, 50, 1);
        assert_eq!(lay.edges.len(), 1);
        assert_eq!(lay.nodes[0].degree, 1);
    }

    #[test]
    fn neighbours_include_the_node_and_its_links() {
        let (n, l) = two_clusters();
        let lay = layout(&n, &l, 600.0, 400.0, 10, 1);
        let h = neighbours(&lay, 4);
        assert_eq!(h.iter().filter(|x| **x).count(), 5, "b1 links to b2 b3 b4 and a1");
        assert!(h[0] && h[4] && !h[1]);
    }

    #[test]
    fn single_and_empty_inputs_work() {
        let one = layout(&[Node::new("x", 0)], &[], 200.0, 100.0, 20, 1);
        assert!((one.nodes[0].x - 100.0).abs() < 1e-9 && (one.nodes[0].y - 50.0).abs() < 1e-9);
        assert_eq!(layout(&[], &[], 200.0, 100.0, 20, 1).nodes, vec![]);
    }
}
