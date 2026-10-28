//! Hot paths of the chart maths: decimation, canvas scatter, Venn and Voronoi layout.
#![allow(missing_docs, clippy::cast_precision_loss, reason = "benchmark")]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hydroxy_charts::hierarchy::Tree;
use hydroxy_charts::raster::{self, Rgba};
use hydroxy_charts::scatter::Dot;
use hydroxy_charts::venn::{self, Intersection};
use hydroxy_charts::{line, voronoi};

fn decimate(c: &mut Criterion) {
    let points: Vec<(f64, f64)> = (0..1_000_000).map(|i| (f64::from(i), (f64::from(i) / 500.0).sin())).collect();
    c.bench_function("line::decimate/1M", |b| b.iter(|| line::decimate(black_box(&points), 1_200)));
}

fn scatter(c: &mut Criterion) {
    let dots: Vec<Dot> = (0..200_000_usize)
        .map(|i| Dot {
            group: i % 3,
            index: i,
            x: (i % 1_000) as f64 / 1_000.0,
            y: (i / 1_000) as f64 / 200.0,
            size: None,
        })
        .collect();
    let colours: [Rgba; 3] = [[200, 60, 60, 255], [60, 200, 60, 255], [60, 60, 200, 255]];
    let mut group = c.benchmark_group("raster");
    group.sample_size(10);
    group.bench_function("scatter/200k", |b| b.iter(|| raster::scatter(black_box(&dots), &colours, 1_200, 800, 2.0)));
    group.finish();
}

fn venn_layout(c: &mut Criterion) {
    let names = ["A", "B", "C", "D", "E"];
    let mut data: Vec<Intersection> = names.iter().map(|n| Intersection::new([*n], 20.0)).collect();
    for i in 0..names.len() {
        for j in i + 1..names.len() {
            data.push(Intersection::new([names[i], names[j]], 6.0));
        }
    }
    c.bench_function("venn::layout/5 sets", |b| b.iter(|| venn::layout(black_box(&data), 600.0, 400.0)));
}

fn voronoi_layout(c: &mut Criterion) {
    let tree = Tree::branch(
        "root",
        (0..8)
            .map(|g| {
                Tree::branch(
                    format!("g{g}"),
                    (0..8).map(|l| Tree::leaf(format!("g{g}l{l}"), 1.0 + f64::from(g * 8 + l) % 5.0)).collect(),
                )
            })
            .collect(),
    );
    let mut group = c.benchmark_group("voronoi");
    group.sample_size(10);
    group.bench_function("layout/64 leaves", |b| {
        b.iter(|| voronoi::layout(black_box(&tree), 600.0, 600.0, voronoi::Shape::Circle, 2));
    });
    group.finish();
}

criterion_group!(benches, decimate, scatter, venn_layout, voronoi_layout);
criterion_main!(benches);
