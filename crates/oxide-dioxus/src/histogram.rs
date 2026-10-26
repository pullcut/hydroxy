//! `Histogram`: how values are distributed, as adjacent equal-width bins.
//! Binning and heights come from `oxide_charts::histogram`; layout is plain CSS.

use dioxus::prelude::*;
use oxide_charts::{format_value, histogram, line};

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// A histogram of raw values. Bins use Sturges' rule unless `bins` is set.
#[component]
pub fn Histogram(
    /// The raw observations. Non-finite values are ignored.
    values: ReadSignal<Vec<f64>>,
    /// What the chart shows, such as "Response time". Used as the accessible name.
    label: String,
    /// Explicit number of bins. Default: Sturges' rule, `ceil(log2 n) + 1`.
    bins: Option<usize>,
    /// Plot height in px, excluding the axis labels.
    #[props(default = 200)]
    height: u16,
    /// Formats bin edges for the axis and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let binned = histogram::bins(&values(), bins);
    let layout = histogram::layout(&binned, 5);
    let n = binned.len();
    let range = |b: &histogram::Bin| format!("{} to {}", fmt(b.lo), fmt(b.hi));

    rsx! {
        figure { "data-scope": "histogram", "data-part": "root",
            div { "data-part": "plot", style: "height: {height}px;", "aria-hidden": "true",
                div { "data-part": "grid",
                    for g in layout.gridlines.iter() {
                        div {
                            "data-part": "gridline",
                            "data-baseline": (g.fraction == 0.0).then_some(""),
                            style: "bottom: {pct(g.fraction)};",
                            span { "data-part": "tick", {format_value(g.value)} }
                        }
                    }
                }
                div { "data-part": "columns",
                    for (i , (b , bar)) in binned.iter().zip(layout.bars.iter()).enumerate() {
                        div {
                            key: "{i}",
                            "data-part": "column",
                            style: "--i: {i};",
                            title: "{range(b)}: {b.count}",
                            span { "data-part": "count", style: "bottom: {pct(bar.fraction)};", "{b.count}" }
                            div { "data-part": "bar", style: "height: {pct(bar.fraction)};" }
                        }
                    }
                }
            }
            div { "data-part": "labels", "aria-hidden": "true",
                if n > 0 {
                    for i in line::label_indices(n + 1, 7) {
                        span {
                            key: "{i}",
                            "data-part": "label",
                            style: "left: {pct(i as f64 / n as f64)};",
                            {fmt(binned.get(i).map_or_else(|| binned[n - 1].hi, |b| b.lo))}
                        }
                    }
                }
            }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Range" }
                        th { scope: "col", "Count" }
                    }
                }
                tbody {
                    for b in binned.iter() {
                        tr {
                            th { scope: "row", {range(b)} }
                            td { "{b.count}" }
                        }
                    }
                }
            }
        }
    }
}
