//! Voronoi treemap: a tree drawn as nested weighted (power) Voronoi cells inside a circle
//! or a rectangle, each cell's area its share of its parent.
//!
//! Each level is solved inside its parent's cell. Sites start at seeded random points
//! (the seed is fixed, so the same data always gives the same picture), then alternate
//! Lloyd steps (each site moves to its cell's centroid, which rounds the cells out) with
//! weight updates (each weight scales by target area over current area) until the areas
//! are each within 1% of their targets or the iteration budget runs out. Power cells are always
//! convex, so every child level clips against a convex polygon.

#![allow(
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    reason = "item counts are small; the geometry reads best in maths notation"
)]

use crate::hierarchy::Tree;
use crate::plot::{Tip, TipRow};
use std::f64::consts::TAU;
use std::fmt::Write as _;

/// A point.
pub type Point = (f64, f64);

/// The outline the whole tree fills.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Shape {
    /// A circle as large as fits the box.
    #[default]
    Circle,
    /// The whole box.
    Rect,
}

/// One cell.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    /// Path from the tree root.
    pub path: Vec<usize>,
    /// Depth, 1 for the root's children.
    pub depth: usize,
    /// Node name.
    pub name: String,
    /// Node size.
    pub value: f64,
    /// Share of the whole tree, 0..=1.
    pub share: f64,
    /// Which top-level child this sits under, for colour.
    pub colour: usize,
    /// Nothing is drawn inside it: a leaf, or a branch at the depth limit.
    pub leaf: bool,
    /// The cell's corners, clockwise.
    pub polygon: Vec<Point>,
    /// SVG `d` for the polygon.
    pub d: String,
    /// Area-weighted centre, for its label.
    pub centroid: Point,
    /// Area in px².
    pub area: f64,
}

/// The laid-out chart.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    /// Cells by depth, then in tree order: draw them in this order and later ones sit on top.
    pub cells: Vec<Cell>,
    /// SVG `d` for the outer outline.
    pub outline: String,
    /// Number of top-level groups.
    pub groups: usize,
    /// Box width.
    pub width: f64,
    /// Box height.
    pub height: f64,
}

impl Cell {
    /// The cell is roomy enough for its name and value on two lines of 11px text.
    #[must_use]
    pub fn label_fits(&self) -> bool {
        let wide = self.name.chars().count() as f64 * 6.4 + 10.0;
        self.area >= 2200.0 && wide <= self.area.sqrt() * 1.2
    }
}

impl Layout {
    /// Tooltip for cell `i`, anchored at its centroid. Cells below the top level name
    /// their top-level group.
    #[must_use]
    pub fn tip(&self, i: usize, fmt: &dyn Fn(f64) -> String) -> Option<Tip> {
        let c = self.cells.get(i)?;
        let mut rows = vec![
            TipRow::new(None, "Value", fmt(c.value)),
            TipRow::new(None, "Share of all", crate::upset::percent(c.share, 1.0)),
        ];
        if c.depth > 1 {
            let group = self
                .cells
                .iter()
                .find(|g| g.depth == 1 && g.path[0] == c.path[0])
                .map(|g| g.name.clone())
                .unwrap_or_default();
            rows.push(TipRow::new(Some(crate::chord::colour(c.colour, self.groups)), "Group", group));
        }
        Some(Tip {
            x: c.centroid.0 / self.width.max(1.0),
            y: 1.0 - c.centroid.1 / self.height.max(1.0),
            title: c.name.clone(),
            rows,
        })
    }
}

/// Signed area of a polygon (positive when its corners run clockwise on screen).
#[must_use]
pub fn area(poly: &[Point]) -> f64 {
    let n = poly.len();
    (0..n).map(|i| poly[i].0 * poly[(i + 1) % n].1 - poly[(i + 1) % n].0 * poly[i].1).sum::<f64>() / 2.0
}

/// Area-weighted centre of a polygon; its first corner when it has no area.
#[must_use]
pub fn centroid(poly: &[Point]) -> Point {
    let a = area(poly);
    if a.abs() < 1e-12 {
        return poly.first().copied().unwrap_or_default();
    }
    let n = poly.len();
    let (mut x, mut y) = (0.0, 0.0);
    for i in 0..n {
        let (p, q) = (poly[i], poly[(i + 1) % n]);
        let k = p.0 * q.1 - q.0 * p.1;
        x += (p.0 + q.0) * k;
        y += (p.1 + q.1) * k;
    }
    (x / (6.0 * a), y / (6.0 * a))
}

/// Keep the part of `poly` where `a·p <= b`.
fn clip(poly: &[Point], a: Point, b: f64) -> Vec<Point> {
    let side = |p: Point| a.0 * p.0 + a.1 * p.1 - b;
    let mut out = Vec::with_capacity(poly.len() + 1);
    for i in 0..poly.len() {
        let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
        let (sp, sq) = (side(p), side(q));
        if sp <= 0.0 {
            out.push(p);
        }
        if (sp < 0.0 && sq > 0.0) || (sp > 0.0 && sq < 0.0) {
            let t = sp / (sp - sq);
            out.push((p.0 + t * (q.0 - p.0), p.1 + t * (q.1 - p.1)));
        }
    }
    out
}

/// Power cells of weighted `sites` inside the convex `container`: each point belongs to
/// the site with the smallest `|p - site|² - weight`.
#[must_use]
pub fn power_cells(container: &[Point], sites: &[Point], weights: &[f64]) -> Vec<Vec<Point>> {
    (0..sites.len())
        .map(|i| {
            let (pi, wi) = (sites[i], weights[i]);
            let mut cell = container.to_vec();
            for j in 0..sites.len() {
                if j == i || cell.len() < 3 {
                    continue;
                }
                let (pj, wj) = (sites[j], weights[j]);
                // |p - pi|² - wi <= |p - pj|² - wj, rearranged to a half-plane.
                let a = (2.0 * (pj.0 - pi.0), 2.0 * (pj.1 - pi.1));
                let b = (pj.0 * pj.0 + pj.1 * pj.1) - (pi.0 * pi.0 + pi.1 * pi.1) - wj + wi;
                if a.0 == 0.0 && a.1 == 0.0 {
                    continue;
                }
                cell = clip(&cell, a, b);
            }
            if cell.len() < 3 { Vec::new() } else { cell }
        })
        .collect()
}

/// A small deterministic random source (xorshift).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn contains(poly: &[Point], p: Point) -> bool {
    let n = poly.len();
    let sign = area(poly).signum();
    (0..n).all(|i| {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        sign * ((b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0)) >= 0.0
    })
}

/// Split the convex `container` into one cell per entry of `values`, each with that
/// share of its area. `seed` picks the starting sites.
#[must_use]
pub fn tessellate(container: &[Point], values: &[f64], seed: u64, iterations: usize) -> Vec<Vec<Point>> {
    let n = values.len();
    if n == 0 || container.len() < 3 {
        return Vec::new();
    }
    if n == 1 {
        return vec![container.to_vec()];
    }
    let whole = area(container).abs();
    let total: f64 = values.iter().sum();
    let target: Vec<f64> = values.iter().map(|v| whole * v / total).collect();
    let (x0, x1, y0, y1) =
        container.iter().fold((f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY), |b, p| {
            (b.0.min(p.0), b.1.max(p.0), b.2.min(p.1), b.3.max(p.1))
        });
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let mut sites: Vec<Point> = Vec::with_capacity(n);
    let mut tries = 0;
    while sites.len() < n {
        let p = (x0 + rng.next() * (x1 - x0), y0 + rng.next() * (y1 - y0));
        tries += 1;
        if contains(container, p) || tries > 10_000 {
            sites.push(if tries > 10_000 { centroid(container) } else { p });
        }
    }
    let mut weights = vec![whole / n as f64 / TAU; n];
    let mut cells = power_cells(container, &sites, &weights);
    for _ in 0..iterations {
        for (s, c) in sites.iter_mut().zip(&cells) {
            if !c.is_empty() {
                *s = centroid(c);
            }
        }
        cells = power_cells(container, &sites, &weights);
        for i in 0..n {
            let a = area(&cells[i]).abs();
            weights[i] = if a > 1e-9 { weights[i] * (target[i] / a).clamp(0.5, 2.0) } else { weights[i] * 2.0 };
        }
        // A weight so far above a neighbour's that the neighbour's own site falls outside
        // its cell would empty that cell; pull it back.
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let d2 = (sites[i].0 - sites[j].0).powi(2) + (sites[i].1 - sites[j].1).powi(2);
                let over = weights[i] - weights[j] - d2;
                if over > 0.0 {
                    weights[i] -= over * 1.001;
                }
            }
        }
        let floor = weights.iter().copied().fold(f64::INFINITY, f64::min);
        if floor <= 0.0 {
            // Only differences matter, so lift every weight to keep them positive.
            let lift = 1e-6 * whole - floor;
            for w in &mut weights {
                *w += lift;
            }
        }
        cells = power_cells(container, &sites, &weights);
        let worst = (0..n).map(|i| (area(&cells[i]).abs() / target[i] - 1.0).abs()).fold(0.0, f64::max);
        if worst < 0.01 {
            break;
        }
    }
    cells
}

fn outline(shape: Shape, width: f64, height: f64) -> Vec<Point> {
    match shape {
        Shape::Rect => vec![(1.0, 1.0), (width - 1.0, 1.0), (width - 1.0, height - 1.0), (1.0, height - 1.0)],
        Shape::Circle => {
            let r = width.min(height) / 2.0 - 1.0;
            (0..96)
                .map(|k| f64::from(k) / 96.0 * TAU)
                .map(|a| (width / 2.0 + r * a.sin(), height / 2.0 - r * a.cos()))
                .collect()
        }
    }
}

fn path_d(poly: &[Point]) -> String {
    let mut d = String::new();
    for (k, p) in poly.iter().enumerate() {
        let _ = write!(d, "{}{:.2} {:.2}", if k == 0 { 'M' } else { 'L' }, p.0, p.1);
    }
    d.push('Z');
    d
}

/// A subtree waiting to be split: node, path, the polygon it fills and its colour slot.
type Job<'a> = (&'a Tree, Vec<usize>, Vec<Point>, Option<usize>);

/// Lay out `root` as a Voronoi treemap in a `width`×`height` box, up to `max_depth` levels.
#[must_use]
pub fn layout(root: &Tree, width: f64, height: f64, shape: Shape, max_depth: usize) -> Layout {
    let container = outline(shape, width, height);
    let whole = root.total();
    let mut levels: Vec<Vec<Cell>> = vec![Vec::new(); max_depth.max(1)];
    let mut stack: Vec<Job> = vec![(root, Vec::new(), container.clone(), None)];
    while let Some((node, path, poly, colour)) = stack.pop() {
        let depth = path.len() + 1;
        if depth > levels.len() {
            continue;
        }
        let kids: Vec<(usize, &Tree, f64)> =
            node.children.iter().enumerate().map(|(i, c)| (i, c, c.total())).filter(|k| k.2 > 0.0).collect();
        let values: Vec<f64> = kids.iter().map(|k| k.2).collect();
        // Seeded by the path, so a subtree's picture doesn't change when a sibling's data does.
        let seed = path.iter().fold(0x5EED_u64, |h, &i| h.wrapping_mul(31).wrapping_add(i as u64 + 1));
        let polys = tessellate(&poly, &values, seed, 160);
        for ((idx, child, value), cell) in kids.into_iter().zip(polys) {
            if cell.len() < 3 {
                continue;
            }
            let mut p = path.clone();
            p.push(idx);
            let col = colour.unwrap_or(idx);
            let leaf = child.children.is_empty() || depth == levels.len();
            levels[depth - 1].push(Cell {
                path: p.clone(),
                depth,
                name: child.name.clone(),
                value,
                share: if whole > 0.0 { value / whole } else { 0.0 },
                colour: col,
                leaf,
                d: path_d(&cell),
                centroid: centroid(&cell),
                area: area(&cell).abs(),
                polygon: cell.clone(),
            });
            if !leaf {
                stack.push((child, p, cell, Some(col)));
            }
        }
    }
    for level in &mut levels {
        level.sort_by(|a, b| a.path.cmp(&b.path));
    }
    let groups = levels.first().map_or(0, Vec::len);
    Layout { cells: levels.into_iter().flatten().collect(), outline: path_d(&container), groups, width, height }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<Point> {
        vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)]
    }

    #[test]
    fn polygon_maths() {
        assert!((area(&square()) - 10_000.0).abs() < 1e-9);
        assert_eq!(centroid(&square()), (50.0, 50.0));
        let half = clip(&square(), (1.0, 0.0), 50.0);
        assert!((area(&half) - 5000.0).abs() < 1e-9);
    }

    #[test]
    fn equal_weights_give_the_plain_voronoi_bisector() {
        let cells = power_cells(&square(), &[(25.0, 50.0), (75.0, 50.0)], &[0.0, 0.0]);
        assert!((area(&cells[0]) - 5000.0).abs() < 1e-9);
        // A heavier site pushes the boundary towards its neighbour.
        let cells = power_cells(&square(), &[(25.0, 50.0), (75.0, 50.0)], &[500.0, 0.0]);
        assert!(area(&cells[0]) > 5000.0);
    }

    #[test]
    fn cells_converge_on_their_target_areas() {
        let values = [40.0, 25.0, 15.0, 10.0, 6.0, 4.0];
        let cells = tessellate(&square(), &values, 7, 200);
        let total: f64 = values.iter().sum();
        let mut covered = 0.0;
        for (c, v) in cells.iter().zip(values) {
            let a = area(c).abs();
            covered += a;
            assert!((a / (10_000.0 * v / total) - 1.0).abs() < 0.02, "{v}: {a}");
        }
        assert!((covered - 10_000.0).abs() < 1e-6, "cells tile the container");
    }

    #[test]
    fn the_same_data_always_gives_the_same_picture() {
        let t = Tree::branch(
            "r",
            vec![
                Tree::branch("a", vec![Tree::leaf("a1", 3.0), Tree::leaf("a2", 2.0)]),
                Tree::leaf("b", 4.0),
                Tree::leaf("c", 1.0),
            ],
        );
        let one = layout(&t, 300.0, 300.0, Shape::Circle, 3);
        assert_eq!(one, layout(&t, 300.0, 300.0, Shape::Circle, 3));
        assert_eq!(one.groups, 3);
        assert_eq!(one.cells.len(), 5);
        let a = one.cells.iter().find(|c| c.name == "a").expect("a");
        assert!(!a.leaf);
        let kids: f64 = one.cells.iter().filter(|c| c.depth == 2).map(|c| c.area).sum();
        assert!((kids - a.area).abs() < 1e-6, "children tile their parent");
        assert!(one.cells.iter().filter(|c| c.depth == 2).all(|c| c.colour == a.colour));
        let leaf = one.cells.iter().position(|c| c.name == "a1").expect("a1");
        assert!(one.cells[leaf].label_fits(), "{}", one.cells[leaf].area);
        let tip = one.tip(leaf, &|v| format!("{v}")).expect("tip");
        assert_eq!((tip.rows[1].value.as_str(), tip.rows[2].value.as_str()), ("30%", "a"));
        let r = 149.0;
        assert!(one.cells.iter().all(|c| c.polygon.iter().all(|p| (p.0 - 150.0).hypot(p.1 - 150.0) <= r + 1e-6)));
    }

    #[test]
    fn depth_limit_and_empty_trees() {
        let t = Tree::branch("r", vec![Tree::branch("a", vec![Tree::leaf("a1", 3.0)]), Tree::leaf("z", 0.0)]);
        let l = layout(&t, 200.0, 100.0, Shape::Rect, 1);
        assert_eq!(l.cells.len(), 1);
        assert!(l.cells[0].leaf && (l.cells[0].area - 198.0 * 98.0).abs() < 1e-6);
        assert_eq!(layout(&Tree::branch("r", vec![]), 100.0, 100.0, Shape::Circle, 2).cells.len(), 0);
    }
}
