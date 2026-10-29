//! Property test: [`Layout`] agrees with a plain prefix-sum model under random sizes,
//! `set_size` and `set_len` calls.

#![allow(clippy::float_cmp, reason = "sizes are whole pixels, so every sum is exact")]

use hydroxy_virtual::Layout;
use proptest::prelude::*;

/// What `Layout` does to a size it is given: at least one pixel, 1 when not finite.
fn normalize(size: f64) -> f64 {
    if size.is_finite() { size.max(1.0) } else { 1.0 }
}

#[derive(Debug, Clone)]
enum Op {
    SetSize(usize, f64),
    SetLen(usize),
}

/// Whole-pixel sizes (so sums are exact), plus the junk `Layout` must normalise.
fn size() -> impl Strategy<Value = f64> {
    prop_oneof![
        8 => (0..200_u32).prop_map(f64::from),
        1 => Just(f64::NAN),
        1 => Just(f64::INFINITY),
        1 => Just(-5.0),
    ]
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        4 => (0..40_usize, size()).prop_map(|(i, s)| Op::SetSize(i, s)),
        1 => (0..40_usize).prop_map(Op::SetLen),
    ]
}

fn check(layout: &Layout, model: &[f64]) {
    assert_eq!(layout.len(), model.len());
    let mut offset = 0.0;
    for (i, &s) in model.iter().enumerate() {
        assert_eq!(layout.size(i), s, "size({i})");
        assert_eq!(layout.offset(i), offset, "offset({i})");
        // The row covering a point just inside it.
        assert_eq!(layout.index_at(offset + s / 2.0), i, "index_at inside row {i}");
        offset += s;
    }
    assert_eq!(layout.total(), offset);
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn layout_matches_prefix_sums(
        sizes in prop::collection::vec(size(), 0..40),
        estimate in size(),
        ops in prop::collection::vec(op(), 0..30),
    ) {
        let mut layout = Layout::from_fn(sizes.len(), estimate, |i| sizes[i]);
        let estimate = normalize(estimate);
        let mut model: Vec<f64> = sizes.iter().copied().map(normalize).collect();
        check(&layout, &model);
        for op in ops {
            match op {
                Op::SetSize(i, s) => {
                    let moved = layout.set_size(i, s);
                    let changed = i < model.len() && model[i] != normalize(s);
                    prop_assert_eq!(moved, changed);
                    if changed {
                        model[i] = normalize(s);
                    }
                }
                Op::SetLen(n) => {
                    layout.set_len(n);
                    model.resize(n, estimate);
                }
            }
            check(&layout, &model);
        }
    }
}
