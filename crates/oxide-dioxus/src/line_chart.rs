//! `LineChart` and `AreaChart`: series over a shared x axis, with a crosshair
//! tooltip. Geometry comes from `oxide_charts::line`; the SVG stretches to fill its
//! container and everything else is positioned with CSS percentages.

use dioxus::html::geometry::ElementPoint;
use dioxus::prelude::*;
use oxide_charts::annotate::{self, Placed};
use oxide_charts::brush::{Brush, Drag, Grab};
use oxide_charts::zoom::{self, Axes, Gesture, View};
use oxide_charts::{format_value, line, raster, scale};

use crate::chart_frame::use_export;

pub use oxide_charts::annotate::Annotation;
pub use oxide_charts::line::{Band, Curve, Series};
pub use oxide_charts::scale::ScaleKind;

/// The placed annotations, as positioned `annotation` parts. Shared by the cartesian charts.
pub(crate) fn annotation_nodes(placed: &[Placed]) -> Element {
    rsx! {
        for (i , p) in placed.iter().enumerate() {
            div { key: "{i}", "data-part": "annotation", "data-shape": p.shape.attr(), style: "{p.style()}",
                if let Some(l) = &p.label {
                    span { "data-part": "annotation-label", "{l}" }
                }
            }
        }
    }
}

/// A point over an element of `size` (width, height), as fractions with y up.
pub(crate) fn plot_fractions(point: ElementPoint, size: (f64, f64)) -> Option<(f64, f64)> {
    (size.0 > 0.0 && size.1 > 0.0).then(|| (point.x / size.0, 1.0 - point.y / size.1))
}

/// Zoom `view` for a wheel or trackpad-pinch event, cancelling the page scroll when it zooms.
pub(crate) fn wheel_zoom(e: &Event<WheelData>, size: (f64, f64), view: View, axes: Axes, min_span: f64) -> Option<View> {
    let at = plot_fractions(e.data().element_coordinates(), size)?;
    let next = view.zoom(at, zoom::wheel_factor(e.data().delta().strip_units().y), axes, min_span);
    (next != view).then(|| {
        e.prevent_default();
        next
    })
}

/// Start a drag or pinch on a zoomable plot.
pub(crate) fn gesture_down(mut gesture: Signal<Gesture>, e: &Event<PointerData>, size: (f64, f64)) {
    let primary = e.data().trigger_button().is_none_or(|b| b == dioxus::html::input_data::MouseButton::Primary);
    if let (true, Some(at)) = (primary, plot_fractions(e.data().element_coordinates(), size)) {
        gesture.write().down(e.data().pointer_id(), at);
    }
}

/// Continue a drag or pinch: the new view, if it changed.
pub(crate) fn gesture_move(mut gesture: Signal<Gesture>, e: &Event<PointerData>, size: (f64, f64), view: View, axes: Axes, min_span: f64) -> Option<View> {
    let at = plot_fractions(e.data().element_coordinates(), size)?;
    gesture.write().moved(e.data().pointer_id(), at, view, axes, min_span)
}

pub(crate) fn cut<T: Clone>(v: &[T], r: &std::ops::Range<usize>) -> Vec<T> {
    v.get(r.start.min(v.len())..r.end.min(v.len())).unwrap_or_default().to_vec()
}

/// Sizes the left gutter to the widest tick label.
pub(crate) fn gutter(labels: &[String], min: f64) -> String {
    format!("--ox-axis: {}px;", oxide_charts::plot::axis_width(labels.iter().map(String::as_str), min))
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
    /// One label per x position, used on the axis, tooltip and table. Leave empty with `x`
    /// set to label points from `x` (dates with `time_axis`), which saves building a string
    /// per point for large data.
    #[props(default)]
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
    bands: ReadSignal<Vec<Band>>,
    /// Add a brush strip under the plot. Drag its edges or window, or focus an edge and use
    /// Left/Right (Shift for bigger steps, Home/End for the limits) to zoom the plot to a range.
    #[props(default)]
    brush: bool,
    /// Reference lines, bands and labelled points. `x` values are point indices, or
    /// numeric x positions when `x` is set. The value axis grows to fit them.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Linear (default), Log or Symlog value axis.
    #[props(default)]
    scale: ScaleKind,
    /// Treat `x` as Unix seconds and label the axis with calendar ticks instead of `labels`.
    #[props(default)]
    time_axis: bool,
    /// Wheel or pinch to zoom around the pointer, drag to pan, and `+`/`-`/Shift+arrows on the
    /// focused plot. Works with `brush`: both move the same window.
    #[props(default)]
    zoom: bool,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let mut active = use_signal(|| None::<usize>);
    let mut size = use_signal(|| (0.0_f64, 0.0_f64));
    let mut window = use_signal(|| Brush::FULL);
    let gesture = use_signal(Gesture::default);
    // one String per point; a lazy label lookup would save memory past ~10M points.
    let full_labels = use_memo(use_reactive((&time_axis,), move |(time_axis,)| {
        let given = labels.read();
        if !given.is_empty() {
            return given.clone();
        }
        x.read().as_ref().map(|x| x.iter().map(|&v| if time_axis { scale::date_time(v) } else { fmt(v) }).collect()).unwrap_or_default()
    }));
    let full_xs = use_memo(move || line::x_positions(full_labels.read().len(), x.read().as_deref()));
    // The plot draws a zoomed slice of the full data; everything below uses the slice.
    // Memos, so a hover never re-cuts or re-lays out the data.
    let range = use_memo(use_reactive((&brush, &zoom), move |(brush, zoom)| {
        if brush || zoom { window().range(&full_xs.read()) } else { 0..full_labels.read().len() }
    }));
    let series_v = use_memo(move || {
        let r = range();
        series.read().iter().map(|s| Series::new(s.name.clone(), cut(&s.values, &r))).collect::<Vec<_>>()
    });
    let labels_v = use_memo(move || cut(&full_labels.read(), &range()));
    let x_v = use_memo(move || x.read().as_ref().map(|x| cut(x, &range())));
    let bands_v = use_memo(move || {
        let r = range();
        bands.read().iter().map(|b| Band::new(b.series, cut(&b.low, &r), cut(&b.high, &r))).collect::<Vec<_>>()
    });
    let layout = use_memo(use_reactive((&curve, &scale), move |(curve, scale)| {
        let xs = line::x_positions(labels_v.read().len(), x_v.read().as_deref());
        let values: Vec<Vec<f64>> = series_v.read().iter().map(|s| s.values.clone()).collect();
        line::layout_scaled(&values, &bands_v.read(), &xs, curve, 5, scale, annotate::y_extent(&annotations.read()))
    }));
    let time_labels = use_memo(use_reactive((&time_axis,), move |(time_axis,)| match x_v.read().as_deref() {
        Some(xv) if time_axis && xv.len() > 1 => {
            scale::time_ticks(xv[0], xv[xv.len() - 1], 6).into_iter().map(|t| (line::x_fraction(t.at, xv.len(), Some(xv)), t.label)).collect()
        }
        _ => Vec::new(),
    }));
    let overview = use_memo(use_reactive((&brush, &curve), move |(brush, curve)| {
        if !brush {
            return Vec::new();
        }
        let values: Vec<Vec<f64>> = series.read().iter().map(|s| s.values.clone()).collect();
        line::layout(&values, &full_xs.read(), curve, 3).series.into_iter().map(|s| s.line).collect::<Vec<_>>()
    }));
    let band_cell = move |b: &Band, high: bool, i: usize| {
        let v = if high { &b.high } else { &b.low };
        v.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt)
    };
    let band_name = move |b: &Band| series.read().get(b.series).map(|s| s.name.clone()).unwrap_or_default();
    let value_at = move |s: &Series, i: usize| s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt);
    use_export(move |()| {
        let full_series = series.read();
        let full_bands = bands.read();
        let head = std::iter::once(String::new())
            .chain(full_series.iter().map(|s| s.name.clone()))
            .chain(full_bands.iter().flat_map(|b| {
                let n = band_name(b);
                [format!("{n} low"), format!("{n} high")]
            }))
            .collect::<Vec<_>>();
        let labels = full_labels.read();
        let body = labels.iter().enumerate().map(|(i, l)| {
            std::iter::once(l.clone())
                .chain(full_series.iter().map(|s| value_at(s, i)))
                .chain(full_bands.iter().flat_map(|b| [band_cell(b, false, i), band_cell(b, true, i)]))
                .collect::<Vec<_>>()
        });
        std::iter::once(head).chain(body).collect()
    });

    let full_series = series.read();
    let full_labels_v = full_labels.read();
    let full_bands = bands.read();
    let series_v = series_v.read();
    let labels_v = labels_v.read();
    let x_r = x_v.read();
    let bands_v = bands_v.read();
    let layout_v = layout.read();
    let xs = &layout_v.xs;
    let len = labels_v.len();
    let range_v = range();
    #[allow(clippy::cast_precision_loss, reason = "point counts are small")]
    let min_span = 1.0 / full_labels_v.len().saturating_sub(1).max(1) as f64;
    let view = move || {
        let b = window();
        View::x_only((b.start, b.end))
    };
    // Once zoomed, annotations stop replaying their entrance as the view moves.
    let mut settled = use_signal(|| false);
    let mut set_view = move |v: View| {
        settled.set(true);
        window.set(Brush { start: v.x.0, end: v.x.1 });
    };
    let notes = annotations.read();
    #[allow(clippy::cast_precision_loss, reason = "point counts are small")]
    let offset = if x_r.is_some() { 0.0 } else { range_v.start as f64 };
    let placed = annotate::place(&notes, |at| line::x_fraction(at - offset, len, x_r.as_deref()), |v| layout_v.scale.map(v));
    let current = active().filter(|&i| i < len);
    let announcement = current.map_or_else(String::new, |i| {
        let parts: Vec<String> = series_v.iter().map(|s| format!("{} {}", s.name, value_at(s, i))).collect();
        format!("{}: {}", labels_v[i], parts.join(", "))
    });
    let move_to = move |e: Event<PointerData>| {
        if zoom && gesture.read().active() {
            if let Some(next) = gesture_move(gesture, &e, size(), view(), Axes::X, min_span) {
                active.set(None);
                set_view(next);
            }
            return;
        }
        if let Some((fx, _)) = plot_fractions(e.data().element_coordinates(), size()) {
            let i = line::nearest_index(&layout.read().xs, fx);
            if active() != i {
                active.set(i);
            }
        }
    };
    let mut move_to_down = move_to;
    let release = move |e: Event<PointerData>| {
        let mut g = gesture;
        g.write().up(e.data().pointer_id());
    };
    let total = full_labels_v.len();
    let multi = series_v.len() > 1;

    rsx! {
        figure {
            "data-scope": "line-chart",
            "data-part": "root",
            "data-area": area.then_some(""),
            "data-scale": scale.attr(),
            "data-zoom": zoom.then_some(""),
            "data-settled": settled().then_some(""),
            style: gutter(&layout_v.gridlines.iter().map(|g| fmt(g.value)).collect::<Vec<_>>(), 36.0),
            div { "data-part": "plot", style: "height: {height}px;",
                div { "data-part": "grid", "aria-hidden": "true",
                    for g in layout_v.gridlines.iter() {
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
                    for (bi , (d , b)) in layout_v.bands.iter().zip(bands_v.iter()).enumerate() {
                        path { key: "band-{bi}", "data-part": "band", "data-index": "{b.series}", d: "{d}", fill: colour(b.series) }
                    }
                    for (i, s) in layout_v.series.iter().enumerate() {
                        g { key: "{i}", "data-part": "series", "data-index": "{i}", style: "--i: {i};",
                            if area {
                                path { "data-part": "area", d: "{s.area}", fill: colour(i) }
                            }
                            path { "data-part": "line", d: "{s.line}", stroke: colour(i), "pathLength": "1" }
                        }
                    }
                }
                div { "data-part": "annotations", "aria-hidden": "true", {annotation_nodes(&placed)} }
                div {
                    "data-part": "overlay",
                    tabindex: "0",
                    role: "group",
                    "aria-label": if zoom {
                        format!("{label}. Use the left and right arrow keys to read values, plus and minus to zoom, Shift and the arrow keys to pan.")
                    } else {
                        format!("{label}. Use the left and right arrow keys to read values.")
                    },
                    onresize: move |e| {
                        if let Ok(s) = e.data().get_border_box_size() {
                            size.set((s.width, s.height));
                        }
                    },
                    onpointermove: move_to,
                    onpointerdown: move |e: Event<PointerData>| {
                        if zoom {
                            gesture_down(gesture, &e, size());
                        }
                        move_to_down(e);
                    },
                    onpointerup: release,
                    onpointercancel: release,
                    onpointerleave: move |e| {
                        release(e);
                        active.set(None);
                    },
                    onwheel: move |e| {
                        if let Some(next) = zoom.then(|| wheel_zoom(&e, size(), view(), Axes::X, min_span)).flatten() {
                            set_view(next);
                        }
                    },
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        let key = e.key().to_string();
                        if let Some(next) = zoom.then(|| view().key(&key, e.modifiers().shift(), Axes::X, min_span)).flatten() {
                            e.prevent_default();
                            set_view(next);
                            return;
                        }
                        if let Some(next) = line::crosshair_key(&key, active(), len) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    if let Some(i) = current {
                        div { "data-part": "crosshair", style: "left: {pct(xs[i])};" }
                        for (si, s) in layout_v.series.iter().enumerate() {
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
                if zoom && !window().is_full() {
                    button { "data-part": "zoom-reset", r#type: "button", onclick: move |_| window.set(Brush::FULL), "Reset zoom" }
                }
            }
            div { "data-part": "x-labels", "aria-hidden": "true",
                if time_labels.read().is_empty() {
                    for i in line::label_indices(len, 6) {
                        span { key: "{i}", "data-part": "x-label", style: "left: {pct(xs[i])};", "{labels_v[i]}" }
                    }
                } else {
                    for (i , (f , t)) in time_labels.read().iter().enumerate() {
                        span { key: "t{i}", "data-part": "x-label", style: "left: {pct(*f)};", "{t}" }
                    }
                }
            }
            if brush {
                BrushStrip {
                    window,
                    overview,
                    len: total,
                    start_label: full_labels_v.get(range_v.start).cloned().unwrap_or_default(),
                    end_label: full_labels_v.get(range_v.end.saturating_sub(1)).cloned().unwrap_or_default(),
                    lo: range_v.start,
                    hi: range_v.end,
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
                caption {
                    "{label}"
                    if total > raster::TABLE_LIMIT {
                        " (first {raster::TABLE_LIMIT} of {total} points)"
                    }
                }
                thead {
                    tr {
                        td {}
                        for s in full_series.iter() {
                            th { scope: "col", "{s.name}" }
                        }
                        for (bi , b) in full_bands.iter().enumerate() {
                            th { key: "bl-{bi}", scope: "col", "{band_name(b)} low" }
                            th { scope: "col", "{band_name(b)} high" }
                        }
                    }
                }
                tbody {
                    for (i, l) in full_labels_v.iter().take(raster::TABLE_LIMIT).enumerate() {
                        tr {
                            th { scope: "row", "{l}" }
                            for s in full_series.iter() {
                                td { {value_at(s, i)} }
                            }
                            for (bi , b) in full_bands.iter().enumerate() {
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
#[allow(clippy::too_many_arguments, reason = "component props")]
fn BrushStrip(
    window: Signal<Brush>,
    overview: Memo<Vec<String>>,
    len: usize,
    start_label: String,
    end_label: String,
    lo: usize,
    hi: usize,
    label: String,
) -> Element {
    let mut window = window;
    let mut drag = use_signal(|| None::<Drag>);
    let mut width = use_signal(|| 0.0_f64);
    #[allow(clippy::cast_precision_loss, reason = "point counts are small")]
    let step = 1.0 / len.saturating_sub(1).max(1) as f64;
    let fraction = move |e: &Event<PointerData>| {
        let w = width();
        (w > 0.0).then(|| (e.data().element_coordinates().x / w).clamp(0.0, 1.0))
    };
    let b = window();
    let edge = |grab: Grab, name: &'static str, index: usize, text: &str| {
        let at = if grab == Grab::Start { b.start } else { b.end };
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
                for (i , d) in overview.read().iter().enumerate() {
                    path { key: "{i}", "data-part": "brush-line", d: "{d}", stroke: colour(i) }
                }
            }
            div { "data-part": "brush-window", style: "left: {b.start * 100.0:.3}%; width: {b.span() * 100.0:.3}%;" }
            {edge(Grab::Start, "start", lo, &start_label)}
            {edge(Grab::End, "end", hi.saturating_sub(1), &end_label)}
        }
    }
}

/// A [`LineChart`] with the area under each line filled.
#[component]
pub fn AreaChart(
    /// The series, drawn in categorical colour order. Up to six read well.
    series: ReadSignal<Vec<Series>>,
    /// One label per x position, used on the axis, tooltip and table. Leave empty with `x`
    /// set to label points from `x` (dates with `time_axis`), which saves building a string
    /// per point for large data.
    #[props(default)]
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
    /// Reference lines, bands and labelled points, as on [`LineChart`].
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Linear (default), Log or Symlog value axis.
    #[props(default)]
    scale: ScaleKind,
    /// Treat `x` as Unix seconds and label the axis with calendar ticks.
    #[props(default)]
    time_axis: bool,
    /// Wheel, pinch, drag and keyboard zoom, as on [`LineChart`].
    #[props(default)]
    zoom: bool,
) -> Element {
    rsx! {
        LineChart {
            series,
            labels,
            x,
            label,
            height,
            curve,
            area: true,
            format,
            annotations,
            scale,
            time_axis,
            zoom,
        }
    }
}
