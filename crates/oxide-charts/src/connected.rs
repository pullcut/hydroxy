//! Connected scatter: two measures plotted against each other, joined in time order, so
//! the line shows how the pair moved together.

use std::fmt::Write as _;

use crate::annotate::Annotation;
use crate::line;
use crate::plot::ValueAxis;
use crate::scale::ScaleKind;

/// One moment: when it was and the two measures then.
#[derive(Debug, Clone, PartialEq)]
pub struct Moment {
    /// When, such as "2019" or "Mar". Shown on labelled points, the tooltip and table.
    pub label: String,
    /// The horizontal measure.
    pub x: f64,
    /// The vertical measure.
    pub y: f64,
}

impl Moment {
    /// A moment at `(x, y)`.
    #[must_use]
    pub fn new(label: impl Into<String>, x: f64, y: f64) -> Self {
        Self { label: label.into(), x, y }
    }
}

/// Everything a connected scatter needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectedLayout {
    /// Each moment as `(x, y)` plot fractions, y from the bottom; `None` when not finite.
    pub points: Vec<Option<(f64, f64)>>,
    /// SVG `d` through the points in time order, in the 100×100 drawing box.
    pub path: String,
    /// How far along the drawn path each point sits, 0..=1, so marks can land as it arrives.
    pub along: Vec<f64>,
    /// Points that carry a time label: the first, the last, and a few evenly between.
    pub labelled: Vec<usize>,
    /// The horizontal axis.
    pub x_axis: ValueAxis,
    /// The vertical axis.
    pub y_axis: ValueAxis,
}

/// Lay out a connected scatter, labelling at most `label_count` points. `aspect` (plot width
/// over height) only evens out the timing in [`ConnectedLayout::along`].
#[must_use]
pub fn layout(data: &[Moment], x_kind: ScaleKind, y_kind: ScaleKind, annotations: &[Annotation], label_count: usize, aspect: f64) -> ConnectedLayout {
    let x_axis = ValueAxis::new(x_kind, data.iter().map(|d| d.x), &[], false);
    let y_axis = ValueAxis::new(y_kind, data.iter().map(|d| d.y), annotations, false);
    let points: Vec<Option<(f64, f64)>> =
        data.iter().map(|d| (d.x.is_finite() && d.y.is_finite()).then(|| (x_axis.at(d.x), y_axis.at(d.y)))).collect();
    let mut path = String::new();
    for run in line::runs(&points) {
        for (i, (x, y)) in run.iter().enumerate() {
            let _ = write!(path, "{}{:.3} {:.3}", if i == 0 { 'M' } else { 'L' }, x * 100.0, (1.0 - y) * 100.0);
        }
        if run.len() == 1 {
            path.push_str("h0");
        }
    }
    // Cumulative length along the drawn segments (gaps add nothing), in even units.
    let a = if aspect.is_finite() && aspect > 0.0 { aspect } else { 1.0 };
    let mut along = Vec::with_capacity(points.len());
    let (mut total, mut prev) = (0.0, None::<(f64, f64)>);
    for p in &points {
        if let (Some((x0, y0)), Some((x1, y1))) = (prev, *p) {
            total += ((x1 - x0) * a).hypot(y1 - y0);
        }
        along.push(total);
        prev = *p;
    }
    if total > 0.0 {
        for t in &mut along {
            *t /= total;
        }
    }
    let present: Vec<usize> = (0..points.len()).filter(|&i| points[i].is_some()).collect();
    let picks = line::label_indices(present.len(), label_count.max(2));
    let step = picks.get(1).copied().unwrap_or(1);
    let mut labelled: Vec<usize> = picks.iter().map(|&k| present[k]).collect();
    if let (Some(&end), Some(&k)) = (present.last(), picks.last())
        && labelled.last() != Some(&end)
    {
        // The thinned labels can stop short of the end, which always gets one; drop a crowding neighbour.
        if picks.len() > 1 && present.len() - 1 - k < step.div_ceil(2) {
            labelled.pop();
        }
        labelled.push(end);
    }
    ConnectedLayout { points, path, along, labelled, x_axis, y_axis }
}

/// The point nearest `p` (plot fractions), if one is within `within` (as a fraction of the
/// plot height). `aspect` is the plot's width over its height.
#[must_use]
pub fn nearest_point(points: &[Option<(f64, f64)>], p: (f64, f64), aspect: f64, within: f64) -> Option<usize> {
    let a = if aspect.is_finite() && aspect > 0.0 { aspect } else { 1.0 };
    points
        .iter()
        .enumerate()
        .filter_map(|(i, q)| q.map(|(x, y)| (i, ((x - p.0) * a).hypot(y - p.1))))
        .filter(|&(_, d)| d <= within)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> Vec<Moment> {
        (0..10).map(|i| Moment::new(format!("{}", 2015 + i), f64::from(i), f64::from(i * i))).collect()
    }

    #[test]
    fn joins_points_in_time_order() {
        let l = layout(&data(), ScaleKind::Linear, ScaleKind::Linear, &[], 4, 1.0);
        assert_eq!(l.path.matches('M').count(), 1);
        assert_eq!(l.path.matches('L').count(), 9);
        assert_eq!(l.along[0], 0.0);
        assert!((l.along[9] - 1.0).abs() < 1e-12);
        assert!(l.along.windows(2).all(|w| w[0] <= w[1]));
        assert_eq!(l.labelled.first(), Some(&0));
        assert_eq!(l.labelled.last(), Some(&9));
        assert!(l.labelled.len() <= 5);
    }

    #[test]
    fn gaps_break_the_line() {
        let mut d = data();
        d[4].y = f64::NAN;
        let l = layout(&d, ScaleKind::Linear, ScaleKind::Linear, &[], 4, 1.0);
        assert_eq!(l.points[4], None);
        assert_eq!(l.path.matches('M').count(), 2);
        assert_eq!(l.along[4], l.along[3], "a gap adds no length");
    }

    #[test]
    fn nearest_point_respects_the_radius() {
        let pts = [Some((0.1, 0.1)), None, Some((0.5, 0.5))];
        assert_eq!(nearest_point(&pts, (0.45, 0.5), 1.0, 0.1), Some(2));
        assert_eq!(nearest_point(&pts, (0.9, 0.9), 1.0, 0.1), None);
    }
}
