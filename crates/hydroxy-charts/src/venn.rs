//! Area-proportional Venn layout for two or three sets. Each circle's area is its set
//! size; centre distances are solved numerically so each pairwise overlap has the area
//! of the members the two sets share. Three sets are placed from their pairwise
//! distances and then nudged to the best fit when no triangle matches them exactly.
//! The triple overlap follows from the geometry and is only approximate.
//!
//! Every exclusive region (members in exactly these sets) gets an outline path built from
//! circle arcs, for hovering, and an interior point for its count label.

#![allow(clippy::cast_precision_loss, reason = "grid sizes are small")]

use std::f64::consts::{PI, TAU};
use std::fmt::Write as _;

use crate::plot::{Tip, TipRow};
pub use crate::upset::Intersection;

/// One set's circle.
#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    /// Set name.
    pub name: String,
    /// Members in the set.
    pub size: f64,
    /// Centre x.
    pub cx: f64,
    /// Centre y.
    pub cy: f64,
    /// Radius.
    pub r: f64,
    /// Where the set's name goes, just outside the circle, as (x, y, text-anchor).
    pub label: (f64, f64, &'static str),
}

/// One exclusive region.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    /// Indices into [`Layout::circles`] of the sets the members are in, ascending.
    pub sets: Vec<usize>,
    /// Members in exactly these sets.
    pub count: f64,
    /// SVG `d` of the region's outline (fill with `evenodd`); empty when the circles leave
    /// it no room.
    pub path: String,
    /// The most interior point of the region, for its count label.
    pub label: Option<(f64, f64)>,
}

/// The laid-out diagram.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    /// One circle per set, in order of first appearance.
    pub circles: Vec<Circle>,
    /// Every combination of the sets, singles first, whether or not it has members.
    pub regions: Vec<Region>,
    /// Box width.
    pub width: f64,
    /// Box height.
    pub height: f64,
}

impl Layout {
    /// Where to pin a tooltip for region `i`, in px.
    #[must_use]
    pub fn anchor(&self, i: usize) -> Option<(f64, f64)> {
        self.regions.get(i)?.label
    }

    /// Every member of the drawn sets, counted once.
    #[must_use]
    pub fn total(&self) -> f64 {
        self.regions.iter().map(|r| r.count).sum()
    }

    /// The name of region `i`, such as "Email only" or "Email ∩ Push".
    #[must_use]
    pub fn region_name(&self, i: usize) -> String {
        let names: Vec<&str> = self
            .regions
            .get(i)
            .map(|r| r.sets.iter().map(|&s| self.circles[s].name.as_str()).collect())
            .unwrap_or_default();
        crate::upset::combo_name(&names)
    }

    /// Tooltip for region `i`.
    #[must_use]
    pub fn tip(&self, i: usize, fmt: &dyn Fn(f64) -> String) -> Option<Tip> {
        let (x, y) = self.anchor(i)?;
        let r = &self.regions[i];
        let mut rows = vec![
            TipRow::new(None, "Members", fmt(r.count)),
            TipRow::new(None, "Share of all", crate::upset::percent(r.count, self.total())),
        ];
        if let [one] = r.sets[..] {
            rows.push(TipRow::new(None, "Set size", fmt(self.circles[one].size)));
        }
        Some(Tip { x: x / self.width.max(1.0), y: 1.0 - y / self.height.max(1.0), title: self.region_name(i), rows })
    }
}

/// Area of the lens where circles of radius `r1` and `r2` with centres `d` apart overlap.
#[must_use]
pub fn lens_area(r1: f64, r2: f64, d: f64) -> f64 {
    if d >= r1 + r2 {
        return 0.0;
    }
    if d <= (r1 - r2).abs() {
        return PI * r1.min(r2).powi(2);
    }
    let a1 = ((d * d + r1 * r1 - r2 * r2) / (2.0 * d * r1)).clamp(-1.0, 1.0).acos();
    let a2 = ((d * d + r2 * r2 - r1 * r1) / (2.0 * d * r2)).clamp(-1.0, 1.0).acos();
    let k = ((-d + r1 + r2) * (d + r1 - r2) * (d - r1 + r2) * (d + r1 + r2)).max(0.0).sqrt();
    r1 * r1 * a1 + r2 * r2 * a2 - 0.5 * k
}

/// The centre distance at which the circles overlap by `overlap`, by bisection. No
/// overlap leaves a small gap; an overlap as big as the smaller circle nests it.
#[must_use]
pub fn distance_for(r1: f64, r2: f64, overlap: f64) -> f64 {
    if overlap <= 0.0 {
        return r1 + r2 + 0.08 * r1.min(r2);
    }
    if overlap >= PI * r1.min(r2).powi(2) {
        return (r1 - r2).abs();
    }
    let (mut lo, mut hi) = ((r1 - r2).abs(), r1 + r2);
    for _ in 0..80 {
        let mid = f64::midpoint(lo, hi);
        if lens_area(r1, r2, mid) > overlap {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    f64::midpoint(lo, hi)
}

/// Place three circles so their centre distances come as close as possible to `d`
/// (`d[0]` A–B, `d[1]` A–C, `d[2]` B–C): a triangle first, then gradient steps on the
/// squared distance error for when the three distances can't form one.
fn place_three(d: [f64; 3]) -> [(f64, f64); 3] {
    let (ab, ac, bc) = (d[0], d[1], d[2]);
    let cos =
        if ab > 0.0 && ac > 0.0 { ((ab * ab + ac * ac - bc * bc) / (2.0 * ab * ac)).clamp(-1.0, 1.0) } else { 0.0 };
    let mut p = [(0.0, 0.0), (ab, 0.0), (ac * cos, ac * (1.0 - cos * cos).sqrt())];
    let pairs = [(0, 1, ab), (0, 2, ac), (1, 2, bc)];
    let scale = ab.max(ac).max(bc).max(f64::MIN_POSITIVE);
    for _ in 0..400 {
        let mut grad = [(0.0, 0.0); 3];
        for &(i, j, want) in &pairs {
            let (dx, dy) = (p[j].0 - p[i].0, p[j].1 - p[i].1);
            let have = dx.hypot(dy).max(1e-9);
            let k = 2.0 * (have - want) / have;
            grad[i].0 -= k * dx;
            grad[i].1 -= k * dy;
            grad[j].0 += k * dx;
            grad[j].1 += k * dy;
        }
        let step = 0.05;
        let mut moved = 0.0f64;
        for (pt, g) in p.iter_mut().zip(grad) {
            pt.0 -= step * g.0;
            pt.1 -= step * g.1;
            moved = moved.max(g.0.hypot(g.1) * step);
        }
        if moved < scale * 1e-9 {
            break;
        }
    }
    p
}

/// Points where two circles cross, as angles on circle `a` (none when apart, nested or tangent).
fn crossings(a: (f64, f64, f64), b: (f64, f64, f64)) -> Option<[(f64, f64); 2]> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let d = dx.hypot(dy);
    if d >= a.2 + b.2 - 1e-9 || d <= (a.2 - b.2).abs() + 1e-9 || d == 0.0 {
        return None;
    }
    let along = (d * d + a.2 * a.2 - b.2 * b.2) / (2.0 * d);
    let h = (a.2 * a.2 - along * along).max(0.0).sqrt();
    let (mx, my) = (a.0 + along * dx / d, a.1 + along * dy / d);
    Some([(mx + h * dy / d, my - h * dx / d), (mx - h * dy / d, my + h * dx / d)])
}

fn inside(p: (f64, f64), c: (f64, f64, f64)) -> bool {
    (p.0 - c.0).hypot(p.1 - c.1) < c.2
}

fn on(c: (f64, f64, f64), angle: f64) -> (f64, f64) {
    (c.0 + c.2 * angle.cos(), c.1 + c.2 * angle.sin())
}

/// A directed arc of circle `c` from angle `a0`, sweeping `span` radians (negative runs backwards).
struct Arc {
    c: (f64, f64, f64),
    a0: f64,
    span: f64,
}

impl Arc {
    fn start(&self) -> (f64, f64) {
        on(self.c, self.a0)
    }
    fn end(&self) -> (f64, f64) {
        on(self.c, self.a0 + self.span)
    }
}

/// The outline of the region inside exactly the circles in `mask`, from the circle arcs
/// that bound it, chained into closed loops.
fn region_path(circles: &[(f64, f64, f64)], mask: &[bool]) -> String {
    let mut arcs: Vec<Arc> = Vec::new();
    for (i, &c) in circles.iter().enumerate() {
        let mut angles: Vec<f64> = circles
            .iter()
            .enumerate()
            .filter(|&(j, _)| j != i)
            .filter_map(|(_, &o)| crossings(c, o))
            .flatten()
            .map(|p| (p.1 - c.1).atan2(p.0 - c.0).rem_euclid(TAU))
            .collect();
        angles.sort_by(f64::total_cmp);
        angles.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        let spans: Vec<(f64, f64)> = if angles.is_empty() {
            vec![(0.0, TAU)]
        } else {
            (0..angles.len())
                .map(|k| {
                    let a0 = angles[k];
                    let a1 = if k + 1 < angles.len() { angles[k + 1] } else { angles[0] + TAU };
                    (a0, a1 - a0)
                })
                .collect()
        };
        for (a0, span) in spans {
            let mid = on(c, a0 + span / 2.0);
            let fits = circles.iter().enumerate().filter(|&(j, _)| j != i).all(|(j, &o)| inside(mid, o) == mask[j]);
            if fits {
                // Keep the region on the same side: forwards round circles it is in, backwards round the rest.
                arcs.push(if mask[i] { Arc { c, a0, span } } else { Arc { c, a0: a0 + span, span: -span } });
            }
        }
    }
    let mut d = String::new();
    let mut used = vec![false; arcs.len()];
    let close = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).hypot(a.1 - b.1);
    while let Some(first) = used.iter().position(|u| !u) {
        used[first] = true;
        let start = arcs[first].start();
        let _ = write!(d, "M{:.2} {:.2}", start.0, start.1);
        let mut cur = first;
        loop {
            let a = &arcs[cur];
            let r = a.c.2;
            if a.span.abs() >= TAU - 1e-9 {
                // A whole circle: two half arcs, since one arc can't close on itself.
                let half = on(a.c, a.a0 + PI);
                let _ = write!(
                    d,
                    "A{r:.2} {r:.2} 0 0 1 {:.2} {:.2}A{r:.2} {r:.2} 0 0 1 {:.2} {:.2}",
                    half.0, half.1, start.0, start.1
                );
                break;
            }
            let end = a.end();
            let large = i32::from(a.span.abs() > PI);
            let sweep = i32::from(a.span > 0.0);
            let _ = write!(d, "A{r:.2} {r:.2} 0 {large} {sweep} {:.2} {:.2}", end.0, end.1);
            let tol = 1e-6 * r.max(1.0);
            if close(end, start) < tol {
                break;
            }
            let next = (0..arcs.len())
                .filter(|&k| !used[k])
                .min_by(|&x, &y| close(arcs[x].start(), end).total_cmp(&close(arcs[y].start(), end)));
            match next {
                Some(k) if close(arcs[k].start(), end) < tol * 1e3 => {
                    used[k] = true;
                    cur = k;
                }
                _ => break,
            }
        }
        d.push('Z');
    }
    d
}

/// Lay out `data` in a `width`×`height` box. Only the three largest sets are drawn; with
/// one set there is a single circle.
#[must_use]
pub fn layout(data: &[Intersection], width: f64, height: f64) -> Layout {
    let (all_names, combos) = crate::upset::index(data);
    let mut size_all = vec![0.0; all_names.len()];
    for (sets, count) in &combos {
        for &s in sets {
            size_all[s] += count;
        }
    }
    let mut keep: Vec<usize> = (0..all_names.len()).collect();
    keep.sort_by(|&a, &b| size_all[b].total_cmp(&size_all[a]).then(a.cmp(&b)));
    keep.truncate(3);
    keep.sort_unstable();
    let n = keep.len();
    if n == 0 {
        return Layout { width, height, ..Layout::default() };
    }
    let names: Vec<String> = keep.iter().map(|&k| all_names[k].clone()).collect();
    // Exclusive counts over the kept sets; members also in a dropped set count where they fall.
    let mut exclusive = vec![0.0; 1 << n];
    for (sets, count) in &combos {
        let mask = keep.iter().enumerate().filter(|(_, k)| sets.contains(k)).fold(0usize, |m, (i, _)| m | (1 << i));
        if mask != 0 {
            exclusive[mask] += count;
        }
    }
    let size: Vec<f64> =
        (0..n).map(|i| (1..1usize << n).filter(|m| m & (1 << i) != 0).map(|m| exclusive[m]).sum()).collect();
    let shared = |i: usize, j: usize| -> f64 {
        (1..1usize << n).filter(|m| m & (1 << i) != 0 && m & (1 << j) != 0).map(|m| exclusive[m]).sum()
    };
    let r: Vec<f64> = size.iter().map(|s| (s / PI).sqrt()).collect();
    let centres: Vec<(f64, f64)> = match n {
        1 => vec![(0.0, 0.0)],
        2 => vec![(0.0, 0.0), (distance_for(r[0], r[1], shared(0, 1)), 0.0)],
        _ => place_three([
            distance_for(r[0], r[1], shared(0, 1)),
            distance_for(r[0], r[2], shared(0, 2)),
            distance_for(r[1], r[2], shared(1, 2)),
        ])
        .to_vec(),
    };

    // Fit the circles into the box, leaving room for the set names around them.
    let (pad_x, pad_y) = (width * 0.12 + 8.0, 30.0);
    let (x0, x1, y0, y1) = (0..n).fold((f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY), |b, i| {
        (
            b.0.min(centres[i].0 - r[i]),
            b.1.max(centres[i].0 + r[i]),
            b.2.min(centres[i].1 - r[i]),
            b.3.max(centres[i].1 + r[i]),
        )
    });
    let k = ((width - 2.0 * pad_x) / (x1 - x0).max(f64::MIN_POSITIVE))
        .min((height - 2.0 * pad_y) / (y1 - y0).max(f64::MIN_POSITIVE))
        .max(0.0);
    let (ox, oy) = (width / 2.0 - k * f64::midpoint(x0, x1), height / 2.0 - k * f64::midpoint(y0, y1));
    let geo: Vec<(f64, f64, f64)> = (0..n).map(|i| (ox + k * centres[i].0, oy + k * centres[i].1, k * r[i])).collect();
    let mid = (geo.iter().map(|c| c.0).sum::<f64>() / n as f64, geo.iter().map(|c| c.1).sum::<f64>() / n as f64);
    let circles = (0..n)
        .map(|i| {
            let c = geo[i];
            let (dx, dy) = (c.0 - mid.0, c.1 - mid.1);
            let len = dx.hypot(dy);
            // Name outside the circle, pointing away from the others; above when there's no direction.
            let (ux, uy) = if len > 1e-6 { (dx / len, dy / len) } else { (0.0, -1.0) };
            let (lx, ly) = (c.0 + ux * (c.2 + 12.0), c.1 + uy * (c.2 + 12.0));
            let anchor = if ux > 0.35 {
                "start"
            } else if ux < -0.35 {
                "end"
            } else {
                "middle"
            };
            Circle {
                name: names[i].clone(),
                size: size[i],
                cx: c.0,
                cy: c.1,
                r: c.2,
                label: (lx, ly + uy.max(0.0) * 8.0, anchor),
            }
        })
        .collect();

    // Most interior grid point of each region: the one furthest from every circle edge.
    let steps = 90u32;
    let mut best: Vec<Option<((f64, f64), f64)>> = vec![None; 1 << n];
    for gy in 0..=steps {
        for gx in 0..=steps {
            let p = (width * f64::from(gx) / f64::from(steps), height * f64::from(gy) / f64::from(steps));
            let mask = (0..n).filter(|&i| inside(p, geo[i])).fold(0usize, |m, i| m | (1 << i));
            if mask == 0 {
                continue;
            }
            let clear = geo.iter().map(|c| (c.2 - (p.0 - c.0).hypot(p.1 - c.1)).abs()).fold(f64::INFINITY, f64::min);
            if best[mask].is_none_or(|(_, b)| clear > b) {
                best[mask] = Some((p, clear));
            }
        }
    }
    let mut masks: Vec<usize> = (1..1usize << n).collect();
    masks.sort_by_key(|m| (m.count_ones(), *m));
    let regions = masks
        .into_iter()
        .map(|m| {
            let flags: Vec<bool> = (0..n).map(|i| m & (1 << i) != 0).collect();
            let label = best[m].map(|(p, _)| p);
            Region {
                sets: (0..n).filter(|&i| flags[i]).collect(),
                count: exclusive[m],
                path: if label.is_some() { region_path(&geo, &flags) } else { String::new() },
                label,
            }
        })
        .collect();
    Layout { circles, regions, width, height }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lens_area_limits_and_solver_round_trip() {
        assert_eq!(lens_area(1.0, 1.0, 2.5), 0.0);
        assert!((lens_area(2.0, 1.0, 0.5) - PI).abs() < 1e-12);
        let d = distance_for(3.0, 2.0, 5.0);
        assert!((lens_area(3.0, 2.0, d) - 5.0).abs() < 1e-6);
    }

    fn area_of(l: &Layout, i: usize) -> f64 {
        PI * l.circles[i].r.powi(2)
    }

    #[test]
    fn two_sets_have_proportional_areas_and_overlap() {
        let l = layout(
            &[Intersection::new(["A"], 300.0), Intersection::new(["B"], 100.0), Intersection::new(["A", "B"], 100.0)],
            400.0,
            300.0,
        );
        assert_eq!(l.circles.len(), 2);
        let (a, b) = (area_of(&l, 0), area_of(&l, 1));
        assert!((a / b - 2.0).abs() < 1e-9, "sizes 400 and 200");
        let d = (l.circles[0].cx - l.circles[1].cx).hypot(l.circles[0].cy - l.circles[1].cy);
        let lens = lens_area(l.circles[0].r, l.circles[1].r, d);
        assert!((lens / a - 0.25).abs() < 1e-6, "the shared 100 is a quarter of A");
        assert_eq!(l.regions.len(), 3);
        assert_eq!(l.regions[2].sets, vec![0, 1]);
        assert!(l.regions.iter().all(|r| r.label.is_some() && r.path.starts_with('M') && r.path.ends_with('Z')));
        let tip = l.tip(2, &|v| format!("{v}")).expect("tip");
        assert_eq!(
            (tip.title.as_str(), tip.rows[0].value.as_str(), tip.rows[1].value.as_str()),
            ("A ∩ B", "100", "20%")
        );
        assert_eq!(l.region_name(0), "A only");
    }

    #[test]
    fn three_sets_fit_their_pairwise_overlaps() {
        let data = [
            Intersection::new(["A"], 200.0),
            Intersection::new(["B"], 150.0),
            Intersection::new(["C"], 120.0),
            Intersection::new(["A", "B"], 60.0),
            Intersection::new(["A", "C"], 40.0),
            Intersection::new(["B", "C"], 30.0),
            Intersection::new(["A", "B", "C"], 20.0),
        ];
        let l = layout(&data, 500.0, 400.0);
        let unit = area_of(&l, 0) / 320.0;
        for (i, j, want) in [(0, 1, 80.0), (0, 2, 60.0), (1, 2, 50.0)] {
            let (a, b) = (&l.circles[i], &l.circles[j]);
            let lens = lens_area(a.r, b.r, (a.cx - b.cx).hypot(a.cy - b.cy));
            assert!((lens / unit / want - 1.0).abs() < 0.01, "{i}{j}: {}", lens / unit);
        }
        assert_eq!(l.regions.len(), 7);
        assert_eq!(l.regions[6].count, 20.0);
        for c in &l.circles {
            assert!(c.cx - c.r >= 0.0 && c.cx + c.r <= 500.0 && c.cy - c.r >= 0.0 && c.cy + c.r <= 400.0);
        }
    }

    #[test]
    fn impossible_distances_still_settle_near_a_best_fit() {
        // 1 + 1 < 5: no triangle, so the points spread along a line as best they can.
        let p = place_three([1.0, 1.0, 5.0]);
        let d = |i: usize, j: usize| (p[i].0 - p[j].0).hypot(p[i].1 - p[j].1);
        assert!(d(1, 2) > 2.0 && d(1, 2) < 5.0);
    }

    #[test]
    fn region_labels_sit_inside_their_regions() {
        let l = layout(
            &[Intersection::new(["A"], 100.0), Intersection::new(["B"], 100.0), Intersection::new(["A", "B"], 50.0)],
            400.0,
            300.0,
        );
        for reg in &l.regions {
            let p = reg.label.expect("label");
            for (i, c) in l.circles.iter().enumerate() {
                assert_eq!((p.0 - c.cx).hypot(p.1 - c.cy) < c.r, reg.sets.contains(&i));
            }
        }
    }

    #[test]
    fn disjoint_and_nested_sets_draw_whole_circles() {
        let apart = layout(&[Intersection::new(["A"], 10.0), Intersection::new(["B"], 10.0)], 300.0, 200.0);
        assert_eq!(apart.regions[2].label, None);
        assert_eq!(apart.regions[2].path, "");
        assert_eq!(apart.regions[0].path.matches('A').count(), 2);
        let nested = layout(&[Intersection::new(["A"], 30.0), Intersection::new(["A", "B"], 10.0)], 300.0, 200.0);
        assert_eq!(nested.regions[1].count, 0.0);
        // A only is a ring: two loops.
        assert_eq!(nested.regions[0].path.matches('M').count(), 2);
    }

    #[test]
    fn extra_sets_are_left_out_and_empty_input_is_empty() {
        let l = layout(
            &[
                Intersection::new(["A"], 5.0),
                Intersection::new(["B"], 4.0),
                Intersection::new(["C"], 3.0),
                Intersection::new(["D"], 1.0),
            ],
            300.0,
            200.0,
        );
        assert_eq!(l.circles.len(), 3);
        assert_eq!(layout(&[], 100.0, 100.0).circles.len(), 0);
    }
}
