//! Pie slices. A slice is a donut segment whose ring is as thick as the radius, so
//! it reuses `donut::segments` and sweeps in from its own start angle the same way.
//! Angles start at 12 o'clock and run clockwise.

use crate::donut;
use std::f64::consts::TAU;

/// Where slice labels go.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Labels {
    /// The share sits inside each slice big enough to hold it.
    #[default]
    Inside,
    /// Every slice gets a leader line out to its name and share.
    Leader,
    /// No labels; the legend and tooltip carry the names.
    None,
}

/// A leader line from a slice's edge out to its label, in box px.
#[derive(Debug, Clone, PartialEq)]
pub struct Leader {
    /// SVG `d` for the elbowed line.
    pub path: String,
    /// Label anchor x.
    pub x: f64,
    /// Label baseline centre y.
    pub y: f64,
    /// True when the label sits left of the pie and reads towards it (`text-anchor: end`).
    pub end: bool,
}

/// One slice.
#[derive(Debug, Clone, PartialEq)]
pub struct Slice {
    /// SVG `d` along the slice's centre line, in pie px. Stroke it `stroke_width` wide.
    pub path: String,
    /// Share of the total, 0..=1.
    pub fraction: f64,
    /// Angle through the middle of the slice, radians clockwise from 12 o'clock.
    pub mid: f64,
    /// A point well inside the slice, as fractions of the box from its top-left.
    pub centroid: (f64, f64),
    /// Unit vector out through the middle of the slice, for pulling it out on hover.
    pub out: (f64, f64),
    /// The leader line, with [`Labels::Leader`].
    pub leader: Option<Leader>,
}

/// A laid-out pie inside a box that leaves room for leader labels.
#[derive(Debug, Clone, PartialEq)]
pub struct PieLayout {
    /// Box width in px.
    pub width: f64,
    /// Box height in px.
    pub height: f64,
    /// Offset of the pie's own `size`×`size` square inside the box.
    pub pad: (f64, f64),
    /// Stroke width for every slice path: the radius.
    pub stroke_width: f64,
    /// One per value.
    pub slices: Vec<Slice>,
}

/// Smallest share that gets an inside label.
pub const INSIDE_MIN: f64 = 0.06;

/// Lay out `values` as a pie `size` px across, with a `gap` px space between slices.
#[must_use]
pub fn layout(values: &[f64], size: f64, gap: f64, labels: Labels) -> PieLayout {
    let segs = donut::segments(values, size, size / 2.0, gap);
    let pad = if labels == Labels::Leader { (110.0, 18.0) } else { (0.0, 0.0) };
    let (width, height) = (size + 2.0 * pad.0, size + 2.0 * pad.1);
    let r = size / 2.0;
    let (cx, cy) = (pad.0 + r, pad.1 + r);
    let mut start = 0.0;
    let mut slices: Vec<Slice> = segs
        .into_iter()
        .map(|s| {
            let mid = start + s.fraction * TAU / 2.0;
            start += s.fraction * TAU;
            let out = (mid.sin(), -mid.cos());
            let at = |d: f64| (cx + d * out.0, cy + d * out.1);
            let (gx, gy) = at(r * 0.62);
            let leader = (labels == Labels::Leader).then(|| {
                let end = out.0 < 0.0;
                let (_, y) = at(r + 10.0);
                Leader { path: String::new(), x: if end { cx - r - 18.0 } else { cx + r + 18.0 }, y, end }
            });
            Slice { path: s.path, fraction: s.fraction, mid, centroid: (gx / width, gy / height), out, leader }
        })
        .collect();
    // Push crowded labels apart, one side at a time, then draw the elbows.
    for left in [false, true] {
        let mut ids: Vec<usize> = (0..slices.len())
            .filter(|&i| slices[i].leader.as_ref().is_some_and(|l| l.end == left) && slices[i].fraction > 0.0)
            .collect();
        ids.sort_by(|&a, &b| leader_y(&slices[a]).total_cmp(&leader_y(&slices[b])));
        let mut floor = f64::NEG_INFINITY;
        for i in ids {
            if let Some(l) = slices[i].leader.as_mut() {
                l.y = l.y.max(floor + 14.0);
                floor = l.y;
            }
        }
    }
    for s in &mut slices {
        let (ox, oy) = s.out;
        if let Some(l) = s.leader.as_mut() {
            let (x0, y0) = (cx + r * 0.92 * ox, cy + r * 0.92 * oy);
            let (x1, y1) = (cx + (r + 10.0) * ox, cy + (r + 10.0) * oy);
            let x2 = if l.end { l.x + 4.0 } else { l.x - 4.0 };
            l.path = format!("M{x0:.2} {y0:.2}L{x1:.2} {y1:.2}L{x2:.2} {:.2}", l.y);
        }
    }
    PieLayout { width, height, pad, stroke_width: r, slices }
}

fn leader_y(s: &Slice) -> f64 {
    s.leader.as_ref().map_or(0.0, |l| l.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slices_cover_the_circle_and_reuse_the_donut_arcs() {
        let p = layout(&[1.0, 1.0, 2.0], 200.0, 0.0, Labels::Inside);
        let sum: f64 = p.slices.iter().map(|s| s.fraction).sum();
        assert!((sum - 1.0).abs() < 1e-9);
        assert_eq!((p.width, p.height, p.stroke_width), (200.0, 200.0, 100.0));
        // The first quarter's middle points at 1:30, so it pulls up and right.
        assert!(p.slices[0].out.0 > 0.0 && p.slices[0].out.1 < 0.0);
        assert!(p.slices[2].path.starts_with('M'));
    }

    #[test]
    fn centroid_sits_inside_the_slice() {
        let p = layout(&[1.0, 1.0], 100.0, 0.0, Labels::Inside);
        // Right half: centroid right of centre at mid-height.
        let (x, y) = p.slices[0].centroid;
        assert!((x - 0.81).abs() < 1e-9 && (y - 0.5).abs() < 1e-9);
    }

    #[test]
    fn leader_labels_do_not_overlap() {
        let p = layout(&[30.0, 1.0, 1.0, 1.0, 1.0], 200.0, 0.0, Labels::Leader);
        assert!(p.width > 200.0);
        let mut ys: Vec<f64> =
            p.slices.iter().filter_map(|s| s.leader.as_ref()).filter(|l| l.end).map(|l| l.y).collect();
        ys.sort_by(f64::total_cmp);
        assert!(ys.windows(2).all(|w| w[1] - w[0] >= 14.0 - 1e-9));
        assert!(p.slices.iter().all(|s| s.leader.as_ref().is_some_and(|l| l.path.matches('L').count() == 2)));
    }
}
