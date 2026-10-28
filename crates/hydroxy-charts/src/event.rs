//! What a chart reports about the mark the user is on: hover, select and view changes.
//! Framework-neutral; the adapters wrap these in `Callback` / `EventHandler`.

use crate::candle::Candle;
use crate::hierarchy::Tree;
use crate::line::Series;
use crate::scatter::{Group, Point};

pub use crate::zoom::View;

/// A node in a hierarchy chart: child indices from the root.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeRef {
    /// Child indices from the root; empty is the root itself.
    pub path: Vec<usize>,
}

/// The mark an event is about. Not to be confused with [`crate::Datum`], a labelled input value.
#[derive(Debug, Clone, PartialEq)]
pub struct Datum {
    /// Index into the series (or groups) passed in; 0 for single-series charts.
    pub series: usize,
    /// Index into that series' data. For hierarchies, the node's index among its siblings.
    pub index: usize,
    /// The category or x label, when there is one.
    pub label: Option<String>,
    /// The y value, or the node value for hierarchies.
    pub value: f64,
    /// Set by hierarchy charts: the node's path.
    pub node: Option<NodeRef>,
}

impl Datum {
    /// A mark in a single-series chart (bars, pies, funnels).
    #[must_use]
    pub fn category(index: usize, label: impl Into<String>, value: f64) -> Self {
        Self { series: 0, index, label: Some(label.into()), value, node: None }
    }

    /// The active point of a line, area or grouped chart. `label` is the x label at `index`.
    /// `None` when either index is out of range.
    #[must_use]
    pub fn from_series(all: &[Series], series: usize, index: usize, label: Option<&str>) -> Option<Self> {
        let value = *all.get(series)?.values.get(index)?;
        Some(Self { series, index, label: label.map(str::to_owned), value, node: None })
    }

    /// The active dot of a scatter chart; the label is the group name and the value is `y`.
    #[must_use]
    pub fn from_point(groups: &[Group], group: usize, index: usize) -> Option<Self> {
        let Point { y, .. } = *groups.get(group)?.points.get(index)?;
        Some(Self { series: group, index, label: Some(groups[group].name.clone()), value: y, node: None })
    }

    /// The active candle; the value is the close.
    #[must_use]
    pub fn from_candle(candles: &[Candle], index: usize, label: Option<&str>) -> Option<Self> {
        let value = candles.get(index)?.close;
        Some(Self { series: 0, index, label: label.map(str::to_owned), value, node: None })
    }

    /// The active node of a hierarchy chart. An empty path is the root (index 0).
    #[must_use]
    pub fn from_node(tree: &Tree, path: &[usize]) -> Option<Self> {
        let node = tree.node_at(path)?;
        Some(Self {
            series: 0,
            index: path.last().copied().unwrap_or(0),
            label: Some(node.name.clone()),
            value: node.total(),
            node: Some(NodeRef { path: path.to_vec() }),
        })
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, reason = "exact small literals")]
mod tests {
    use super::*;

    #[test]
    fn category_and_series() {
        let d = Datum::category(2, "c", 7.0);
        assert_eq!((d.series, d.index, d.label.as_deref(), d.value), (0, 2, Some("c"), 7.0));
        let s = [Series::new("a", vec![1.0, 2.0]), Series::new("b", vec![3.0, 4.0])];
        let d = Datum::from_series(&s, 1, 0, Some("Mon")).unwrap();
        assert_eq!((d.series, d.index, d.value), (1, 0, 3.0));
        assert_eq!(d.label.as_deref(), Some("Mon"));
        assert!(Datum::from_series(&s, 2, 0, None).is_none());
        assert!(Datum::from_series(&s, 0, 2, None).is_none());
    }

    #[test]
    fn scatter_and_candle() {
        let g = [Group::new("g", vec![Point::new(1.0, 5.0), Point::new(2.0, 6.0)])];
        let d = Datum::from_point(&g, 0, 1).unwrap();
        assert_eq!((d.index, d.value, d.label.as_deref()), (1, 6.0, Some("g")));
        assert!(Datum::from_point(&g, 0, 2).is_none());
        let c = [Candle::new(1.0, 3.0, 0.5, 2.0)];
        let d = Datum::from_candle(&c, 0, None).unwrap();
        assert_eq!((d.value, d.label), (2.0, None));
        assert!(Datum::from_candle(&c, 1, None).is_none());
    }

    #[test]
    fn hierarchy_node() {
        let t = Tree::branch("root", vec![Tree::leaf("a", 1.0), Tree::branch("b", vec![Tree::leaf("c", 4.0)])]);
        let d = Datum::from_node(&t, &[1, 0]).unwrap();
        assert_eq!((d.index, d.value, d.label.as_deref()), (0, 4.0, Some("c")));
        assert_eq!(d.node, Some(NodeRef { path: vec![1, 0] }));
        assert_eq!(Datum::from_node(&t, &[]).unwrap().value, 5.0);
        assert!(Datum::from_node(&t, &[9]).is_none());
    }
}
