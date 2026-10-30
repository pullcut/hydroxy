//! `Histogram`: how values are distributed, as adjacent equal-width bins.
//! Binning and heights come from `hydroxy_charts::histogram`; layout is plain CSS.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{annotate, format_value, histogram, line, nav};

use crate::bar_chart::edge_side;
use crate::callbacks::{select_active, use_hover};
use crate::line_chart::annotation_nodes;
use crate::plot_kit::pct;
pub use hydroxy_charts::annotate::Annotation;

/// A histogram of raw values. Bins use Sturges' rule unless `bins` is set. Hover a bin, or
/// focus the plot and use the arrow keys, to read its range and count.
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
    /// Reference lines, bands and labelled points. `x` values are on the value axis (such as
    /// a target time), `y` values are counts. The count axis grows to fit them.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Called when the pointer or keyboard moves to a bin, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a bin.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let binned = histogram::bins(&values(), bins);
    let notes = annotations();
    let layout = histogram::layout_with_extent(&binned, 5, annotate::y_extent(&notes));
    let placed = annotate::place(&notes, |x| histogram::x_fraction(&binned, x), |v| v / layout.max);
    let n = binned.len();
    let range = |b: &histogram::Bin| format!("{} to {}", fmt(b.lo), fmt(b.hi));
    let mut active = use_signal(|| None::<usize>);
    let current = active().filter(|&i| i < n);
    #[allow(clippy::cast_precision_loss, reason = "counts are small")]
    let now = current.map(|i| EventDatum::category(i, range(&binned[i]), binned[i].count as f64));
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    use_hover(Some(datum), on_hover);
    let announcement = current.map(|i| format!("{}: {}", range(&binned[i]), binned[i].count)).unwrap_or_default();

    rsx! {
        figure { "data-scope": "histogram", "data-part": "root",
            div { "data-part": "plot", style: "height: {height}px;",
                div { "data-part": "grid", "aria-hidden": "true",
                    for g in layout.gridlines.iter() {
                        div {
                            "data-part": "gridline",
                            "data-baseline": (g.fraction == 0.0).then_some(""),
                            style: "bottom: {pct(g.fraction)};",
                            span { "data-part": "tick", {format_value(g.value)} }
                        }
                    }
                }
                div { "data-part": "annotations", "aria-hidden": "true", {annotation_nodes(&placed)} }
                div {
                    "data-part": "columns",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to read each bin.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onclick: move |_| select_active(Some(datum), on_select),
                    onkeydown: move |e| {
                        if e.key() == dioxus::html::Key::Enter {
                            select_active(Some(datum), on_select);
                        }
                        if let Some(next) = nav::list_key(&e.key().to_string(), active(), n) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    for (i , (b , bar)) in binned.iter().zip(layout.bars.iter()).enumerate() {
                        div {
                            key: "{i}",
                            "data-part": "column",
                            "aria-hidden": "true",
                            "data-active": (current == Some(i)).then_some(""),
                            style: "--i: {i}; --ox-n: {n};",
                            onpointerenter: move |_| active.set(Some(i)),
                            span { "data-part": "count", style: "bottom: {pct(bar.fraction)};", "{b.count}" }
                            div { "data-part": "bar", style: "height: {pct(bar.fraction)};" }
                            if current == Some(i) {
                                div { "data-part": "tooltip", "data-side": edge_side(i, n), style: "bottom: {pct(bar.fraction)};",
                                    div { "data-part": "tooltip-label", {range(b)} }
                                    ul {
                                        li { "data-part": "tooltip-row",
                                            span { "data-part": "swatch" }
                                            span { "data-part": "tooltip-name", "Count" }
                                            span { "data-part": "tooltip-value", "{b.count}" }
                                        }
                                    }
                                }
                            }
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
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
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
