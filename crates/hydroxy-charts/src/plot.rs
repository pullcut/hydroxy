//! Shared pieces for the single-value-axis cartesian charts (grouped bars, waterfall,
//! stacked area and friends): the value axis, axis and legend entries, tooltip content,
//! category picking, series folding and annotation placement.

use crate::annotate::{self, Annotation, Placed};
use crate::bar::Gridline;
use crate::line::{self, Curve, Series};
use crate::scale::{ScaleKind, ValueScale};

/// `var(--chart-N)` for series `i`: the one colour rule for every chart. The palette has six
/// hues and the index is capped, never wrapped, so series seven onwards share the sixth hue
/// in every chart. Charts that need more fold the extras into "Other" first.
#[must_use]
pub fn colour(i: usize) -> String {
    format!("var(--chart-{})", i.min(5) + 1)
}

/// `var(--chart-N)` for slot `i` while `i` is among the first `n` slots (at most six), else
/// the muted "Other" colour. Charts with more groups than hues give the first few their own
/// colour and let the rest share the muted one rather than reusing a hue.
#[must_use]
pub fn colour_or_muted(i: usize, n: usize) -> String {
    if i < n.min(6) { format!("var(--chart-{})", i + 1) } else { "var(--chart-muted)".to_string() }
}

/// A fraction as a CSS percentage with three decimals, such as `0.5` as `50.000%`.
#[must_use]
pub fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// A value axis: its scale and gridlines.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueAxis {
    /// The mapping from values to 0..=1.
    pub scale: ValueScale,
    /// Gridlines from the bottom up, at the scale's ticks.
    pub gridlines: Vec<Gridline>,
}

impl ValueAxis {
    /// An axis covering every finite value in `values` and the annotations' y values.
    /// `zero` keeps zero in view (bars start there); log axes ignore it and skip values at or below zero.
    #[must_use]
    pub fn new(kind: ScaleKind, values: impl IntoIterator<Item = f64>, annotations: &[Annotation], zero: bool) -> Self {
        let log = kind == ScaleKind::Log;
        let extra = annotate::y_extent(annotations).map_or([f64::NAN; 2], |(a, b)| [a, b]);
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for v in values.into_iter().chain(extra) {
            if v.is_finite() && (!log || v > 0.0) {
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        if zero && !log {
            lo = lo.min(0.0);
            hi = hi.max(0.0);
        }
        if !lo.is_finite() {
            (lo, hi) = if log { (1.0, 10.0) } else { (0.0, 1.0) };
        }
        if hi <= lo {
            hi = if log { lo * 10.0 } else { lo + 1.0 };
        }
        Self::over(kind, lo, hi)
    }

    /// An axis over exactly `[lo, hi]`, widened to nice ticks.
    #[must_use]
    pub fn over(kind: ScaleKind, lo: f64, hi: f64) -> Self {
        let scale = ValueScale::new(kind, lo, hi, 5);
        let (a, b) = scale.domain;
        let slack = (b - a).abs() * 1e-9;
        let gridlines = scale
            .ticks(5)
            .into_iter()
            .filter(|v| (a - slack..=b + slack).contains(v))
            .map(|value| Gridline { value, fraction: scale.map(value) })
            .collect();
        Self { scale, gridlines }
    }

    /// Where `value` sits, clamped to the plot (0 = bottom).
    #[must_use]
    pub fn at(&self, value: f64) -> f64 {
        self.scale.map(value).clamp(0.0, 1.0)
    }

    /// Where bars start: zero, or the nearest edge when zero is out of range (always the bottom on a log axis).
    #[must_use]
    pub fn base(&self) -> f64 {
        self.at(0.0)
    }

    /// The gridlines as axis ticks, labelled with `format`.
    #[must_use]
    pub fn ticks(&self, format: impl Fn(f64) -> String) -> Vec<Tick> {
        self.gridlines
            .iter()
            .map(|g| Tick { at: g.fraction, text: format(g.value), baseline: g.value == 0.0 })
            .collect()
    }
}

/// One labelled position on an axis.
#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    /// Position along the axis, 0..=1 (from the bottom for values, from the start for categories).
    pub at: f64,
    /// The label.
    pub text: String,
    /// The zero line, drawn a little stronger.
    pub baseline: bool,
}

/// Category axis labels at the centre of each band, thinned to at most `max`.
#[allow(clippy::cast_precision_loss, reason = "category counts are small")]
#[must_use]
pub fn category_ticks(names: &[String], max: usize) -> Vec<Tick> {
    let n = names.len().max(1) as f64;
    line::label_indices(names.len(), max)
        .into_iter()
        .map(|i| Tick { at: (i as f64 + 0.5) / n, text: names[i].clone(), baseline: false })
        .collect()
}

/// Labels for evenly spaced line positions (see [`line::x_positions`]), thinned to at most `max`.
#[must_use]
pub fn point_ticks(names: &[String], xs: &[f64], max: usize) -> Vec<Tick> {
    line::label_indices(names.len().min(xs.len()), max)
        .into_iter()
        .map(|i| Tick { at: xs[i], text: names[i].clone(), baseline: false })
        .collect()
}

/// The smallest gap between neighbouring ticks (0..=1), so labels can be kept from overlapping.
#[must_use]
pub fn tick_room(ticks: &[Tick]) -> f64 {
    ticks.windows(2).map(|w| (w[1].at - w[0].at).abs()).fold(1.0, f64::min)
}

/// Left gutter in px that fits the widest y tick label (11px monospace, about 6.8px a
/// character, plus the 8px gap and a little air), and never less than `min`.
#[allow(clippy::cast_precision_loss, reason = "label lengths are short")]
#[must_use]
pub fn axis_width<'a>(labels: impl IntoIterator<Item = &'a str>, min: f64) -> f64 {
    let widest = labels.into_iter().map(|l| l.chars().count()).max().unwrap_or(0);
    (widest as f64 * 6.8 + 12.0).ceil().max(min)
}

/// A legend entry.
#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    /// The series name.
    pub name: String,
    /// A CSS colour.
    pub colour: String,
    /// Drawn as a dashed line rather than a solid swatch.
    pub dashed: bool,
}

impl Key {
    /// A solid legend entry.
    #[must_use]
    pub fn new(name: impl Into<String>, colour: impl Into<String>) -> Self {
        Self { name: name.into(), colour: colour.into(), dashed: false }
    }
}

/// One row of a tooltip.
#[derive(Debug, Clone, PartialEq)]
pub struct TipRow {
    /// Swatch colour, if the row belongs to a series.
    pub colour: Option<String>,
    /// What the value is.
    pub name: String,
    /// The formatted value.
    pub value: String,
    /// The row for the mark under the pointer or focus.
    pub active: bool,
}

impl TipRow {
    /// A row with a swatch.
    #[must_use]
    pub fn new(colour: Option<String>, name: impl Into<String>, value: impl Into<String>) -> Self {
        Self { colour, name: name.into(), value: value.into(), active: false }
    }
}

/// Tooltip content and position.
#[derive(Debug, Clone, PartialEq)]
pub struct Tip {
    /// Anchor from the left of the plot, 0..=1.
    pub x: f64,
    /// Anchor from the bottom of the plot, 0..=1.
    pub y: f64,
    /// The heading, usually the category or x label.
    pub title: String,
    /// The values.
    pub rows: Vec<TipRow>,
}

impl Tip {
    /// The `data-side` value: the tooltip opens away from the nearer edge.
    #[must_use]
    pub fn side(&self) -> &'static str {
        if self.x > 0.5 { "left" } else { "right" }
    }

    /// The `data-vertical` value.
    #[must_use]
    pub fn vertical(&self) -> &'static str {
        if self.y > 0.5 { "below" } else { "above" }
    }

    /// What the live region says for this tooltip.
    #[must_use]
    /// Only the active row when there is one, otherwise every row.
    pub fn announce(&self) -> String {
        let active: Vec<&TipRow> = self.rows.iter().filter(|r| r.active).collect();
        let rows = if active.is_empty() { self.rows.iter().collect() } else { active };
        let rows: Vec<String> = rows
            .iter()
            .map(|r| if r.name.is_empty() { r.value.clone() } else { format!("{} {}", r.name, r.value) })
            .collect();
        format!("{}: {}", self.title, rows.join(", "))
    }
}

/// Which of `n` equal bands contains position `p` (0..=1). `None` when there are none.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "band counts are small"
)]
#[must_use]
pub fn band_at(p: f64, n: usize) -> Option<usize> {
    (n > 0).then(|| ((p.clamp(0.0, 1.0) * n as f64) as usize).min(n - 1))
}

/// Centre of band `i` of `n`, 0..=1.
#[allow(clippy::cast_precision_loss, reason = "band counts are small")]
#[must_use]
pub fn band_centre(i: usize, n: usize) -> f64 {
    (i as f64 + 0.5) / n.max(1) as f64
}

/// Where a category index sits along the axis, 0..=1. Fractional indices fall between
/// categories, so an annotation at 1.5 sits between the second and third.
#[allow(clippy::cast_precision_loss, reason = "band counts are small")]
#[must_use]
pub fn category_at(x: f64, n: usize) -> f64 {
    (x + 0.5) / n.max(1) as f64
}

/// Where a point index sits along a line chart's x axis (see [`line::x_positions`]), 0..=1.
/// Fractional indices fall between points, for annotations.
#[allow(clippy::cast_precision_loss, reason = "point counts are small")]
#[must_use]
pub fn index_at(x: f64, n: usize) -> f64 {
    if n > 1 { x / (n - 1) as f64 } else { 0.5 }
}

/// Arrow keys for a chart whose categories run down the page: Down/Up step through
/// categories and Left/Right across series, so they swap places before [`crate::nav`] sees them.
#[must_use]
pub fn turn_key(key: &str) -> &str {
    match key {
        "ArrowDown" => "ArrowRight",
        "ArrowRight" => "ArrowDown",
        "ArrowUp" => "ArrowLeft",
        "ArrowLeft" => "ArrowUp",
        other => other,
    }
}

/// Place annotations given in data units. `x` maps an annotation's x (a category index
/// or numeric x) to 0..=1 along the category axis; `y` maps values. When `horizontal`, the
/// category axis runs down the page (first at the top) and values run left to right.
pub fn place(
    annotations: &[Annotation],
    horizontal: bool,
    x: impl Fn(f64) -> f64,
    y: impl Fn(f64) -> f64,
) -> Vec<Placed> {
    if !horizontal {
        return annotate::place(annotations, x, y);
    }
    let turned: Vec<Annotation> = annotations
        .iter()
        .map(|a| match a.clone() {
            Annotation::YLine { value, label } => Annotation::XLine { at: value, label },
            Annotation::YBand { from, to, label } => Annotation::XBand { from, to, label },
            Annotation::XLine { at, label } => Annotation::YLine { value: at, label },
            Annotation::XBand { from, to, label } => Annotation::YBand { from, to, label },
            Annotation::Point { x, y, label } => Annotation::Point { x: y, y: x, label },
        })
        .collect();
    annotate::place(&turned, y, |c| 1.0 - x(c))
}

/// Keep at most `max` series: the `max - 1` largest by total stay in order, the rest sum into "Other".
#[must_use]
pub fn fold_series(series: &[Series], max: usize) -> Vec<Series> {
    if series.len() <= max || max < 2 {
        return series.to_vec();
    }
    let total = |s: &Series| s.values.iter().filter(|v| v.is_finite()).sum::<f64>();
    let mut by_size: Vec<usize> = (0..series.len()).collect();
    by_size.sort_by(|&a, &b| total(&series[b]).total_cmp(&total(&series[a])));
    let keep = &by_size[..max - 1];
    let len = series.iter().map(|s| s.values.len()).max().unwrap_or(0);
    let mut out: Vec<Series> = (0..series.len()).filter(|i| keep.contains(i)).map(|i| series[i].clone()).collect();
    let rest = (0..len)
        .map(|j| {
            (0..series.len())
                .filter(|i| !keep.contains(i))
                .filter_map(|i| series[i].values.get(j).copied().filter(|v| v.is_finite()))
                .sum()
        })
        .collect::<Vec<f64>>();
    out.push(Series::new("Other", rest));
    out
}

/// Each value as a point in the 100×100 drawing box, `None` for gaps.
#[must_use]
pub fn drawn(values: &[f64], xs: &[f64], axis: &ValueAxis) -> Vec<Option<(f64, f64)>> {
    xs.iter()
        .enumerate()
        .map(|(i, &x)| {
            values.get(i).copied().filter(|v| v.is_finite()).map(|v| (x * 100.0, (1.0 - axis.scale.map(v)) * 100.0))
        })
        .collect()
}

/// SVG `d` through `points`, breaking at gaps.
#[must_use]
pub fn line_path(points: &[Option<(f64, f64)>], curve: Curve) -> String {
    line::runs(points).iter().map(|r| line::path(r, curve)).collect()
}

/// Height of each value as a fraction of the plot, `None` for gaps.
#[must_use]
pub fn heights(values: &[f64], len: usize, axis: &ValueAxis) -> Vec<Option<f64>> {
    (0..len).map(|i| values.get(i).copied().filter(|v| v.is_finite()).map(|v| axis.scale.map(v))).collect()
}

/// A value formatted, or an en dash for gaps.
pub fn cell(value: Option<f64>, format: impl Fn(f64) -> String) -> String {
    value.filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_includes_zero_and_annotations() {
        let a = ValueAxis::new(ScaleKind::Linear, [12.0, 87.0], &[Annotation::y_line(140.0, "Goal")], true);
        assert_eq!(a.scale.domain, (0.0, 150.0));
        assert!((a.at(75.0) - 0.5).abs() < 1e-9);
        assert_eq!(a.base(), 0.0);
        assert_eq!(a.ticks(|v| format!("{v}"))[0].text, "0");
    }

    #[test]
    fn log_axis_skips_non_positive_values() {
        let a = ValueAxis::new(ScaleKind::Log, [0.0, -3.0, 5.0, 900.0], &[], true);
        assert_eq!(a.scale.domain, (1.0, 1000.0));
        assert_eq!(a.base(), 0.0);
        let empty = ValueAxis::new(ScaleKind::Linear, [f64::NAN], &[], false);
        assert!(empty.scale.domain.1 > empty.scale.domain.0);
    }

    #[test]
    fn bands_pick_and_centre() {
        assert_eq!(band_at(0.0, 4), Some(0));
        assert_eq!(band_at(0.49, 4), Some(1));
        assert_eq!(band_at(1.0, 4), Some(3));
        assert_eq!(band_at(0.5, 0), None);
        assert!((band_centre(1, 4) - 0.375).abs() < 1e-12);
        assert!((index_at(1.5, 4) - 0.5).abs() < 1e-12);
        assert!((index_at(0.0, 1) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn horizontal_annotations_turn() {
        let placed = place(&[Annotation::y_line(50.0, "Target")], true, |c| c / 4.0, |v| v / 100.0);
        assert_eq!(placed[0].shape, annotate::Shape::VLine);
        assert!((placed[0].x0 - 0.5).abs() < 1e-12);
        assert_eq!(turn_key("ArrowDown"), "ArrowRight");
        assert_eq!(turn_key("Home"), "Home");
    }

    #[test]
    fn folds_small_series_into_other() {
        let s: Vec<Series> = (0..8).map(|i| Series::new(format!("s{i}"), [f64::from(i), 1.0])).collect();
        let f = fold_series(&s, 6);
        assert_eq!(f.len(), 6);
        assert_eq!(f[5].name, "Other");
        // s0, s1 and s2 are the smallest: (0 + 1 + 2, 1 + 1 + 1).
        assert_eq!(f[5].values, vec![3.0, 3.0]);
        assert_eq!(f[0].name, "s3");
    }

    #[test]
    fn tips_flip_and_announce() {
        let t = Tip { x: 0.8, y: 0.2, title: "Q1".into(), rows: vec![TipRow::new(None, "Sales", "12")] };
        assert_eq!((t.side(), t.vertical()), ("left", "above"));
        assert_eq!(t.announce(), "Q1: Sales 12");
        let both = Tip {
            rows: vec![TipRow::new(None, "A", "1"), TipRow { active: true, ..TipRow::new(None, "B", "2") }],
            ..t.clone()
        };
        assert_eq!(both.announce(), "Q1: B 2", "only the active row is read");
        assert_eq!(colour(9), "var(--chart-6)");
        assert_eq!(cell(Some(f64::NAN), |v| v.to_string()), "–");
    }

    #[test]
    fn pct_and_muted_colours() {
        assert_eq!(pct(0.5), "50.000%");
        assert_eq!(pct(1.0 / 3.0), "33.333%");
        assert_eq!(colour_or_muted(0, 6), "var(--chart-1)");
        assert_eq!(colour_or_muted(5, 6), "var(--chart-6)");
        assert_eq!(colour_or_muted(6, 6), "var(--chart-muted)");
        assert_eq!(colour_or_muted(4, 5), "var(--chart-5)");
        assert_eq!(colour_or_muted(5, 5), "var(--chart-muted)");
        assert_eq!(colour_or_muted(3, 99), "var(--chart-4)");
        assert_eq!(colour_or_muted(7, 99), "var(--chart-muted)");
    }

    #[test]
    fn axis_width_grows_with_long_labels() {
        assert!((axis_width(["0", "10"], 36.0) - 36.0).abs() < 1e-9);
        assert!((axis_width(["3.00 mm/s"], 36.0) - 74.0).abs() < 1e-9);
        assert!((axis_width([], 40.0) - 40.0).abs() < 1e-9);
    }
}
