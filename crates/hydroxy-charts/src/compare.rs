//! Categorical comparison with dots: dot plots (with a dumbbell bar when a row has two
//! or more series) and lollipops (a stick from the baseline plus a dot).

use crate::annotate::Annotation;
use crate::line::Series;
use crate::plot::ValueAxis;
use crate::scale::ScaleKind;

/// Everything a dot plot needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct DotLayout {
    /// Per row, each series' position along the value axis (0..=1), `None` when missing.
    pub dots: Vec<Vec<Option<f64>>>,
    /// Per row, the span from the lowest to the highest dot when the row has two or more.
    pub spans: Vec<Option<(f64, f64)>>,
    /// The value axis. It does not force zero in, so close values stay readable.
    pub axis: ValueAxis,
}

/// Lay out `series` over `rows` categories.
#[must_use]
pub fn dots(series: &[Series], rows: usize, kind: ScaleKind, annotations: &[Annotation]) -> DotLayout {
    let axis =
        ValueAxis::new(kind, series.iter().flat_map(|s| s.values.iter().take(rows).copied()), annotations, false);
    let dots: Vec<Vec<Option<f64>>> = (0..rows)
        .map(|r| {
            series.iter().map(|s| s.values.get(r).copied().filter(|v| v.is_finite()).map(|v| axis.at(v))).collect()
        })
        .collect();
    let spans = dots
        .iter()
        .map(|row| {
            let present: Vec<f64> = row.iter().flatten().copied().collect();
            (present.len() >= 2).then(|| {
                (present.iter().copied().fold(f64::INFINITY, f64::min), present.iter().copied().fold(0.0, f64::max))
            })
        })
        .collect();
    DotLayout { dots, spans, axis }
}

/// The series in `row` whose dot is nearest to value position `p`.
#[must_use]
pub fn nearest_dot(row: &[Option<f64>], p: f64) -> Option<usize> {
    row.iter()
        .enumerate()
        .filter_map(|(i, d)| d.map(|d| (i, (d - p).abs())))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// Everything a lollipop chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct LollipopLayout {
    /// Each dot's position along the value axis, 0..=1; `None` when missing.
    pub heads: Vec<Option<f64>>,
    /// Where sticks start: zero, or the bottom of a log axis.
    pub base: f64,
    /// The value axis.
    pub axis: ValueAxis,
}

/// Lay out one lollipop per value.
#[must_use]
pub fn lollipops(values: &[f64], kind: ScaleKind, annotations: &[Annotation]) -> LollipopLayout {
    let axis = ValueAxis::new(kind, values.iter().copied(), annotations, true);
    let heads = values.iter().map(|&v| v.is_finite().then(|| axis.at(v))).collect();
    LollipopLayout { heads, base: axis.base(), axis }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dumbbells_span_the_row() {
        let s = [Series::new("2024", [40.0, 55.0]), Series::new("2025", [60.0, f64::NAN])];
        let l = dots(&s, 2, ScaleKind::Linear, &[]);
        let (lo, hi) = l.spans[0].expect("two dots");
        assert!((lo - l.axis.at(40.0)).abs() < 1e-12 && (hi - l.axis.at(60.0)).abs() < 1e-12);
        assert_eq!(l.spans[1], None, "one dot, no bar");
        assert!(l.axis.scale.domain.0 > 0.0, "dot plots do not force zero");
        assert_eq!(nearest_dot(&l.dots[0], 0.9), Some(1));
        assert_eq!(nearest_dot(&[None], 0.5), None);
    }

    #[test]
    fn lollipops_start_at_zero() {
        let l = lollipops(&[5.0, 20.0, f64::NAN], ScaleKind::Linear, &[]);
        assert_eq!(l.base, 0.0);
        assert_eq!(l.heads[2], None);
        assert!(l.heads[1] > l.heads[0]);
        let log = lollipops(&[5.0, 500.0], ScaleKind::Log, &[]);
        assert_eq!(log.base, 0.0, "log sticks start at the bottom");
    }
}
