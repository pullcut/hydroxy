//! `LineChart` and `AreaChart`: series over a shared x axis, with a crosshair
//! tooltip. Geometry comes from `oxide_charts::line`; the SVG stretches to fill its
//! container and everything else is positioned with CSS percentages.

use dioxus::prelude::*;
use oxide_charts::brush::{Brush, Drag, Grab};
use oxide_charts::{format_value, line};

pub use oxide_charts::line::{Band, Curve, Series};

fn cut<T: Clone>(v: &[T], r: &std::ops::Range<usize>) -> Vec<T> {
    v.get(r.start.min(v.len())..r.end.min(v.len())).unwrap_or_default().to_vec()
}

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
    /// Confidence bands: a shaded low-to-high region around a series, picked by index.
    /// The value axis grows to fit them. Default: none.
    #[props(default)]
    bands: Vec<Band>,
    /// Add a brush strip under the plot. Drag its edges or window, or focus an edge and use
    /// Left/Right (Shift for bigger steps, Home/End for the limits) to zoom the plot to a range.
    #[props(default)]
    brush: bool,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let mut active = use_signal(|| None::<usize>);
    let mut width = use_signal(|| 0.0_f64);
    let window = use_signal(|| Brush::FULL);
    // The plot draws a zoomed slice of the full data; everything below uses the slice.
    let full_series = series();
    let full_labels = labels();
    let full_x = x();
    let full_xs = line::x_positions(full_labels.len(), full_x.as_deref());
    let range = if brush { window().range(&full_xs) } else { 0..full_labels.len() };
    let series_v: Vec<Series> = full_series.iter().map(|s| Series::new(s.name.clone(), cut(&s.values, &range))).collect();
    let labels_v = cut(&full_labels, &range);
    let x_v = full_x.as_ref().map(|x| cut(x, &range));
    let bands_v: Vec<Band> = bands.iter().map(|b| Band::new(b.series, cut(&b.low, &range), cut(&b.high, &range))).collect();
    let len = labels_v.len();
    let xs = line::x_positions(len, x_v.as_deref());
    let values: Vec<Vec<f64>> = series_v.iter().map(|s| s.values.clone()).collect();
    let layout = line::layout_with_bands(&values, &bands_v, &xs, curve, 5);
    let overview: Vec<String> = if brush {
        let full_values: Vec<Vec<f64>> = full_series.iter().map(|s| s.values.clone()).collect();
        line::layout(&full_values, &full_xs, curve, 3).series.into_iter().map(|s| s.line).collect()
    } else {
        Vec::new()
    };
    let band_cell = |b: &Band, high: bool, i: usize| {
        let v = if high { &b.high } else { &b.low };
        v.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt)
    };
    let band_name = |b: &Band| series_v.get(b.series).map(|s| s.name.clone()).unwrap_or_default();
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
                    for (bi , (d , b)) in layout.bands.iter().zip(bands_v.iter()).enumerate() {
                        path { key: "band-{bi}", "data-part": "band", "data-index": "{b.series}", d: "{d}", fill: colour(b.series) }
                    }
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
            if brush {
                BrushStrip {
                    window,
                    overview,
                    labels: full_labels.clone(),
                    lo: range.start,
                    hi: range.end,
                    label: label.clone(),
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
                        for (bi , b) in bands_v.iter().enumerate() {
                            th { key: "bl-{bi}", scope: "col", "{band_name(b)} low" }
                            th { scope: "col", "{band_name(b)} high" }
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
                            for (bi , b) in bands_v.iter().enumerate() {
                                td { key: "bl-{bi}", {band_cell(b, false, i)} }
                                td { {band_cell(b, true, i)} }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The overview strip under a zoomable `LineChart`: a faint copy of every line over the
/// full range, with a draggable window and two keyboard-operable edges.
#[component]
fn BrushStrip(window: Signal<Brush>, overview: Vec<String>, labels: Vec<String>, lo: usize, hi: usize, label: String) -> Element {
    let mut window = window;
    let mut drag = use_signal(|| None::<Drag>);
    let mut width = use_signal(|| 0.0_f64);
    let len = labels.len();
    #[allow(clippy::cast_precision_loss, reason = "point counts are small")]
    let step = 1.0 / len.saturating_sub(1).max(1) as f64;
    let fraction = move |e: &Event<PointerData>| {
        let w = width();
        (w > 0.0).then(|| (e.data().element_coordinates().x / w).clamp(0.0, 1.0))
    };
    let b = window();
    let edge = |grab: Grab, name: &'static str, index: usize| {
        let at = if grab == Grab::Start { b.start } else { b.end };
        let text = labels.get(index).cloned().unwrap_or_default();
        rsx! {
            div {
                "data-part": "brush-handle",
                "data-edge": name,
                role: "slider",
                tabindex: "0",
                "aria-label": "{label} range {name}",
                "aria-valuemin": "0",
                "aria-valuemax": "{len.saturating_sub(1)}",
                "aria-valuenow": "{index}",
                "aria-valuetext": "{text}",
                style: "left: {at * 100.0:.3}%;",
                onkeydown: move |e| {
                    let shift = e.modifiers().shift();
                    if let Some(next) = window().key(grab, &e.key().to_string(), shift, step, step) {
                        e.prevent_default();
                        window.set(next);
                    }
                },
            }
        }
    };

    rsx! {
        div {
            "data-part": "brush",
            role: "group",
            "aria-label": "{label}: zoom range",
            onresize: move |e| {
                if let Ok(size) = e.data().get_border_box_size() {
                    width.set(size.width);
                }
            },
            onpointerdown: move |e| {
                if let Some(f) = fraction(&e) {
                    let (next, d) = window().begin(f, 0.025);
                    window.set(next);
                    drag.set(Some(d));
                }
            },
            onpointermove: move |e| {
                if let (Some(d), Some(f)) = (drag(), fraction(&e)) {
                    window.set(d.apply(f, step));
                }
            },
            onpointerup: move |_| drag.set(None),
            onpointercancel: move |_| drag.set(None),
            onpointerleave: move |_| drag.set(None),
            svg { "data-part": "brush-marks", view_box: "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                for (i , d) in overview.iter().enumerate() {
                    path { key: "{i}", "data-part": "brush-line", d: "{d}", stroke: colour(i) }
                }
            }
            div { "data-part": "brush-window", style: "left: {b.start * 100.0:.3}%; width: {b.span() * 100.0:.3}%;" }
            {edge(Grab::Start, "start", lo)}
            {edge(Grab::End, "end", hi.saturating_sub(1))}
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
