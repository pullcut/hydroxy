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
    /// Indices into `dots`, left to right (ties bottom to top), for keyboard stepping.
    pub order: Vec<usize>,
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
pub fn layout_scaled(groups: &[Group], tick_count: usize, kinds: (ScaleKind, ScaleKind), view: View, extent: Option<(f64, f64)>) -> ScatterLayout {
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
        let (lo, hi) = if lo.is_finite() { (lo, hi) } else if kind == ScaleKind::Log { (1.0, 10.0) } else { (0.0, 1.0) };
        ValueScale::new(kind, lo, if hi > lo { hi } else { lo + 1.0 }, tick_count)
    };
    let sx = span(&|p| p.x, kinds.0, None);
    let sy = span(&|p| p.y, kinds.1, extent);
    let max_size = finite.iter().filter_map(|t| t.2.size).filter(|s| s.is_finite()).fold(0.0, f64::max);
    let inside = |f: f64| (-1e-9..=1.0 + 1e-9).contains(&f);

    let dots: Vec<Dot> = finite
        .iter()
        .map(|&(group, index, p)| Dot {
            group,
            index,
            x: view.project_x(sx.map(p.x).clamp(0.0, 1.0)),
            y: view.project_y(sy.map(p.y).clamp(0.0, 1.0)),
            size: p.size.filter(|_| max_size > 0.0).map(|s| (s.max(0.0) / max_size).sqrt()),
        })
        .filter(|d| inside(d.x) && inside(d.y))
        .collect();
    let mut order: Vec<usize> = (0..dots.len()).collect();
    order.sort_by(|&a, &b| dots[a].x.total_cmp(&dots[b].x).then(dots[a].y.total_cmp(&dots[b].y)));
    ScatterLayout {
        dots,
        order,
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
    fn keys_walk_left_to_right() {
        let l = layout(&sample(), 5);
        let first = step_key("ArrowRight", None, &l.order);
        assert_eq!(first, Some(Some(l.order[0])));
        assert_eq!(step_key("End", None, &l.order), Some(Some(l.order[2])));
        assert_eq!(step_key("Escape", Some(0), &l.order), Some(None));
        assert_eq!(step_key("a", None, &l.order), None);
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
    fn empty_data_still_has_axes() {
        let l = layout(&[], 5);
        assert!(l.dots.is_empty() && !l.x_ticks.is_empty());
    }
}
