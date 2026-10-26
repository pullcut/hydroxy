//! `BarChart`: one series of vertical bars. Geometry comes from `oxide_charts::bar`;
//! layout is plain CSS, so it fills its container without measuring.

use dioxus::prelude::*;
use oxide_charts::{bar, format_value};

pub use oxide_charts::Datum;
pub use oxide_charts::Orientation;
pub use oxide_charts::bar::Highlight;

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Horizontal bars take their size from `--p` in CSS; vertical ones keep inline `height`/`bottom`.
fn place(horizontal: bool, prop: &str, f: f64) -> String {
    if horizontal { format!("--p: {};", pct(f)) } else { format!("{prop}: {}; --p: {};", pct(f), pct(f)) }
}

/// A single-series bar chart: grey bars with one picked out in the accent colour.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn BarChart(
    /// The values, in display order.
    data: ReadSignal<Vec<Datum>>,
    /// What the chart shows, such as "Interviews per week". Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the category labels.
    #[props(default = 180)]
    height: u16,
    /// Which bar gets the accent colour. Default: the last.
    #[props(default)]
    highlight: Highlight,
    /// Vertical (default) or horizontal bars. Horizontal suits long category names.
    #[props(default)]
    orientation: Orientation,
    /// Show every value above its bar, not only the highlighted one and the hovered one.
    #[props(default)]
    show_values: bool,
    /// Formats values for the axis, labels and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let horizontal = orientation == Orientation::Horizontal;
    let items = data();
    let values: Vec<f64> = items.iter().map(|d| d.value).collect();
    let layout = bar::layout(&values, highlight.resolve(values.len()), 5);

    rsx! {
        figure {
            "data-scope": "bar-chart",
            "data-part": "root",
            "data-orientation": orientation.attr(),
            "data-show-values": show_values.then_some(""),
            div { "data-part": "plot", style: "height: {height}px;", "aria-hidden": "true",
                div { "data-part": "grid",
                    for g in layout.gridlines.iter() {
                        div {
                            "data-part": "gridline",
                            "data-baseline": (g.fraction == 0.0).then_some(""),
                            style: "{place(horizontal, \"bottom\", g.fraction)}",
                            span { "data-part": "tick", {fmt(g.value)} }
                        }
                    }
                }
                div { "data-part": "bars",
                    // Keyed by label: a changed value glides its own bar; only new bars grow in.
                    for (i, (d, b)) in items.iter().zip(layout.bars.iter()).enumerate() {
                        div {
                            key: "{d.label}",
                            "data-part": "column",
                            "data-highlighted": b.highlighted.then_some(""),
                            style: "--i: {i};",
                            span { "data-part": "value", style: "{place(horizontal, \"bottom\", b.fraction)}", {fmt(d.value)} }
                            div { "data-part": "bar", style: "{place(horizontal, \"height\", b.fraction)}" }
                        }
                    }
                }
            }
            div { "data-part": "labels", "aria-hidden": "true",
                for d in items.iter() {
                    span { "data-part": "label", title: "{d.label}", "{d.label}" }
                }
            }
            table { "data-part": "sr-table",
                caption { "{label}" }
                tbody {
                    for d in items.iter() {
                        tr {
                            th { scope: "row", "{d.label}" }
                            td { {fmt(d.value)} }
                        }
                    }
                }
            }
        }
    }
}
