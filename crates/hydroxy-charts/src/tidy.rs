//! Tidy tree layout (Buchheim, Jünger and Leipert's linear-time Reingold–Tilford):
//! parents centred over their children, subtrees as close as they can be without
//! overlapping, mirror images drawn as mirror images. Collapsed nodes hide their
//! children. Positions are 0..=1 fractions: `breadth` across, `level` down from the root.

#![allow(clippy::cast_precision_loss, reason = "item counts are small")]

use crate::Orientation;
use crate::hierarchy::Tree;
use crate::partition::slot;

/// One visible node.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode {
    /// Path from the tree root.
    pub path: Vec<usize>,
    /// 0 for the root.
    pub depth: usize,
    /// Node name.
    pub name: String,
    /// Node size, as [`Tree::total`].
    pub value: f64,
    /// Position across the tree, 0..=1.
    pub breadth: f64,
    /// Position from root (0) to the deepest visible level (1).
    pub level: f64,
    /// Index of the parent in the node list.
    pub parent: Option<usize>,
    /// It has children, shown or not.
    pub has_children: bool,
    /// Its children are hidden.
    pub collapsed: bool,
    /// Colour slot of its top-level branch; `None` for the root.
    pub colour: Option<usize>,
}

struct W {
    parent: Option<usize>,
    kids: Vec<usize>,
    number: usize,
    prelim: f64,
    modifier: f64,
    shift: f64,
    change: f64,
    thread: Option<usize>,
    ancestor: usize,
    x: f64,
}

struct Walk {
    n: Vec<W>,
}

impl Walk {
    fn left_sibling(&self, v: usize) -> Option<usize> {
        let p = self.n[v].parent?;
        let k = self.n[v].number;
        (k > 0).then(|| self.n[p].kids[k - 1])
    }

    fn leftmost_sibling(&self, v: usize) -> usize {
        self.n[v].parent.map_or(v, |p| self.n[p].kids[0])
    }

    fn next_left(&self, v: usize) -> Option<usize> {
        self.n[v].kids.first().copied().or(self.n[v].thread)
    }

    fn next_right(&self, v: usize) -> Option<usize> {
        self.n[v].kids.last().copied().or(self.n[v].thread)
    }

    fn first_walk(&mut self, v: usize) {
        let kids = self.n[v].kids.clone();
        let left = self.left_sibling(v);
        if kids.is_empty() {
            self.n[v].prelim = left.map_or(0.0, |w| self.n[w].prelim + 1.0);
            return;
        }
        let mut default_ancestor = kids[0];
        for &w in &kids {
            self.first_walk(w);
            default_ancestor = self.apportion(w, default_ancestor);
        }
        self.execute_shifts(v);
        let mid = f64::midpoint(self.n[kids[0]].prelim, self.n[kids[kids.len() - 1]].prelim);
        if let Some(w) = left {
            self.n[v].prelim = self.n[w].prelim + 1.0;
            self.n[v].modifier = self.n[v].prelim - mid;
        } else {
            self.n[v].prelim = mid;
        }
    }

    fn apportion(&mut self, v: usize, mut default_ancestor: usize) -> usize {
        let Some(w) = self.left_sibling(v) else { return default_ancestor };
        let (mut vir, mut vor, mut vil, mut vol) = (v, v, w, self.leftmost_sibling(v));
        let (mut sir, mut sor) = (self.n[v].modifier, self.n[v].modifier);
        let (mut sil, mut sol) = (self.n[vil].modifier, self.n[vol].modifier);
        while let (Some(nil), Some(nir)) = (self.next_right(vil), self.next_left(vir)) {
            vil = nil;
            vir = nir;
            vol = self.next_left(vol).unwrap_or(vol);
            vor = self.next_right(vor).unwrap_or(vor);
            self.n[vor].ancestor = v;
            let shift = (self.n[vil].prelim + sil) - (self.n[vir].prelim + sir) + 1.0;
            if shift > 0.0 {
                let a = self.ancestor_of(vil, v, default_ancestor);
                self.move_subtree(a, v, shift);
                sir += shift;
                sor += shift;
            }
            sil += self.n[vil].modifier;
            sir += self.n[vir].modifier;
            sol += self.n[vol].modifier;
            sor += self.n[vor].modifier;
        }
        if let Some(t) = self.next_right(vil).filter(|_| self.next_right(vor).is_none()) {
            self.n[vor].thread = Some(t);
            self.n[vor].modifier += sil - sor;
        }
        if let Some(t) = self.next_left(vir).filter(|_| self.next_left(vol).is_none()) {
            self.n[vol].thread = Some(t);
            self.n[vol].modifier += sir - sol;
            default_ancestor = v;
        }
        default_ancestor
    }

    fn ancestor_of(&self, vil: usize, v: usize, default_ancestor: usize) -> usize {
        let a = self.n[vil].ancestor;
        if self.n[a].parent == self.n[v].parent { a } else { default_ancestor }
    }

    fn move_subtree(&mut self, wl: usize, wr: usize, shift: f64) {
        let subtrees = (self.n[wr].number - self.n[wl].number).max(1) as f64;
        self.n[wr].change -= shift / subtrees;
        self.n[wr].shift += shift;
        self.n[wl].change += shift / subtrees;
        self.n[wr].prelim += shift;
        self.n[wr].modifier += shift;
    }

    fn execute_shifts(&mut self, v: usize) {
        let (mut shift, mut change) = (0.0, 0.0);
        for &w in self.n[v].kids.clone().iter().rev() {
            self.n[w].prelim += shift;
            self.n[w].modifier += shift;
            change += self.n[w].change;
            shift += self.n[w].shift + change;
        }
    }

    fn second_walk(&mut self, v: usize, m: f64) {
        self.n[v].x = self.n[v].prelim + m;
        let m = m + self.n[v].modifier;
        for w in self.n[v].kids.clone() {
            self.second_walk(w, m);
        }
    }
}

fn collect(
    node: &Tree,
    path: &[usize],
    depth: usize,
    parent: Option<usize>,
    colour: Option<usize>,
    collapsed: &[Vec<usize>],
    out: &mut Vec<TreeNode>,
) {
    let hidden = collapsed.iter().any(|c| c == path);
    let me = out.len();
    out.push(TreeNode {
        path: path.to_vec(),
        depth,
        name: node.name.clone(),
        value: node.total(),
        breadth: 0.0,
        level: 0.0,
        parent,
        has_children: !node.children.is_empty(),
        collapsed: hidden && !node.children.is_empty(),
        colour,
    });
    if hidden {
        return;
    }
    for (i, child) in node.children.iter().enumerate() {
        let mut p = path.to_vec();
        p.push(i);
        collect(child, &p, depth + 1, Some(me), if depth == 0 { Some(slot(i)) } else { colour }, collapsed, out);
    }
}

/// Visible nodes in pre-order, root first, with nodes at `collapsed` paths folded.
#[must_use]
pub fn layout(root: &Tree, collapsed: &[Vec<usize>]) -> Vec<TreeNode> {
    let mut nodes = Vec::new();
    collect(root, &[], 0, None, None, collapsed, &mut nodes);
    let mut walk = Walk {
        n: nodes
            .iter()
            .enumerate()
            .map(|(i, _)| W {
                parent: None,
                kids: Vec::new(),
                number: 0,
                prelim: 0.0,
                modifier: 0.0,
                shift: 0.0,
                change: 0.0,
                thread: None,
                ancestor: i,
                x: 0.0,
            })
            .collect(),
    };
    for (i, n) in nodes.iter().enumerate() {
        if let Some(p) = n.parent {
            walk.n[i].parent = Some(p);
            walk.n[i].number = walk.n[p].kids.len();
            walk.n[p].kids.push(i);
        }
    }
    walk.first_walk(0);
    walk.second_walk(0, 0.0);
    let (lo, hi) = walk.n.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), w| (a.min(w.x), b.max(w.x)));
    let deepest = nodes.iter().map(|n| n.depth).max().unwrap_or(0);
    for (n, w) in nodes.iter_mut().zip(&walk.n) {
        n.breadth = if hi > lo { (w.x - lo) / (hi - lo) } else { 0.5 };
        n.level = if deepest > 0 { n.depth as f64 / deepest as f64 } else { 0.0 };
    }
    nodes
}

impl TreeNode {
    /// Position in a `width`×`height` box. Vertical trees grow down from the top with
    /// room for labels under the leaves; horizontal trees grow right with room for
    /// labels after them.
    #[must_use]
    pub fn point(&self, orientation: Orientation, width: f64, height: f64) -> (f64, f64) {
        match orientation {
            Orientation::Vertical => (40.0 + self.breadth * (width - 80.0), 20.0 + self.level * (height - 50.0)),
            Orientation::Horizontal => (30.0 + self.level * (width - 170.0), 16.0 + self.breadth * (height - 32.0)),
        }
    }
}

/// Gap below a vertical tree's parent node that its label occupies.
pub const LABEL_CLEARANCE: f64 = 28.0;

/// SVG `d` for a curved link from a parent at `a` to a child at `b`.
#[must_use]
pub fn link(a: (f64, f64), b: (f64, f64), orientation: Orientation) -> String {
    match orientation {
        Orientation::Vertical => {
            // Leave the parent below its label, so the link never runs through the text.
            let a = (a.0, a.1 + LABEL_CLEARANCE);
            let m = f64::midpoint(a.1, b.1);
            format!("M{:.2} {:.2}C{:.2} {m:.2} {:.2} {m:.2} {:.2} {:.2}", a.0, a.1, a.0, b.0, b.0, b.1)
        }
        Orientation::Horizontal => {
            let m = f64::midpoint(a.0, b.0);
            format!("M{:.2} {:.2}C{m:.2} {:.2} {m:.2} {:.2} {:.2} {:.2}", a.0, a.1, a.1, b.1, b.0, b.1)
        }
    }
}

/// What a key does in tree navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeKey {
    /// Move focus to a node, or clear it.
    Focus(Option<usize>),
    /// Expand or collapse a node.
    Toggle(usize),
}

/// WAI-ARIA tree keys over the visible pre-order nodes. Up/Down step, Right expands
/// or enters the first child, Left collapses or climbs to the parent, Home/End jump,
/// Enter and Space toggle, Escape clears. `None` when the key isn't handled.
#[must_use]
pub fn key(key: &str, current: Option<usize>, nodes: &[TreeNode]) -> Option<TreeKey> {
    let last = nodes.len().checked_sub(1)?;
    let Some(i) = current.filter(|&i| i <= last) else {
        return match key {
            "ArrowDown" | "ArrowUp" | "ArrowRight" | "ArrowLeft" | "Home" => Some(TreeKey::Focus(Some(0))),
            "End" => Some(TreeKey::Focus(Some(last))),
            _ => None,
        };
    };
    let n = &nodes[i];
    match key {
        "ArrowDown" => Some(TreeKey::Focus(Some((i + 1).min(last)))),
        "ArrowUp" => Some(TreeKey::Focus(Some(i.saturating_sub(1)))),
        "Home" => Some(TreeKey::Focus(Some(0))),
        "End" => Some(TreeKey::Focus(Some(last))),
        "Escape" => Some(TreeKey::Focus(None)),
        "ArrowRight" if n.collapsed => Some(TreeKey::Toggle(i)),
        "ArrowRight" if n.has_children => Some(TreeKey::Focus(Some(i + 1))),
        "ArrowLeft" if n.has_children && !n.collapsed => Some(TreeKey::Toggle(i)),
        "ArrowLeft" => n.parent.map(|p| TreeKey::Focus(Some(p))),
        "Enter" | " " if n.has_children => Some(TreeKey::Toggle(i)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Tree {
        Tree::branch(
            "CEO",
            vec![
                Tree::branch("CTO", vec![Tree::leaf("Dev", 1.0), Tree::leaf("Ops", 1.0), Tree::leaf("QA", 1.0)]),
                Tree::leaf("CFO", 1.0),
                Tree::branch("COO", vec![Tree::leaf("Sales", 1.0)]),
            ],
        )
    }

    #[test]
    fn parents_centre_over_children_and_siblings_do_not_overlap() {
        let n = layout(&sample(), &[]);
        assert_eq!(n.len(), 8);
        let at = |name: &str| n.iter().find(|x| x.name == name).expect(name).breadth;
        assert!((at("CTO") - f64::midpoint(at("Dev"), at("QA"))).abs() < 1e-9);
        assert!((at("CEO") - f64::midpoint(at("CTO"), at("COO"))).abs() < 1e-9);
        assert!(at("Dev") < at("Ops") && at("Ops") < at("QA") && at("QA") < at("Sales"));
        let unit = at("Ops") - at("Dev");
        assert!(at("Sales") - at("QA") >= unit - 1e-9);
        assert!((n[0].level).abs() < 1e-9 && (n[2].level - 1.0).abs() < 1e-9);
        assert_eq!(n[2].colour, Some(0));
        assert_eq!(n[0].colour, None);
        let (x, y) = n[0].point(Orientation::Horizontal, 800.0, 300.0);
        assert!((x - 30.0).abs() < 1e-9 && y > 16.0 && y < 284.0);
        assert!(
            link((0.0, 0.0), (10.0, 100.0), Orientation::Vertical).starts_with("M0.00 28.00C0.00 64.00 10.00 64.00"),
            "starts below the label"
        );
    }

    #[test]
    fn collapsed_nodes_hide_their_children() {
        let n = layout(&sample(), &[vec![0]]);
        assert_eq!(n.len(), 5);
        assert!(n[1].collapsed && n[1].has_children);
    }

    #[test]
    fn keys_follow_the_aria_tree_pattern() {
        let n = layout(&sample(), &[vec![2]]);
        assert_eq!(key("ArrowDown", None, &n), Some(TreeKey::Focus(Some(0))));
        assert_eq!(key("End", None, &n), Some(TreeKey::Focus(Some(n.len() - 1))));
        assert_eq!(key("ArrowRight", Some(1), &n), Some(TreeKey::Focus(Some(2))));
        assert_eq!(key("ArrowLeft", Some(2), &n), Some(TreeKey::Focus(Some(1))));
        assert_eq!(key("ArrowLeft", Some(1), &n), Some(TreeKey::Toggle(1)));
        let coo = n.iter().position(|x| x.name == "COO").expect("COO");
        assert_eq!(key("ArrowRight", Some(coo), &n), Some(TreeKey::Toggle(coo)));
        assert_eq!(key("Enter", Some(2), &n), None);
        assert_eq!(key("Escape", Some(2), &n), Some(TreeKey::Focus(None)));
        assert_eq!(key("Escape", None, &n), None);
        assert_eq!(key("ArrowLeft", Some(0), &n), Some(TreeKey::Toggle(0)));
    }
}
