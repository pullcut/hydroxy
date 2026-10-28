//! Arc diagram layout: nodes in a row along a baseline, each link a semicircle above it.
//! Coordinates are px in a `width`×`height` box; `height` follows the widest arc.

#![allow(clippy::cast_precision_loss, reason = "node counts are small")]

use std::collections::HashMap;

pub use crate::network::{Link, Node};

/// Margin left and right of the first and last node.
pub const PAD: f64 = 28.0;
/// Room under the baseline for the slanted node labels.
const LABEL_ROOM: f64 = 84.0;
/// Room above the tallest arc.
const HEAD: f64 = 14.0;

/// How the nodes are ordered along the line.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Order {
    /// By group, then most-linked first within each group. Keeps clusters' arcs short.
    #[default]
    Group,
    /// Most-linked first, whatever the group.
    Degree,
    /// As given.
    Input,
}

/// A placed node.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    /// The node's id, also its label.
    pub id: String,
    /// The node's group.
    pub group: usize,
    /// Centre x on the baseline.
    pub x: f64,
    /// Dot radius, growing with the number of links.
    pub radius: f64,
    /// Number of links at this node.
    pub degree: usize,
    /// Sum of its links' values.
    pub weight: f64,
}

/// A placed link, by node index.
#[derive(Debug, Clone, PartialEq)]
pub struct Arc {
    /// Index of the node on the left.
    pub source: usize,
    /// Index of the node on the right.
    pub target: usize,
    /// The link's value, cleaned to a positive number.
    pub value: f64,
    /// SVG `d`: a semicircle from left to right, above the baseline.
    pub path: String,
    /// Stroke width in px.
    pub width: f64,
}

/// The laid-out diagram.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    /// Nodes in input order.
    pub nodes: Vec<Placed>,
    /// Arcs, shortest first, so short arcs draw first and long ones sweep over them.
    pub arcs: Vec<Arc>,
    /// Node indices from left to right, for keyboard stepping.
    pub order: Vec<usize>,
    /// Drawing width.
    pub width: f64,
    /// Drawing height, from the tallest arc to the bottom of the labels.
    pub height: f64,
    /// y of the line the nodes sit on.
    pub baseline: f64,
}

/// CSS colour for `group`: the first six get a `--chart-N` hue each, later groups share
/// the muted colour rather than cycling.
#[must_use]
pub fn colour(group: usize) -> String {
    crate::plot::colour_or_muted(group, 6)
}

/// Lay out `nodes` and `links` across `width` px. Links to unknown ids and self-links are
/// dropped; non-finite or non-positive values count as 1. With `weighted`, stroke width
/// follows value (1 to 4 px); otherwise every arc is 1.5 px.
#[must_use]
pub fn layout(nodes: &[Node], links: &[Link], width: f64, order: Order, weighted: bool) -> Layout {
    let n = nodes.len();
    let index: HashMap<&str, usize> = nodes.iter().enumerate().map(|(i, n)| (n.id.as_str(), i)).collect();
    let edges: Vec<(usize, usize, f64)> = links
        .iter()
        .filter_map(|l| {
            let (s, t) = (*index.get(l.source.as_str())?, *index.get(l.target.as_str())?);
            let value = if l.value.is_finite() && l.value > 0.0 { l.value } else { 1.0 };
            (s != t).then_some((s, t, value))
        })
        .collect();
    let mut degree = vec![0usize; n];
    let mut weight = vec![0.0; n];
    for &(s, t, v) in &edges {
        degree[s] += 1;
        degree[t] += 1;
        weight[s] += v;
        weight[t] += v;
    }
    let mut seq: Vec<usize> = (0..n).collect();
    match order {
        Order::Group => seq.sort_by_key(|&i| (nodes[i].group, std::cmp::Reverse(degree[i]))),
        Order::Degree => seq.sort_by_key(|&i| std::cmp::Reverse(degree[i])),
        Order::Input => {}
    }
    let step = if n > 1 { (width - 2.0 * PAD) / (n - 1) as f64 } else { 0.0 };
    let mut xs = vec![width / 2.0; n];
    for (slot, &i) in seq.iter().enumerate() {
        if n > 1 {
            xs[i] = PAD + slot as f64 * step;
        }
    }
    let max_r = edges.iter().map(|&(s, t, _)| (xs[s] - xs[t]).abs() / 2.0).fold(0.0, f64::max);
    let baseline = HEAD + max_r;
    let max_v = edges.iter().map(|e| e.2).fold(0.0, f64::max);
    let mut arcs: Vec<Arc> = edges
        .iter()
        .map(|&(s, t, value)| {
            let (a, b) = if xs[s] <= xs[t] { (s, t) } else { (t, s) };
            let r = (xs[b] - xs[a]) / 2.0;
            Arc {
                source: a,
                target: b,
                value,
                path: format!("M{:.2} {baseline:.2}A{r:.2} {r:.2} 0 0 1 {:.2} {baseline:.2}", xs[a], xs[b]),
                width: if weighted { 1.0 + 3.0 * value / max_v } else { 1.5 },
            }
        })
        .collect();
    arcs.sort_by(|a, b| (xs[a.target] - xs[a.source]).total_cmp(&(xs[b.target] - xs[b.source])));
    Layout {
        nodes: nodes
            .iter()
            .enumerate()
            .map(|(i, node)| Placed {
                id: node.id.clone(),
                group: node.group,
                x: xs[i],
                radius: (3.5 + 1.5 * (degree[i] as f64).sqrt()).min(9.0),
                degree: degree[i],
                weight: weight[i],
            })
            .collect(),
        arcs,
        order: seq,
        width,
        height: baseline + LABEL_ROOM,
        baseline,
    }
}

/// Which nodes are `i` or linked to it.
#[must_use]
pub fn neighbours(l: &Layout, i: usize) -> Vec<bool> {
    let mut on = vec![false; l.nodes.len()];
    if let Some(slot) = on.get_mut(i) {
        *slot = true;
    }
    for a in &l.arcs {
        if a.source == i {
            on[a.target] = true;
        } else if a.target == i {
            on[a.source] = true;
        }
    }
    on
}

/// The ids linked to node `i`, in left-to-right order.
#[must_use]
pub fn linked(l: &Layout, i: usize) -> Vec<String> {
    let near = neighbours(l, i);
    l.order.iter().filter(|&&j| j != i && near[j]).map(|&j| l.nodes[j].id.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> (Vec<Node>, Vec<Link>) {
        let nodes = vec![Node::new("a", 1), Node::new("b", 0), Node::new("c", 1), Node::new("d", 0)];
        let links = vec![
            Link::new("a", "c", 2.0),
            Link::new("b", "c", 1.0),
            Link::new("a", "zz", 1.0),
            Link::new("d", "d", 1.0),
            Link::new("a", "b", f64::NAN),
        ];
        (nodes, links)
    }

    #[test]
    fn group_order_clusters_and_ranks_by_degree() {
        let (nodes, links) = sample();
        let l = layout(&nodes, &links, 400.0, Order::Group, true);
        // Group 0: b (2 links) before d (0); group 1: a (2) and c (2) keep input order.
        assert_eq!(l.order, vec![1, 3, 0, 2]);
        assert!((l.nodes[1].x - PAD).abs() < 1e-9 && (l.nodes[2].x - (400.0 - PAD)).abs() < 1e-9);
    }

    #[test]
    fn arcs_are_semicircles_above_the_baseline() {
        let (nodes, links) = sample();
        let l = layout(&nodes, &links, 400.0, Order::Input, true);
        assert_eq!(l.arcs.len(), 3, "unknown id and self-link dropped");
        assert!(l.arcs.iter().all(|a| l.nodes[a.source].x < l.nodes[a.target].x));
        let widest = (l.nodes[2].x - l.nodes[0].x) / 2.0;
        assert!((l.baseline - HEAD - widest).abs() < 1e-9);
        assert_eq!((l.arcs[2].source, l.arcs[2].target), (0, 2), "widest arc last");
        assert!((l.arcs[2].width - 4.0).abs() < 1e-9, "heaviest arc is 4px");
        assert!(l.arcs[2].path.contains("0 0 1"), "sweeps clockwise, so over the top");
    }

    #[test]
    fn neighbours_and_empty_input() {
        let (nodes, links) = sample();
        let l = layout(&nodes, &links, 400.0, Order::Degree, false);
        assert_eq!(neighbours(&l, 2), vec![true, true, true, false]);
        assert_eq!(linked(&l, 3), Vec::<String>::new());
        assert!(l.arcs.iter().all(|a| (a.width - 1.5).abs() < 1e-9));
        let e = layout(&[], &[], 300.0, Order::Group, true);
        assert!(e.nodes.is_empty() && e.arcs.is_empty());
        assert_eq!(layout(&[Node::new("x", 0)], &[], 300.0, Order::Group, true).nodes[0].x, 150.0);
    }

    #[test]
    fn colours_never_cycle() {
        assert_eq!(colour(0), "var(--chart-1)");
        assert_eq!(colour(6), "var(--chart-muted)");
    }
}
