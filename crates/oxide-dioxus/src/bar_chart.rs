//! `BarChart`: one series of vertical bars. Geometry comes from `oxide_charts::bar`;
//! layout is plain CSS, so it fills its container without measuring.

use dioxus::prelude::*;
use oxide_charts::{annotate, bar, format_value, nav};

use crate::line_chart::annotation_nodes;
pub use oxide_charts::Datum;
pub use oxide_charts::Orientation;
pub use oxide_charts::annotate::Annotation;
pub use oxide_charts::bar::Highlight;

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Horizontal bars take their size from `--p` in CSS; vertical ones keep inline `height`/`bottom`.
fn place(horizontal: bool, prop: &str, f: f64) -> String {
    if horizontal { format!("--p: {};", pct(f)) } else { format!("{prop}: {}; --p: {};", pct(f), pct(f)) }
}

/// Which way an edge tooltip leans so it stays inside the plot: the first third of the
/// marks "start", the last third "end".
pub(crate) fn edge_side(i: usize, len: usize) -> &'static str {
    if i * 3 < len {
        "start"
    } else if i * 3 >= len * 2 {
        "end"
    } else {
        ""
    }
}

/// A single-series bar chart: grey bars with one picked out in the accent colour.
/// Hover a bar, or focus the plot and use the arrow keys, to read its value.
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
    /// Reference lines, bands and labelled points. `x` values are category indices; `y`
    /// values are on the value axis, which grows to fit them. Horizontal bars turn them round.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let horizontal = orientation == Orientation::Horizontal;
    let items = data();
    let n = items.len();
    let values: Vec<f64> = items.iter().map(|d| d.value).collect();
    let notes = annotations();
    let layout = bar::layout_with_extent(&values, highlight.resolve(values.len()), 5, annotate::y_extent(&notes));
    let placed = annotate::place(&notes, |i| bar::category_centre(i, n), |v| v / layout.max);
    let placed = if horizontal { placed.into_iter().map(bar::transpose).collect() } else { placed };
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    let announcement = current.map(|i| format!("{}: {}", items[i].label, fmt(items[i].value))).unwrap_or_default();

    rsx! {
        figure {
            "data-scope": "bar-chart",
            "data-part": "root",
            "data-orientation": orientation.attr(),
            "data-show-values": show_values.then_some(""),
            div { "data-part": "plot", style: "height: {height}px;",
                div { "data-part": "grid", "aria-hidden": "true",
                    for g in layout.gridlines.iter() {
                        div {
                            "data-part": "gridline",
                            "data-baseline": (g.fraction == 0.0).then_some(""),
                            style: "{place(horizontal, \"bottom\", g.fraction)}",
                            span { "data-part": "tick", {fmt(g.value)} }
                        }
                    }
                }
                div { "data-part": "annotations", "aria-hidden": "true", {annotation_nodes(&placed)} }
                div {
                    "data-part": "bars",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to read each bar.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if let Some(next) = nav::list_key(&e.key().to_string(), active(), n) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    // Keyed by label: a changed value glides its own bar; only new bars grow in.
                    for (i, (d, b)) in items.iter().zip(layout.bars.iter()).enumerate() {
                        div {
                            key: "{d.label}",
                            "data-part": "column",
                            "aria-hidden": "true",
                            "data-highlighted": b.highlighted.then_some(""),
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--i: {i};",
                            onpointerenter: move |_| active.set(Some(i)),
                            span { "data-part": "value", style: "{place(horizontal, \"bottom\", b.fraction)}", {fmt(d.value)} }
                            div { "data-part": "bar", style: "{place(horizontal, \"height\", b.fraction)}" }
                            if current == Some(i) {
                                div {
                                    "data-part": "tooltip",
                                    "data-side": edge_side(i, n),
                                    style: "{place(horizontal, \"bottom\", b.fraction)}",
                                    div { "data-part": "tooltip-label", "{d.label}" }
                                    ul {
                                        li { "data-part": "tooltip-row",
                                            span { "data-part": "swatch" }
                                            span { "data-part": "tooltip-name", "{label}" }
                                            span { "data-part": "tooltip-value", {fmt(d.value)} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { "data-part": "labels", "aria-hidden": "true",
                for d in items.iter() {
                    span { "data-part": "label", title: "{d.label}", "{d.label}" }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        td {}
                        th { scope: "col", "{label}" }
                    }
                }
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
