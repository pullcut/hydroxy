//! Nested donut: concentric rings, innermost first. Each ring is either one level of a
//! tree (children sit under their parent's sweep) or a separate series over the same
//! categories. Segments are the same stroked centre-line arcs as [`crate::donut`], each
//! ring drawn in its own square box and offset so all rings share one centre.

#![allow(clippy::cast_precision_loss, reason = "item counts are small")]

use crate::Datum;
use crate::donut;
use crate::hierarchy::Tree;
use crate::plot::{Tip, TipRow};
use crate::scale::non_negative;

/// One part of a ring.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    /// Name.
    pub label: String,
    /// Size. Non-finite and negative values count as zero.
    pub value: f64,
    /// Palette slot. `None` keeps the space empty, so a leaf's missing children leave a gap
    /// in the next ring out instead of shifting everything after them.
    pub colour: Option<usize>,
    /// What the part belongs to, such as its parent's name, for the tooltip.
    pub group: Option<String>,
}

/// One ring.
#[derive(Debug, Clone, PartialEq)]
pub struct Ring {
    /// Name, such as a year or a tree level.
    pub name: String,
    /// The parts, clockwise from 12 o'clock.
    pub parts: Vec<Part>,
    /// Opacity of the ring's marks, 0..=1. Deeper tree levels fade so parents lead.
    pub opacity: f64,
}

/// Rings from a tree, one per level up to `max_depth`, innermost first. Every part is
/// coloured by the top-level branch it sits under.
#[must_use]
pub fn levels(root: &Tree, max_depth: usize) -> Vec<Ring> {
    fn walk(node: &Tree, depth: usize, colour: Option<usize>, parent: Option<&str>, rings: &mut [Ring]) {
        if depth >= rings.len() {
            return;
        }
        let kids: Vec<(usize, &Tree)> = node.children.iter().enumerate().filter(|(_, c)| c.total() > 0.0).collect();
        if kids.is_empty() {
            // A leaf below the top: hold its sweep open in every ring further out.
            if depth > 0 {
                for r in rings.iter_mut().skip(depth) {
                    r.parts.push(Part { label: String::new(), value: node.total(), colour: None, group: None });
                }
            }
            return;
        }
        for (i, child) in kids {
            let col = colour.unwrap_or(i);
            rings[depth].parts.push(Part {
                label: child.name.clone(),
                value: child.total(),
                colour: Some(col),
                group: parent.map(str::to_string),
            });
            walk(child, depth + 1, Some(col), Some(&child.name), rings);
        }
    }
    let mut rings: Vec<Ring> = (0..max_depth.max(1))
        .map(|d| Ring {
            name: format!("Level {}", d + 1),
            parts: Vec::new(),
            opacity: [1.0, 0.72, 0.5, 0.36].get(d).copied().unwrap_or(0.3),
        })
        .collect();
    walk(root, 0, None, None, &mut rings);
    rings.retain(|r| r.parts.iter().any(|p| p.colour.is_some()));
    rings
}

/// Rings from series over the same categories, innermost first: the category decides the
/// colour, so the same category matches across rings.
#[must_use]
pub fn series(series: &[(String, Vec<Datum>)]) -> Vec<Ring> {
    let mut cats: Vec<String> = Vec::new();
    for (_, data) in series {
        for d in data {
            if !cats.contains(&d.label) {
                cats.push(d.label.clone());
            }
        }
    }
    series
        .iter()
        .map(|(name, data)| Ring {
            name: name.clone(),
            parts: data
                .iter()
                .map(|d| Part {
                    label: d.label.clone(),
                    value: d.value,
                    colour: cats.iter().position(|c| *c == d.label),
                    group: None,
                })
                .collect(),
            opacity: 1.0,
        })
        .collect()
}

/// Legend names by palette slot: the first part found in each slot, innermost ring first.
#[must_use]
pub fn legend(rings: &[Ring]) -> Vec<String> {
    let mut out: Vec<Option<String>> = Vec::new();
    for p in rings.iter().flat_map(|r| &r.parts) {
        if let Some(c) = p.colour {
            if out.len() <= c {
                out.resize(c + 1, None);
            }
            out[c].get_or_insert_with(|| p.label.clone());
        }
    }
    out.into_iter().map(Option::unwrap_or_default).collect()
}

/// A drawn segment.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// Ring index, 0 innermost.
    pub ring: usize,
    /// Index into that ring's parts.
    pub part: usize,
    /// SVG `d` along the ring's centre line, in the ring's own box: draw it inside a group
    /// translated by the ring's [`RingLayout::offset`].
    pub path: String,
    /// Share of the ring's total, 0..=1.
    pub fraction: f64,
}

/// A placed ring.
#[derive(Debug, Clone, PartialEq)]
pub struct RingLayout {
    /// Side of the ring's own square box.
    pub size: f64,
    /// Translate the ring's box by this much on both axes to centre it.
    pub offset: f64,
    /// Stroke width: the ring's thickness.
    pub thickness: f64,
    /// Sum of the ring's part values.
    pub total: f64,
}

/// The laid-out chart.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    /// One per ring, innermost first.
    pub rings: Vec<RingLayout>,
    /// Every coloured part with something to draw, ring by ring.
    pub segments: Vec<Segment>,
    /// Number of palette slots in use.
    pub palette: usize,
    /// Radius of the empty centre.
    pub hole: f64,
    /// Side of the whole chart's square box.
    pub size: f64,
}

impl Layout {
    /// Tooltip for segment `s`.
    #[must_use]
    pub fn tip(&self, rings: &[Ring], s: usize, fmt: &dyn Fn(f64) -> String) -> Option<Tip> {
        let (x, y) = self.anchor(rings, s)?;
        let seg = &self.segments[s];
        let ring = &rings[seg.ring];
        let part = &ring.parts[seg.part];
        let mut rows = vec![
            TipRow::new(None, "Value", fmt(part.value)),
            TipRow::new(None, format!("Share of {}", ring.name), crate::upset::percent(seg.fraction, 1.0)),
        ];
        if let Some(g) = &part.group {
            rows.push(TipRow::new(None, "Part of", g.clone()));
        }
        Some(Tip { x, y: 1.0 - y, title: part.label.clone(), rows })
    }

    /// CSS colour for segment `s`.
    #[must_use]
    pub fn colour(&self, rings: &[Ring], s: usize) -> String {
        let slot =
            self.segments.get(s).and_then(|seg| rings.get(seg.ring)?.parts.get(seg.part)?.colour).unwrap_or(usize::MAX);
        crate::chord::colour(slot, self.palette)
    }

    /// Where segment `s` sits for a tooltip, as 0..=1 fractions of the box from the left
    /// and top.
    #[must_use]
    pub fn anchor(&self, rings: &[Ring], s: usize) -> Option<(f64, f64)> {
        let seg = self.segments.get(s)?;
        let ring = self.rings.get(seg.ring)?;
        let values: Vec<f64> = rings.get(seg.ring)?.parts.iter().map(|p| p.value).collect();
        let (fx, fy) = donut::anchor(&values, seg.part, ring.size, ring.thickness)?;
        Some(((ring.offset + fx * ring.size) / self.size, (ring.offset + fy * ring.size) / self.size))
    }
}

/// Lay out `rings` in a `size`×`size` box. `hole` is the empty centre's share of the
/// radius, `ring_gap` the px between rings and `gap` the px between segments.
#[must_use]
pub fn layout(rings: &[Ring], size: f64, hole: f64, ring_gap: f64, gap: f64) -> Layout {
    let n = rings.len();
    let outer = size / 2.0 - 1.0;
    let hole_r = outer * hole.clamp(0.0, 0.9);
    let thickness = if n == 0 { 0.0 } else { ((outer - hole_r - ring_gap * (n - 1) as f64) / n as f64).max(1.0) };
    let mut out = Layout { hole: hole_r, size, ..Layout::default() };
    for (k, ring) in rings.iter().enumerate() {
        let inner = (thickness + ring_gap).mul_add(k as f64, hole_r);
        let box_size = 2.0 * (inner + thickness);
        let values: Vec<f64> = ring.parts.iter().map(|p| non_negative(p.value)).collect();
        out.rings.push(RingLayout {
            size: box_size,
            offset: size / 2.0 - (inner + thickness),
            thickness,
            total: values.iter().sum(),
        });
        for (part, seg) in donut::segments(&values, box_size, thickness, gap).into_iter().enumerate() {
            let Some(colour) = ring.parts[part].colour else { continue };
            out.palette = out.palette.max(colour + 1);
            if !seg.path.is_empty() {
                out.segments.push(Segment { ring: k, part, path: seg.path, fraction: seg.fraction });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Tree {
        Tree::branch(
            "Spend",
            vec![
                Tree::branch("Engineering", vec![Tree::leaf("Platform", 30.0), Tree::leaf("Product", 50.0)]),
                Tree::leaf("Legal", 20.0),
                Tree::branch("Sales", vec![Tree::leaf("EMEA", 25.0), Tree::leaf("Americas", 0.0)]),
            ],
        )
    }

    #[test]
    fn levels_keep_children_under_their_parents() {
        let rings = levels(&tree(), 3);
        assert_eq!(rings.len(), 2, "no third level has anything to draw");
        let inner: Vec<&str> = rings[0].parts.iter().map(|p| p.label.as_str()).collect();
        assert_eq!(inner, ["Engineering", "Legal", "Sales"]);
        let outer: Vec<(Option<usize>, f64)> = rings[1].parts.iter().map(|p| (p.colour, p.value)).collect();
        // Legal has no children, so its sweep stays open in the outer ring.
        assert_eq!(outer, [(Some(0), 30.0), (Some(0), 50.0), (None, 20.0), (Some(2), 25.0)]);
        assert_eq!(rings[1].parts[0].group.as_deref(), Some("Engineering"));
        let l = layout(&rings, 300.0, 0.4, 3.0, 2.0);
        assert_eq!(l.segments.len(), 6);
        assert_eq!(l.palette, 3);
        assert_eq!(legend(&rings), ["Engineering", "Legal", "Sales"]);
    }

    #[test]
    fn rings_share_a_centre_and_stack_outwards() {
        let rings = series(&[
            ("2024".into(), vec![Datum::new("Web", 3.0), Datum::new("App", 1.0)]),
            ("2025".into(), vec![Datum::new("App", 2.0), Datum::new("Web", 2.0)]),
        ]);
        assert_eq!(rings[1].parts[0].colour, Some(1), "colours follow the category, not the position");
        let l = layout(&rings, 200.0, 0.4, 4.0, 2.0);
        for r in &l.rings {
            assert!((r.offset + r.size / 2.0 - 100.0).abs() < 1e-9);
        }
        assert!(l.rings[1].size > l.rings[0].size);
        assert!((l.rings[1].size / 2.0 - 99.0).abs() < 1e-9, "the outer ring touches the edge");
        assert!((l.rings[0].size / 2.0 - l.rings[0].thickness - l.hole).abs() < 1e-9);
        let (x, y) = l.anchor(&rings, 0).expect("anchor");
        assert!(x > 0.5 && (0.0..1.0).contains(&y));
        assert_eq!(l.anchor(&rings, 9), None);
        let tip = l.tip(&rings, 1, &|v| format!("{v}")).expect("tip");
        assert_eq!(
            (tip.title.as_str(), tip.rows[1].name.as_str(), tip.rows[1].value.as_str()),
            ("App", "Share of 2024", "25%")
        );
        assert_eq!(l.colour(&rings, 1), "var(--chart-2)");
    }

    #[test]
    fn nothing_in_nothing_out() {
        assert_eq!(layout(&[], 100.0, 0.4, 2.0, 2.0).segments.len(), 0);
        assert_eq!(levels(&Tree::branch("r", vec![]), 2).len(), 0);
    }
}
