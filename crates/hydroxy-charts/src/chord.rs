//! Chord diagram geometry from a square flow matrix, d3-chord style: each group's arc
//! is sized by its row total, and each pair of groups shares one ribbon whose ends are
//! as wide as the flow each way. Coordinates are px in a `SIZE`×`SIZE` box; angles run
//! clockwise from twelve o'clock.

#![allow(clippy::cast_precision_loss, reason = "group counts are small")]
#![allow(clippy::needless_range_loop, reason = "i and j index the matrix and its sub-arcs together")]

use std::f64::consts::TAU;
use std::fmt::Write as _;

use crate::plot::{Tip, TipRow};
use crate::scale::non_negative;

/// Width and height of the drawing.
pub const SIZE: f64 = 400.0;
const OUTER: f64 = 150.0;
const INNER: f64 = 140.0;
const LABEL: f64 = 160.0;

/// One group's arc.
#[derive(Debug, Clone, PartialEq)]
pub struct Arc {
    /// Start angle in radians.
    pub start: f64,
    /// End angle in radians.
    pub end: f64,
    /// Row total: everything this group sends.
    pub out: f64,
    /// Column total: everything this group receives.
    pub incoming: f64,
    /// SVG `d` for the annular sector.
    pub path: String,
    /// Where the label sits, outside the ring.
    pub label: (f64, f64),
    /// `text-anchor` for the label: "start" on the right half, "end" on the left.
    pub anchor: &'static str,
}

/// One ribbon between two groups (or a group and itself).
#[derive(Debug, Clone, PartialEq)]
pub struct Ribbon {
    /// The lower group index.
    pub source: usize,
    /// The higher group index.
    pub target: usize,
    /// Flow from `source` to `target`.
    pub value: f64,
    /// Flow from `target` back to `source`.
    pub back: f64,
    /// SVG `d`.
    pub path: String,
    /// A point inside the ribbon for the tooltip.
    pub anchor: (f64, f64),
    /// The group whose colour the ribbon takes: the side that sends more.
    pub colour: usize,
}

/// A laid-out chord diagram.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Chord {
    /// One per matrix row.
    pub arcs: Vec<Arc>,
    /// One per pair with any flow.
    pub ribbons: Vec<Ribbon>,
}

fn point(r: f64, a: f64) -> (f64, f64) {
    crate::geom::polar(SIZE / 2.0, r, a)
}

/// Lay out `matrix`, where `matrix[i][j]` is the flow from group `i` to group `j`.
/// Missing entries and negative or non-finite values count as zero. `pad` is the gap
/// between groups in radians.
#[must_use]
pub fn layout(matrix: &[Vec<f64>], pad: f64) -> Chord {
    let n = matrix.len();
    let at = |i: usize, j: usize| matrix.get(i).and_then(|r| r.get(j)).copied().map_or(0.0, non_negative);
    let out: Vec<f64> = (0..n).map(|i| (0..n).map(|j| at(i, j)).sum()).collect();
    let total: f64 = out.iter().sum();
    if n == 0 {
        return Chord::default();
    }
    let pad = if total > 0.0 { pad.clamp(0.0, TAU / n as f64 / 2.0) } else { 0.0 };
    let k = if total > 0.0 { (TAU - pad * n as f64) / total } else { 0.0 };
    // Sub-arc start for flow i→j, laid out within group i in column order.
    let mut sub = vec![vec![(0.0, 0.0); n]; n];
    let mut arcs = Vec::with_capacity(n);
    let mut angle = 0.0;
    for i in 0..n {
        let start = angle;
        for j in 0..n {
            let w = at(i, j) * k;
            sub[i][j] = (angle, angle + w);
            angle += w;
        }
        let end = angle;
        angle += pad;
        let mid = f64::midpoint(start, end);
        let large = u8::from(end - start > TAU / 2.0);
        let (o0, o1, i1, i0) = (point(OUTER, start), point(OUTER, end), point(INNER, end), point(INNER, start));
        let path = format!(
            "M{:.2} {:.2}A{OUTER} {OUTER} 0 {large} 1 {:.2} {:.2}L{:.2} {:.2}A{INNER} {INNER} 0 {large} 0 {:.2} {:.2}Z",
            o0.0, o0.1, o1.0, o1.1, i1.0, i1.1, i0.0, i0.1
        );
        arcs.push(Arc {
            start,
            end,
            out: out[i],
            incoming: (0..n).map(|j| at(j, i)).sum(),
            path,
            label: point(LABEL, mid),
            anchor: if mid.sin() >= 0.0 { "start" } else { "end" },
        });
    }
    let mut ribbons = Vec::with_capacity(n * (n + 1) / 2);
    for i in 0..n {
        for j in i..n {
            let (value, back) = (at(i, j), if i == j { 0.0 } else { at(j, i) });
            if value + back <= 0.0 {
                continue;
            }
            let ((a0, a1), (b0, b1)) = (sub[i][j], sub[j][i]);
            let c = SIZE / 2.0;
            let p = |a: f64| point(INNER, a);
            let (s0, s1, t0, t1) = (p(a0), p(a1), p(b0), p(b1));
            let mut path = String::new();
            let arc = |path: &mut String, from: f64, to: f64, end: (f64, f64)| {
                let large = u8::from(to - from > TAU / 2.0);
                let _ = write!(path, "A{INNER} {INNER} 0 {large} 1 {:.2} {:.2}", end.0, end.1);
            };
            let _ = write!(path, "M{:.2} {:.2}", s0.0, s0.1);
            arc(&mut path, a0, a1, s1);
            let _ = write!(path, "Q{c} {c} {:.2} {:.2}", t0.0, t0.1);
            arc(&mut path, b0, b1, t1);
            let _ = write!(path, "Q{c} {c} {:.2} {:.2}Z", s0.0, s0.1);
            let (ma, mb) = (point(INNER * 0.55, f64::midpoint(a0, a1)), point(INNER * 0.55, f64::midpoint(b0, b1)));
            ribbons.push(Ribbon {
                source: i,
                target: j,
                value,
                back,
                path,
                anchor: (f64::midpoint(ma.0, mb.0), f64::midpoint(ma.1, mb.1)),
                colour: if back > value { j } else { i },
            });
        }
    }
    Chord { arcs, ribbons }
}

/// Palette slot for group `i` of `n`: `Some(0..6)` for a `--chart-N` colour, `None` when it
/// folds into the muted "Other" colour. Six or fewer groups each get their own; with
/// more, the first five do and the rest share "Other".
#[must_use]
pub const fn slot(i: usize, n: usize) -> Option<usize> {
    if n <= 6 || i < 5 { Some(i) } else { None }
}

/// CSS colour for group `i` of `n`.
#[must_use]
pub fn colour(i: usize, n: usize) -> String {
    crate::plot::colour_or_muted(i, if n <= 6 { 6 } else { 5 })
}

/// What the pointer or keyboard is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hit {
    /// A group's arc, by index.
    Group(usize),
    /// A ribbon, by index into [`Chord::ribbons`].
    Ribbon(usize),
}

/// The tooltip for `hit`, anchored at the arc's label or inside the ribbon. `name` gives a
/// group's display name and `format` formats flows. `None` when `hit` is out of range.
#[must_use]
pub fn tooltip(lay: &Chord, hit: Hit, name: impl Fn(usize) -> String, format: &dyn Fn(f64) -> String) -> Option<Tip> {
    let (title, rows, at) = match hit {
        Hit::Group(i) => {
            let arc = lay.arcs.get(i)?;
            let rows =
                vec![TipRow::new(None, "Sends", format(arc.out)), TipRow::new(None, "Receives", format(arc.incoming))];
            (name(i), rows, arc.label)
        }
        Hit::Ribbon(k) => {
            let rib = lay.ribbons.get(k)?;
            let (from, to) = (name(rib.source), name(rib.target));
            let mut rows = vec![TipRow::new(None, format!("{from} → {to}"), format(rib.value))];
            if rib.source != rib.target {
                rows.push(TipRow::new(None, format!("{to} → {from}"), format(rib.back)));
            }
            let title = if rib.source == rib.target { from } else { format!("{from} and {to}") };
            (title, rows, rib.anchor)
        }
    };
    Some(Tip { x: at.0 / SIZE, y: 1.0 - at.1 / SIZE, title, rows })
}

/// The swatch colour for `hit`'s tooltip: the group's, or the side of the ribbon that sends more.
#[must_use]
pub fn swatch(lay: &Chord, hit: Hit) -> Option<String> {
    let groups = lay.arcs.len();
    match hit {
        Hit::Group(i) => lay.arcs.get(i).map(|_| colour(i, groups)),
        Hit::Ribbon(k) => lay.ribbons.get(k).map(|r| colour(r.colour, groups)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Vec<f64>> {
        vec![vec![0.0, 4.0, 2.0], vec![1.0, 0.0, 0.0], vec![3.0, 0.0, 2.0]]
    }

    #[test]
    fn arcs_share_the_circle_by_row_total() {
        let c = layout(&sample(), 0.0);
        let spans: Vec<f64> = c.arcs.iter().map(|a| a.end - a.start).collect();
        assert!((spans.iter().sum::<f64>() - TAU).abs() < 1e-9);
        assert!((spans[0] / spans[1] - 6.0).abs() < 1e-9, "row totals 6 and 1");
        assert!((c.arcs[0].incoming - 4.0).abs() < 1e-9);
    }

    #[test]
    fn one_ribbon_per_pair_with_both_directions() {
        let c = layout(&sample(), 0.05);
        let pairs: Vec<(usize, usize)> = c.ribbons.iter().map(|r| (r.source, r.target)).collect();
        assert_eq!(pairs, vec![(0, 1), (0, 2), (2, 2)], "1→2 and 2→1 are both zero");
        assert_eq!((c.ribbons[0].value, c.ribbons[0].back), (4.0, 1.0));
        assert_eq!(c.ribbons[1].colour, 2, "2 sends 3, more than the 2 it receives");
    }

    #[test]
    fn bad_values_and_empty_input_are_safe() {
        let c = layout(&[vec![f64::NAN, -1.0], vec![2.0]], 0.1);
        assert_eq!(c.arcs[0].out, 0.0);
        assert_eq!(c.ribbons.len(), 1);
        assert_eq!(layout(&[], 0.1), Chord::default());
        assert_eq!(layout(&[vec![0.0]], 0.1).ribbons, vec![]);
    }

    #[test]
    fn colours_never_cycle() {
        assert_eq!(colour(5, 6), "var(--chart-6)");
        assert_eq!(colour(5, 7), "var(--chart-muted)");
        assert_eq!(colour(4, 9), "var(--chart-5)");
    }

    #[test]
    fn tooltips_read_groups_and_ribbons() {
        let c = layout(&sample(), 0.0);
        let name = |i: usize| format!("G{i}");
        let fmt = |v: f64| format!("{v}");
        let g = tooltip(&c, Hit::Group(0), name, &fmt).unwrap();
        assert_eq!(g.title, "G0");
        assert_eq!((g.rows[0].name.as_str(), g.rows[0].value.as_str()), ("Sends", "6"));
        assert_eq!(g.rows[1].value, "4");
        assert!((0.0..=1.0).contains(&g.x) && (0.0..=1.0).contains(&g.y));
        let r = tooltip(&c, Hit::Ribbon(0), name, &fmt).unwrap();
        assert_eq!(r.title, "G0 and G1");
        assert_eq!(r.rows.len(), 2);
        assert_eq!(r.rows[0].name, "G0 → G1");
        let own = tooltip(&c, Hit::Ribbon(2), name, &fmt).unwrap();
        assert_eq!((own.title.as_str(), own.rows.len()), ("G2", 1));
        assert!(tooltip(&c, Hit::Group(9), name, &fmt).is_none());
        assert_eq!(swatch(&c, Hit::Ribbon(1)).as_deref(), Some("var(--chart-3)"));
        assert_eq!(swatch(&c, Hit::Group(9)), None);
    }
}
