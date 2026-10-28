//! Scatter and bubble layout. Positions are fractions of the plot area (0..=1, y up),
//! so adapters place dots with plain CSS percentages.

use crate::bar::Gridline;
use crate::scale::{ScaleKind, ValueScale};
use crate::zoom::{self, View};

/// One observation. `size` makes it a bubble.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// Horizontal value.
    pub x: f64,
    /// Vertical value.
    pub y: f64,
    /// Optional third measure, drawn as the dot's area.
    pub size: Option<f64>,
}

impl Point {
    /// A plain dot.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y, size: None }
    }

    /// A bubble whose area follows `size`.
    #[must_use]
    pub const fn sized(x: f64, y: f64, size: f64) -> Self {
        Self { x, y, size: Some(size) }
    }
}

/// A named group of points, drawn in one colour.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    /// Shown in the legend and tooltip.
    pub name: String,
    /// The points.
    pub points: Vec<Point>,
}

impl Group {
    /// A named group.
    #[must_use]
    pub fn new(name: impl Into<String>, points: Vec<Point>) -> Self {
        Self { name: name.into(), points }
    }
}

/// A dot ready to place.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dot {
    /// Index of its group.
    pub group: usize,
    /// Index within the group.
    pub index: usize,
    /// Distance from the left, 0..=1.
    pub x: f64,
    /// Distance from the bottom, 0..=1.
    pub y: f64,
    /// Relative diameter, 0..=1, from the square root of `size` so area tracks value.
    /// `None` for points without a size.
    pub size: Option<f64>,
}

/// Everything a scatter chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct ScatterLayout {
    /// Every finite point, group by group.
    pub dots: Vec<Dot>,
    /// Vertical positions of the horizontal gridlines.
    pub y_ticks: Vec<Gridline>,
    /// Horizontal positions of the vertical gridlines.
    pub x_ticks: Vec<Gridline>,
    /// The full x axis; visible positions are `view.project_x(x_scale.map(v))`.
    pub x_scale: ValueScale,
    /// The full y axis.
    pub y_scale: ValueScale,
    /// The visible window.
    pub view: View,
}

/// Lay out `groups` on nice linear axes. Points with non-finite coordinates are dropped.
#[must_use]
pub fn layout(groups: &[Group], tick_count: usize) -> ScatterLayout {
    layout_scaled(groups, tick_count, (ScaleKind::Linear, ScaleKind::Linear), View::FULL, None)
}

/// [`layout`] with a scale per axis (`(x, y)`), a zoomed `view`, and the y axis widened to
/// cover `extent` (such as [`crate::annotate::y_extent`]). Only dots inside the view are kept.
#[must_use]
pub fn layout_scaled(
    groups: &[Group],
    tick_count: usize,
    kinds: (ScaleKind, ScaleKind),
    view: View,
    extent: Option<(f64, f64)>,
) -> ScatterLayout {
    let finite: Vec<(usize, usize, Point)> = groups
        .iter()
        .enumerate()
        .flat_map(|(g, grp)| grp.points.iter().enumerate().map(move |(i, p)| (g, i, *p)))
        .filter(|(_, _, p)| p.x.is_finite() && p.y.is_finite())
        .collect();
    let span = |f: &dyn Fn(&Point) -> f64, kind: ScaleKind, extra: Option<(f64, f64)>| {
        let values = finite.iter().map(|t| f(&t.2)).chain(extra.into_iter().flat_map(|(a, b)| [a, b]));
        let values: Vec<f64> = values.filter(|v| kind != ScaleKind::Log || *v > 0.0).collect();
        let lo = values.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let (lo, hi) = if lo.is_finite() {
            (lo, hi)
        } else if kind == ScaleKind::Log {
            (1.0, 10.0)
        } else {
            (0.0, 1.0)
        };
        ValueScale::new(kind, lo, if hi > lo { hi } else { lo + 1.0 }, tick_count)
    };
    let sx = span(&|p| p.x, kinds.0, None);
    let sy = span(&|p| p.y, kinds.1, extent);
    let max_size = finite.iter().filter_map(|t| t.2.size).filter(|s| s.is_finite()).fold(0.0, f64::max);
    let full = ScatterLayout {
        dots: finite
            .iter()
            .map(|&(group, index, p)| Dot {
                group,
                index,
                x: sx.map(p.x).clamp(0.0, 1.0),
                y: sy.map(p.y).clamp(0.0, 1.0),
                size: p.size.filter(|_| max_size > 0.0).map(|s| (s.max(0.0) / max_size).sqrt()),
            })
            .collect(),
        x_ticks: Vec::new(),
        y_ticks: Vec::new(),
        x_scale: sx,
        y_scale: sy,
        view: View::FULL,
    };
    reframe(&full, view, tick_count)
}

/// Show `full` (laid out at [`View::FULL`]) through `view`: dots re-projected and those
/// outside dropped, gridlines re-ticked. Much cheaper than a fresh layout, so zooming a
/// big cloud lays it out once and reframes it per step.
#[must_use]
pub fn reframe(full: &ScatterLayout, view: View, tick_count: usize) -> ScatterLayout {
    let inside = |f: f64| (-1e-9..=1.0 + 1e-9).contains(&f);
    let dots = full
        .dots
        .iter()
        .map(|d| Dot { x: view.project_x(d.x), y: view.project_y(d.y), ..*d })
        .filter(|d| inside(d.x) && inside(d.y))
        .collect();
    let (sx, sy) = (full.x_scale, full.y_scale);
    ScatterLayout {
        dots,
        x_ticks: visible_ticks(sx, view.x, tick_count, |f| view.project_x(f)),
        y_ticks: visible_ticks(sy, view.y, tick_count, |f| view.project_y(f)),
        x_scale: sx,
        y_scale: sy,
        view,
    }
}

/// Gridlines for the visible window `(a, b)` (full-extent fractions) of `scale`.
fn visible_ticks(scale: ValueScale, (a, b): (f64, f64), count: usize, project: impl Fn(f64) -> f64) -> Vec<Gridline> {
    let values = if a <= 1e-9 && b >= 1.0 - 1e-9 {
        scale.ticks(count)
    } else {
        let (lo, hi) = (zoom::invert(scale, a), zoom::invert(scale, b));
        ValueScale::new(scale.kind, lo, hi, count).ticks(count)
    };
    values
        .into_iter()
        .map(|value| Gridline { value, fraction: project(scale.map(value)) })
        .filter(|g| (-1e-6..=1.0 + 1e-6).contains(&g.fraction))
        .collect()
}

/// The dot nearest to `(x, y)` (plot fractions), if one lies within `radius`.
/// Distance is measured in fraction space, so a wide plot reaches slightly further sideways.
// fractions, not pixels; pass the plot's aspect ratio if that ever feels off.
#[must_use]
pub fn nearest(dots: &[Dot], x: f64, y: f64, radius: f64) -> Option<usize> {
    dots.iter()
        .enumerate()
        .map(|(i, d)| (i, (d.x - x).hypot(d.y - y)))
        .filter(|(_, dist)| *dist <= radius)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// A grid over the plot that finds the nearest dot without checking every one,
/// for scatters with many thousands of points.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Index {
    side: usize,
    /// `starts[c]..starts[c + 1]` indexes `items` for cell `c`.
    starts: Vec<u32>,
    items: Vec<u32>,
}

impl Index {
    /// Bucket `dots` into about four per cell.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "cell maths on clamped fractions"
    )]
    #[must_use]
    pub fn new(dots: &[Dot]) -> Self {
        let side = ((dots.len() as f64 / 4.0).sqrt() as usize).clamp(1, 512);
        let cell = |d: &Dot| Self::cell_of(side, d.x, d.y);
        let mut starts = vec![0u32; side * side + 1];
        for d in dots {
            starts[cell(d) + 1] += 1;
        }
        for c in 0..side * side {
            starts[c + 1] += starts[c];
        }
        let mut fill = starts.clone();
        let mut items = vec![0u32; dots.len()];
        for (i, d) in dots.iter().enumerate() {
            let c = cell(d);
            items[fill[c] as usize] = i as u32;
            fill[c] += 1;
        }
        Self { side, starts, items }
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "clamped fractions"
    )]
    fn axis(side: usize, f: f64) -> usize {
        ((f.clamp(0.0, 1.0) * side as f64) as usize).min(side - 1)
    }

    fn cell_of(side: usize, x: f64, y: f64) -> usize {
        Self::axis(side, y) * side + Self::axis(side, x)
    }

    /// Same answer as [`nearest`], checking only the cells within `radius`.
    #[must_use]
    pub fn nearest(&self, dots: &[Dot], x: f64, y: f64, radius: f64) -> Option<usize> {
        self.scan(dots, (x, y), (1.0, 1.0), radius).map(|(i, _)| i)
    }

    /// Same answer as [`pick`]: the nearest dot in pixels on a `size` (width, height) plot,
    /// within `radius` px. Searches outwards ring by ring, so an unbounded radius stays
    /// cheap wherever the dots are dense.
    #[allow(clippy::cast_precision_loss, reason = "grid side is at most 512")]
    #[must_use]
    pub fn pick(&self, dots: &[Dot], at: (f64, f64), size: (f64, f64), radius: f64) -> Option<usize> {
        let cap = radius.min(size.0.hypot(size.1));
        let mut r = (size.0.max(size.1) / self.side.max(1) as f64).min(cap);
        loop {
            if let Some((i, _)) = self.scan(dots, at, size, r) {
                return Some(i);
            }
            if r >= cap || r <= 0.0 {
                return None;
            }
            r = (r * 2.0).min(cap);
        }
    }

    /// The nearest dot to `at` (fractions) within `radius`, measuring with x scaled by
    /// `size.0` and y by `size.1`. Ties go to the lower index.
    #[allow(clippy::many_single_char_names, reason = "x, y, w, h read best for plot maths")]
    fn scan(&self, dots: &[Dot], (x, y): (f64, f64), (w, h): (f64, f64), radius: f64) -> Option<(usize, f64)> {
        if self.starts.len() < 2 || w <= 0.0 || h <= 0.0 {
            return None;
        }
        let (rx, ry) = (radius / w, radius / h);
        let (c0, c1) = (Self::axis(self.side, x - rx), Self::axis(self.side, x + rx));
        let (r0, r1) = (Self::axis(self.side, y - ry), Self::axis(self.side, y + ry));
        let mut best: Option<(usize, f64)> = None;
        for row in r0..=r1 {
            let cells = row * self.side + c0..=row * self.side + c1;
            let (from, to) = (self.starts[*cells.start()] as usize, self.starts[cells.end() + 1] as usize);
            for &i in &self.items[from..to] {
                let i = i as usize;
                let d = ((dots[i].x - x) * w).hypot((dots[i].y - y) * h);
                if d <= radius && best.is_none_or(|(b, bd)| d < bd || (d <= bd && i < b)) {
                    best = Some((i, d));
                }
            }
        }
        best
    }
}

/// The dot whose Voronoi cell holds the pointer: the nearest to `at` (plot fractions,
/// y up) in pixels on a `size` (width, height) plot, if within `radius` px. Pass
/// `f64::INFINITY` to pick from anywhere in the plot. With an [`Index`] it skips the full scan.
#[allow(clippy::many_single_char_names, reason = "x, y, w, h read best for plot maths")]
#[must_use]
pub fn pick(dots: &[Dot], index: Option<&Index>, at: (f64, f64), size: (f64, f64), radius: f64) -> Option<usize> {
    if let Some(ix) = index {
        return ix.pick(dots, at, size, radius);
    }
    let (w, h) = size;
    dots.iter()
        .enumerate()
        .map(|(i, d)| (i, ((d.x - at.0) * w).hypot((d.y - at.1) * h)))
        .filter(|(_, dist)| *dist <= radius)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// Each dot's Voronoi cell as an SVG `d` in px on a `size` (width, height) plot, y down:
/// the region that picks that dot on hover.
#[must_use]
pub fn cells(dots: &[Dot], size: (f64, f64)) -> Vec<String> {
    let (w, h) = size;
    let points: Vec<(f64, f64)> = dots.iter().map(|d| (d.x * w, (1.0 - d.y) * h)).collect();
    crate::delaunay::voronoi(&points, w, h).iter().map(|c| crate::delaunay::path(c)).collect()
}

/// Indices into `dots`, left to right (ties bottom to top), for [`step_key`]. Kept out of
/// the layout so zooming a big cloud doesn't re-sort it until a key is pressed.
#[must_use]
pub fn order(dots: &[Dot]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..dots.len()).collect();
    order.sort_unstable_by(|&a, &b| dots[a].x.total_cmp(&dots[b].x).then(dots[a].y.total_cmp(&dots[b].y)));
    order
}

/// Keyboard stepping through `order`. Right/Up go forward, Left/Down back, Home/End jump,
/// Escape clears. Returns `None` for other keys, `Some(None)` to clear.
#[must_use]
pub fn step_key(key: &str, active: Option<usize>, order: &[usize]) -> Option<Option<usize>> {
    if order.is_empty() {
        return None;
    }
    let pos = active.and_then(|a| order.iter().position(|&o| o == a));
    let last = order.len() - 1;
    let to = match key {
        "ArrowRight" | "ArrowUp" => pos.map_or(0, |p| (p + 1).min(last)),
        "ArrowLeft" | "ArrowDown" => pos.map_or(last, |p| p.saturating_sub(1)),
        "Home" => 0,
        "End" => last,
        "Escape" => return Some(None),
        _ => return None,
    };
    Some(Some(order[to]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Group> {
        vec![
            Group::new("a", vec![Point::new(0.0, 0.0), Point::sized(10.0, 50.0, 4.0)]),
            Group::new("b", vec![Point::new(5.0, f64::NAN), Point::sized(20.0, 100.0, 1.0)]),
        ]
    }

    #[test]
    fn drops_non_finite_and_keeps_group_indices() {
        let l = layout(&sample(), 5);
        assert_eq!(l.dots.len(), 3);
        assert_eq!((l.dots[2].group, l.dots[2].index), (1, 1));
        assert!(l.dots.iter().all(|d| (0.0..=1.0).contains(&d.x) && (0.0..=1.0).contains(&d.y)));
    }

    #[test]
    fn bubble_area_follows_size() {
        let l = layout(&sample(), 5);
        assert_eq!(l.dots[0].size, None);
        assert!((l.dots[1].size.unwrap_or(0.0) - 1.0).abs() < 1e-9);
        assert!((l.dots[2].size.unwrap_or(0.0) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn nearest_respects_the_radius() {
        let l = layout(&sample(), 5);
        let d = l.dots[1];
        assert_eq!(nearest(&l.dots, d.x + 0.01, d.y, 0.1), Some(1));
        assert_eq!(nearest(&l.dots, 0.5, 0.97, 0.01), None);
    }

    #[test]
    fn index_agrees_with_a_full_scan() {
        let points: Vec<Point> =
            (0..5_000).map(|i| Point::new(f64::from(i * 37 % 1000), f64::from(i * 91 % 997))).collect();
        let l = layout(&[Group::new("a", points)], 5);
        let index = Index::new(&l.dots);
        for k in 0..200 {
            let (x, y) = (f64::from(k * 13 % 100) / 100.0, f64::from(k * 7 % 100) / 100.0);
            assert_eq!(index.nearest(&l.dots, x, y, 0.03), nearest(&l.dots, x, y, 0.03));
        }
        assert_eq!(Index::new(&[]).nearest(&[], 0.5, 0.5, 0.1), None);
    }

    #[test]
    fn pick_measures_in_pixels_and_agrees_with_the_index() {
        let points: Vec<Point> =
            (0..3_000).map(|i| Point::new(f64::from(i * 37 % 1000), f64::from(i * 91 % 997))).collect();
        let l = layout(&[Group::new("a", points)], 5);
        let index = Index::new(&l.dots);
        for k in 0..200 {
            let at = (f64::from(k * 13 % 100) / 100.0, f64::from(k * 7 % 100) / 100.0);
            let full = pick(&l.dots, None, at, (900.0, 300.0), f64::INFINITY);
            assert!(full.is_some());
            assert_eq!(pick(&l.dots, Some(&index), at, (900.0, 300.0), f64::INFINITY), full);
            assert_eq!(
                pick(&l.dots, Some(&index), at, (900.0, 300.0), 6.0),
                pick(&l.dots, None, at, (900.0, 300.0), 6.0)
            );
        }
        // On a wide plot a dot 0.1 away across is 90px; one 0.2 away down is only 60px.
        let dots = [
            Dot { group: 0, index: 0, x: 0.6, y: 0.5, size: None },
            Dot { group: 0, index: 1, x: 0.5, y: 0.7, size: None },
        ];
        assert_eq!(pick(&dots, None, (0.5, 0.5), (900.0, 300.0), f64::INFINITY), Some(1));
        assert_eq!(pick(&dots, None, (0.5, 0.5), (900.0, 300.0), 50.0), None);
        assert_eq!(pick(&[], Some(&Index::new(&[])), (0.5, 0.5), (900.0, 300.0), f64::INFINITY), None);
        assert_eq!(cells(&dots, (900.0, 300.0)).len(), 2);
    }

    #[test]
    fn keys_walk_left_to_right() {
        let l = layout(&sample(), 5);
        let o = order(&l.dots);
        let first = step_key("ArrowRight", None, &o);
        assert_eq!(first, Some(Some(o[0])));
        assert_eq!(step_key("End", None, &o), Some(Some(o[2])));
        assert_eq!(step_key("Escape", Some(0), &o), Some(None));
        assert_eq!(step_key("a", None, &o), None);
    }

    #[test]
    fn zoomed_views_drop_hidden_dots_and_retick() {
        let groups = vec![Group::new("a", (0..=10).map(|i| Point::new(f64::from(i), f64::from(i))).collect())];
        let view = View { x: (0.0, 0.5), y: (0.0, 0.5) };
        let l = layout_scaled(&groups, 5, (ScaleKind::Linear, ScaleKind::Linear), view, None);
        assert_eq!(l.dots.len(), 6);
        assert!(l.x_ticks.iter().all(|g| g.value <= 5.0) && l.x_ticks.len() >= 3);
        let log = layout_scaled(
            &[Group::new("b", vec![Point::new(1.0, 10.0), Point::new(2.0, 10_000.0)])],
            5,
            (ScaleKind::Linear, ScaleKind::Log),
            View::FULL,
            Some((0.0, 50_000.0)),
        );
        assert_eq!(log.y_scale.domain, (10.0, 100_000.0));
        assert!((log.dots[1].y - 0.75).abs() < 1e-9);
    }

    #[test]
    fn reframing_matches_a_fresh_layout() {
        let groups = vec![Group::new("a", (0..=40).map(|i| Point::new(f64::from(i), f64::from(i * i))).collect())];
        let kinds = (ScaleKind::Linear, ScaleKind::Linear);
        let view = View { x: (0.2, 0.6), y: (0.1, 0.5) };
        let full = layout_scaled(&groups, 5, kinds, View::FULL, None);
        assert_eq!(reframe(&full, view, 5), layout_scaled(&groups, 5, kinds, view, None));
    }

    #[test]
    fn empty_data_still_has_axes() {
        let l = layout(&[], 5);
        assert!(l.dots.is_empty() && !l.x_ticks.is_empty());
    }
}
