//! Rectangular partition of a tree by depth, for icicle charts and flame graphs.
//! Each row is one level; each cell's width is its share of the zoomed node.
//! Widths are 0..=1 fractions; rows are counted from the zoomed node (depth 0).

#![allow(clippy::cast_precision_loss, reason = "item counts are small")]

use crate::hierarchy::Tree;
use crate::scale::non_negative;

/// Colour slot (0..6) for the `i`th branch. Six hues at most: with more than six
/// branches, the sixth slot is shared by every branch past the fifth ("Other").
#[must_use]
pub const fn slot(i: usize) -> usize {
    if i < 5 { i } else { 5 }
}

/// Legend names for a node's branches, matching [`slot`]: every name when there are
/// six or fewer, else the first five and "Other".
#[must_use]
pub fn legend(node: &Tree) -> Vec<String> {
    let names = node.children.iter().map(|c| c.name.clone());
    if node.children.len() > 6 {
        names.take(5).chain(std::iter::once("Other".to_string())).collect()
    } else {
        names.collect()
    }
}

/// A stable colour slot (0..6) from a name, so a function keeps its colour everywhere it appears.
#[must_use]
pub fn name_slot(name: &str) -> usize {
    let hash = name.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3));
    usize::try_from(hash % 6).unwrap_or(0)
}

/// Whether `name` contains `query`, ignoring case. An empty query matches nothing.
#[must_use]
pub fn matches(name: &str, query: &str) -> bool {
    let q = query.trim();
    !q.is_empty() && name.to_lowercase().contains(&q.to_lowercase())
}

/// A node's size. For an icicle a branch is its children summed, as in [`Tree::total`].
/// For a flame graph a branch's own `value` is its total time when that is larger
/// than its children's, and the difference is self time.
#[must_use]
pub fn size(node: &Tree, flame: bool) -> f64 {
    if node.children.is_empty() {
        return non_negative(node.value);
    }
    let kids: f64 = node.children.iter().map(|c| size(c, flame)).sum();
    if flame { non_negative(node.value).max(kids) } else { kids }
}

/// Every node in pre-order as ("Root > Branch > Node", size, self size), for a text
/// alternative. `flame` sizes branches as [`size`] describes.
#[must_use]
pub fn rows(root: &Tree, flame: bool) -> Vec<(String, f64, f64)> {
    fn walk(node: &Tree, prefix: &str, flame: bool, out: &mut Vec<(String, f64, f64)>) {
        let name = if prefix.is_empty() { node.name.clone() } else { format!("{prefix} > {}", node.name) };
        let total = size(node, flame);
        let kids: f64 = node.children.iter().map(|c| size(c, flame)).sum();
        out.push((name.clone(), total, (total - kids).max(0.0)));
        for c in &node.children {
            walk(c, &name, flame, out);
        }
    }
    let mut out = Vec::new();
    walk(root, "", flame, &mut out);
    out
}

/// `name` cut to fit `width` px at roughly `char_px` per character, with an ellipsis;
/// `None` when not even three characters fit.
#[must_use]
pub fn fit_label(name: &str, width: f64, char_px: f64) -> Option<String> {
    let room = ((width - 8.0) / char_px).floor();
    if room < 3.0 {
        return None;
    }
    let n = name.chars().count();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "room is small and positive")]
    let room = room as usize;
    Some(if n <= room { name.to_string() } else { format!("{}…", name.chars().take(room - 1).collect::<String>()) })
}

/// One cell.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    /// Path from the tree root.
    pub path: Vec<usize>,
    /// Row, 0 for the zoomed node.
    pub depth: usize,
    /// Node name.
    pub name: String,
    /// Total size.
    pub value: f64,
    /// Size not covered by children: self time in a flame graph.
    pub self_value: f64,
    /// Left edge, 0..=1.
    pub x0: f64,
    /// Right edge, 0..=1.
    pub x1: f64,
    /// Colour slot from [`slot`]; `None` for the zoomed node itself.
    pub colour: Option<usize>,
    /// It has children, so it can be zoomed into.
    pub zoomable: bool,
}

/// The laid-out partition.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Partition {
    /// Cells in pre-order, the zoomed node first.
    pub cells: Vec<Cell>,
    /// Number of rows.
    pub levels: usize,
}

fn place(
    node: &Tree,
    path: &[usize],
    (x0, x1): (f64, f64),
    depth: usize,
    colour: Option<usize>,
    flame: bool,
    out: &mut Vec<Cell>,
) {
    let total = size(node, flame);
    let mut x = x0;
    for (i, child) in node.children.iter().enumerate() {
        let v = size(child, flame);
        if v <= 0.0 || total <= 0.0 {
            continue;
        }
        let w = (x1 - x0) * v / total;
        let mut p = path.to_vec();
        p.push(i);
        let col = colour.or(Some(slot(i)));
        let kids: f64 = child.children.iter().map(|c| size(c, flame)).sum();
        out.push(Cell {
            path: p.clone(),
            depth,
            name: child.name.clone(),
            value: v,
            self_value: (v - kids).max(0.0),
            x0: x,
            x1: x + w,
            colour: col,
            zoomable: !child.children.is_empty(),
        });
        place(child, &p, (x, x + w), depth + 1, col, flame, out);
        x += w;
    }
}

impl Cell {
    /// The cell's `(x, y, width, row height)` in a `width`×`height` box with `levels`
    /// rows, leaving a 1px gap to the right. Flame graphs stack bottom-up.
    #[must_use]
    pub fn rect(&self, levels: usize, width: f64, height: f64, flame: bool) -> (f64, f64, f64, f64) {
        let levels = levels.max(1);
        let rh = height / levels as f64;
        let row = if flame { levels.saturating_sub(1 + self.depth) } else { self.depth };
        (self.x0 * width, row as f64 * rh, ((self.x1 - self.x0) * width - 1.0).max(0.5), rh)
    }
}

/// Cells for the node at `zoom` (clamped to a valid path), which spans the full width.
/// `flame` sizes branches as [`size`] describes.
#[must_use]
pub fn layout(root: &Tree, zoom: &[usize], flame: bool) -> Partition {
    let zoom = root.valid_path(zoom);
    let Some(node) = root.node_at(&zoom) else { return Partition::default() };
    let total = size(node, flame);
    let kids: f64 = node.children.iter().map(|c| size(c, flame)).sum();
    let mut cells = vec![Cell {
        path: zoom.clone(),
        depth: 0,
        name: node.name.clone(),
        value: total,
        self_value: (total - kids).max(0.0),
        x0: 0.0,
        x1: 1.0,
        colour: None,
        zoomable: !node.children.is_empty(),
    }];
    place(node, &zoom, (0.0, 1.0), 1, None, flame, &mut cells);
    let levels = cells.iter().map(|c| c.depth).max().unwrap_or(0) + 1;
    Partition { cells, levels }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Tree {
        Tree::branch(
            "main",
            vec![
                Tree { name: "parse".into(), value: 10.0, children: vec![Tree::leaf("lex", 6.0)] },
                Tree::leaf("eval", 30.0),
                Tree::leaf("idle", 0.0),
            ],
        )
    }

    #[test]
    fn icicle_widths_are_shares_of_the_zoomed_node() {
        let p = layout(&sample(), &[], false);
        assert_eq!(p.levels, 3);
        assert_eq!(p.cells.len(), 4, "zero-sized nodes are dropped");
        let parse = &p.cells[1];
        assert!((parse.x1 - 6.0 / 36.0).abs() < 1e-9);
        assert_eq!(parse.self_value, 0.0);
        assert!((p.cells[3].x1 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn flame_branches_keep_self_time() {
        let p = layout(&sample(), &[], true);
        let parse = &p.cells[1];
        assert!((parse.value - 10.0).abs() < 1e-9 && (parse.self_value - 4.0).abs() < 1e-9);
        let lex = &p.cells[2];
        assert!((lex.x1 - lex.x0 - 0.6 * (parse.x1 - parse.x0)).abs() < 1e-9);
    }

    #[test]
    fn zooming_rescales_the_subtree_to_full_width() {
        let p = layout(&sample(), &[0], false);
        assert_eq!(p.cells[0].name, "parse");
        assert!((p.cells[1].x0).abs() < 1e-9 && (p.cells[1].x1 - 1.0).abs() < 1e-9);
        assert_eq!(p.cells[1].path, vec![0, 0]);
        let (_, y_top, w, rh) = p.cells[0].rect(p.levels, 800.0, 100.0, false);
        assert!(y_top.abs() < 1e-9 && (w - 799.0).abs() < 1e-9 && (rh - 50.0).abs() < 1e-9);
        assert!((p.cells[0].rect(p.levels, 800.0, 100.0, true).1 - 50.0).abs() < 1e-9, "flame roots sit at the bottom");
    }

    #[test]
    fn colours_cap_at_six_and_names_hash_stably() {
        assert_eq!(slot(3), 3);
        assert_eq!(slot(8), 5);
        let many = Tree::branch("r", (0..8).map(|i| Tree::leaf(format!("n{i}"), 1.0)).collect());
        assert_eq!(legend(&many).len(), 6);
        assert_eq!(legend(&many)[5], "Other");
        assert_eq!(name_slot("render"), name_slot("render"));
        assert!(name_slot("x") < 6);
        assert_eq!(fit_label("render", 100.0, 6.0).as_deref(), Some("render"));
        assert_eq!(fit_label("renderFrame", 50.0, 6.0).as_deref(), Some("render…"));
        assert_eq!(fit_label("x", 20.0, 6.0), None);
        assert_eq!(rows(&sample(), true)[1], ("main > parse".to_string(), 10.0, 4.0));
        assert!(matches("parseJSON", "json") && !matches("eval", "") && !matches("eval", "x"));
    }
}
