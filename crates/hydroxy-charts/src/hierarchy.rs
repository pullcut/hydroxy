//! A value tree shared by the treemap and sunburst, plus path helpers for zooming.
//! A path is a list of child indices from the root.

#![allow(clippy::cast_precision_loss, reason = "item counts are small")]

/// One node: a leaf with a value, or a branch whose value is the sum of its children.
#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    /// Name shown in labels, breadcrumbs and the table.
    pub name: String,
    /// Leaf value. Ignored on branches. Non-finite and negative values count as zero.
    pub value: f64,
    /// Children. Empty for a leaf.
    pub children: Vec<Tree>,
}

impl Tree {
    /// A leaf.
    #[must_use]
    pub fn leaf(name: impl Into<String>, value: f64) -> Self {
        Self { name: name.into(), value, children: Vec::new() }
    }

    /// A branch.
    #[must_use]
    pub fn branch(name: impl Into<String>, children: Vec<Tree>) -> Self {
        Self { name: name.into(), value: 0.0, children }
    }

    /// The node's size: a leaf's own value, a branch's children summed.
    #[must_use]
    pub fn total(&self) -> f64 {
        if self.children.is_empty() {
            if self.value.is_finite() { self.value.max(0.0) } else { 0.0 }
        } else {
            self.children.iter().map(Tree::total).sum()
        }
    }

    /// The node at `path`, if every index exists.
    #[must_use]
    pub fn node_at(&self, path: &[usize]) -> Option<&Tree> {
        path.iter().try_fold(self, |node, &i| node.children.get(i))
    }

    /// The longest prefix of `path` that exists and leads through branches with a size.
    #[must_use]
    pub fn valid_path(&self, path: &[usize]) -> Vec<usize> {
        let mut out = Vec::new();
        let mut node = self;
        for &i in path {
            match node.children.get(i) {
                Some(c) if c.total() > 0.0 => {
                    out.push(i);
                    node = c;
                }
                _ => break,
            }
        }
        out
    }

    /// Breadcrumb entries from the root down to `path`: each name with the path that selects it.
    #[must_use]
    pub fn crumbs(&self, path: &[usize]) -> Vec<(String, Vec<usize>)> {
        let path = self.valid_path(path);
        let mut node = self;
        let mut out = vec![(node.name.clone(), Vec::new())];
        for k in 0..path.len() {
            node = &node.children[path[k]];
            out.push((node.name.clone(), path[..=k].to_vec()));
        }
        out
    }

    /// Every leaf as ("Branch > Leaf", value), in tree order, for a text alternative.
    #[must_use]
    pub fn flatten(&self) -> Vec<(String, f64)> {
        fn walk(node: &Tree, prefix: &str, out: &mut Vec<(String, f64)>) {
            let name = if prefix.is_empty() { node.name.clone() } else { format!("{prefix} > {}", node.name) };
            if node.children.is_empty() {
                out.push((name, node.total()));
            } else {
                for c in &node.children {
                    walk(c, &name, out);
                }
            }
        }
        let mut out = Vec::new();
        for c in &self.children {
            walk(c, &self.name, &mut out);
        }
        if self.children.is_empty() {
            out.push((self.name.clone(), self.total()));
        }
        out
    }
}

/// What a key press does on a zoomable hierarchy chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Drill {
    /// Move the active mark (or clear it with `None`).
    Move(Option<usize>),
    /// Zoom into mark `i`.
    Into(usize),
    /// Zoom out one level.
    Up,
}

/// Keys for a treemap or sunburst whose visible marks are flagged in `zoomable`: arrows,
/// Home and End move (see [`crate::nav::list_key`]); Enter or Space zooms into the active
/// mark if it has parts; Backspace zooms out; Escape clears the active mark, or zooms out
/// when none is active. `at_top` is true at the root, where nothing zooms out.
#[must_use]
pub fn drill_key(key: &str, active: Option<usize>, zoomable: &[bool], at_top: bool) -> Option<Drill> {
    match key {
        "Enter" | " " => active.filter(|&i| zoomable.get(i).copied().unwrap_or(false)).map(Drill::Into),
        "Backspace" => (!at_top).then_some(Drill::Up),
        "Escape" if active.is_none() => (!at_top).then_some(Drill::Up),
        _ => crate::nav::list_key(key, active, zoomable.len()).map(Drill::Move),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drill_keys_move_zoom_and_climb() {
        let z = [true, false, true];
        assert_eq!(drill_key("ArrowRight", None, &z, true), Some(Drill::Move(Some(0))));
        assert_eq!(drill_key("Enter", Some(2), &z, true), Some(Drill::Into(2)));
        assert_eq!(drill_key("Enter", Some(1), &z, true), None, "leaves don't zoom");
        assert_eq!(drill_key("Enter", None, &z, true), None);
        assert_eq!(drill_key("Backspace", Some(1), &z, false), Some(Drill::Up));
        assert_eq!(drill_key("Backspace", None, &z, true), None);
        assert_eq!(drill_key("Escape", Some(1), &z, false), Some(Drill::Move(None)), "Escape clears first");
        assert_eq!(drill_key("Escape", None, &z, false), Some(Drill::Up));
        assert_eq!(drill_key("Escape", None, &z, true), None);
    }

    fn sample() -> Tree {
        Tree::branch(
            "All",
            vec![
                Tree::branch("A", vec![Tree::leaf("a1", 3.0), Tree::leaf("a2", 1.0)]),
                Tree::leaf("B", 6.0),
                Tree::leaf("Z", 0.0),
            ],
        )
    }

    #[test]
    fn totals_sum_leaves_and_clean_bad_values() {
        assert!((sample().total() - 10.0).abs() < 1e-9);
        assert_eq!(Tree::leaf("x", f64::NAN).total(), 0.0);
        assert_eq!(Tree::leaf("x", -4.0).total(), 0.0);
    }

    #[test]
    fn paths_resolve_and_clamp() {
        let t = sample();
        assert_eq!(t.node_at(&[0, 1]).map(|n| n.name.as_str()), Some("a2"));
        assert!(t.node_at(&[5]).is_none());
        assert_eq!(t.valid_path(&[0, 9]), vec![0]);
        assert_eq!(t.valid_path(&[2]), Vec::<usize>::new(), "zero-sized nodes can't be zoomed into");
    }

    #[test]
    fn crumbs_start_at_the_root() {
        let c = sample().crumbs(&[0]);
        assert_eq!(c, vec![("All".to_string(), vec![]), ("A".to_string(), vec![0])]);
    }

    #[test]
    fn flatten_names_each_leaf_by_its_path() {
        let rows = sample().flatten();
        assert_eq!(rows[0], ("All > A > a1".to_string(), 3.0));
        assert_eq!(rows[2], ("All > B".to_string(), 6.0));
    }
}
