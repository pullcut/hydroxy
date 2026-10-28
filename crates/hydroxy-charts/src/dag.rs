//! Layered (Sugiyama-style) layout for a directed graph drawn left to right: cycles are
//! broken by reversing back edges, nodes get the longest-path layer, long edges pass
//! through dummy points, and barycentre sweeps reduce crossings. Coordinates are px in
//! a `width`×`height` box.

#![allow(clippy::cast_precision_loss, reason = "node counts are small")]

use crate::network::{Link, Node};
use std::collections::HashMap;
use std::fmt::Write as _;

/// A placed node.
#[derive(Debug, Clone, PartialEq)]
pub struct DagNode {
    /// The node's id, also its label.
    pub id: String,
    /// The node's group.
    pub group: usize,
    /// Column, 0 for nodes nothing points to.
    pub layer: usize,
    /// Centre x.
    pub x: f64,
    /// Centre y.
    pub y: f64,
}

/// A placed edge, by node index.
#[derive(Debug, Clone, PartialEq)]
pub struct DagEdge {
    /// Where the edge starts.
    pub source: usize,
    /// Where it ends.
    pub target: usize,
    /// The link's weight, cleaned to a positive number.
    pub value: f64,
    /// SVG `d`, a smooth curve through any dummy points.
    pub path: String,
    /// True when the edge closes a cycle and was reversed for layering.
    pub reversed: bool,
}

/// The laid-out graph.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dag {
    /// Nodes, in input order.
    pub nodes: Vec<DagNode>,
    /// Edges whose ends both exist; self-links are dropped.
    pub edges: Vec<DagEdge>,
    /// Node indices by layer, then top to bottom: reading order for the keyboard.
    pub order: Vec<usize>,
    /// Number of layers.
    pub layers: usize,
    /// Edge crossings left after ordering.
    pub crossings: usize,
}

/// Crossings between consecutive layers. `pos[v]` is a vertex's index in its layer;
/// `segs` are (upper, lower) vertex pairs one layer apart.
fn count_crossings(segs: &[(usize, usize)], pos: &[usize]) -> usize {
    let mut n = 0;
    for (k, &(a, b)) in segs.iter().enumerate() {
        for &(c, d) in &segs[k + 1..] {
            let (pa, pb, pc, pd) = (pos[a], pos[b], pos[c], pos[d]);
            if (pa < pc && pb > pd) || (pa > pc && pb < pd) {
                n += 1;
            }
        }
    }
    n
}

fn positions(rows: &[Vec<usize>], vertices: usize) -> Vec<usize> {
    let mut pos = vec![0; vertices];
    for row in rows {
        for (k, &v) in row.iter().enumerate() {
            pos[v] = k;
        }
    }
    pos
}

/// Barycentre sweeps, down then up, keeping the ordering with the fewest crossings.
fn reduce_crossings(mut rows: Vec<Vec<usize>>, segs: &[(usize, usize)], vertices: usize) -> (Vec<Vec<usize>>, usize) {
    let mut best = (rows.clone(), count_crossings(segs, &positions(&rows, vertices)));
    let mut up: Vec<Vec<usize>> = vec![Vec::new(); vertices];
    let mut down: Vec<Vec<usize>> = vec![Vec::new(); vertices];
    for &(a, b) in segs {
        down[a].push(b);
        up[b].push(a);
    }
    for sweep in 0..8 {
        let downward = sweep % 2 == 0;
        let layers: Vec<usize> =
            if downward { (1..rows.len()).collect() } else { (0..rows.len().saturating_sub(1)).rev().collect() };
        for l in layers {
            let pos = positions(&rows, vertices);
            let mut keyed: Vec<(f64, usize)> = rows[l]
                .iter()
                .enumerate()
                .map(|(k, &v)| {
                    let nb = if downward { &up[v] } else { &down[v] };
                    let key = if nb.is_empty() {
                        k as f64
                    } else {
                        nb.iter().map(|&u| pos[u] as f64).sum::<f64>() / nb.len() as f64
                    };
                    (key, v)
                })
                .collect();
            keyed.sort_by(|a, b| a.0.total_cmp(&b.0));
            rows[l] = keyed.into_iter().map(|(_, v)| v).collect();
        }
        let c = count_crossings(segs, &positions(&rows, vertices));
        if c < best.1 {
            best = (rows.clone(), c);
        }
    }
    best
}

/// Lay out `nodes` and `links` (source → target) in a `width`×`height` box. `node_width`
/// is the drawn width of a node box, so the first and last columns stay inside.
#[must_use]
#[allow(clippy::too_many_lines, reason = "the Sugiyama steps read best in sequence")]
pub fn layout(nodes: &[Node], links: &[Link], width: f64, height: f64, node_width: f64) -> Dag {
    let n = nodes.len();
    let index: HashMap<&str, usize> = nodes.iter().enumerate().map(|(i, node)| (node.id.as_str(), i)).collect();
    let mut edges: Vec<DagEdge> = links
        .iter()
        .filter_map(|l| {
            let (s, t) = (*index.get(l.source.as_str())?, *index.get(l.target.as_str())?);
            let value = if l.value.is_finite() && l.value > 0.0 { l.value } else { 1.0 };
            (s != t).then(|| DagEdge { source: s, target: t, value, path: String::new(), reversed: false })
        })
        .collect();

    // 1. Break cycles: an iterative DFS marks edges that point back up the stack.
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (k, e) in edges.iter().enumerate() {
        out[e.source].push(k);
    }
    let mut state = vec![0u8; n]; // 0 new, 1 on stack, 2 done
    for root in 0..n {
        if state[root] != 0 {
            continue;
        }
        let mut stack = vec![(root, 0usize)];
        state[root] = 1;
        while let Some(&mut (v, ref mut next)) = stack.last_mut() {
            if let Some(&k) = out[v].get(*next) {
                *next += 1;
                let t = edges[k].target;
                match state[t] {
                    0 => {
                        state[t] = 1;
                        stack.push((t, 0));
                    }
                    1 => edges[k].reversed = true,
                    _ => {}
                }
            } else {
                state[v] = 2;
                stack.pop();
            }
        }
    }
    let dir = |e: &DagEdge| if e.reversed { (e.target, e.source) } else { (e.source, e.target) };

    // 2. Longest-path layering in topological order (Kahn).
    let mut in_degree = vec![0usize; n];
    let mut succ: Vec<Vec<usize>> = vec![Vec::new(); n];
    for e in &edges {
        let (s, t) = dir(e);
        succ[s].push(t);
        in_degree[t] += 1;
    }
    let mut layer = vec![0usize; n];
    let mut queue: Vec<usize> = (0..n).filter(|&v| in_degree[v] == 0).collect();
    let mut head = 0;
    while head < queue.len() {
        let v = queue[head];
        head += 1;
        for &t in &succ[v] {
            layer[t] = layer[t].max(layer[v] + 1);
            in_degree[t] -= 1;
            if in_degree[t] == 0 {
                queue.push(t);
            }
        }
    }
    let layers = layer.iter().max().map_or(0, |m| m + 1);

    // 3. Dummy vertices so every segment spans one layer.
    let mut vlayer = layer.clone();
    let mut chains: Vec<Vec<usize>> = Vec::with_capacity(edges.len());
    let mut segs = Vec::new();
    for e in &edges {
        let (s, t) = dir(e);
        let mut chain = vec![s];
        for l in (layer[s] + 1)..layer[t] {
            vlayer.push(l);
            chain.push(vlayer.len() - 1);
        }
        chain.push(t);
        segs.extend(chain.windows(2).map(|w| (w[0], w[1])));
        chains.push(chain);
    }
    let vertices = vlayer.len();
    let mut rows: Vec<Vec<usize>> = vec![Vec::new(); layers];
    for (v, &l) in vlayer.iter().enumerate() {
        rows[l].push(v);
    }

    // 4. Crossing reduction.
    let (rows, crossings) = reduce_crossings(rows, &segs, vertices);

    // 5. Coordinates: columns spread across the width, each column centred vertically.
    let pad_x = node_width / 2.0 + 4.0;
    let widest = rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
    let gap = ((height - 40.0) / widest as f64).min(56.0);
    let mut xy = vec![(0.0, 0.0); vertices];
    for (l, row) in rows.iter().enumerate() {
        let x = if layers > 1 { pad_x + (width - 2.0 * pad_x) * l as f64 / (layers - 1) as f64 } else { width / 2.0 };
        for (k, &v) in row.iter().enumerate() {
            xy[v] = (x, height / 2.0 + (k as f64 - (row.len() - 1) as f64 / 2.0) * gap);
        }
    }

    // 6. Paths: horizontal-tangent cubics between consecutive chain points, starting
    // and ending at the node boxes' sides.
    let half = node_width / 2.0;
    for (e, chain) in edges.iter_mut().zip(&chains) {
        let mut pts: Vec<(f64, f64)> = chain.iter().map(|&v| xy[v]).collect();
        if let Some(first) = pts.first_mut() {
            first.0 += half;
        }
        if let Some(last) = pts.last_mut() {
            last.0 -= half;
        }
        if e.reversed {
            pts.reverse();
        }
        let mut d = format!("M{:.1} {:.1}", pts[0].0, pts[0].1);
        for w in pts.windows(2) {
            let ((x0, y0), (x1, y1)) = (w[0], w[1]);
            let mx = f64::midpoint(x0, x1);
            let _ = write!(d, "C{mx:.1} {y0:.1} {mx:.1} {y1:.1} {x1:.1} {y1:.1}");
        }
        e.path = d;
    }

    let order = rows.iter().flatten().copied().filter(|&v| v < n).collect();
    let nodes = nodes
        .iter()
        .enumerate()
        .map(|(i, nd)| DagNode { id: nd.id.clone(), group: nd.group, layer: layer[i], x: xy[i].0, y: xy[i].1 })
        .collect();
    Dag { nodes, edges, order, layers, crossings }
}

/// Everything upstream of node `i` (what it depends on, transitively) and everything
/// downstream (what depends on it), as one flag per node. `i` itself is in neither.
#[must_use]
pub fn reach(dag: &Dag, i: usize) -> (Vec<bool>, Vec<bool>) {
    let n = dag.nodes.len();
    let walk = |forward: bool| {
        let mut seen = vec![false; n];
        let mut stack = vec![i];
        while let Some(v) = stack.pop() {
            for e in &dag.edges {
                let (from, to) = if forward { (e.source, e.target) } else { (e.target, e.source) };
                if from == v && to != i && !seen[to] {
                    seen[to] = true;
                    stack.push(to);
                }
            }
        }
        seen
    };
    if i >= n {
        return (vec![false; n], vec![false; n]);
    }
    (walk(false), walk(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(pairs: &[(&str, &str)]) -> (Vec<Node>, Vec<Link>) {
        let mut ids: Vec<&str> = pairs.iter().flat_map(|(a, b)| [*a, *b]).collect();
        ids.dedup();
        let mut seen = Vec::new();
        for id in ids {
            if !seen.contains(&id) {
                seen.push(id);
            }
        }
        (seen.iter().map(|s| Node::new(*s, 0)).collect(), pairs.iter().map(|(a, b)| Link::new(*a, *b, 1.0)).collect())
    }

    #[test]
    fn layers_follow_the_longest_path() {
        let (n, l) = graph(&[("a", "b"), ("b", "c"), ("a", "c"), ("c", "d")]);
        let d = layout(&n, &l, 600.0, 300.0, 80.0);
        let layers: Vec<usize> = d.nodes.iter().map(|n| n.layer).collect();
        assert_eq!(layers, vec![0, 1, 2, 3]);
        assert!(d.nodes[0].x < d.nodes[1].x && d.nodes[3].x < 600.0);
        assert_eq!(d.edges[2].path.matches('C').count(), 2, "a→c passes one dummy point");
    }

    #[test]
    fn cycles_are_broken_and_flagged() {
        let (n, l) = graph(&[("a", "b"), ("b", "c"), ("c", "a")]);
        let d = layout(&n, &l, 600.0, 300.0, 80.0);
        assert_eq!(d.edges.iter().filter(|e| e.reversed).count(), 1);
        assert_eq!(d.layers, 3);
    }

    #[test]
    fn sweeps_remove_avoidable_crossings() {
        // Input order crosses a→y with b→x; swapping one column fixes it.
        let (n, l) = graph(&[("a", "y"), ("b", "x"), ("a", "y2"), ("b", "x2")]);
        let initial = count_crossings(&[(0, 1), (2, 3), (0, 4), (2, 5)], &[0, 0, 1, 1, 2, 3]);
        let d = layout(&n, &l, 600.0, 300.0, 80.0);
        assert!(d.crossings <= initial);
        assert_eq!(d.crossings, 0);
    }

    #[test]
    fn reach_walks_both_ways() {
        let (n, l) = graph(&[("a", "b"), ("b", "c"), ("x", "b")]);
        let d = layout(&n, &l, 600.0, 300.0, 80.0);
        let (up, down) = reach(&d, 1);
        assert_eq!(up, vec![true, false, false, true]);
        assert_eq!(down, vec![false, false, true, false]);
    }

    #[test]
    fn empty_and_dangling_inputs_are_safe() {
        assert_eq!(layout(&[], &[], 100.0, 100.0, 40.0), Dag::default());
        let d =
            layout(&[Node::new("a", 0)], &[Link::new("a", "nope", 1.0), Link::new("a", "a", 1.0)], 100.0, 100.0, 40.0);
        assert_eq!(d.edges, vec![]);
        assert_eq!(d.order, vec![0]);
    }
}
