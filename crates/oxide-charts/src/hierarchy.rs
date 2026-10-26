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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Tree {
        Tree::branch(
            "All",
            vec![Tree::branch("A", vec![Tree::leaf("a1", 3.0), Tree::leaf("a2", 1.0)]), Tree::leaf("B", 6.0), Tree::leaf("Z", 0.0)],
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
