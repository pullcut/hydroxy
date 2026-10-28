//! Alluvial layout: members' categories across ordered steps. Each step is a column of
//! stacked category nodes, always in the same category order so a category keeps its
//! place; ribbons between neighbouring columns carry the members who moved from one
//! category to another. Ribbon bands use the sankey's curve (a cubic with both control
//! points at the horizontal midpoint), drawn as a filled band so wide flows keep their
//! true thickness through the bend. Coordinates are px in a `width`×`height` box.

#![allow(clippy::cast_precision_loss, clippy::many_single_char_names, reason = "item counts are small; the geometry reads best in maths notation")]

use crate::plot::{Tip, TipRow};
use std::fmt::Write as _;

/// A group of members that share a path through the steps.
#[derive(Debug, Clone, PartialEq)]
pub struct Cohort {
    /// The category at each step, first step first.
    pub path: Vec<String>,
    /// How many members. Zero, negative and non-finite counts are ignored.
    pub count: f64,
}

impl Cohort {
    /// `count` members who took `path`.
    #[must_use]
    pub fn new<S: Into<String>>(path: impl IntoIterator<Item = S>, count: f64) -> Self {
        Self { path: path.into_iter().map(Into::into).collect(), count }
    }
}

/// A category at one step.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Step (column), 0 on the left.
    pub step: usize,
    /// Index into [`Layout::categories`].
    pub category: usize,
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Height, proportional to `value`.
    pub height: f64,
    /// Members in this category at this step.
    pub value: f64,
}

/// Members moving from one node to a node in the next column.
#[derive(Debug, Clone, PartialEq)]
pub struct Ribbon {
    /// Index of the source node.
    pub source: usize,
    /// Index of the target node.
    pub target: usize,
    /// How many members.
    pub value: f64,
    /// Band thickness.
    pub width: f64,
    /// Top of the band where it leaves the source.
    pub y0: f64,
    /// Top of the band where it enters the target.
    pub y1: f64,
    /// SVG `d` for the filled band.
    pub path: String,
}

/// The laid-out diagram.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    /// Category names, in order of first appearance; nodes stack in this order.
    pub categories: Vec<String>,
    /// Number of steps.
    pub steps: usize,
    /// Nodes, column by column, top to bottom.
    pub nodes: Vec<Node>,
    /// Ribbons, column by column.
    pub ribbons: Vec<Ribbon>,
    /// Width of every node bar.
    pub node_width: f64,
    /// Left edge of each column.
    pub columns: Vec<f64>,
}

impl Layout {
    /// Where to pin a tooltip for mark `mark`, in px. Marks count the nodes first, then
    /// the ribbons: a node anchors at the middle of its bar, a ribbon at its midpoint.
    #[must_use]
    pub fn anchor(&self, mark: usize) -> Option<(f64, f64)> {
        if let Some(n) = self.nodes.get(mark) {
            return Some((n.x + self.node_width / 2.0, n.y + n.height / 2.0));
        }
        let r = self.ribbons.get(mark - self.nodes.len())?;
        let (s, t) = (&self.nodes[r.source], &self.nodes[r.target]);
        Some((f64::midpoint(s.x + self.node_width, t.x), f64::midpoint(r.y0, r.y1) + r.width / 2.0))
    }

    /// CSS colour for a category.
    #[must_use]
    pub fn colour(&self, category: usize) -> String {
        crate::chord::colour(category, self.categories.len())
    }

    /// Members at each step (every step holds everyone).
    #[must_use]
    pub fn total(&self) -> f64 {
        self.nodes.iter().filter(|n| n.step == 0).map(|n| n.value).sum()
    }

    /// Tooltip for mark `mark` (nodes, then ribbons) in a `width`×`height` box. `steps`
    /// names the columns.
    #[must_use]
    pub fn tip(&self, mark: usize, steps: &[String], width: f64, height: f64, fmt: &dyn Fn(f64) -> String) -> Option<Tip> {
        let (x, y) = self.anchor(mark)?;
        let step = |t: usize| steps.get(t).cloned().unwrap_or_else(|| format!("Step {}", t + 1));
        let total = self.total();
        let (title, rows) = if let Some(n) = self.nodes.get(mark) {
            let rows = vec![
                TipRow::new(None, "Step", step(n.step)),
                TipRow::new(None, "Members", fmt(n.value)),
                TipRow::new(None, "Share of step", crate::upset::percent(n.value, total)),
            ];
            (self.categories[n.category].clone(), rows)
        } else {
            let r = &self.ribbons[mark - self.nodes.len()];
            let (s, t) = (&self.nodes[r.source], &self.nodes[r.target]);
            let title = if s.category == t.category {
                format!("Stayed {}", self.categories[s.category])
            } else {
                format!("{} → {}", self.categories[s.category], self.categories[t.category])
            };
            let rows = vec![
                TipRow::new(None, "Steps", format!("{} → {}", step(s.step), step(t.step))),
                TipRow::new(None, "Members", fmt(r.value)),
                TipRow::new(None, format!("Share of {}", self.categories[s.category]), crate::upset::percent(r.value, s.value)),
            ];
            (title, rows)
        };
        Some(Tip { x: x / width.max(1.0), y: 1.0 - y / height.max(1.0), title, rows })
    }

    /// Which nodes and ribbons mark `mark` (nodes, then ribbons) lights: a node lights
    /// itself and every ribbon in or out of it, a ribbon lights itself and its two nodes.
    #[must_use]
    pub fn lit(&self, mark: usize) -> (Vec<bool>, Vec<bool>) {
        let mut nodes = vec![false; self.nodes.len()];
        let mut ribbons = vec![false; self.ribbons.len()];
        if mark < nodes.len() {
            nodes[mark] = true;
            for (k, r) in self.ribbons.iter().enumerate() {
                ribbons[k] = r.source == mark || r.target == mark;
            }
        } else if let Some(r) = self.ribbons.get(mark - nodes.len()) {
            ribbons[mark - nodes.len()] = true;
            nodes[r.source] = true;
            nodes[r.target] = true;
        }
        (nodes, ribbons)
    }

    /// The node for `category` at `step`, if any members are there.
    #[must_use]
    pub fn node(&self, step: usize, category: usize) -> Option<usize> {
        self.nodes.iter().position(|n| n.step == step && n.category == category)
    }
}

/// Lay out `cohorts` in a `width`×`height` box (`top` px at the top are left for step
/// names). The step count is the longest path; shorter paths are ignored. `pad` is the
/// gap between nodes in a column.
#[must_use]
pub fn layout(cohorts: &[Cohort], width: f64, height: f64, top: f64, node_width: f64, pad: f64) -> Layout {
    let steps = cohorts.iter().filter(|c| c.count.is_finite() && c.count > 0.0).map(|c| c.path.len()).max().unwrap_or(0);
    let mut categories: Vec<String> = Vec::new();
    let mut paths: Vec<(Vec<usize>, f64)> = Vec::new();
    for c in cohorts {
        if !(c.count.is_finite() && c.count > 0.0) || c.path.len() != steps {
            continue;
        }
        let ids = c
            .path
            .iter()
            .map(|s| {
                categories.iter().position(|n| n == s).unwrap_or_else(|| {
                    categories.push(s.clone());
                    categories.len() - 1
                })
            })
            .collect();
        paths.push((ids, c.count));
    }
    let k = categories.len();
    if steps == 0 || paths.is_empty() {
        return Layout { categories, node_width, ..Layout::default() };
    }
    // value[step][category]
    let mut value = vec![vec![0.0; k]; steps];
    for (p, n) in &paths {
        for (t, &c) in p.iter().enumerate() {
            value[t][c] += n;
        }
    }
    let room = (height - top).max(0.0);
    let ky = value
        .iter()
        .map(|col| {
            let present = col.iter().filter(|v| **v > 0.0).count();
            pad.mul_add(-(present.saturating_sub(1) as f64), room).max(0.0) / col.iter().sum::<f64>().max(f64::MIN_POSITIVE)
        })
        .fold(f64::INFINITY, f64::min);
    let step_x = if steps > 1 { (width - node_width) / (steps - 1) as f64 } else { 0.0 };
    let columns: Vec<f64> = (0..steps).map(|t| t as f64 * step_x).collect();
    let mut nodes = Vec::new();
    for (t, col) in value.iter().enumerate() {
        let present = col.iter().filter(|v| **v > 0.0).count();
        let used = col.iter().sum::<f64>() * ky + pad * present.saturating_sub(1) as f64;
        let mut y = top + (room - used) / 2.0;
        for (c, &v) in col.iter().enumerate() {
            if v > 0.0 {
                nodes.push(Node { step: t, category: c, x: columns[t], y, height: v * ky, value: v });
                y += v * ky + pad;
            }
        }
    }
    let node_at = |t: usize, c: usize| nodes.iter().position(|n| n.step == t && n.category == c);

    let mut ribbons = Vec::new();
    for t in 0..steps.saturating_sub(1) {
        let mut flow = vec![vec![0.0; k]; k];
        for (p, n) in &paths {
            flow[p[t]][p[t + 1]] += n;
        }
        // Attach in category order at both ends, which is also the nodes' stacking order,
        // so no two ribbons cross where they leave or enter a node.
        let mut out_off = vec![0.0; k];
        let mut in_off = vec![0.0; k];
        let mut pending = Vec::new();
        for (a, row) in flow.iter().enumerate() {
            for (b, &v) in row.iter().enumerate() {
                if v > 0.0 {
                    let w = v * ky;
                    pending.push((a, b, v, w, out_off[a]));
                    out_off[a] += w;
                }
            }
        }
        let mut by_target = pending.clone();
        by_target.sort_by_key(|&(a, b, ..)| (b, a));
        let mut enter = vec![vec![0.0; k]; k];
        for (a, b, _, w, _) in by_target {
            enter[a][b] = in_off[b];
            in_off[b] += w;
        }
        for (a, b, v, w, off) in pending {
            let (Some(s), Some(e)) = (node_at(t, a), node_at(t + 1, b)) else { continue };
            let (x0, x1) = (nodes[s].x + node_width, nodes[e].x);
            let (y0, y1) = (nodes[s].y + off, nodes[e].y + enter[a][b]);
            let xm = f64::midpoint(x0, x1);
            let mut path = String::new();
            let _ = write!(
                path,
                "M{x0:.2} {y0:.2}C{xm:.2} {y0:.2} {xm:.2} {y1:.2} {x1:.2} {y1:.2}L{x1:.2} {:.2}C{xm:.2} {:.2} {xm:.2} {:.2} {x0:.2} {:.2}Z",
                y1 + w,
                y1 + w,
                y0 + w,
                y0 + w
            );
            ribbons.push(Ribbon { source: s, target: e, value: v, width: w, y0, y1, path });
        }
    }
    Layout { categories, steps, nodes, ribbons, node_width, columns }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Cohort> {
        vec![
            Cohort::new(["Free", "Free", "Pro"], 50.0),
            Cohort::new(["Free", "Pro", "Pro"], 30.0),
            Cohort::new(["Pro", "Pro", "Team"], 20.0),
            Cohort::new(["Pro", "Churned", "Churned"], 10.0),
            Cohort::new(["Pro", "Pro"], 99.0),
            Cohort::new(["Free", "Free", "Free"], 0.0),
        ]
    }

    #[test]
    fn nodes_stack_in_category_order_and_scale_by_value() {
        let l = layout(&sample(), 400.0, 220.0, 20.0, 10.0, 8.0);
        assert_eq!(l.steps, 3);
        assert_eq!(l.categories, ["Free", "Pro", "Team", "Churned"]);
        let col1: Vec<(usize, f64)> = l.nodes.iter().filter(|n| n.step == 1).map(|n| (n.category, n.value)).collect();
        assert_eq!(col1, [(0, 50.0), (1, 50.0), (3, 10.0)]);
        for t in 0..3 {
            let ns: Vec<&Node> = l.nodes.iter().filter(|n| n.step == t).collect();
            assert!(ns.windows(2).all(|p| p[0].category < p[1].category && p[0].y + p[0].height + 8.0 <= p[1].y + 1e-9));
            assert!(ns.iter().all(|n| n.y >= 20.0 - 1e-9 && n.y + n.height <= 220.0 + 1e-9));
        }
        assert_eq!(l.node(2, 2).map(|i| l.nodes[i].value), Some(20.0));
    }

    #[test]
    fn ribbons_fill_their_nodes_without_crossing_at_the_ends() {
        let l = layout(&sample(), 400.0, 220.0, 20.0, 10.0, 8.0);
        for (i, n) in l.nodes.iter().enumerate() {
            let out: f64 = l.ribbons.iter().filter(|r| r.source == i).map(|r| r.width).sum();
            let inn: f64 = l.ribbons.iter().filter(|r| r.target == i).map(|r| r.width).sum();
            if n.step < 2 {
                assert!((out - n.height).abs() < 1e-9);
            }
            if n.step > 0 {
                assert!((inn - n.height).abs() < 1e-9);
            }
        }
        let from_free: Vec<&Ribbon> = l.ribbons.iter().filter(|r| r.source == 0).collect();
        assert_eq!(from_free.len(), 2);
        assert!(from_free[0].y0 < from_free[1].y0 && from_free[0].y1 < from_free[1].y1);
        assert!(l.ribbons.iter().all(|r| r.path.starts_with('M') && r.path.ends_with('Z')));
    }

    #[test]
    fn anchors_and_empty_input() {
        let l = layout(&sample(), 400.0, 220.0, 20.0, 10.0, 8.0);
        let (x, _) = l.anchor(0).expect("node");
        assert!((x - 5.0).abs() < 1e-9);
        let (x, _) = l.anchor(l.nodes.len()).expect("ribbon");
        assert!(x > 10.0 && x < l.columns[1]);
        assert_eq!(l.anchor(l.nodes.len() + l.ribbons.len()), None);
        let steps = ["Jan".to_string(), "Feb".to_string(), "Mar".to_string()];
        let fmt = |v: f64| format!("{v}");
        let tip = l.tip(l.nodes.len(), &steps, 400.0, 220.0, &fmt).expect("ribbon tip");
        assert_eq!((tip.title.as_str(), tip.rows[0].value.as_str(), tip.rows[2].value.as_str()), ("Stayed Free", "Jan → Feb", "62%"));
        assert_eq!(l.tip(0, &steps, 400.0, 220.0, &fmt).expect("node").rows[2].value, "73%");
        let (nodes, ribbons) = l.lit(0);
        assert!(nodes[0] && !nodes[1] && ribbons.iter().filter(|x| **x).count() == 2);
        assert_eq!(layout(&[], 100.0, 100.0, 0.0, 10.0, 4.0).nodes.len(), 0);
    }
}
