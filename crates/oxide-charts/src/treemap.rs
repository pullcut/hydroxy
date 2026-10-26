//! Squarified treemap with two levels of nesting. Paths are absolute (from the tree's
//! root), so a cell's path is also the zoom target for clicking it.

#![allow(clippy::many_single_char_names, reason = "rectangle maths reads best as x, y, w, h")]

use crate::hierarchy::Tree;

/// A rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// One drawn cell.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    /// Path from the tree root to this node; zoom here by passing it to [`layout`].
    pub path: Vec<usize>,
    /// 1 for the zoomed node's children, 2 for their children.
    pub depth: usize,
    /// Node name.
    pub name: String,
    /// Node size.
    pub value: f64,
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width.
    pub w: f64,
    /// Height.
    pub h: f64,
    /// Which top-level child this sits under, for colour.
    pub colour: usize,
    /// It has children, so a click can zoom into it.
    pub zoomable: bool,
    /// Big enough to print its name.
    pub label: bool,
    /// Its children are drawn inside it, below a header strip.
    pub nested: bool,
}

/// Height of a parent cell's header strip, where its name sits.
pub const HEADER: f64 = 20.0;
const GAP: f64 = 2.0;

fn worst(sum: f64, max: f64, min: f64, side: f64) -> f64 {
    let s2 = sum * sum;
    f64::max(side * side * max / s2, s2 / (side * side * min))
}

/// Squarify `areas` (descending, summing to the rect's area) into `r`.
fn squarify(areas: &[f64], r: Rect) -> Vec<Rect> {
    let mut out = Vec::with_capacity(areas.len());
    let Rect { mut x, mut y, mut w, mut h } = r;
    let mut i = 0;
    while i < areas.len() {
        if areas[i] <= 0.0 || w <= 0.0 || h <= 0.0 {
            out.push(Rect { x, y, w: 0.0, h: 0.0 });
            i += 1;
            continue;
        }
        let side = w.min(h);
        let mut j = i + 1;
        let mut sum = areas[i];
        let mut ratio = worst(sum, areas[i], areas[i], side);
        while j < areas.len() && areas[j] > 0.0 {
            let next = worst(sum + areas[j], areas[i], areas[j], side);
            if next > ratio {
                break;
            }
            sum += areas[j];
            ratio = next;
            j += 1;
        }
        // The row is a strip along the short side.
        if w >= h {
            let strip = sum / h;
            let mut cy = y;
            for a in &areas[i..j] {
                let ch = a / strip;
                out.push(Rect { x, y: cy, w: strip, h: ch });
                cy += ch;
            }
            x += strip;
            w -= strip;
        } else {
            let strip = sum / w;
            let mut cx = x;
            for a in &areas[i..j] {
                let cw = a / strip;
                out.push(Rect { x: cx, y, w: cw, h: strip });
                cx += cw;
            }
            y += strip;
            h -= strip;
        }
        i = j;
    }
    out
}

fn place(node: &Tree, path: &[usize], r: Rect, depth: usize, colour: Option<usize>, out: &mut Vec<Cell>) {
    let mut kids: Vec<(usize, &Tree, f64)> = node.children.iter().enumerate().map(|(i, c)| (i, c, c.total())).filter(|k| k.2 > 0.0).collect();
    kids.sort_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));
    let total: f64 = kids.iter().map(|k| k.2).sum();
    let areas: Vec<f64> = kids.iter().map(|k| k.2 / total * r.w * r.h).collect();
    for ((idx, child, value), rect) in kids.iter().zip(squarify(&areas, r)) {
        let mut p = path.to_vec();
        p.push(*idx);
        let c = colour.unwrap_or(*idx);
        let (x, y) = (rect.x + GAP / 2.0, rect.y + GAP / 2.0);
        let (w, h) = ((rect.w - GAP).max(0.0), (rect.h - GAP).max(0.0));
        let zoomable = !child.children.is_empty() && *value > 0.0;
        let nested = depth == 1 && zoomable && w >= 72.0 && h >= 56.0;
        out.push(Cell { path: p.clone(), depth, name: child.name.clone(), value: *value, x, y, w, h, colour: c, zoomable, label: w >= 40.0 && h >= 20.0, nested });
        if nested {
            let inner = Rect { x: x + 4.0, y: y + HEADER, w: w - 8.0, h: h - HEADER - 4.0 };
            place(child, &p, inner, 2, Some(c), out);
        }
    }
}

/// Cells for the children of the node at `zoom` (clamped to a valid path), fitted to
/// `width`×`height`, with the children of big cells nested inside them. Cells come in
/// paint order: a parent before its children.
#[must_use]
pub fn layout(root: &Tree, zoom: &[usize], width: f64, height: f64) -> Vec<Cell> {
    let zoom = root.valid_path(zoom);
    let Some(node) = root.node_at(&zoom) else { return Vec::new() };
    let mut out = Vec::new();
    place(node, &zoom, Rect { x: 0.0, y: 0.0, w: width, h: height }, 1, None, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Tree {
        Tree::branch(
            "All",
            vec![
                Tree::branch("A", vec![Tree::leaf("a1", 30.0), Tree::leaf("a2", 20.0), Tree::leaf("a3", 10.0)]),
                Tree::leaf("B", 25.0),
                Tree::leaf("C", 10.0),
                Tree::leaf("D", 5.0),
                Tree::leaf("Zero", 0.0),
            ],
        )
    }

    #[test]
    fn areas_are_proportional_and_tile_the_box() {
        let cells: Vec<Cell> = layout(&sample(), &[], 400.0, 300.0).into_iter().filter(|c| c.depth == 1).collect();
        assert_eq!(cells.len(), 4, "zero-sized children are skipped");
        let total: f64 = cells.iter().map(|c| (c.w + 2.0) * (c.h + 2.0)).sum();
        assert!((total - 120_000.0).abs() < 1e-6, "{total}");
        let a = cells.iter().find(|c| c.name == "A").expect("A");
        let b = cells.iter().find(|c| c.name == "B").expect("B");
        let ratio = ((a.w + 2.0) * (a.h + 2.0)) / ((b.w + 2.0) * (b.h + 2.0));
        assert!((ratio - 60.0 / 25.0).abs() < 1e-6);
    }

    #[test]
    fn cells_do_not_overlap_and_stay_inside() {
        let cells: Vec<Cell> = layout(&sample(), &[], 400.0, 300.0).into_iter().filter(|c| c.depth == 1).collect();
        for (i, a) in cells.iter().enumerate() {
            assert!(a.x >= 0.0 && a.y >= 0.0 && a.x + a.w <= 400.0 + 1e-9 && a.y + a.h <= 300.0 + 1e-9);
            for b in &cells[i + 1..] {
                let apart = a.x + a.w <= b.x + 1e-9 || b.x + b.w <= a.x + 1e-9 || a.y + a.h <= b.y + 1e-9 || b.y + b.h <= a.y + 1e-9;
                assert!(apart, "{a:?} overlaps {b:?}");
            }
        }
    }

    #[test]
    fn cells_are_squarish() {
        let cells = layout(&Tree::branch("r", (0..8).map(|i| Tree::leaf(format!("n{i}"), 10.0 - f64::from(i))).collect()), &[], 400.0, 300.0);
        assert!(cells.iter().all(|c| (c.w / c.h).max(c.h / c.w) < 4.5));
    }

    #[test]
    fn big_parents_nest_their_children_inside() {
        let cells = layout(&sample(), &[], 400.0, 300.0);
        let a = cells.iter().find(|c| c.name == "A" && c.depth == 1).expect("A");
        assert!(a.nested && a.zoomable);
        let kids: Vec<&Cell> = cells.iter().filter(|c| c.depth == 2).collect();
        assert_eq!(kids.len(), 3);
        for k in kids {
            assert_eq!(k.path[0], 0);
            assert_eq!(k.colour, a.colour);
            assert!(k.y >= a.y + HEADER - 1e-9 && k.x >= a.x && k.x + k.w <= a.x + a.w + 1e-9 && k.y + k.h <= a.y + a.h + 1e-9);
        }
        // Parents come before their children in paint order.
        assert!(cells.iter().position(|c| c.depth == 2) > cells.iter().position(|c| c.name == "A"));
    }

    #[test]
    fn zooming_lays_out_the_chosen_node_and_clamps_bad_paths() {
        let zoomed = layout(&sample(), &[0], 400.0, 300.0);
        assert!(zoomed.iter().all(|c| c.path.starts_with(&[0])));
        assert_eq!(zoomed.iter().filter(|c| c.depth == 1).count(), 3);
        assert_eq!(layout(&sample(), &[9, 9], 400.0, 300.0), layout(&sample(), &[], 400.0, 300.0));
    }

    #[test]
    fn an_empty_tree_has_no_cells() {
        assert_eq!(layout(&Tree::branch("r", vec![]), &[], 100.0, 100.0), vec![]);
    }
}
