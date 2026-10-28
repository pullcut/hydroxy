//! Span (range) bars: one bar per category from a start value to an end value, such as a
//! salary band, with an optional marker for a point inside it (a median, today's price).

use crate::annotate::Annotation;
use crate::plot::ValueAxis;
use crate::scale::ScaleKind;

/// One category's range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    /// Where the range starts.
    pub start: f64,
    /// Where it ends. A start above the end is swapped.
    pub end: f64,
    /// A point of interest inside the range, such as the median.
    pub marker: Option<f64>,
}

impl Span {
    /// A range from `start` to `end`.
    #[must_use]
    pub fn new(start: f64, end: f64) -> Self {
        Self { start, end, marker: None }
    }

    /// The same range with a marker at `value`.
    #[must_use]
    pub fn with_marker(self, value: f64) -> Self {
        Self { marker: Some(value), ..self }
    }

    /// The range low to high, or `None` when either end is missing.
    #[must_use]
    pub fn bounds(&self) -> Option<(f64, f64)> {
        (self.start.is_finite() && self.end.is_finite()).then(|| (self.start.min(self.end), self.start.max(self.end)))
    }
}

/// One drawn bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpanBar {
    /// Start along the value axis, 0..=1.
    pub a: f64,
    /// End along the value axis, 0..=1.
    pub b: f64,
    /// The marker along the value axis, 0..=1.
    pub marker: Option<f64>,
}

/// Everything a span chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct SpanLayout {
    /// One per row, `None` when the row's range is missing.
    pub bars: Vec<Option<SpanBar>>,
    /// The value axis. It fits the data rather than forcing zero in, with room past both
    /// ends for the end labels.
    pub axis: ValueAxis,
}

/// Lay out `spans`. `annotations` widen the axis to keep reference lines in view.
#[must_use]
pub fn layout(spans: &[Span], kind: ScaleKind, annotations: &[Annotation]) -> SpanLayout {
    let values = spans.iter().flat_map(|s| s.bounds().map_or([f64::NAN; 3], |(a, b)| [a, b, s.marker.unwrap_or(f64::NAN)]));
    let fitted = ValueAxis::new(kind, values, annotations, false);
    let (lo, hi) = fitted.scale.domain;
    let axis = if kind == ScaleKind::Linear {
        // A tenth of the range either side keeps the end labels off the plot edges.
        let pad = (hi - lo) * 0.1;
        ValueAxis::over(kind, lo - pad, hi + pad)
    } else {
        fitted
    };
    let bars = spans
        .iter()
        .map(|s| {
            let (a, b) = s.bounds()?;
            Some(SpanBar { a: axis.at(a), b: axis.at(b), marker: s.marker.filter(|m| m.is_finite()).map(|m| axis.at(m)) })
        })
        .collect();
    SpanLayout { bars, axis }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_fit_with_room_for_labels() {
        let l = layout(&[Span::new(40.0, 60.0).with_marker(52.0), Span::new(90.0, 70.0), Span::new(f64::NAN, 3.0)], ScaleKind::Linear, &[]);
        let first = l.bars[0].expect("a range");
        assert!(first.a > 0.0 && first.b < 1.0, "padded past the data");
        assert!(first.marker.is_some_and(|m| m > first.a && m < first.b));
        let swapped = l.bars[1].expect("a range");
        assert!(swapped.a < swapped.b, "start above end is swapped");
        assert_eq!(l.bars[2], None);
        assert!(l.axis.scale.domain.0 > 0.0, "spans do not force zero");
    }

    #[test]
    fn spans_report_bounds() {
        assert_eq!(Span::new(5.0, 2.0).bounds(), Some((2.0, 5.0)));
        assert_eq!(Span::new(5.0, f64::INFINITY).bounds(), None);
        let empty = layout(&[], ScaleKind::Linear, &[]);
        assert!(empty.bars.is_empty() && empty.axis.scale.domain.1 > empty.axis.scale.domain.0);
    }
}
