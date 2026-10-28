//! One line drawn in two styles: coloured above and below a threshold, or switching
//! style at an index (actual, then projected).

use crate::annotate::Annotation;
use crate::line::{self, Curve};
use crate::plot::{self, ValueAxis};
use crate::scale::ScaleKind;

/// Everything a threshold chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct ThresholdLayout {
    /// Horizontal position of each value, 0..=1.
    pub xs: Vec<f64>,
    /// SVG `d` for the line, in the 100×100 box. Adapters draw it twice, clipped above and below `threshold`.
    pub line: String,
    /// SVG `d` for the fill between the line and the threshold.
    pub area: String,
    /// Height of each value, 0..=1; `None` for gaps.
    pub points: Vec<Option<f64>>,
    /// Height of the threshold, 0..=1.
    pub threshold: f64,
    /// The value axis, including the threshold.
    pub axis: ValueAxis,
}

/// Lay out `values` against `threshold`. Non-finite values leave gaps.
#[must_use]
pub fn threshold(
    values: &[f64],
    threshold: f64,
    kind: ScaleKind,
    curve: Curve,
    annotations: &[Annotation],
) -> ThresholdLayout {
    let xs = line::x_positions(values.len(), None);
    let axis = ValueAxis::new(kind, values.iter().copied().chain([threshold]), annotations, false);
    let drawn = plot::drawn(values, &xs, &axis);
    let t = axis.at(threshold);
    let area = line::runs(&drawn).iter().map(|r| line::area(r, (1.0 - t) * 100.0, curve)).collect();
    ThresholdLayout {
        line: plot::line_path(&drawn, curve),
        area,
        points: plot::heights(values, xs.len(), &axis),
        threshold: t,
        xs,
        axis,
    }
}

/// Everything a split line chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitLayout {
    /// Horizontal position of each value, 0..=1.
    pub xs: Vec<f64>,
    /// SVG `d` up to the split.
    pub before: String,
    /// SVG `d` from the last point before the split onwards, so the two meet.
    pub after: String,
    /// Height of each value, 0..=1; `None` for gaps.
    pub points: Vec<Option<f64>>,
    /// The value axis.
    pub axis: ValueAxis,
}

/// Lay out `values`, switching style at index `split` (the first value drawn the second way).
#[must_use]
pub fn split_line(
    values: &[f64],
    split: usize,
    kind: ScaleKind,
    curve: Curve,
    annotations: &[Annotation],
) -> SplitLayout {
    let xs = line::x_positions(values.len(), None);
    let axis = ValueAxis::new(kind, values.iter().copied(), annotations, false);
    let drawn = plot::drawn(values, &xs, &axis);
    let split = split.min(drawn.len());
    SplitLayout {
        before: plot::line_path(&drawn[..split], curve),
        after: if split < drawn.len() {
            plot::line_path(&drawn[split.saturating_sub(1)..], curve)
        } else {
            String::new()
        },
        points: plot::heights(values, xs.len(), &axis),
        xs,
        axis,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_sits_in_the_axis_and_closes_the_fill() {
        let l = threshold(&[10.0, 30.0, 20.0], 50.0, ScaleKind::Linear, Curve::Linear, &[]);
        assert!(l.axis.scale.domain.1 >= 50.0);
        assert!((l.threshold - l.axis.at(50.0)).abs() < 1e-12);
        let ty = (1.0 - l.threshold) * 100.0;
        assert!(l.area.contains(&format!("{ty:.3}Z")));
        assert_eq!(l.points.len(), 3);
    }

    #[test]
    fn split_halves_share_a_point() {
        let l = split_line(&[1.0, 2.0, 3.0, 4.0], 2, ScaleKind::Linear, Curve::Linear, &[]);
        assert_eq!(l.before.matches('L').count(), 1);
        assert_eq!(l.after.matches('L').count(), 2, "starts at the last actual point");
        let all = split_line(&[1.0, 2.0], 9, ScaleKind::Linear, Curve::Linear, &[]);
        assert_eq!(all.after, "", "a split past the end is all actual");
        assert_eq!(split_line(&[1.0, 2.0], 0, ScaleKind::Linear, Curve::Linear, &[]).before, "");
    }
}
