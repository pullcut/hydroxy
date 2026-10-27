//! The readout shared by the distribution and financial charts: axis ticks, tooltip
//! stops, the screen-reader table and placed annotations. Each chart's maths fills one
//! in, so adapters render the frame once and only draw the marks per chart.

use crate::annotate::{Annotation, Placed};
use crate::boxplot::Group;
use crate::scale::ValueScale;

/// A value waiting for the chart's formatter.
#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    /// Plain text, shown as is.
    Text(String),
    /// A data value, run through the chart's `format`.
    Value(f64),
    /// A count.
    Count(usize),
    /// A size or volume, shortened: 12.5k.
    Compact(f64),
    /// A small unitless number such as a density, to three significant figures.
    Small(f64),
}

impl Cell {
    /// The text to show, using `fmt` for [`Cell::Value`].
    pub fn show(&self, fmt: impl Fn(f64) -> String) -> String {
        match self {
            Cell::Text(t) => t.clone(),
            Cell::Value(v) => fmt(*v),
            Cell::Count(n) => n.to_string(),
            Cell::Compact(v) => crate::candle::compact(*v),
            Cell::Small(v) => small(*v),
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "decimal places are tiny")]
fn small(v: f64) -> String {
    if v == 0.0 || !v.is_finite() {
        return "0".into();
    }
    let places = (2.0 - v.abs().log10().floor()).clamp(0.0, 12.0) as usize;
    let text = format!("{v:.places$}");
    if text.contains('.') { text.trim_end_matches('0').trim_end_matches('.').to_string() } else { text }
}

/// An axis tick or category label, `fraction` along the axis (0 = left or bottom).
#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    /// Position along the axis, 0..=1.
    pub fraction: f64,
    /// Text.
    pub label: Cell,
}

/// One line of a tooltip.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// What the value is.
    pub name: String,
    /// Series colour slot for a swatch, if any.
    pub colour: Option<usize>,
    /// The value.
    pub value: Cell,
}

impl Row {
    /// A row without a swatch.
    #[must_use]
    pub fn new(name: impl Into<String>, value: Cell) -> Self {
        Self { name: name.into(), colour: None, value }
    }
}

/// A place the pointer or keyboard can land, with what its tooltip says.
#[derive(Debug, Clone, PartialEq)]
pub struct Stop {
    /// From the left, 0..=1.
    pub x: f64,
    /// From the bottom, 0..=1.
    pub y: f64,
    /// Tooltip heading.
    pub title: Cell,
    /// Colour slot for the heading swatch.
    pub colour: Option<usize>,
    /// Tooltip rows.
    pub rows: Vec<Row>,
}

/// How the pointer picks a stop.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Pick {
    /// The nearest stop within a small radius, for dots.
    #[default]
    Near,
    /// The stop nearest horizontally, for columns and crosshairs.
    X,
    /// The row whose stop sits at or below the pointer, for stacked rows.
    Row,
}

/// Everything around the marks.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Readout {
    /// Ticks along the bottom.
    pub x_ticks: Vec<Tick>,
    /// Ticks up the side.
    pub y_ticks: Vec<Tick>,
    /// The bottom ticks are category labels, without gridlines.
    pub x_bands: bool,
    /// Tooltip stops, in keyboard order.
    pub stops: Vec<Stop>,
    /// How the pointer picks a stop.
    pub pick: Pick,
    /// Legend entries: name and colour slot.
    pub legend: Vec<(String, usize)>,
    /// Annotations, placed.
    pub notes: Vec<Placed>,
    /// Table header.
    pub head: Vec<String>,
    /// Table rows; the first cell is the row header.
    pub rows: Vec<Vec<Cell>>,
}

/// The stop the pointer at (`x`, `y`) points at.
#[must_use]
pub fn nearest(stops: &[Stop], x: f64, y: f64, pick: Pick) -> Option<usize> {
    if pick == Pick::Row {
        let below = stops.iter().enumerate().filter(|(_, s)| s.y <= y + 1e-9).max_by(|a, b| a.1.y.total_cmp(&b.1.y));
        return below.or_else(|| stops.iter().enumerate().min_by(|a, b| a.1.y.total_cmp(&b.1.y))).map(|(i, _)| i);
    }
    let dist = |s: &Stop| if pick == Pick::X { (s.x - x).abs() } else { (s.x - x).hypot(s.y - y) };
    let (i, d) = stops.iter().map(dist).enumerate().min_by(|a, b| a.1.total_cmp(&b.1))?;
    (pick != Pick::Near || d <= 0.08).then_some(i)
}

/// One sentence for the live region: "Title: name value, name value".
pub fn describe(stop: &Stop, fmt: impl Fn(f64) -> String) -> String {
    let rows: Vec<String> = stop.rows.iter().map(|r| format!("{} {}", r.name, r.value.show(&fmt))).collect();
    format!("{}: {}", stop.title.show(&fmt), rows.join(", "))
}

/// The colour slot for group `g` of `n`. Past six groups every mark shares the first
/// colour (position already says which group it is), so colours never repeat.
#[must_use]
pub const fn slot(g: usize, n: usize) -> usize {
    if n > 6 { 0 } else { g }
}

/// Legend entries for `names`, or none when there is one name or more than six.
#[must_use]
pub fn legend(names: impl IntoIterator<Item = String>) -> Vec<(String, usize)> {
    let names: Vec<String> = names.into_iter().collect();
    if names.len() < 2 || names.len() > 6 {
        return Vec::new();
    }
    names.into_iter().enumerate().map(|(i, n)| (n, i)).collect()
}

/// Gridline ticks for a value axis, inside its domain, labelled by `label`.
#[must_use]
pub fn value_ticks(scale: ValueScale, count: usize, label: fn(f64) -> Cell) -> Vec<Tick> {
    // `ValueScale::ticks` re-nices the domain and can step past it (195..225 gives 190..230).
    scale
        .ticks(count)
        .into_iter()
        .map(|v| Tick { fraction: scale.map(v), label: label(v) })
        .filter(|t| (-1e-9..=1.0 + 1e-9).contains(&t.fraction))
        .collect()
}

#[allow(clippy::cast_precision_loss, reason = "category counts are small")]
/// The centre of band `i` of `n`, as a fraction.
#[must_use]
pub fn band(i: usize, n: usize) -> f64 {
    (i as f64 + 0.5) / n.max(1) as f64
}

/// The screen-reader table for raw observations: one row per value.
#[must_use]
pub fn observations(groups: &[Group]) -> (Vec<String>, Vec<Vec<Cell>>) {
    let rows = groups
        .iter()
        .flat_map(|g| g.values.iter().filter(|v| v.is_finite()).map(|&v| vec![Cell::Text(g.label.clone()), Cell::Value(v)]))
        .collect();
    (vec!["Group".into(), "Value".into()], rows)
}

/// The finite low and high of every group's values, or `None` when there are none.
#[must_use]
pub fn extent(groups: &[Group]) -> Option<(f64, f64)> {
    let mut all = groups.iter().flat_map(|g| g.values.iter().copied()).filter(|v| v.is_finite()).peekable();
    all.peek()?;
    Some(all.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| (lo.min(v), hi.max(v))))
}

/// The x values annotations need visible, for charts whose value axis runs across.
#[must_use]
pub fn x_extent(annotations: &[Annotation]) -> Option<(f64, f64)> {
    let values = annotations.iter().flat_map(|a| match a {
        Annotation::XLine { at, .. } => vec![*at],
        Annotation::XBand { from, to, .. } => vec![*from, *to],
        Annotation::Point { x, .. } => vec![*x],
        Annotation::YLine { .. } | Annotation::YBand { .. } => vec![],
    });
    values.filter(|v| v.is_finite()).fold(None, |acc, v| Some(acc.map_or((v, v), |(lo, hi): (f64, f64)| (lo.min(v), hi.max(v)))))
}

/// Widen `(lo, hi)` to cover `extra`.
#[must_use]
pub fn widen((lo, hi): (f64, f64), extra: Option<(f64, f64)>) -> (f64, f64) {
    extra.map_or((lo, hi), |(a, b)| (lo.min(a), hi.max(b)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stop(x: f64, y: f64) -> Stop {
        Stop { x, y, title: Cell::Text("t".into()), colour: None, rows: vec![Row::new("v", Cell::Value(2.0))] }
    }

    #[test]
    fn picks_by_mode() {
        let stops = [stop(0.1, 0.1), stop(0.5, 0.9)];
        assert_eq!(nearest(&stops, 0.12, 0.1, Pick::Near), Some(0));
        assert_eq!(nearest(&stops, 0.3, 0.5, Pick::Near), None);
        assert_eq!(nearest(&stops, 0.45, 0.0, Pick::X), Some(1));
        assert_eq!(nearest(&stops, 0.0, 0.95, Pick::Row), Some(1));
        assert_eq!(nearest(&stops, 0.0, 0.5, Pick::Row), Some(0));
        assert_eq!(nearest(&stops, 0.0, 0.0, Pick::Row), Some(0));
        assert_eq!(nearest(&[], 0.0, 0.0, Pick::X), None);
    }

    #[test]
    fn cells_and_sentences() {
        assert_eq!(Cell::Small(0.02346).show(crate::format_value), "0.0235");
        assert_eq!(Cell::Small(12.345).show(crate::format_value), "12.3");
        assert_eq!(Cell::Small(0.08).show(crate::format_value), "0.08");
        assert_eq!(Cell::Small(120.0).show(crate::format_value), "120");
        assert_eq!(Cell::Compact(12_500.0).show(crate::format_value), "12.5k");
        assert_eq!(describe(&stop(0.0, 0.0), |v| format!("{v}!")), "t: v 2!");
    }

    #[test]
    fn colours_never_repeat() {
        assert_eq!(slot(3, 4), 3);
        assert_eq!(slot(6, 8), 0);
        assert_eq!(legend(["a".to_string()]), vec![]);
        assert_eq!(legend(["a".to_string(), "b".to_string()]).len(), 2);
    }

    #[test]
    fn value_ticks_stay_inside_the_domain() {
        let s = ValueScale::new(crate::scale::ScaleKind::Linear, 198.0, 224.0, 5);
        let t = value_ticks(s, 5, Cell::Value);
        assert!(!t.is_empty() && t.iter().all(|t| (0.0..=1.0).contains(&t.fraction)));
    }

    #[test]
    fn extents_skip_non_finite() {
        let g = [Group::new("a", [3.0, f64::NAN, -1.0]), Group::new("b", [7.0])];
        assert_eq!(extent(&g), Some((-1.0, 7.0)));
        assert_eq!(extent(&[]), None);
        assert_eq!(observations(&g).1.len(), 3);
    }
}
