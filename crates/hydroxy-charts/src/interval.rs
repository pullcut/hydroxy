//! Uncertainty: error bars, a confidence band around a line, and a forecast fan.
//! The line charts here reuse `line`'s path maths on a [`ValueAxis`], so they take any scale.

use crate::annotate::Annotation;
use crate::line::{self, Curve};
use crate::plot::{self, ValueAxis};
use crate::scale::ScaleKind;

/// An estimate with a low and high bound.
#[derive(Debug, Clone, PartialEq)]
pub struct Interval {
    /// Category name.
    pub label: String,
    /// The estimate.
    pub value: f64,
    /// Lower bound.
    pub low: f64,
    /// Upper bound.
    pub high: f64,
}

impl Interval {
    /// An estimate between `low` and `high`.
    #[must_use]
    pub fn new(label: impl Into<String>, value: f64, low: f64, high: f64) -> Self {
        Self { label: label.into(), value, low, high }
    }
}

/// One error bar, as plot fractions. `None` where the input was not finite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Whisker {
    /// The estimate.
    pub value: Option<f64>,
    /// Lower bound.
    pub low: Option<f64>,
    /// Upper bound.
    pub high: Option<f64>,
}

/// Everything an error bar chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct WhiskerLayout {
    /// One per interval.
    pub whiskers: Vec<Whisker>,
    /// The value axis.
    pub axis: ValueAxis,
}

/// Lay out error bars. `bars` keeps zero in view, since bars start there.
#[must_use]
pub fn whiskers(data: &[Interval], kind: ScaleKind, bars: bool, annotations: &[Annotation]) -> WhiskerLayout {
    let axis = ValueAxis::new(kind, data.iter().flat_map(|d| [d.value, d.low, d.high]), annotations, bars);
    let at = |v: f64| v.is_finite().then(|| axis.at(v));
    let whiskers = data.iter().map(|d| Whisker { value: at(d.value), low: at(d.low), high: at(d.high) }).collect();
    WhiskerLayout { whiskers, axis }
}

/// Everything a confidence band chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct BandLayout {
    /// Horizontal position of each value, 0..=1.
    pub xs: Vec<f64>,
    /// SVG `d` for the line.
    pub line: String,
    /// SVG `d` for the band between low and high.
    pub band: String,
    /// Height of each value, 0..=1; `None` for gaps.
    pub points: Vec<Option<f64>>,
    /// The value axis, wide enough for the band.
    pub axis: ValueAxis,
}

/// A line with one band from `low` to `high`.
#[must_use]
pub fn band(
    values: &[f64],
    low: &[f64],
    high: &[f64],
    kind: ScaleKind,
    curve: Curve,
    annotations: &[Annotation],
) -> BandLayout {
    let xs = line::x_positions(values.len(), None);
    let n = xs.len();
    let axis = ValueAxis::new(
        kind,
        values.iter().chain(low.iter().take(n)).chain(high.iter().take(n)).copied(),
        annotations,
        false,
    );
    let drawn = plot::drawn(values, &xs, &axis);
    BandLayout {
        line: plot::line_path(&drawn, curve),
        band: line::band_path(&plot::drawn(high, &xs, &axis), &plot::drawn(low, &xs, &axis), curve),
        points: plot::heights(values, n, &axis),
        xs,
        axis,
    }
}

/// One forecast interval, aligned with the forecast values.
#[derive(Debug, Clone, PartialEq)]
pub struct FanBand {
    /// Shown in the tooltip, legend and table, such as "80%".
    pub name: String,
    /// Lower bound per forecast step.
    pub low: Vec<f64>,
    /// Upper bound per forecast step.
    pub high: Vec<f64>,
}

impl FanBand {
    /// A named interval.
    #[must_use]
    pub fn new(name: impl Into<String>, low: impl Into<Vec<f64>>, high: impl Into<Vec<f64>>) -> Self {
        Self { name: name.into(), low: low.into(), high: high.into() }
    }
}

/// Everything a forecast fan chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct FanLayout {
    /// Horizontal position of every step (history then forecast), 0..=1.
    pub xs: Vec<f64>,
    /// SVG `d` for the actual values.
    pub actual: String,
    /// SVG `d` for the forecast, starting from the last actual value.
    pub forecast: String,
    /// SVG `d` per band, in input order. Each starts as a point at the last actual value and widens.
    pub bands: Vec<String>,
    /// Height of the actual or forecast value at each step, 0..=1.
    pub points: Vec<Option<f64>>,
    /// Index of the first forecast step.
    pub split: usize,
    /// The value axis, wide enough for every band.
    pub axis: ValueAxis,
}

/// Actual values followed by a forecast with nested bands (widest first reads best).
#[must_use]
pub fn fan(
    history: &[f64],
    forecast: &[f64],
    bands: &[FanBand],
    kind: ScaleKind,
    curve: Curve,
    annotations: &[Annotation],
) -> FanLayout {
    let split = history.len();
    let len = split + forecast.len();
    let xs = line::x_positions(len, None);
    let anchor = history.last().copied().unwrap_or(f64::NAN);
    // Pad forecast-aligned values out to the full x range, joined to the last actual value.
    let full = |future: &[f64]| -> Vec<f64> {
        let mut v = vec![f64::NAN; split];
        if let Some(last) = v.last_mut() {
            *last = anchor;
        }
        v.extend((0..forecast.len()).map(|i| future.get(i).copied().unwrap_or(f64::NAN)));
        v
    };
    let values = history.iter().chain(bands.iter().flat_map(|b| b.low.iter().chain(&b.high)).chain(forecast)).copied();
    let axis = ValueAxis::new(kind, values, annotations, false);
    let draw = |v: &[f64]| plot::drawn(v, &xs, &axis);
    let mut combined = history.to_vec();
    combined.extend_from_slice(forecast);
    FanLayout {
        actual: plot::line_path(&draw(history), curve),
        forecast: plot::line_path(&draw(&full(forecast)), curve),
        bands: bands.iter().map(|b| line::band_path(&draw(&full(&b.high)), &draw(&full(&b.low)), curve)).collect(),
        points: plot::heights(&combined, len, &axis),
        split,
        xs,
        axis,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whiskers_cover_their_bounds() {
        let l = whiskers(
            &[Interval::new("A", 10.0, 8.0, 14.0), Interval::new("B", f64::NAN, 1.0, 2.0)],
            ScaleKind::Linear,
            true,
            &[],
        );
        assert_eq!(l.axis.scale.domain.0, 0.0, "bars keep zero");
        let w = l.whiskers[0];
        assert!(w.low < w.value && w.value < w.high);
        assert_eq!(l.whiskers[1].value, None);
        let points = whiskers(&[Interval::new("A", 10.0, 8.0, 14.0)], ScaleKind::Linear, false, &[]);
        assert!(points.axis.scale.domain.0 > 0.0, "points can float");
    }

    #[test]
    fn band_widens_the_axis() {
        let l = band(&[10.0, 11.0, 12.0], &[9.0, 9.0, 9.0], &[11.0, 30.0, 13.0], ScaleKind::Linear, Curve::Linear, &[]);
        assert!(l.axis.scale.domain.1 >= 30.0);
        assert!(l.band.ends_with('Z'));
        assert_eq!(l.points.len(), 3);
    }

    #[test]
    fn fan_starts_at_the_last_actual_value() {
        let b = FanBand::new("80%", [9.0, 8.0], [13.0, 16.0]);
        let l = fan(&[5.0, 8.0, 10.0], &[11.0, 12.0], &[b], ScaleKind::Linear, Curve::Linear, &[]);
        assert_eq!(l.split, 3);
        assert_eq!(l.xs.len(), 5);
        // The forecast line and band both start at x = 50 (index 2 of 5), the last actual point.
        assert!(l.forecast.starts_with("M50.000"));
        assert!(l.bands[0].starts_with("M50.000"));
        assert_eq!(l.actual.matches('L').count(), 2);
        assert!(l.points.iter().all(Option::is_some));
    }
}
