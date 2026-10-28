//! Hierarchical edge bundling (Holten 2006): leaves of a tree sit round a circle and each
//! link follows the tree from one leaf up to their common ancestor and back down, drawn
//! as a B-spline through those nodes. `beta` sets how tightly links hug the tree: 1
//! follows it exactly, 0 is a straight line. Coordinates are px in a `size`×`size` box;
//! angles run clockwise from twelve o'clock.

#![allow(clippy::cast_precision_loss, reason = "leaf counts are small")]
#![allow(clippy::many_single_char_names, reason = "geometry follows the published formulas")]

use std::collections::HashMap;
use std::f64::consts::{PI, TAU};
use std::fmt::Write as _;

pub use crate::hierarchy::Tree;
pub use crate::network::Link;

/// Room between the ring of leaves and the box edge, for the labels.
const LABEL_ROOM: f64 = 96.0;

/// A leaf on the ring.
#[derive(Debug, Clone, PartialEq)]
pub struct Leaf {
    /// The leaf's name, also its label.
    pub name: String,
    /// Index of its top-level branch.
    pub group: usize,
    /// Angle in radians, clockwise from twelve o'clock.
    pub angle: f64,
    /// Position on the ring.
    pub x: f64,
    /// Position on the ring.
    pub y: f64,
    /// SVG `transform` for the label: rotated to point outwards, flipped on the left half
    /// so it never reads upside down.
    pub transform: String,
    /// `text-anchor` for the label.
    pub anchor: &'static str,
}

/// A bundled link, by leaf index.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    /// Index of the leaf it leaves from.
    pub source: usize,
    /// Index of the leaf it goes to.
    pub target: usize,
    /// SVG `d`.
    pub path: String,
}

/// The laid-out diagram.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Bundle {
    /// Leaves in tree order, which is also clockwise order.
    pub leaves: Vec<Leaf>,
    /// Links between two different known leaves.
    pub edges: Vec<Edge>,
    /// Names of the top-level branches, by group.
    pub groups: Vec<String>,
    /// Width and height of the drawing.
    pub size: f64,
    /// Radius of the ring of leaves.
    pub radius: f64,
}

/// One tree node flattened: its parent, its leaf index if it is a leaf, and its children.
struct Flat {
    parent: Option<usize>,
    children: Vec<usize>,
    leaf: Option<usize>,
}

fn flatten(tree: &Tree, parent: Option<usize>, depth: usize, out: &mut Vec<Flat>, leaves: &mut Vec<(usize, String, usize)>, group: usize) -> usize {
    let id = out.len();
    out.push(Flat { parent, children: Vec::new(), leaf: None });
    if tree.children.is_empty() {
        out[id].leaf = Some(leaves.len());
        leaves.push((id, tree.name.clone(), group));
    } else {
        for (k, child) in tree.children.iter().enumerate() {
            let g = if depth == 0 { k } else { group };
            let c = flatten(child, Some(id), depth + 1, out, leaves, g);
            out[id].children.push(c);
        }
    }
    id
}

/// d3's `curveBasis`: a uniform cubic B-spline through `pts`, clamped to both ends.
fn basis(pts: &[(f64, f64)]) -> String {
    let mut d = String::new();
    let Some(&(x, y)) = pts.first() else { return d };
    let _ = write!(d, "M{x:.2} {y:.2}");
    if pts.len() == 2 {
        let _ = write!(d, "L{:.2} {:.2}", pts[1].0, pts[1].1);
        return d;
    }
    let seg = |d: &mut String, p0: (f64, f64), p1: (f64, f64), p: (f64, f64)| {
        let _ = write!(
            d,
            "C{:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
            (2.0 * p0.0 + p1.0) / 3.0,
            (2.0 * p0.1 + p1.1) / 3.0,
            (p0.0 + 2.0 * p1.0) / 3.0,
            (p0.1 + 2.0 * p1.1) / 3.0,
            (p0.0 + 4.0 * p1.0 + p.0) / 6.0,
            (p0.1 + 4.0 * p1.1 + p.1) / 6.0,
        );
    };
    let (p0, p1) = (pts[0], pts[1]);
    let _ = write!(d, "L{:.2} {:.2}", (5.0 * p0.0 + p1.0) / 6.0, (5.0 * p0.1 + p1.1) / 6.0);
    for w in pts.windows(3) {
        seg(&mut d, w[0], w[1], w[2]);
    }
    let (a, b) = (pts[pts.len() - 2], pts[pts.len() - 1]);
    seg(&mut d, a, b, b);
    let _ = write!(d, "L{:.2} {:.2}", b.0, b.1);
    d
}

/// Lay out the leaves of `tree` round a circle in a `size` px box and bundle `links`
/// (by leaf name; the first leaf wins if names repeat) through the tree with strength
/// `beta` (clamped to 0..=1). Self-links and links to unknown names are dropped.
/// Top-level branches are separated by a one-leaf gap.
#[must_use]
#[allow(clippy::too_many_lines, reason = "one pass per stage of the layout, each short")]
pub fn layout(tree: &Tree, links: &[Link], size: f64, beta: f64) -> Bundle {
    let beta = if beta.is_finite() { beta.clamp(0.0, 1.0) } else { 0.85 };
    let radius = (size / 2.0 - LABEL_ROOM).max(10.0);
    let mut flat = Vec::new();
    let mut found = Vec::new();
    flatten(tree, None, 0, &mut flat, &mut found, 0);
    let groups: Vec<String> = if tree.children.is_empty() { vec![tree.name.clone()] } else { tree.children.iter().map(|c| c.name.clone()).collect() };
    // Height of each node: leaves 0, a branch one more than its tallest child.
    let mut height = vec![0usize; flat.len()];
    for id in (0..flat.len()).rev() {
        height[id] = flat[id].children.iter().map(|&c| height[c] + 1).max().unwrap_or(0);
    }
    let top = height[0].max(1) as f64;
    // Leaf slots, with a gap between top-level branches.
    let mut slot = Vec::with_capacity(found.len());
    let mut next = 0.0_f64;
    for (k, f) in found.iter().enumerate() {
        if k > 0 && f.2 != found[k - 1].2 {
            next += 1.0;
        }
        slot.push(next);
        next += 1.0;
    }
    let slots = if groups.len() > 1 { next + 1.0 } else { next.max(1.0) };
    // Every node's angle: a leaf's slot, a branch the mean of its children.
    let mut angle = vec![0.0; flat.len()];
    for id in (0..flat.len()).rev() {
        angle[id] = match flat[id].leaf {
            Some(l) => slot[l] / slots * TAU,
            None => flat[id].children.iter().map(|&c| angle[c]).sum::<f64>() / flat[id].children.len() as f64,
        };
    }
    let c = size / 2.0;
    let at = |id: usize| {
        let r = radius * (1.0 - height[id] as f64 / top);
        (c + r * angle[id].sin(), c - r * angle[id].cos())
    };
    let leaves: Vec<Leaf> = found
        .iter()
        .map(|(id, name, group)| {
            let a = angle[*id];
            let (x, y) = at(*id);
            let deg = a.to_degrees() - 90.0;
            let left = a > PI;
            let transform = if left {
                format!("rotate({deg:.2} {c:.2} {c:.2}) translate({:.2} {c:.2}) rotate(180)", c + radius + 8.0)
            } else {
                format!("rotate({deg:.2} {c:.2} {c:.2}) translate({:.2} {c:.2})", c + radius + 8.0)
            };
            Leaf { name: name.clone(), group: *group, angle: a, x, y, transform, anchor: if left { "end" } else { "start" } }
        })
        .collect();
    let mut by_name: HashMap<&str, usize> = HashMap::new();
    for (i, l) in leaves.iter().enumerate() {
        by_name.entry(l.name.as_str()).or_insert(i);
    }
    let ancestors = |mut id: usize| {
        let mut up = vec![id];
        while let Some(p) = flat[id].parent {
            up.push(p);
            id = p;
        }
        up
    };
    let edges = links
        .iter()
        .filter_map(|l| {
            let (s, t) = (*by_name.get(l.source.as_str())?, *by_name.get(l.target.as_str())?);
            if s == t {
                return None;
            }
            let (up, down) = (ancestors(found[s].0), ancestors(found[t].0));
            // Climb from the source to the lowest common ancestor, then down to the target.
            let common = up.iter().position(|a| down.contains(a)).unwrap_or(up.len() - 1);
            let lca = up[common];
            let mut route: Vec<usize> = up[..=common].to_vec();
            let back = down.iter().position(|&a| a == lca).unwrap_or(0);
            route.extend(down[..back].iter().rev());
            let raw: Vec<(f64, f64)> = route.iter().map(|&id| at(id)).collect();
            // Straighten towards the chord by 1 - beta.
            let (p0, pn) = (raw[0], raw[raw.len() - 1]);
            let last = (raw.len() - 1) as f64;
            let pts: Vec<(f64, f64)> = raw
                .iter()
                .enumerate()
                .map(|(k, p)| {
                    let f = k as f64 / last;
                    (beta * p.0 + (1.0 - beta) * (p0.0 + f * (pn.0 - p0.0)), beta * p.1 + (1.0 - beta) * (p0.1 + f * (pn.1 - p0.1)))
                })
                .collect();
            Some(Edge { source: s, target: t, path: basis(&pts) })
        })
        .collect();
    Bundle { leaves, edges, groups, size, radius }
}

/// How leaf `j` relates to the picked leaf `i`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// `i` links to it.
    Target,
    /// It links to `i`.
    Source,
    /// Links both ways.
    Both,
}

impl Role {
    /// The `data-role` value.
    #[must_use]
    pub const fn attr(self) -> &'static str {
        match self {
            Role::Target => "target",
            Role::Source => "source",
            Role::Both => "both",
        }
    }
}

/// Each leaf's relation to leaf `i`; `None` for leaves it has no link with, and for `i`.
#[must_use]
pub fn roles(b: &Bundle, i: usize) -> Vec<Option<Role>> {
    let mut out = vec![None; b.leaves.len()];
    for e in &b.edges {
        let (j, role) = if e.source == i {
            (e.target, Role::Target)
        } else if e.target == i {
            (e.source, Role::Source)
        } else {
            continue;
        };
        out[j] = Some(match out[j] {
            Some(r) if r != role => Role::Both,
            _ => role,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> (Tree, Vec<Link>) {
        let tree = Tree::branch(
            "root",
            vec![
                Tree::branch("app", vec![Tree::leaf("ui", 1.0), Tree::leaf("api", 1.0)]),
                Tree::branch("core", vec![Tree::branch("data", vec![Tree::leaf("db", 1.0), Tree::leaf("cache", 1.0)]), Tree::leaf("log", 1.0)]),
            ],
        );
        let links = vec![Link::new("ui", "api", 1.0), Link::new("ui", "db", 1.0), Link::new("db", "ui", 1.0), Link::new("ui", "ui", 1.0), Link::new("ui", "nope", 1.0)];
        (tree, links)
    }

    #[test]
    fn leaves_ring_the_circle_with_gaps_between_branches() {
        let (tree, links) = sample();
        let b = layout(&tree, &links, 500.0, 0.85);
        let names: Vec<&str> = b.leaves.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["ui", "api", "db", "cache", "log"]);
        assert_eq!(b.groups, ["app", "core"]);
        // 5 leaves + 1 gap between branches + 1 gap closing the circle = 7 slots.
        assert!((b.leaves[2].angle - 3.0 / 7.0 * TAU).abs() < 1e-9);
        for l in &b.leaves {
            assert!(((l.x - 250.0).hypot(l.y - 250.0) - b.radius).abs() < 1e-9);
        }
        assert_eq!((b.leaves[0].anchor, b.leaves[4].anchor), ("start", "end"));
    }

    #[test]
    fn links_follow_the_tree() {
        let (tree, links) = sample();
        let b = layout(&tree, &links, 500.0, 1.0);
        assert_eq!(b.edges.len(), 3, "self-link and unknown name dropped");
        // Siblings: leaf, parent, leaf, so one spline segment plus end lines.
        assert_eq!(b.edges[0].path.matches('C').count(), 2);
        // ui → app → root → core → data → db: six control points, five curves.
        assert_eq!(b.edges[1].path.matches('C').count(), 5);
        assert!(b.edges[1].path.starts_with(&format!("M{:.2} {:.2}", b.leaves[0].x, b.leaves[0].y)));
        assert!(b.edges[1].path.ends_with(&format!("L{:.2} {:.2}", b.leaves[2].x, b.leaves[2].y)));
    }

    #[test]
    fn beta_zero_is_a_straight_line() {
        let (tree, links) = sample();
        let b = layout(&tree, &links, 500.0, 0.0);
        let (s, t) = (&b.leaves[0], &b.leaves[2]);
        // Every number pair in the path lies on the chord from s to t.
        let nums: Vec<f64> = b.edges[1].path.split(|c: char| c.is_ascii_alphabetic() || c == ' ').filter_map(|v| v.parse().ok()).collect();
        for p in nums.chunks(2) {
            let cross = (t.x - s.x) * (p[1] - s.y) - (t.y - s.y) * (p[0] - s.x);
            assert!(cross.abs() / (t.x - s.x).hypot(t.y - s.y) < 0.02, "{p:?} off the chord");
        }
    }

    #[test]
    fn roles_split_in_and_out() {
        let (tree, links) = sample();
        let b = layout(&tree, &links, 500.0, 0.85);
        assert_eq!(roles(&b, 0), vec![None, Some(Role::Target), Some(Role::Both), None, None]);
        assert_eq!(roles(&b, 1), vec![Some(Role::Source), None, None, None, None]);
    }

    #[test]
    fn a_lone_leaf_is_safe() {
        let b = layout(&Tree::leaf("solo", 1.0), &[Link::new("solo", "solo", 1.0)], 300.0, 0.5);
        assert_eq!(b.leaves.len(), 1);
        assert_eq!(b.edges, Vec::<Edge>::new());
    }
}
