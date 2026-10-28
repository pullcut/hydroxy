//! Sankey layout: nodes in columns by longest path, link widths proportional to value,
//! a few relaxation sweeps to cut crossings, and path highlighting. Coordinates are px
//! in a `width`×`height` box. Links that would close a cycle are ignored.

#![allow(clippy::cast_precision_loss, reason = "item counts are small")]

use std::collections::HashMap;
use std::fmt::Write as _;

/// A flow from one named node to another.
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    /// Source node name.
    pub source: String,
    /// Target node name.
    pub target: String,
    /// Amount flowing. Zero, negative and non-finite links are ignored.
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
pub struct Node {
    /// Name, in order of first appearance in the links.
    pub name: String,
    /// Column, 0 on the left.
    pub column: usize,
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Height, proportional to `value`.
    pub height: f64,
    /// The larger of the node's inflow and outflow.
    pub value: f64,
}

/// A placed link: stroke `path` with `width`.
#[derive(Debug, Clone, PartialEq)]
pub struct Ribbon {
    /// Index of the source node.
    pub source: usize,
    /// Index of the target node.
    pub target: usize,
    /// The link's value.
    pub value: f64,
    /// Stroke width.
    pub width: f64,
    /// Centre-line y where it leaves the source.
    pub y0: f64,
    /// Centre-line y where it enters the target.
    pub y1: f64,
    /// SVG `d` for a cubic curve along the centre line.
    pub path: String,
}

/// The laid-out diagram.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    /// Nodes.
    pub nodes: Vec<Node>,
    /// Links that survived cycle removal.
    pub links: Vec<Ribbon>,
    /// Width of every node bar.
    pub node_width: f64,
}

/// Which nodes and links are on a path.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Highlight {
    /// One flag per node.
    pub nodes: Vec<bool>,
    /// One flag per link.
    pub links: Vec<bool>,
}

fn clean(links: &[Link]) -> (Vec<String>, Vec<(usize, usize, f64)>) {
    let mut names: Vec<String> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::new();
    for l in links {
        if !(l.value.is_finite() && l.value > 0.0) || l.source == l.target {
            continue;
        }
        let mut ids = [0usize; 2];
        for (slot, name) in ids.iter_mut().zip([&l.source, &l.target]) {
            *slot = *index.entry(name.clone()).or_insert_with(|| {
                names.push(name.clone());
                names.len() - 1
            });
        }
        out.push((ids[0], ids[1], l.value));
    }
    (names, out)
}

/// Drop links that point back at a node still being explored (they close a cycle).
fn break_cycles(n: usize, links: &[(usize, usize, f64)]) -> Vec<(usize, usize, f64)> {
    // 0 unseen, 1 on the stack, 2 done.
    let mut state = vec![0u8; n];
    let mut back = vec![false; links.len()];
    let mut out_of: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (k, l) in links.iter().enumerate() {
        out_of[l.0].push(k);
    }
    for start in 0..n {
        if state[start] != 0 {
            continue;
        }
        let mut stack = vec![(start, 0usize)];
        state[start] = 1;
        while let Some(top) = stack.last_mut() {
            let node = top.0;
            if let Some(&k) = out_of[node].get(top.1) {
                top.1 += 1;
                let t = links[k].1;
                match state[t] {
                    1 => back[k] = true,
                    0 => {
                        state[t] = 1;
                        stack.push((t, 0));
                    }
                    _ => {}
                }
            } else {
                state[node] = 2;
                stack.pop();
            }
        }
    }
    links.iter().enumerate().filter(|(k, _)| !back[*k]).map(|(_, l)| *l).collect()
}

/// Longest-path column per node; nodes with no outflow move to the last column.
fn columns(n: usize, links: &[(usize, usize, f64)]) -> Vec<usize> {
    let mut indeg = vec![0usize; n];
    let mut has_out = vec![false; n];
    for l in links {
        indeg[l.1] += 1;
        has_out[l.0] = true;
    }
    let mut col = vec![0usize; n];
    let mut ready: Vec<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
    while let Some(u) = ready.pop() {
        for l in links.iter().filter(|l| l.0 == u) {
            col[l.1] = col[l.1].max(col[u] + 1);
            indeg[l.1] -= 1;
            if indeg[l.1] == 0 {
                ready.push(l.1);
            }
        }
    }
    let last = col.iter().copied().max().unwrap_or(0);
    for i in 0..n {
        if !has_out[i] && indeg_in(links, i) {
            col[i] = last;
        }
    }
    col
}

fn indeg_in(links: &[(usize, usize, f64)], i: usize) -> bool {
    links.iter().any(|l| l.1 == i)
}

/// Push overlapping nodes apart, keeping them inside `0..height`.
fn resolve(col: &mut [usize], y: &mut [f64], h: &[f64], pad: f64, height: f64) {
    col.sort_by(|&a, &b| y[a].total_cmp(&y[b]).then(a.cmp(&b)));
    let mut top = 0.0;
    for &i in col.iter() {
        y[i] = y[i].max(top);
        top = y[i] + h[i] + pad;
    }
    let mut bottom = height;
    for &i in col.iter().rev() {
        y[i] = y[i].min(bottom - h[i]).max(0.0);
        bottom = y[i] - pad;
    }
}

/// Lay out `links` in a `width`×`height` box. `node_width` is each bar's width, `pad`
/// the gap between bars in a column, and `passes` the number of relaxation sweeps.
#[must_use]
pub fn layout(links: &[Link], width: f64, height: f64, node_width: f64, pad: f64, passes: usize) -> Layout {
    let (names, raw) = clean(links);
    let n = names.len();
    if n == 0 {
        return Layout { node_width, ..Layout::default() };
    }
    let kept = break_cycles(n, &raw);
    let column = columns(n, &kept);
    let ncols = column.iter().copied().max().unwrap_or(0) + 1;
    let mut inflow = vec![0.0; n];
    let mut outflow = vec![0.0; n];
    for l in &kept {
        outflow[l.0] += l.2;
        inflow[l.1] += l.2;
    }
    let value: Vec<f64> = (0..n).map(|i| inflow[i].max(outflow[i])).collect();
    let mut by_col: Vec<Vec<usize>> = vec![Vec::new(); ncols];
    for i in 0..n {
        by_col[column[i]].push(i);
    }
    // One scale for every column, so equal values are equal heights everywhere.
    let ky = by_col
        .iter()
        .map(|c| {
            let room = pad.mul_add(-(c.len().saturating_sub(1) as f64), height).max(0.0);
            room / c.iter().map(|&i| value[i]).sum::<f64>().max(f64::MIN_POSITIVE)
        })
        .fold(f64::INFINITY, f64::min);
    let h: Vec<f64> = value.iter().map(|v| v * ky).collect();
    let mut y = vec![0.0; n];
    for c in &by_col {
        let mut top = 0.0;
        for &i in c {
            y[i] = top;
            top += h[i] + pad;
        }
    }
    let centre_of = |y: &[f64], i: usize| y[i] + h[i] / 2.0;
    for _ in 0..passes {
        for col in by_col.iter_mut().take(ncols.saturating_sub(1)).rev() {
            sweep(col, &mut y, &h, &kept, |l| (l.0, l.1), pad, height, &centre_of);
        }
        for col in by_col.iter_mut().skip(1) {
            sweep(col, &mut y, &h, &kept, |l| (l.1, l.0), pad, height, &centre_of);
        }
    }

    // Where each link attaches along its two nodes, ordered to avoid crossing.
    let mut out_off = vec![0.0; kept.len()];
    let mut in_off = vec![0.0; kept.len()];
    for i in 0..n {
        let mut outs: Vec<usize> = (0..kept.len()).filter(|&k| kept[k].0 == i).collect();
        outs.sort_by(|&a, &b| centre_of(&y, kept[a].1).total_cmp(&centre_of(&y, kept[b].1)).then(a.cmp(&b)));
        let mut off = 0.0;
        for k in outs {
            out_off[k] = off;
            off += kept[k].2 * ky;
        }
        let mut ins: Vec<usize> = (0..kept.len()).filter(|&k| kept[k].1 == i).collect();
        ins.sort_by(|&a, &b| centre_of(&y, kept[a].0).total_cmp(&centre_of(&y, kept[b].0)).then(a.cmp(&b)));
        let mut off = 0.0;
        for k in ins {
            in_off[k] = off;
            off += kept[k].2 * ky;
        }
    }

    let step = if ncols > 1 { (width - node_width) / (ncols - 1) as f64 } else { 0.0 };
    let x_of = |i: usize| column[i] as f64 * step;
    let nodes = (0..n)
        .map(|i| Node { name: names[i].clone(), column: column[i], x: x_of(i), y: y[i], height: h[i], value: value[i] })
        .collect();
    let links = kept
        .iter()
        .enumerate()
        .map(|(k, &(s, t, v))| {
            let w = v * ky;
            let (x0, x1) = (x_of(s) + node_width, x_of(t));
            let (y0, y1) = (y[s] + out_off[k] + w / 2.0, y[t] + in_off[k] + w / 2.0);
            let xm = f64::midpoint(x0, x1);
            let mut path = String::new();
            let _ = write!(path, "M{x0:.2} {y0:.2}C{xm:.2} {y0:.2} {xm:.2} {y1:.2} {x1:.2} {y1:.2}");
            Ribbon { source: s, target: t, value: v, width: w, y0, y1, path }
        })
        .collect();
    Layout { nodes, links, node_width }
}

/// Move each node in `col` towards the value-weighted centre of its neighbours on one
/// side (`ends` maps a link to (this node, neighbour)), then resolve overlaps.
#[allow(clippy::too_many_arguments, reason = "internal helper over shared layout arrays")]
fn sweep(
    col: &mut [usize],
    y: &mut [f64],
    h: &[f64],
    links: &[(usize, usize, f64)],
    ends: impl Fn(&(usize, usize, f64)) -> (usize, usize),
    pad: f64,
    height: f64,
    centre_of: &impl Fn(&[f64], usize) -> f64,
) {
    let want: Vec<(usize, Option<f64>)> = col
        .iter()
        .map(|&i| {
            let (sum, weight) = links
                .iter()
                .filter(|l| ends(l).0 == i)
                .fold((0.0, 0.0), |(s, w), l| (l.2.mul_add(centre_of(y, ends(l).1), s), w + l.2));
            (i, (weight > 0.0).then(|| sum / weight))
        })
        .collect();
    for (i, c) in want {
        if let Some(c) = c {
            y[i] = c - h[i] / 2.0;
        }
    }
    resolve(col, y, h, pad, height);
}

fn closure(layout: &Layout, start: &[usize], forward: bool, nodes: &mut [bool], links: &mut [bool]) {
    let mut stack = start.to_vec();
    while let Some(u) = stack.pop() {
        for (k, l) in layout.links.iter().enumerate() {
            let (from, to) = if forward { (l.source, l.target) } else { (l.target, l.source) };
            if from == u {
                links[k] = true;
                if !nodes[to] {
                    nodes[to] = true;
                    stack.push(to);
                }
            }
        }
    }
}

/// Everything upstream and downstream of node `i`, including the node itself.
#[must_use]
pub fn highlight_node(layout: &Layout, i: usize) -> Highlight {
    let mut h = Highlight { nodes: vec![false; layout.nodes.len()], links: vec![false; layout.links.len()] };
    if i < h.nodes.len() {
        h.nodes[i] = true;
        closure(layout, &[i], true, &mut h.nodes, &mut h.links);
        closure(layout, &[i], false, &mut h.nodes, &mut h.links);
    }
    h
}

/// Link `k` with everything upstream of its source and downstream of its target.
#[must_use]
pub fn highlight_link(layout: &Layout, k: usize) -> Highlight {
    let mut h = Highlight { nodes: vec![false; layout.nodes.len()], links: vec![false; layout.links.len()] };
    if let Some(l) = layout.links.get(k) {
        h.links[k] = true;
        h.nodes[l.source] = true;
        h.nodes[l.target] = true;
        closure(layout, &[l.source], false, &mut h.nodes, &mut h.links);
        closure(layout, &[l.target], true, &mut h.nodes, &mut h.links);
    }
    h
}

/// Inflow and outflow of node `i`.
#[must_use]
pub fn flows(layout: &Layout, i: usize) -> (f64, f64) {
    layout.links.iter().fold((0.0, 0.0), |(inn, out), l| {
        (inn + if l.target == i { l.value } else { 0.0 }, out + if l.source == i { l.value } else { 0.0 })
    })
}

/// Where to pin a tooltip for mark `mark`, in px. Marks count the nodes first, then the
/// links: a node anchors at the middle of its bar, a link at its midpoint.
#[must_use]
pub fn anchor(layout: &Layout, mark: usize) -> Option<(f64, f64)> {
    if let Some(node) = layout.nodes.get(mark) {
        return Some((node.x + layout.node_width / 2.0, node.y + node.height / 2.0));
    }
    let l = layout.links.get(mark.checked_sub(layout.nodes.len())?)?;
    let x0 = layout.nodes[l.source].x + layout.node_width;
    let x1 = layout.nodes[l.target].x;
    Some((f64::midpoint(x0, x1), f64::midpoint(l.y0, l.y1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flows_and_anchors() {
        let lay = layout(&[l("a", "b", 2.0), l("b", "c", 1.5)], 300.0, 100.0, 10.0, 4.0, 1);
        assert_eq!(flows(&lay, 1), (2.0, 1.5));
        let (x, y) = anchor(&lay, 0).expect("node");
        assert!((x - 5.0).abs() < 1e-9 && y > 0.0);
        let (x, _) = anchor(&lay, 3).expect("first link");
        assert!(x > lay.node_width && x < lay.nodes[1].x);
        assert_eq!(anchor(&lay, 5), None);
    }

    fn l(s: &str, t: &str, v: f64) -> Link {
        Link::new(s, t, v)
    }

    fn col(lay: &Layout, name: &str) -> usize {
        lay.nodes.iter().find(|n| n.name == name).expect("node").column
    }

    #[test]
    fn longest_path_sets_columns_and_sinks_go_last() {
        let lay = layout(&[l("a", "b", 2.0), l("b", "c", 2.0), l("a", "c", 1.0), l("a", "d", 1.0)], 400.0, 200.0, 10.0, 8.0, 2);
        assert_eq!((col(&lay, "a"), col(&lay, "b"), col(&lay, "c")), (0, 1, 2));
        assert_eq!(col(&lay, "d"), 2, "a dead end sits with the other sinks");
    }

    #[test]
    fn cycles_and_bad_links_are_dropped() {
        let lay = layout(&[l("a", "b", 1.0), l("b", "a", 1.0), l("a", "a", 1.0), l("a", "b", 0.0), l("b", "c", f64::NAN)], 300.0, 100.0, 10.0, 4.0, 1);
        assert_eq!(lay.links.len(), 1);
        assert_eq!(lay.nodes.len(), 2);
    }

    #[test]
    fn widths_follow_values_and_nodes_stay_in_bounds() {
        let lay = layout(&[l("a", "x", 6.0), l("a", "y", 2.0), l("b", "x", 4.0)], 300.0, 160.0, 12.0, 10.0, 4);
        let w: Vec<f64> = lay.links.iter().map(|k| k.width).collect();
        assert!((w[0] / w[1] - 3.0).abs() < 1e-9);
        for n in &lay.nodes {
            assert!(n.y >= -1e-9 && n.y + n.height <= 160.0 + 1e-9, "{n:?}");
        }
        for c in 0..2 {
            let mut ns: Vec<&Node> = lay.nodes.iter().filter(|n| n.column == c).collect();
            ns.sort_by(|a, b| a.y.total_cmp(&b.y));
            assert!(ns.windows(2).all(|p| p[0].y + p[0].height + 10.0 <= p[1].y + 1e-9));
        }
    }

    fn crossings(lay: &Layout) -> usize {
        let mut n = 0;
        for (i, a) in lay.links.iter().enumerate() {
            for b in &lay.links[i + 1..] {
                if lay.nodes[a.source].column == lay.nodes[b.source].column && (a.y0 - b.y0) * (a.y1 - b.y1) < -1e-9 {
                    n += 1;
                }
            }
        }
        n
    }

    #[test]
    fn relaxation_untangles_crossed_links() {
        let links = [l("s1", "t1", 3.0), l("s2", "t2", 3.0), l("s1", "t2", 0.5), l("s3", "t1", 2.0), l("s4", "t3", 2.0), l("s3", "t3", 1.0)];
        // Targets first appear in an order that makes the naive stacking cross.
        let tangled = [l("s1", "t3", 1.0), l("s2", "t2", 1.0), l("s3", "t1", 1.0), l("s1", "t1", 1.0)];
        assert!(crossings(&layout(&tangled, 300.0, 200.0, 10.0, 8.0, 0)) > 0);
        assert_eq!(crossings(&layout(&tangled, 300.0, 200.0, 10.0, 8.0, 6)), 0);
        assert_eq!(layout(&links, 300.0, 200.0, 10.0, 8.0, 6).links.len(), 6);
    }

    #[test]
    fn highlighting_follows_the_whole_path() {
        let lay = layout(&[l("a", "b", 1.0), l("b", "c", 1.0), l("a", "d", 1.0), l("e", "c", 1.0)], 300.0, 100.0, 10.0, 4.0, 1);
        let idx = |n: &str| lay.nodes.iter().position(|x| x.name == n).expect("node");
        let h = highlight_node(&lay, idx("b"));
        assert!(h.nodes[idx("a")] && h.nodes[idx("b")] && h.nodes[idx("c")]);
        assert!(!h.nodes[idx("d")] && !h.nodes[idx("e")]);
        assert_eq!(h.links.iter().filter(|x| **x).count(), 2);
        let h = highlight_link(&lay, 1);
        assert!(h.nodes[idx("a")] && h.nodes[idx("c")] && !h.nodes[idx("e")]);
    }

    #[test]
    fn empty_input_is_an_empty_layout() {
        assert_eq!(layout(&[], 100.0, 100.0, 10.0, 4.0, 2).nodes, vec![]);
    }
}
