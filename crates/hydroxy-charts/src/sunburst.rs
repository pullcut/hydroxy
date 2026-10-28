//! Radial partition: each ring is one level of the tree, each arc's sweep is its share
//! of the zoomed node. Angles start at 12 o'clock and run clockwise.

#![allow(clippy::cast_precision_loss, reason = "item counts are small")]

use crate::hierarchy::Tree;
use std::f64::consts::{PI, TAU};
use std::fmt::Write as _;

/// One arc.
#[derive(Debug, Clone, PartialEq)]
pub struct Arc {
    /// Path from the tree root; zoom here by passing it to [`layout`].
    pub path: Vec<usize>,
    /// Ring number, 1 next to the centre.
    pub depth: usize,
    /// Node name.
    pub name: String,
    /// Node size.
    pub value: f64,
    /// Share of the zoomed node, 0..=1.
    pub share: f64,
    /// Start angle in radians.
    pub start: f64,
    /// End angle in radians.
    pub end: f64,
    /// Which top-level child this sits under, for colour.
    pub colour: usize,
    /// It has children, so a click can zoom into it.
    pub zoomable: bool,
    /// SVG `d` for the filled sector.
    pub path_d: String,
}

/// The laid-out chart.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    /// Arcs in tree order, rings outward.
    pub arcs: Vec<Arc>,
    /// Radius of the clickable centre disc.
    pub hole: f64,
    /// Width of one ring.
    pub ring: f64,
    /// Side of the square box the layout fills.
    pub size: f64,
}

impl Layout {
    /// Where arc `i` sits for a tooltip: the middle of its sweep, halfway across its
    /// ring, as 0..=1 fractions of the box from the left and top.
    #[must_use]
    pub fn anchor(&self, i: usize) -> Option<(f64, f64)> {
        let arc = self.arcs.get(i)?;
        if self.size <= 0.0 {
            return None;
        }
        let radius = self.ring.mul_add(arc.depth as f64 - 0.5, self.hole);
        let (px, py) = polar(self.size / 2.0, radius, f64::midpoint(arc.start, arc.end));
        Some((px / self.size, py / self.size))
    }
}

fn polar(c: f64, r: f64, a: f64) -> (f64, f64) {
    (c + r * a.sin(), c - r * a.cos())
}

fn sector(c: f64, r0: f64, r1: f64, a0: f64, a1: f64) -> String {
    // One arc can't close on itself, so a full turn is drawn just short of it.
    let a1 = a1.min(a0 + TAU - 1e-4);
    let large = i32::from(a1 - a0 > PI);
    let (ox0, oy0) = polar(c, r1, a0);
    let (ox1, oy1) = polar(c, r1, a1);
    let (ix1, iy1) = polar(c, r0, a1);
    let (ix0, iy0) = polar(c, r0, a0);
    let mut d = String::new();
    let _ = write!(d, "M{ox0:.3} {oy0:.3}A{r1:.3} {r1:.3} 0 {large} 1 {ox1:.3} {oy1:.3}L{ix1:.3} {iy1:.3}A{r0:.3} {r0:.3} 0 {large} 0 {ix0:.3} {iy0:.3}Z");
    d
}

struct Ctx {
    c: f64,
    hole: f64,
    ring: f64,
    max_depth: usize,
}

#[allow(clippy::too_many_arguments, reason = "recursive helper over one shared context")]
fn place(node: &Tree, path: &[usize], (a0, a1): (f64, f64), depth: usize, colour: Option<usize>, zoom_total: f64, cx: &Ctx, out: &mut Vec<Arc>) {
    let kids: Vec<(usize, &Tree, f64)> = node.children.iter().enumerate().map(|(i, c)| (i, c, c.total())).filter(|k| k.2 > 0.0).collect();
    let total: f64 = kids.iter().map(|k| k.2).sum();
    let mut start = a0;
    for (idx, child, value) in kids {
        let end = start + (a1 - a0) * value / total;
        let mut p = path.to_vec();
        p.push(idx);
        let col = colour.unwrap_or(idx);
        let r0 = cx.ring.mul_add((depth - 1) as f64, cx.hole) + 1.0;
        let r1 = cx.ring.mul_add(depth as f64, cx.hole);
        // A hairline of padding between neighbours, in radians at the middle of the ring.
        let pad = (0.75 / f64::midpoint(r0, r1)).min((end - start) / 4.0);
        out.push(Arc {
            path: p.clone(),
            depth,
            name: child.name.clone(),
            value,
            share: value / zoom_total,
            start,
            end,
            colour: col,
            zoomable: !child.children.is_empty(),
            path_d: sector(cx.c, r0, r1, start + pad, end - pad),
        });
        if depth < cx.max_depth {
            place(child, &p, (start, end), depth + 1, Some(col), zoom_total, cx, out);
        }
        start = end;
    }
}

/// Arcs for the node at `zoom` (clamped to a valid path) in a `size`×`size` box, up to
/// `max_depth` rings out. The centre hole is left for the zoomed node's own label.
#[must_use]
pub fn layout(root: &Tree, zoom: &[usize], size: f64, max_depth: usize) -> Layout {
    let zoom = root.valid_path(zoom);
    let Some(node) = root.node_at(&zoom) else { return Layout::default() };
    let max_depth = max_depth.max(1);
    let outer = size / 2.0 - 2.0;
    let hole = outer * 0.26;
    let cx = Ctx { c: size / 2.0, hole, ring: (outer - hole) / max_depth as f64, max_depth };
    let mut arcs = Vec::new();
    place(node, &zoom, (0.0, TAU), 1, None, node.total(), &cx, &mut arcs);
    Layout { arcs, hole, ring: cx.ring, size }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Tree {
        Tree::branch(
            "All",
            vec![
                Tree::branch("A", vec![Tree::leaf("a1", 30.0), Tree::leaf("a2", 10.0)]),
                Tree::leaf("B", 40.0),
                Tree::leaf("C", 20.0),
                Tree::leaf("Zero", 0.0),
            ],
        )
    }

    #[test]
    fn top_ring_sweeps_sum_to_a_full_turn() {
        let l = layout(&sample(), &[], 300.0, 3);
        let top: Vec<&Arc> = l.arcs.iter().filter(|a| a.depth == 1).collect();
        assert_eq!(top.len(), 3);
        let sweep: f64 = top.iter().map(|a| a.end - a.start).sum();
        assert!((sweep - TAU).abs() < 1e-9);
        assert!((top[0].share - 0.4).abs() < 1e-9 && (top[1].share - 0.4).abs() < 1e-9);
    }

    #[test]
    fn children_sit_inside_their_parents_sweep_one_ring_out() {
        let l = layout(&sample(), &[], 300.0, 3);
        let a = l.arcs.iter().find(|x| x.name == "A").expect("A");
        let kids: Vec<&Arc> = l.arcs.iter().filter(|x| x.depth == 2).collect();
        assert_eq!(kids.len(), 2);
        assert!(kids.iter().all(|k| k.start >= a.start - 1e-9 && k.end <= a.end + 1e-9 && k.colour == a.colour));
        assert!((kids[0].end - kids[0].start - 0.75 * (a.end - a.start)).abs() < 1e-9);
    }

    #[test]
    fn anchors_sit_mid_sweep_mid_ring() {
        // One leaf fills the circle; its middle is at 6 o'clock, halfway across the ring.
        let l = layout(&Tree::branch("r", vec![Tree::leaf("only", 5.0)]), &[], 200.0, 1);
        let (x, y) = l.anchor(0).expect("anchor");
        assert!((x - 0.5).abs() < 1e-9);
        assert!((y - (100.0 + l.hole + l.ring / 2.0) / 200.0).abs() < 1e-9);
        assert_eq!(l.anchor(1), None);
    }

    #[test]
    fn max_depth_limits_the_rings() {
        assert!(layout(&sample(), &[], 300.0, 1).arcs.iter().all(|a| a.depth == 1));
    }

    #[test]
    fn zooming_makes_the_chosen_node_the_whole_circle() {
        let l = layout(&sample(), &[0], 300.0, 3);
        assert_eq!(l.arcs.len(), 2);
        assert!(l.arcs.iter().all(|a| a.path[0] == 0 && a.depth == 1));
        assert!((l.arcs[0].start).abs() < 1e-9 && (l.arcs[1].end - TAU).abs() < 1e-9);
        assert_eq!(layout(&sample(), &[7], 300.0, 3), layout(&sample(), &[], 300.0, 3));
    }

    #[test]
    fn a_lone_child_still_draws_a_closed_sector() {
        let t = Tree::branch("r", vec![Tree::leaf("only", 5.0)]);
        let a = &layout(&t, &[], 200.0, 2).arcs[0];
        assert!(a.path_d.starts_with('M') && a.path_d.ends_with('Z'));
        assert!(!a.path_d.contains("NaN"));
    }

    #[test]
    fn arcs_are_ordered_outwards_from_the_hole() {
        let l = layout(&sample(), &[], 300.0, 2);
        assert!(l.hole > 0.0 && l.hole < 150.0);
        assert_eq!(layout(&Tree::branch("r", vec![]), &[], 100.0, 2).arcs, vec![]);
    }
}
