//! Circle packing: siblings packed with a front chain (Wang et al., as in d3's
//! `packSiblings`), each parent the smallest circle enclosing its children (Welzl).
//! A leaf's area is proportional to its value. Coordinates are px in a square box.

#![allow(clippy::cast_precision_loss, reason = "item counts are small")]
#![allow(clippy::many_single_char_names, reason = "geometry follows the published formulas")]

use crate::hierarchy::Tree;
use crate::partition::slot;

/// A circle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    /// Centre x.
    pub x: f64,
    /// Centre y.
    pub y: f64,
    /// Radius.
    pub r: f64,
}

const fn c(x: f64, y: f64, r: f64) -> Circle {
    Circle { x, y, r }
}

fn place(b: Circle, a: Circle, r: f64) -> Circle {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let d2 = dx * dx + dy * dy;
    if d2 <= 0.0 {
        return c(a.x + r, a.y, r);
    }
    let (a2, b2) = ((a.r + r).powi(2), (b.r + r).powi(2));
    if a2 > b2 {
        let x = (d2 + b2 - a2) / (2.0 * d2);
        let y = (b2 / d2 - x * x).max(0.0).sqrt();
        c(b.x - x * dx - y * dy, b.y - x * dy + y * dx, r)
    } else {
        let x = (d2 + a2 - b2) / (2.0 * d2);
        let y = (a2 / d2 - x * x).max(0.0).sqrt();
        c(a.x + x * dx - y * dy, a.y + x * dy + y * dx, r)
    }
}

fn intersects(a: Circle, b: Circle) -> bool {
    let dr = a.r + b.r - 1e-6;
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    dr > 0.0 && dr * dr > dx * dx + dy * dy
}

/// Place circles of the given radii side by side without overlap, the first ones near
/// the origin. Returns the circles and the circle enclosing them, centred on the origin.
#[must_use]
pub fn siblings(radii: &[f64]) -> (Vec<Circle>, Circle) {
    let n = radii.len();
    let mut cs: Vec<Circle> = radii.iter().map(|&r| c(0.0, 0.0, r)).collect();
    if n == 0 {
        return (cs, c(0.0, 0.0, 0.0));
    }
    if n == 1 {
        return (cs, c(0.0, 0.0, radii[0]));
    }
    cs[0].x = -radii[1];
    cs[1].x = radii[0];
    if n > 2 {
        cs[2] = place(cs[1], cs[0], radii[2]);
        // The front chain as a ring of indices: a -> b -> c -> a.
        let mut next = vec![0; n];
        let mut prev = vec![0; n];
        let (mut a, mut b) = (0, 1);
        (next[0], next[1], next[2]) = (1, 2, 0);
        (prev[0], prev[1], prev[2]) = (2, 0, 1);
        let mut i = 3;
        'pack: while i < n {
            cs[i] = place(cs[a], cs[b], radii[i]);
            let (mut j, mut k) = (next[b], prev[a]);
            let (mut sj, mut sk) = (cs[b].r, cs[a].r);
            loop {
                if sj <= sk {
                    if intersects(cs[j], cs[i]) {
                        b = j;
                        next[a] = b;
                        prev[b] = a;
                        continue 'pack;
                    }
                    sj += cs[j].r;
                    j = next[j];
                } else {
                    if intersects(cs[k], cs[i]) {
                        a = k;
                        next[a] = b;
                        prev[b] = a;
                        continue 'pack;
                    }
                    sk += cs[k].r;
                    k = prev[k];
                }
                if j == next[k] {
                    break;
                }
            }
            prev[i] = a;
            next[i] = b;
            next[a] = i;
            prev[b] = i;
            b = i;
            // Pick the new pair closest to the origin.
            let score = |p: usize, next: &[usize]| {
                let (u, v) = (cs[p], cs[next[p]]);
                let ab = u.r + v.r;
                let (dx, dy) = ((u.x * v.r + v.x * u.r) / ab, (u.y * v.r + v.y * u.r) / ab);
                dx * dx + dy * dy
            };
            let mut best = score(a, &next);
            let mut cur = next[b];
            while cur != b {
                let s = score(cur, &next);
                if s < best {
                    a = cur;
                    best = s;
                }
                cur = next[cur];
            }
            b = next[a];
            i += 1;
        }
        let mut chain = vec![cs[b]];
        let mut cur = next[b];
        while cur != b {
            chain.push(cs[cur]);
            cur = next[cur];
        }
        let e = enclose(&chain);
        for p in &mut cs {
            p.x -= e.x;
            p.y -= e.y;
        }
        return (cs, c(0.0, 0.0, e.r));
    }
    let e = enclose(&cs);
    for p in &mut cs {
        p.x -= e.x;
        p.y -= e.y;
    }
    (cs, c(0.0, 0.0, e.r))
}

fn encloses_not(a: Circle, b: Circle) -> bool {
    let dr = a.r - b.r;
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    dr < 0.0 || dr * dr < dx * dx + dy * dy
}

fn encloses_weak(a: Circle, b: Circle) -> bool {
    let dr = a.r - b.r + a.r.max(b.r).max(1.0) * 1e-9;
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    dr > 0.0 && dr * dr > dx * dx + dy * dy
}

fn encloses_all(a: Circle, set: &[Circle]) -> bool {
    set.iter().all(|&b| encloses_weak(a, b))
}

fn basis2(a: Circle, b: Circle) -> Circle {
    let (x21, y21, r21) = (b.x - a.x, b.y - a.y, b.r - a.r);
    let l = x21.hypot(y21);
    if l <= 0.0 {
        return c(a.x, a.y, a.r.max(b.r));
    }
    c((a.x + b.x + x21 / l * r21) / 2.0, (a.y + b.y + y21 / l * r21) / 2.0, (l + a.r + b.r) / 2.0)
}

fn basis3(a: Circle, b: Circle, cc: Circle) -> Circle {
    let (x1, y1, r1) = (a.x, a.y, a.r);
    let (a2, a3) = (x1 - b.x, x1 - cc.x);
    let (b2, b3) = (y1 - b.y, y1 - cc.y);
    let (c2, c3) = (b.r - r1, cc.r - r1);
    let d1 = x1 * x1 + y1 * y1 - r1 * r1;
    let d2 = d1 - b.x * b.x - b.y * b.y + b.r * b.r;
    let d3 = d1 - cc.x * cc.x - cc.y * cc.y + cc.r * cc.r;
    let ab = a3 * b2 - a2 * b3;
    let xa = (b2 * d3 - b3 * d2) / (ab * 2.0) - x1;
    let xb = (b3 * c2 - b2 * c3) / ab;
    let ya = (a3 * d2 - a2 * d3) / (ab * 2.0) - y1;
    let yb = (a2 * c3 - a3 * c2) / ab;
    let qa = xb * xb + yb * yb - 1.0;
    let qb = 2.0 * (r1 + xa * xb + ya * yb);
    let qc = xa * xa + ya * ya - r1 * r1;
    let r = -(if qa.abs() > 1e-6 { (qb + (qb * qb - 4.0 * qa * qc).max(0.0).sqrt()) / (2.0 * qa) } else { qc / qb });
    c(x1 + xa + xb * r, y1 + ya + yb * r, r)
}

fn extend(basis: &[Circle], p: Circle) -> Option<Vec<Circle>> {
    if encloses_all(p, basis) {
        return Some(vec![p]);
    }
    for &b in basis {
        if encloses_not(p, b) && encloses_all(basis2(b, p), basis) {
            return Some(vec![b, p]);
        }
    }
    for (i, &bi) in basis.iter().enumerate() {
        for &bj in &basis[i + 1..] {
            if encloses_not(basis2(bi, bj), p)
                && encloses_not(basis2(bi, p), bj)
                && encloses_not(basis2(bj, p), bi)
                && encloses_all(basis3(bi, bj, p), basis)
            {
                return Some(vec![bi, bj, p]);
            }
        }
    }
    None
}

/// The smallest circle enclosing every circle in `set` (Welzl's move-to-front form).
#[must_use]
pub fn enclose(set: &[Circle]) -> Circle {
    let mut basis: Vec<Circle> = Vec::new();
    let mut e: Option<Circle> = None;
    let mut i = 0;
    while i < set.len() {
        let p = set[i];
        if e.is_some_and(|e| encloses_weak(e, p)) {
            i += 1;
            continue;
        }
        let Some(b) = extend(&basis, p) else { break };
        basis = b;
        e = Some(match basis.as_slice() {
            [a] => *a,
            [a, b] => basis2(*a, *b),
            [a, b, cc] => basis3(*a, *b, *cc),
            _ => unreachable!("a basis holds one to three circles"),
        });
        i = 0;
    }
    // Numerical trouble only: fall back to a circle around everything.
    let e = e.unwrap_or(c(0.0, 0.0, 0.0));
    if set.iter().all(|&p| encloses_weak(e, p) || !encloses_not(e, p)) {
        e
    } else {
        let r = set.iter().map(|p| (p.x - e.x).hypot(p.y - e.y) + p.r).fold(0.0, f64::max);
        c(e.x, e.y, r)
    }
}

/// One packed node.
#[derive(Debug, Clone, PartialEq)]
pub struct Bubble {
    /// Path from the tree root.
    pub path: Vec<usize>,
    /// 0 for the zoomed node.
    pub depth: usize,
    /// Node name.
    pub name: String,
    /// Node size.
    pub value: f64,
    /// Centre x in px.
    pub x: f64,
    /// Centre y in px.
    pub y: f64,
    /// Radius in px.
    pub r: f64,
    /// Colour slot from [`slot`]; `None` for the zoomed node.
    pub colour: Option<usize>,
    /// It has children, so it can be zoomed into.
    pub zoomable: bool,
}

struct Packed {
    r: f64,
    kids: Vec<(usize, f64, f64, Packed)>,
}

fn pack(node: &Tree, pad: f64) -> Packed {
    if node.children.is_empty() {
        return Packed { r: node.total().sqrt(), kids: Vec::new() };
    }
    let kids: Vec<(usize, Packed)> =
        node.children.iter().enumerate().filter(|(_, c)| c.total() > 0.0).map(|(i, c)| (i, pack(c, pad))).collect();
    let radii: Vec<f64> = kids.iter().map(|(_, p)| p.r + pad).collect();
    let (circles, e) = siblings(&radii);
    let r = if kids.is_empty() { 0.0 } else { e.r + pad };
    Packed { r, kids: kids.into_iter().zip(circles).map(|((i, p), c)| (i, c.x, c.y, p)).collect() }
}

#[allow(clippy::too_many_arguments, reason = "recursive helper")]
fn flatten(
    node: &Tree,
    packed: &Packed,
    path: &[usize],
    (x, y, k): (f64, f64, f64),
    depth: usize,
    colour: Option<usize>,
    out: &mut Vec<Bubble>,
) {
    for (i, cx, cy, p) in &packed.kids {
        let child = &node.children[*i];
        let mut path = path.to_vec();
        path.push(*i);
        let col = colour.or_else(|| Some(slot(*i)));
        let (bx, by) = (x + cx * k, y + cy * k);
        out.push(Bubble {
            path: path.clone(),
            depth,
            name: child.name.clone(),
            value: child.total(),
            x: bx,
            y: by,
            r: p.r * k,
            colour: col,
            zoomable: !child.children.is_empty(),
        });
        flatten(child, p, &path, (bx, by, k), depth + 1, col, out);
    }
}

/// Circles for the node at `zoom` (clamped to a valid path) filling a `size`×`size`
/// box, with `padding` px between siblings and inside parents. Pre-order, the zoomed
/// node first.
#[must_use]
pub fn layout(root: &Tree, zoom: &[usize], size: f64, padding: f64) -> Vec<Bubble> {
    let zoom = root.valid_path(zoom);
    let Some(node) = root.node_at(&zoom) else { return Vec::new() };
    let half = size / 2.0 - 1.0;
    // Pack once unpadded to find the scale, then again with the padding in those units.
    let first = pack(node, 0.0);
    let k0 = if first.r > 0.0 { half / first.r } else { 1.0 };
    let packed = pack(node, padding / k0);
    let k = if packed.r > 0.0 { half / packed.r } else { 1.0 };
    let mut out = vec![Bubble {
        path: zoom.clone(),
        depth: 0,
        name: node.name.clone(),
        value: node.total(),
        x: size / 2.0,
        y: size / 2.0,
        r: half,
        colour: None,
        zoomable: !node.children.is_empty(),
    }];
    flatten(node, &packed, &zoom, (size / 2.0, size / 2.0, k), 1, None, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overlap(a: Circle, b: Circle) -> bool {
        (a.x - b.x).hypot(a.y - b.y) < a.r + b.r - 1e-6
    }

    #[test]
    fn siblings_never_overlap_and_stay_enclosed() {
        let radii: Vec<f64> = (1..=25).map(|i| f64::from(i % 7 + 1)).collect();
        let (cs, e) = siblings(&radii);
        for (i, a) in cs.iter().enumerate() {
            for b in &cs[i + 1..] {
                assert!(!overlap(*a, *b), "{a:?} overlaps {b:?}");
            }
            assert!(a.x.hypot(a.y) + a.r <= e.r + 1e-6);
        }
        // Packing is reasonably tight: circle area is at least half the enclosure.
        let area: f64 = radii.iter().map(|r| r * r).sum();
        assert!(area / (e.r * e.r) > 0.5);
    }

    #[test]
    fn enclosing_two_circles_spans_both() {
        let e = enclose(&[c(0.0, 0.0, 1.0), c(4.0, 0.0, 1.0)]);
        assert!((e.r - 3.0).abs() < 1e-9 && (e.x - 2.0).abs() < 1e-9);
    }

    #[test]
    fn layout_fits_the_box_and_nests_children() {
        let t = Tree::branch(
            "All",
            vec![
                Tree::branch("A", vec![Tree::leaf("a1", 9.0), Tree::leaf("a2", 4.0)]),
                Tree::leaf("B", 16.0),
                Tree::leaf("Z", 0.0),
            ],
        );
        let b = layout(&t, &[], 200.0, 2.0);
        assert_eq!(b.len(), 5);
        let a = &b[1];
        for kid in &b[2..4] {
            assert!((kid.x - a.x).hypot(kid.y - a.y) + kid.r <= a.r + 1e-6);
            assert_eq!(kid.colour, a.colour);
        }
        assert!(b.iter().all(|x| x.x - x.r >= -1e-6 && x.x + x.r <= 200.0 + 1e-6));
        // Leaf area follows value.
        assert!((b[2].r / b[3].r - 1.5).abs() < 0.05);
        let zoomed = layout(&t, &[0], 200.0, 2.0);
        assert_eq!(zoomed[0].name, "A");
        assert_eq!(zoomed.len(), 3);
    }
}
