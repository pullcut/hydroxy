//! `LineChart` and `AreaChart`: series over a shared x axis, with a crosshair
//! tooltip. Geometry comes from `hydroxy_charts::line`; the SVG stretches to fill its
//! container and everything else is positioned with CSS percentages.

use dioxus::prelude::*;
use hydroxy_charts::annotate;
use hydroxy_charts::brush::Brush;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::zoom::{Axes, Gesture, View};
use hydroxy_charts::{format_value, line, nav, raster, scale, small_multiples};

use crate::callbacks::{select_active, use_hover};
use crate::chart_frame::use_export;
use crate::plot_kit::{colour, pct};

mod area_chart;
mod brush_strip;
mod gestures;

pub use area_chart::AreaChart;
use brush_strip::BrushStrip;
pub(crate) use gestures::{
    annotation_nodes, cut, gesture_down, gesture_move, gutter, link_view, plot_fractions, wheel_zoom,
};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::line::{Band, Curve, Series};
pub use hydroxy_charts::scale::ScaleKind;

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
    /// A value range the axis must cover, such as the shared domain of small multiples.
    y_domain: Option<(f64, f64)>,
    /// The visible window, to control it from outside (link charts by sharing one signal).
    /// Only the x span is used. Default: the chart keeps its own.
    view: ReadSignal<Option<View>>,
    /// Called once a zoom, pan, brush drag or reset has settled, with the new view.
    on_view_change: Option<EventHandler<View>>,
    /// Called when the point under the pointer or keyboard changes (the first series' value at
    /// that position), and with `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on the active position.
    on_select: Option<EventHandler<EventDatum>>,
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
        x.read()
            .as_ref()
            .map(|x| x.iter().map(|&v| if time_axis { scale::date_time(v) } else { fmt(v) }).collect())
            .unwrap_or_default()
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
    let layout = use_memo(use_reactive((&curve, &scale, &y_domain), move |(curve, scale, y_domain)| {
        let xs = line::x_positions(labels_v.read().len(), x_v.read().as_deref());
        let values: Vec<Vec<f64>> = series_v.read().iter().map(|s| s.values.clone()).collect();
        line::layout_scaled(
            &values,
            &bands_v.read(),
            &xs,
            curve,
            5,
            scale,
            small_multiples::widen(annotate::y_extent(&annotations.read()), y_domain),
        )
    }));
    let time_labels =
        use_memo(use_reactive((&time_axis,), move |(time_axis,)| line::time_labels(x_v.read().as_deref(), time_axis)));
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
    let value_at = move |s: &Series, i: usize| {
        s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt)
    };
    use_export(move |()| line::export_table(&full_labels.read(), &series.read(), &bands.read(), &fmt));

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
    let current_view = move || {
        let b = window();
        View::x_only((b.start, b.end))
    };
    // Once zoomed, annotations stop replaying their entrance as the view moves.
    let mut settled = use_signal(|| false);
    let mut set_view = move |v: View| {
        settled.set(true);
        window.set(Brush { start: v.x.0, end: v.x.1 });
    };
    link_view(
        view,
        on_view_change,
        current_view,
        move || {
            let b = *window.peek();
            View::x_only((b.start, b.end))
        },
        move |v| {
            settled.set(true);
            window.set(Brush { start: v.x.0, end: v.x.1 });
        },
    );
    let notes = annotations.read();
    #[allow(clippy::cast_precision_loss, reason = "point counts are small")]
    let offset = if x_r.is_some() { 0.0 } else { range_v.start as f64 };
    let placed =
        annotate::place(&notes, |at| line::x_fraction(at - offset, len, x_r.as_deref()), |v| layout_v.scale.map(v));
    let current = active().filter(|&i| i < len);
    let now = current.and_then(|i| {
        let at = i + range_v.start;
        EventDatum::from_series(&full_series, 0, at, full_labels_v.get(at).map(String::as_str))
    });
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    use_hover(Some(datum), on_hover);
    let announcement = current.map_or_else(String::new, |i| line::announce(&labels_v, &series_v, i, &fmt));
    let move_to = move |e: Event<PointerData>| {
        if zoom && gesture.read().is_active() {
            if let Some(next) = gesture_move(gesture, &e, size(), current_view(), Axes::X, min_span) {
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
                        nav::hinted(&label, nav::HINT_READ_VALUES_LR)
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
                        if let Some(next) = zoom.then(|| wheel_zoom(&e, size(), current_view(), Axes::X, min_span)).flatten() {
                            set_view(next);
                        }
                    },
                    onblur: move |_| active.set(None),
                    onclick: move |_| select_active(Some(datum), on_select),
                    onkeydown: move |e| {
                        if e.key() == dioxus::html::Key::Enter {
                            select_active(Some(datum), on_select);
                        }
                        let key = e.key().to_string();
                        if let Some(next) = zoom.then(|| current_view().key(&key, e.modifiers().shift(), Axes::X, min_span)).flatten() {
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
