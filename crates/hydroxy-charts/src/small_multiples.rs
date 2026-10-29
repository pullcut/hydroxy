//! Small multiples: one chart per facet, every facet on the same value axis so their
//! heights compare directly. This module finds that shared domain.

use crate::line::Series;

/// The chart each facet draws.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FacetKind {
    /// Lines over a shared x axis.
    #[default]
    Line,
    /// Lines with the area under them filled.
    Area,
    /// Series stacked from zero.
    StackedArea,
    /// Bars of the first series.
    Bar,
}

impl FacetKind {
    /// The `data-kind` value.
    #[must_use]
    pub const fn attr(self) -> &'static str {
        match self {
            FacetKind::Line => "line",
            FacetKind::Area => "area",
            FacetKind::StackedArea => "stacked-area",
            FacetKind::Bar => "bar",
        }
    }
}

/// One panel: a title and its series.
#[derive(Debug, Clone, PartialEq)]
pub struct Facet {
    /// Shown above the panel, such as a region or product name.
    pub title: String,
    /// The panel's series, in the same order (and so the same colours) in every facet.
    pub series: Vec<Series>,
}

impl Facet {
    /// A titled panel.
    #[must_use]
    pub fn new(title: impl Into<String>, series: Vec<Series>) -> Self {
        Self { title: title.into(), series }
    }
}

/// The value range every facet must show, or `None` when there are no finite values.
/// Stacked facets count each x position's stacked total (negatives as zero, as the stack
/// does); bar facets only their first series, which is all they draw.
#[must_use]
pub fn shared_domain(facets: &[Facet], kind: FacetKind) -> Option<(f64, f64)> {
    let finite = |v: &f64| v.is_finite();
    let values: Vec<f64> = match kind {
        FacetKind::Line | FacetKind::Area => {
            facets.iter().flat_map(|f| f.series.iter().flat_map(|s| s.values.iter().copied())).filter(finite).collect()
        }
        FacetKind::Bar => facets
            .iter()
            .filter_map(|f| f.series.first())
            .flat_map(|s| s.values.iter().copied())
            .filter(finite)
            .collect(),
        FacetKind::StackedArea => facets
            .iter()
            .flat_map(|f| {
                let len = f.series.iter().map(|s| s.values.len()).max().unwrap_or(0);
                (0..len).map(|i| {
                    f.series
                        .iter()
                        .filter_map(|s| s.values.get(i).copied().filter(finite))
                        .map(|v| v.max(0.0))
                        .sum::<f64>()
                })
            })
            .collect(),
    };
    let lo = values.iter().copied().reduce(f64::min)?;
    let hi = values.iter().copied().reduce(f64::max)?;
    Some((lo, hi))
}

/// `extent` grown to cover `domain` too. Charts take a shared domain this way: it
/// joins the annotation extent their value axis already fits.
#[must_use]
pub fn widen(extent: Option<(f64, f64)>, domain: Option<(f64, f64)>) -> Option<(f64, f64)> {
    match (extent, domain) {
        (Some((a, b)), Some((c, d))) => Some((a.min(c), b.max(d))),
        (e, d) => e.or(d),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facets() -> Vec<Facet> {
        vec![
            Facet::new("North", vec![Series::new("A", [1.0, 4.0]), Series::new("B", [2.0, f64::NAN])]),
            Facet::new("South", vec![Series::new("A", [-3.0, 9.0]), Series::new("B", [5.0, 1.0])]),
        ]
    }

    #[test]
    fn line_domain_spans_every_facet() {
        assert_eq!(shared_domain(&facets(), FacetKind::Line), Some((-3.0, 9.0)));
        assert_eq!(shared_domain(&facets(), FacetKind::Bar), Some((-3.0, 9.0)));
        assert_eq!(shared_domain(&[], FacetKind::Line), None);
    }

    #[test]
    fn stacked_domain_uses_totals() {
        // North: 3, 4. South: max(-3,0)+5 = 5, 9+1 = 10.
        assert_eq!(shared_domain(&facets(), FacetKind::StackedArea), Some((3.0, 10.0)));
    }

    #[test]
    fn widen_joins_extents() {
        assert_eq!(widen(Some((0.0, 5.0)), Some((-1.0, 3.0))), Some((-1.0, 5.0)));
        assert_eq!(widen(None, Some((1.0, 2.0))), Some((1.0, 2.0)));
        assert_eq!(widen(Some((1.0, 2.0)), None), Some((1.0, 2.0)));
        assert_eq!(widen(None, None), None);
    }
}
