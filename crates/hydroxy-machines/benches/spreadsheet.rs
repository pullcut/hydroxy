//! Recalculating a sheet of 10k formulas.
#![allow(missing_docs, clippy::cast_precision_loss, reason = "benchmark")]

use std::collections::BTreeMap;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hydroxy_machines::spreadsheet_grid::formula::{Cell, recalc};

fn recalculation(c: &mut Criterion) {
    let rows = 10_000;
    let mut cells: BTreeMap<Cell, String> = BTreeMap::new();
    for r in 0..rows {
        cells.insert((r, 0), (r + 1).to_string());
        let formula = if r == 0 { "=A1*2".to_string() } else { format!("=A{}*2+B{}", r + 1, r) };
        cells.insert((r, 1), formula);
    }
    let mut group = c.benchmark_group("spreadsheet");
    group.sample_size(10);
    group.bench_function("recalc/10k formulas", |b| b.iter(|| recalc(black_box(&cells), (rows, 2))));
    group.finish();
}

criterion_group!(benches, recalculation);
criterion_main!(benches);
