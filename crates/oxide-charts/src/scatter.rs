//! Scatter and bubble layout. Positions are fractions of the plot area (0..=1, y up),
//! so adapters place dots with plain CSS percentages.

use crate::bar::Gridline;
use crate::scale::{Linear, ticks};

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
}

/// Lay out `groups` on nice axes. Points with non-finite coordinates are dropped.
#[must_use]
pub fn layout(groups: &[Group], tick_count: usize) -> ScatterLayout {
    let finite: Vec<(usize, usize, Point)> = groups
        .iter()
        .enumerate()
        .flat_map(|(g, grp)| grp.points.iter().enumerate().map(move |(i, p)| (g, i, *p)))
        .filter(|(_, _, p)| p.x.is_finite() && p.y.is_finite())
        .collect();
    let span = |f: fn(&Point) -> f64| {
        let lo = finite.iter().map(|t| f(&t.2)).fold(f64::INFINITY, f64::min);
        let hi = finite.iter().map(|t| f(&t.2)).fold(f64::NEG_INFINITY, f64::max);
        if finite.is_empty() { (0.0, 1.0) } else { (lo, hi) }
    };
    let (x_lo, x_hi) = span(|p| p.x);
    let (y_lo, y_hi) = span(|p| p.y);
    let xt = ticks(x_lo, x_hi, tick_count);
    let yt = ticks(y_lo, y_hi, tick_count);
    let sx = Linear::new((xt[0], xt[xt.len() - 1]), (0.0, 1.0));
    let sy = Linear::new((yt[0], yt[yt.len() - 1]), (0.0, 1.0));
    let max_size = finite.iter().filter_map(|t| t.2.size).filter(|s| s.is_finite()).fold(0.0, f64::max);

    let dots: Vec<Dot> = finite
        .iter()
        .map(|&(group, index, p)| Dot {
            group,
            index,
            x: sx.map(p.x).clamp(0.0, 1.0),
            y: sy.map(p.y).clamp(0.0, 1.0),
            size: p.size.filter(|_| max_size > 0.0).map(|s| (s.max(0.0) / max_size).sqrt()),
        })
        .collect();
    let mut order: Vec<usize> = (0..dots.len()).collect();
    order.sort_by(|&a, &b| dots[a].x.total_cmp(&dots[b].x).then(dots[a].y.total_cmp(&dots[b].y)));
    let grid = |t: &[f64], s: Linear| t.iter().map(|&value| Gridline { value, fraction: s.map(value) }).collect();
    ScatterLayout { dots, order, x_ticks: grid(&xt, sx), y_ticks: grid(&yt, sy) }
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
    fn empty_data_still_has_axes() {
        let l = layout(&[], 5);
        assert!(l.dots.is_empty() && !l.x_ticks.is_empty());
    }
}
