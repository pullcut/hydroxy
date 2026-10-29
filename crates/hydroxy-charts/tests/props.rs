//! Property tests: every layout finishes on any input, NaN and infinity included, and
//! never emits non-finite geometry.

use hydroxy_charts::hierarchy::Tree;
use hydroxy_charts::line::{self, Curve};
use hydroxy_charts::pie::{self, Labels};
use hydroxy_charts::scatter::{self, Group, Point};
use hydroxy_charts::{bar, boxplot, delaunay, donut, histogram, sankey, scale, treemap, venn, voronoi};
use proptest::prelude::*;

/// Panics if `out`'s `Debug` text holds a NaN or an infinity.
fn assert_finite(what: &str, out: &impl std::fmt::Debug) {
    let text = format!("{out:?}");
    let bad = text.split(|c: char| !c.is_alphanumeric()).find(|w| w.ends_with("NaN") || w.ends_with("inf"));
    assert!(bad.is_none(), "{what} emitted {bad:?}: {text}");
}

fn values() -> impl Strategy<Value = Vec<f64>> {
    prop::collection::vec(any::<f64>(), 0..12)
}

/// A drawing size: finite, zero included.
fn size() -> impl Strategy<Value = f64> {
    0.0..2000.0_f64
}

const NAMES: [&str; 4] = ["a", "b", "c", "d"];

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn ticks_are_bounded(a in any::<f64>(), b in any::<f64>(), n in 0..20_usize) {
        prop_assert!(scale::ticks(a, b, n).len() <= 4 * n.max(2) + 2);
    }

    #[test]
    fn line_layout_is_finite(series in prop::collection::vec(values(), 0..4), curve in prop_oneof![Just(Curve::Linear), Just(Curve::Monotone)]) {
        let len = series.iter().map(Vec::len).max().unwrap_or(0);
        let xs = line::x_positions(len, None);
        assert_finite("line::layout", &line::layout(&series, &xs, curve, 5));
    }

    #[test]
    fn scatter_layout_is_finite(points in prop::collection::vec((any::<f64>(), any::<f64>(), prop::option::of(any::<f64>())), 0..12)) {
        let points = points.into_iter().map(|(x, y, size)| Point { x, y, size }).collect();
        assert_finite("scatter::layout", &scatter::layout(&[Group::new("g", points)], 5));
    }

    #[test]
    fn histogram_is_finite(v in values(), count in prop::option::of(0..40_usize)) {
        let bins = histogram::bins(&v, count);
        assert_finite("histogram::bins", &bins);
        assert_finite("histogram::layout", &histogram::layout(&bins, 5));
    }

    #[test]
    fn boxplot_is_finite(groups in prop::collection::vec(values(), 0..4)) {
        assert_finite("boxplot::layout", &boxplot::layout(&groups, 5));
    }

    #[test]
    fn bar_is_finite(v in values(), highlight in prop::option::of(0..12_usize)) {
        assert_finite("bar::layout", &bar::layout(&v, highlight, 5));
    }

    #[test]
    fn pie_and_donut_are_finite(v in values(), s in size(), gap in 0.0..20.0_f64) {
        assert_finite("pie::layout", &pie::layout(&v, s, gap, Labels::Leader));
        assert_finite("donut::segments", &donut::segments(&v, s, s / 4.0, gap));
    }

    #[test]
    fn venn_is_finite(data in prop::collection::vec((prop::sample::subsequence(NAMES.to_vec(), 1..=3), any::<f64>()), 0..8), w in size(), h in size()) {
        let data: Vec<_> = data.into_iter().map(|(sets, count)| venn::Intersection::new(sets, count)).collect();
        assert_finite("venn::layout", &venn::layout(&data, w, h));
    }

    #[test]
    fn sankey_is_finite(links in prop::collection::vec((0..4_usize, 0..4_usize, any::<f64>()), 0..8), w in size(), h in size()) {
        let links: Vec<_> = links.into_iter().map(|(s, t, v)| sankey::Link::new(NAMES[s], NAMES[t], v)).collect();
        assert_finite("sankey::layout", &sankey::layout(&links, w, h, 12.0, 8.0, 6));
    }

    #[test]
    fn delaunay_and_voronoi_are_finite(points in prop::collection::vec((any::<f64>(), any::<f64>()), 0..10), w in size(), h in size()) {
        let tris = delaunay::triangulate(&points);
        prop_assert!(tris.iter().flatten().all(|&i| i < points.len()));
        assert_finite("delaunay::voronoi", &delaunay::voronoi(&points, w, h));
    }

    #[test]
    fn voronoi_treemap_and_treemap_are_finite(leaves in prop::collection::vec(any::<f64>(), 0..8), w in size(), h in size()) {
        let root = Tree::branch("root", leaves.iter().enumerate().map(|(i, &v)| Tree::leaf(i.to_string(), v)).collect());
        assert_finite("voronoi::layout", &voronoi::layout(&root, w, h, voronoi::Shape::Circle, 2));
        assert_finite("treemap::layout", &treemap::layout(&root, &[], w, h));
    }
}
