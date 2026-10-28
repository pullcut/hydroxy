//! Hot paths of the virtual list layout at 100k rows.
#![allow(missing_docs, clippy::cast_precision_loss, reason = "benchmark")]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hydroxy_virtual::Layout;

fn layout(c: &mut Criterion) {
    let uniform = Layout::fixed(100_000, 36.0);
    let measured = Layout::from_fn(100_000, 36.0, |i| 24.0 + (i % 7) as f64 * 4.0);
    c.bench_function("range/fixed", |b| b.iter(|| uniform.range(black_box(1_800_000.0), 600.0, 4)));
    c.bench_function("range/measured", |b| b.iter(|| measured.range(black_box(1_800_000.0), 600.0, 4)));
    c.bench_function("index_at/measured", |b| b.iter(|| measured.index_at(black_box(2_500_000.0))));
    c.bench_function("from_fn/100k", |b| {
        b.iter(|| Layout::from_fn(100_000, 36.0, |i| 24.0 + (i % 7) as f64 * 4.0));
    });
}

criterion_group!(benches, layout);
criterion_main!(benches);
