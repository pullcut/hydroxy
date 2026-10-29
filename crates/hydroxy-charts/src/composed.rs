//! Composed charts, ported from hecto's `ComposedChart`: layers of bars, lines, areas and
//! points over shared categories and one value axis. Bar layers sit side by side within
//! each category; lines, areas and points run through the category centres.
//!
//! Bars and points are plot fractions (0..=1, y up) for CSS; lines and areas are SVG paths
//! in a 100×100 box (y down), like [`crate::line`].

use crate::bar::Gridline;
use crate::line::{self, Curve};
use crate::scale::{ScaleKind, ValueScale};

/// How a layer is drawn.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum LayerKind {
    /// A bar per category.
    #[default]
    Bar,
    /// A line through the category centres.
    Line,
    /// A line with the area under it filled.
    Area,
    /// A dot per category.
    Points,
}

impl LayerKind {
    /// The `data-kind` value.
    #[must_use]
    pub const fn attr(self) -> &'static str {
        match self {
            LayerKind::Bar => "bar",
            LayerKind::Line => "line",
            LayerKind::Area => "area",
            LayerKind::Points => "points",
        }
    }
}

/// One named layer: a value per category. Non-finite values are gaps.
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    /// Shown in the legend, tooltip and table.
    pub name: String,
    /// How it is drawn.
    pub kind: LayerKind,
    /// One value per category, in order.
    pub values: Vec<f64>,
}

impl Layer {
    /// A layer of any kind.
    #[must_use]
    pub fn new(kind: LayerKind, name: impl Into<String>, values: impl Into<Vec<f64>>) -> Self {
        Self { name: name.into(), kind, values: values.into() }
    }

    /// A bar layer.
    #[must_use]
    pub fn bar(name: impl Into<String>, values: impl Into<Vec<f64>>) -> Self {
        Self::new(LayerKind::Bar, name, values)
    }

    /// A line layer.
    #[must_use]
    pub fn line(name: impl Into<String>, values: impl Into<Vec<f64>>) -> Self {
        Self::new(LayerKind::Line, name, values)
    }

    /// An area layer.
    #[must_use]
    pub fn area(name: impl Into<String>, values: impl Into<Vec<f64>>) -> Self {
        Self::new(LayerKind::Area, name, values)
    }

    /// A points layer.
    #[must_use]
    pub fn points(name: impl Into<String>, values: impl Into<Vec<f64>>) -> Self {
        Self::new(LayerKind::Points, name, values)
    }

    /// The value at category `i`, if finite.
    #[must_use]
    pub fn value(&self, i: usize) -> Option<f64> {
        self.values.get(i).copied().filter(|v| v.is_finite())
    }
}

/// A bar, in plot fractions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarRect {
    /// Left edge.
    pub left: f64,
    /// Width.
    pub width: f64,
    /// Bottom edge (the zero line for positive values, the value for negative ones).
    pub bottom: f64,
    /// Height.
    pub height: f64,
    /// Below zero: the bar hangs down from the zero line.
    pub negative: bool,
}

/// One layer, ready to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerGeom {
    /// SVG `d` for a line or area layer's line; empty for bars and points.
    pub line: String,
    /// SVG `d` for an area layer's fill; empty otherwise.
    pub area: String,
    /// Height of each value (0 = bottom), for dots and the crosshair; `None` for gaps.
    pub points: Vec<Option<f64>>,
    /// A bar layer's bars, one per category; empty for other kinds.
    pub bars: Vec<Option<BarRect>>,
}

/// Everything a composed chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct ComposedLayout {
    /// Horizontal centre of each category, 0..=1.
    pub centres: Vec<f64>,
    /// One per layer, in order.
    pub layers: Vec<LayerGeom>,
    /// Gridlines from the bottom up.
    pub gridlines: Vec<Gridline>,
    /// The value axis, for placing annotations.
    pub scale: ValueScale,
}

/// Lay out `layers` over `categories` columns. The value axis includes zero (bars start
/// there) and widens to cover `extent` (such as [`crate::annotate::y_extent`]).
#[allow(clippy::cast_precision_loss, reason = "category and layer counts are small")]
#[must_use]
pub fn layout(
    layers: &[Layer],
    categories: usize,
    curve: Curve,
    tick_count: usize,
    extent: Option<(f64, f64)>,
) -> ComposedLayout {
    let values = || {
        layers
            .iter()
            .flat_map(|l| (0..categories).filter_map(|i| l.value(i)))
            .chain(extent.into_iter().flat_map(|(a, b)| [a, b]))
    };
    let lo = values().filter(|v| v.is_finite()).fold(0.0, f64::min);
    let hi = values().filter(|v| v.is_finite()).fold(0.0, f64::max);
    let scale = ValueScale::new(ScaleKind::Linear, lo, if hi > lo { hi } else { lo + 1.0 }, tick_count);
    let slot = 1.0 / categories.max(1) as f64;
    let centres: Vec<f64> = (0..categories).map(|i| (i as f64 + 0.5) * slot).collect();
    let zero = scale.map(0.0).clamp(0.0, 1.0);
    let bar_layers: Vec<usize> =
        layers.iter().enumerate().filter(|(_, l)| l.kind == LayerKind::Bar).map(|(i, _)| i).collect();
    let group = slot * 0.7;
    let bar_width = group / bar_layers.len().max(1) as f64;

    let geoms = layers
        .iter()
        .enumerate()
        .map(|(li, layer)| {
            let points: Vec<Option<f64>> = (0..categories).map(|i| layer.value(i).map(|v| scale.map(v))).collect();
            let mut geom = LayerGeom { line: String::new(), area: String::new(), points, bars: Vec::new() };
            match layer.kind {
                LayerKind::Bar => {
                    let j = bar_layers.iter().position(|&b| b == li).unwrap_or(0) as f64;
                    geom.bars = geom
                        .points
                        .iter()
                        .zip(&centres)
                        .map(|(p, c)| {
                            p.map(|f| BarRect {
                                left: c - group / 2.0 + j * bar_width,
                                width: bar_width,
                                bottom: f.min(zero),
                                height: (f - zero).abs(),
                                negative: f < zero,
                            })
                        })
                        .collect();
                }
                LayerKind::Line | LayerKind::Area => {
                    let drawn: Vec<Option<(f64, f64)>> = geom
                        .points
                        .iter()
                        .zip(&centres)
                        .map(|(p, &c)| p.map(|f| (c * 100.0, (1.0 - f) * 100.0)))
                        .collect();
                    let runs = line::runs(&drawn);
                    geom.line = runs.iter().map(|r| line::path(r, curve)).collect();
                    if layer.kind == LayerKind::Area {
                        geom.area = runs.iter().map(|r| line::area(r, (1.0 - zero) * 100.0, curve)).collect();
                    }
                }
                LayerKind::Points => {}
            }
            geom
        })
        .collect();

    ComposedLayout {
        centres,
        layers: geoms,
        gridlines: scale
            .ticks(tick_count)
            .into_iter()
            .map(|value| Gridline { value, fraction: scale.map(value) })
            .collect(),
        scale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_sit_side_by_side_and_lines_run_through_centres() {
        let layers = [
            Layer::bar("Orders", [40.0, 60.0]),
            Layer::bar("Returns", [-10.0, 5.0]),
            Layer::line("Target", [50.0, f64::NAN]),
        ];
        let l = layout(&layers, 2, Curve::Linear, 5, None);
        assert_eq!(l.centres, vec![0.25, 0.75]);
        let (a, b) = (
            l.layers[0].bars[0].unwrap_or_else(|| unreachable!()),
            l.layers[1].bars[0].unwrap_or_else(|| unreachable!()),
        );
        assert!((a.left + a.width - b.left).abs() < 1e-12, "the second bar starts where the first ends");
        assert!(b.bottom < a.bottom && b.negative && !a.negative, "negative bars hang below zero");
        assert!(l.layers[2].line.starts_with("M25.000") && l.layers[2].points[1].is_none());
        assert_eq!(l.layers[2].bars.len(), 0);
        assert!(l.scale.domain.0 < 0.0 && l.scale.domain.1 >= 60.0);
    }

    #[test]
    fn areas_close_at_zero_and_extents_widen_the_axis() {
        let l = layout(&[Layer::area("Visits", [10.0, 20.0, 30.0])], 3, Curve::Linear, 5, Some((0.0, 80.0)));
        assert!(l.layers[0].area.ends_with('Z'));
        assert!(l.scale.domain.1 >= 80.0);
        assert_eq!(LayerKind::Points.attr(), "points");
    }
}
