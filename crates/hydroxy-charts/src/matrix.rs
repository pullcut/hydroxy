//! Adjacency matrix: nodes × nodes, each cell the weight of the link between a row and a
//! column node. Rows and columns share one order: by name, by degree, or by clustering
//! so that nodes with similar links sit together.

#![allow(clippy::cast_precision_loss, reason = "node counts are small")]

use crate::network::{Link, Node};
use std::collections::BTreeMap;

/// How rows and columns are ordered.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum MatrixOrder {
    /// Alphabetically by id.
    #[default]
    Name,
    /// Most links first, ties by id.
    Degree,
    /// Average-linkage hierarchical clustering over each node's link profile, so
    /// groups of similar nodes form blocks along the diagonal.
    Cluster,
}

/// One cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    /// Display row.
    pub row: usize,
    /// Display column.
    pub col: usize,
    /// Summed link weight, or `None` when there is no link.
    pub value: Option<f64>,
    /// `value` over the largest value, 0..=1.
    pub level: f64,
}

/// A laid-out matrix.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Matrix {
    /// Node indices (into the input) in display order.
    pub order: Vec<usize>,
    /// Row-major cells, `order.len()` squared.
    pub cells: Vec<Cell>,
    /// Largest cell value.
    pub max: f64,
    /// Links per node, in input order.
    pub degree: Vec<usize>,
}

/// Dense weights by input index. Undirected graphs mirror each link.
fn weights(nodes: &[Node], links: &[Link], directed: bool) -> Vec<Vec<f64>> {
    let n = nodes.len();
    let index: BTreeMap<&str, usize> = nodes.iter().enumerate().map(|(i, node)| (node.id.as_str(), i)).collect();
    let mut grid = vec![vec![0.0; n]; n];
    for link in links {
        let (Some(&s), Some(&t)) = (index.get(link.source.as_str()), index.get(link.target.as_str())) else { continue };
        let v = if link.value.is_finite() && link.value > 0.0 { link.value } else { 1.0 };
        grid[s][t] += v;
        if !directed && s != t {
            grid[t][s] += v;
        }
    }
    grid
}

/// Average-linkage agglomerative clustering; returns the leaf order.
/// O(n³), fine for the few hundred nodes a matrix can show.
fn cluster_order(w: &[Vec<f64>]) -> Vec<usize> {
    let n = w.len();
    let max = w.iter().flatten().copied().fold(0.0, f64::max).max(1.0);
    // Each node's profile: its link strength to every node, counting itself as fully linked.
    let profile = |i: usize, k: usize| if i == k { max } else { w[i][k] + w[k][i] };
    let dist: Vec<Vec<f64>> = (0..n)
        .map(|i| (0..n).map(|j| (0..n).map(|k| (profile(i, k) - profile(j, k)).powi(2)).sum::<f64>().sqrt()).collect())
        .collect();
    let mut clusters: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
    while clusters.len() > 1 {
        let avg = |a: &[usize], b: &[usize]| {
            a.iter().flat_map(|&i| b.iter().map(move |&j| (i, j))).map(|(i, j)| dist[i][j]).sum::<f64>()
                / (a.len() * b.len()) as f64
        };
        let mut best = (0, 1, f64::INFINITY);
        for a in 0..clusters.len() {
            for b in (a + 1)..clusters.len() {
                let d = avg(&clusters[a], &clusters[b]);
                if d < best.2 - 1e-12 {
                    best = (a, b, d);
                }
            }
        }
        let merged = clusters.remove(best.1);
        clusters[best.0].extend(merged);
    }
    clusters.pop().unwrap_or_default()
}

/// Lay out the matrix for `nodes` and `links`. Links to unknown ids are ignored;
/// repeated links add up. When `directed` is false each link fills both mirrored cells.
#[must_use]
pub fn layout(nodes: &[Node], links: &[Link], order: MatrixOrder, directed: bool) -> Matrix {
    let n = nodes.len();
    let w = weights(nodes, links, directed);
    let degree: Vec<usize> =
        (0..n).map(|i| (0..n).filter(|&j| j != i && (w[i][j] > 0.0 || w[j][i] > 0.0)).count()).collect();
    let mut by_name: Vec<usize> = (0..n).collect();
    by_name.sort_by(|&a, &b| nodes[a].id.cmp(&nodes[b].id));
    let order = match order {
        MatrixOrder::Name => by_name,
        MatrixOrder::Degree => {
            let mut o = by_name;
            o.sort_by(|&a, &b| degree[b].cmp(&degree[a]));
            o
        }
        MatrixOrder::Cluster => cluster_order(&w),
    };
    let max = w.iter().flatten().copied().fold(0.0, f64::max);
    let cells = order
        .iter()
        .enumerate()
        .flat_map(|(row, &i)| {
            let w = &w;
            order.iter().enumerate().map(move |(col, &j)| {
                let v = w[i][j];
                Cell { row, col, value: (v > 0.0).then_some(v), level: if max > 0.0 { v / max } else { 0.0 } }
            })
        })
        .collect();
    Matrix { order, cells, max, degree }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph() -> (Vec<Node>, Vec<Link>) {
        let nodes = ["d", "a", "c", "b"].iter().map(|s| Node::new(*s, 0)).collect();
        let links = vec![
            Link::new("a", "b", 2.0),
            Link::new("a", "c", 1.0),
            Link::new("a", "d", 1.0),
            Link::new("c", "d", 3.0),
            Link::new("a", "zz", 1.0),
        ];
        (nodes, links)
    }

    #[test]
    fn orders_by_name_and_degree() {
        let (n, l) = graph();
        let ids = |m: &Matrix| m.order.iter().map(|&i| n[i].id.clone()).collect::<String>();
        assert_eq!(ids(&layout(&n, &l, MatrixOrder::Name, false)), "abcd");
        assert_eq!(ids(&layout(&n, &l, MatrixOrder::Degree, false)), "acdb");
    }

    #[test]
    fn undirected_mirrors_and_directed_does_not() {
        let (n, l) = graph();
        let m = layout(&n, &l, MatrixOrder::Name, false);
        let at = |m: &Matrix, r: usize, c: usize| m.cells[r * 4 + c].value;
        assert_eq!(at(&m, 0, 1), Some(2.0));
        assert_eq!(at(&m, 1, 0), Some(2.0));
        assert_eq!(m.max, 3.0);
        let d = layout(&n, &l, MatrixOrder::Name, true);
        assert_eq!(at(&d, 1, 0), None);
    }

    #[test]
    fn clustering_keeps_cliques_together() {
        let nodes: Vec<Node> = ["a1", "b1", "a2", "b2", "a3", "b3"].iter().map(|s| Node::new(*s, 0)).collect();
        let mut links = Vec::new();
        for g in ["a", "b"] {
            for (x, y) in [(1, 2), (2, 3), (1, 3)] {
                links.push(Link::new(format!("{g}{x}"), format!("{g}{y}"), 1.0));
            }
        }
        let m = layout(&nodes, &links, MatrixOrder::Cluster, false);
        let firsts: String = m.order.iter().map(|&i| nodes[i].id.chars().next().unwrap_or(' ')).collect();
        assert!(firsts == "aaabbb" || firsts == "bbbaaa", "{firsts}");
        assert_eq!(m, layout(&nodes, &links, MatrixOrder::Cluster, false), "deterministic");
    }

    #[test]
    fn empty_is_safe() {
        assert_eq!(layout(&[], &[], MatrixOrder::Cluster, true), Matrix::default());
    }
}
