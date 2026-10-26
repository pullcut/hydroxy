//! `LineChart` and `AreaChart`: series over a shared x axis, with a crosshair
//! tooltip. Geometry comes from `oxide_charts::line`; the SVG stretches to fill its
//! container and everything else is positioned with CSS percentages.

use dioxus::prelude::*;
use oxide_charts::{format_value, line};

pub use oxide_charts::line::{Curve, Series};

fn colour(i: usize) -> String {
    format!("var(--chart-{})", i % 6 + 1)
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Lines over a shared x axis. Hover, touch or focus the plot and use Left/Right to
/// read every series at one x position; Escape hides the crosshair.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn LineChart(
    /// The series, drawn in categorical colour order. Up to six read well.
    series: ReadSignal<Vec<Series>>,
    /// One label per x position, used on the axis, tooltip and table.
    labels: ReadSignal<Vec<String>>,
    /// Numeric x positions (ascending), one per label, for uneven spacing. Default: evenly spaced.
    x: ReadSignal<Option<Vec<f64>>>,
    /// What the chart shows, such as "Weekly active users". Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[props(default = 220)]
    height: u16,
    /// Straight segments (default) or a monotone curve.
    #[props(default)]
    curve: Curve,
    /// Fill the area under each line.
    #[props(default)]
    area: bool,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let mut active = use_signal(|| None::<usize>);
    let mut width = use_signal(|| 0.0_f64);
    let series_v = series();
    let labels_v = labels();
    let len = labels_v.len();
    let xs = line::x_positions(len, x().as_deref());
    let values: Vec<Vec<f64>> = series_v.iter().map(|s| s.values.clone()).collect();
    let layout = line::layout(&values, &xs, curve, 5);
    let value_at = move |s: &Series, i: usize| s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt);
    let current = active().filter(|&i| i < len);
    let announcement = current.map_or_else(String::new, |i| {
        let parts: Vec<String> = series_v.iter().map(|s| format!("{} {}", s.name, value_at(s, i))).collect();
        format!("{}: {}", labels_v[i], parts.join(", "))
    });
    let xs_for_pointer = xs.clone();
    let move_to = move |e: Event<PointerData>| {
        let w = width();
        if w > 0.0 {
            let i = line::nearest_index(&xs_for_pointer, e.data().element_coordinates().x / w);
            if active() != i {
                active.set(i);
            }
        }
    };
    let move_to_down = move_to.clone();
    let multi = series_v.len() > 1;

    rsx! {
        figure { "data-scope": "line-chart", "data-part": "root", "data-area": area.then_some(""),
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
                svg {
                    "data-part": "marks",
                    view_box: "0 0 100 100",
                    "preserveAspectRatio": "none",
                    "aria-hidden": "true",
                    for (i, s) in layout.series.iter().enumerate() {
                        g { key: "{i}", "data-part": "series", "data-index": "{i}", style: "--i: {i};",
                            if area {
                                path { "data-part": "area", d: "{s.area}", fill: colour(i) }
                            }
                            path { "data-part": "line", d: "{s.line}", stroke: colour(i), "pathLength": "1" }
                        }
                    }
                }
                div {
                    "data-part": "overlay",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the left and right arrow keys to read values.",
                    onresize: move |e| {
                        if let Ok(size) = e.data().get_border_box_size() {
                            width.set(size.width);
                        }
                    },
                    onpointermove: move_to,
                    onpointerdown: move_to_down,
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if let Some(next) = line::crosshair_key(&e.key().to_string(), active(), len) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    if let Some(i) = current {
                        div { "data-part": "crosshair", style: "left: {pct(xs[i])};" }
                        for (si, s) in layout.series.iter().enumerate() {
                            if let Some(f) = s.points.get(i).copied().flatten() {
                                span {
                                    "data-part": "dot",
                                    style: "left: {pct(xs[i])}; bottom: {pct(f)}; background: {colour(si)};",
                                }
                            }
                        }
                        div {
                            "data-part": "tooltip",
                            "data-side": if xs[i] > 0.5 { "left" } else { "right" },
                            style: "left: {pct(xs[i])};",
                            "aria-hidden": "true",
                            div { "data-part": "tooltip-label", "{labels_v[i]}" }
                            ul {
                                for (si, s) in series_v.iter().enumerate() {
                                    li { "data-part": "tooltip-row",
                                        span { "data-part": "swatch", style: "background: {colour(si)};" }
                                        span { "data-part": "tooltip-name", "{s.name}" }
                                        span { "data-part": "tooltip-value", {value_at(s, i)} }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { "data-part": "x-labels", "aria-hidden": "true",
                for i in line::label_indices(len, 6) {
                    span { "data-part": "x-label", style: "left: {pct(xs[i])};", "{labels_v[i]}" }
                }
            }
            if multi {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i, s) in series_v.iter().enumerate() {
                        li { "data-part": "legend-item", "data-index": "{i}",
                            span { "data-part": "swatch", style: "background: {colour(i)};" }
                            "{s.name}"
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
                        for s in series_v.iter() {
                            th { scope: "col", "{s.name}" }
                        }
                    }
                }
                tbody {
                    for (i, l) in labels_v.iter().enumerate() {
                        tr {
                            th { scope: "row", "{l}" }
                            for s in series_v.iter() {
                                td { {value_at(s, i)} }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A [`LineChart`] with the area under each line filled.
#[component]
pub fn AreaChart(
    /// The series, drawn in categorical colour order. Up to six read well.
    series: ReadSignal<Vec<Series>>,
    /// One label per x position, used on the axis, tooltip and table.
    labels: ReadSignal<Vec<String>>,
    /// Numeric x positions (ascending), one per label, for uneven spacing. Default: evenly spaced.
    x: ReadSignal<Option<Vec<f64>>>,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[props(default = 220)]
    height: u16,
    /// Straight segments (default) or a monotone curve.
    #[props(default)]
    curve: Curve,
    /// Formats values for the axis, tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    rsx! { LineChart { series, labels, x, label, height, curve, area: true, format } }
}
