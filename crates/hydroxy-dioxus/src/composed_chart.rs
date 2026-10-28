//! `ComposedChart`: bars, lines, areas and points over shared categories and one value
//! axis, with a crosshair tooltip that reads every layer. Geometry comes from
//! `hydroxy_charts::composed`.

use dioxus::prelude::*;
use hydroxy_charts::composed::LayerKind;
use hydroxy_charts::{annotate, bar, composed, format_value, line, nav};

use crate::line_chart::{annotation_nodes, plot_fractions};
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::composed::Layer;
pub use hydroxy_charts::line::Curve;

fn colour(i: usize) -> String {
    format!("var(--chart-{})", i % 6 + 1)
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Layers of bars, lines, areas and points over one set of categories. Hover, touch or
/// focus the plot and use Left/Right to read every layer at one category.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn ComposedChart(
    /// The layers, drawn in order (later ones on top) in categorical colours.
    /// `Layer::bar`, `Layer::line`, `Layer::area` or `Layer::points`.
    layers: ReadSignal<Vec<Layer>>,
    /// One label per category, used on the axis, tooltip and table.
    labels: ReadSignal<Vec<String>>,
    /// What the chart shows, such as "Orders against target". Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[props(default = 240)]
    height: u16,
    /// How line and area layers join their points.
    #[props(default)]
    curve: Curve,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points. `x` values are category indices.
    /// The value axis grows to fit them.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let layers_v = layers();
    let labels_v = labels();
    let n = labels_v.len();
    let notes = annotations();
    let layout = composed::layout(&layers_v, n, curve, 5, annotate::y_extent(&notes));
    let placed = annotate::place(&notes, |i| bar::category_centre(i, n), |v| layout.scale.map(v));
    let mut active = use_signal(|| None::<usize>);
    let mut size = use_signal(|| (0.0_f64, 0.0_f64));
    let current = active().filter(|&i| i < n);
    let value_at = move |l: &Layer, i: usize| l.value(i).map_or_else(|| "–".to_string(), fmt);
    let announcement = current.map_or_else(String::new, |i| {
        let parts: Vec<String> = layers_v.iter().map(|l| format!("{} {}", l.name, value_at(l, i))).collect();
        format!("{}: {}", labels_v[i], parts.join(", "))
    });
    let centres = layout.centres.clone();
    let move_to = move |e: Event<PointerData>| {
        if let Some((fx, _)) = plot_fractions(e.data().element_coordinates(), size()) {
            let i = line::nearest_index(&centres, fx);
            if active() != i {
                active.set(i);
            }
        }
    };
    let move_to_down = move_to.clone();
    let bars: Vec<(usize, usize, composed::BarRect)> = layout
        .layers
        .iter()
        .enumerate()
        .flat_map(|(li, g)| g.bars.iter().enumerate().filter_map(move |(i, b)| b.map(|b| (li, i, b))))
        .collect();
    let points: Vec<(usize, usize, f64, f64)> = layout
        .layers
        .iter()
        .enumerate()
        .filter(|(i, _)| layers_v.get(*i).map(|l| l.kind) == Some(LayerKind::Points))
        .flat_map(|(li, g)| {
            g.points.iter().zip(&layout.centres).enumerate().filter_map(move |(i, (p, &x))| p.map(|f| (li, i, x, f)))
        })
        .collect();

    rsx! {
        figure { "data-scope": "composed-chart", "data-part": "root",
            div { "data-part": "plot", style: "height: {height}px;",
                div { "data-part": "grid", "aria-hidden": "true",
                    for g in layout.gridlines.iter() {
                        div {
                            "data-part": "gridline",
                            "data-baseline": (g.value == 0.0).then_some(""),
                            style: "bottom: {pct(g.fraction)};",
                            span { "data-part": "tick", {fmt(g.value)} }
                        }
                    }
                }
                div { "data-part": "annotations", "aria-hidden": "true", {annotation_nodes(&placed)} }
                div { "data-part": "bars", "aria-hidden": "true",
                    for (li , i , b) in bars {
                        div {
                            key: "{li}-{i}",
                            "data-part": "bar",
                            "data-index": "{li}",
                            "data-negative": b.negative.then_some(""),
                            style: "left: {pct(b.left)}; width: {pct(b.width)}; bottom: {pct(b.bottom)}; height: {pct(b.height)}; background: {colour(li)}; --i: {i};",
                        }
                    }
                }
                svg {
                    "data-part": "marks",
                    view_box: "0 0 100 100",
                    "preserveAspectRatio": "none",
                    "aria-hidden": "true",
                    for (i , g) in layout.layers.iter().enumerate() {
                        if matches!(layers_v.get(i).map(|l| l.kind), Some(LayerKind::Line | LayerKind::Area)) {
                            g { key: "{i}", "data-part": "series", "data-index": "{i}", style: "--i: {i};",
                                if !g.area.is_empty() {
                                    path { "data-part": "area", d: "{g.area}", fill: colour(i) }
                                }
                                path { "data-part": "line", d: "{g.line}", stroke: colour(i), "pathLength": "1" }
                            }
                        }
                    }
                }
                div { "data-part": "points", "aria-hidden": "true",
                    for (li , i , x , f) in points {
                        span {
                            key: "{li}-{i}",
                            "data-part": "point",
                            style: "left: {pct(x)}; bottom: {pct(f)}; background: {colour(li)}; --i: {i};",
                        }
                    }
                }
                div {
                    "data-part": "overlay",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the left and right arrow keys to read values.",
                    onresize: move |e| {
                        if let Ok(s) = e.data().get_border_box_size() {
                            size.set((s.width, s.height));
                        }
                    },
                    onpointermove: move_to,
                    onpointerdown: move_to_down,
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if let Some(next) = nav::list_key(&e.key().to_string(), active(), n) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    if let Some(i) = current {
                        div { "data-part": "crosshair", style: "left: {pct(layout.centres[i])};" }
                        for (li , g) in layout.layers.iter().enumerate() {
                            if let Some(f) = g.points.get(i).copied().flatten() {
                                span {
                                    key: "{li}",
                                    "data-part": "dot",
                                    style: "left: {pct(layout.centres[i])}; bottom: {pct(f)}; background: {colour(li)};",
                                }
                            }
                        }
                        div {
                            "data-part": "tooltip",
                            "data-side": if layout.centres[i] > 0.5 { "left" } else { "right" },
                            style: "left: {pct(layout.centres[i])};",
                            "aria-hidden": "true",
                            div { "data-part": "tooltip-label", "{labels_v[i]}" }
                            ul {
                                for (li , l) in layers_v.iter().enumerate() {
                                    li { "data-part": "tooltip-row",
                                        span { "data-part": "swatch", "data-kind": l.kind.attr(), style: "background: {colour(li)};" }
                                        span { "data-part": "tooltip-name", "{l.name}" }
                                        span { "data-part": "tooltip-value", {value_at(l, i)} }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { "data-part": "x-labels", "aria-hidden": "true",
                for i in line::label_indices(n, 8) {
                    span { key: "{i}", "data-part": "x-label", style: "left: {pct(layout.centres[i])};", "{labels_v[i]}" }
                }
            }
            if layers_v.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i , l) in layers_v.iter().enumerate() {
                        li { "data-part": "legend-item", "data-index": "{i}",
                            span { "data-part": "swatch", "data-kind": l.kind.attr(), style: "background: {colour(i)};" }
                            "{l.name}"
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        td {}
                        for l in layers_v.iter() {
                            th { scope: "col", "{l.name}" }
                        }
                    }
                }
                tbody {
                    for (i , name) in labels_v.iter().enumerate() {
                        tr {
                            th { scope: "row", "{name}" }
                            for l in layers_v.iter() {
                                td { {value_at(l, i)} }
                            }
                        }
                    }
                }
            }
        }
    }
}
