//! `ScatterChart`: numeric x and y, with optional bubble size and one colour per group.
//! Geometry comes from `hydroxy_charts::scatter`; dots are positioned with CSS percentages.

use dioxus::prelude::*;
use hydroxy_charts::annotate;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::{HINT_POINTS, hinted};
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::zoom::{Axes, Gesture, View};
use hydroxy_charts::{format_value, raster, scatter};

use crate::chart_frame::use_export;
use crate::line_chart::{annotation_nodes, gesture_down, gesture_move, link_view, plot_fractions, wheel_zoom};
use crate::plot_kit::{ChartTip, colour, pct};
use crate::raster_canvas::{Frame, RasterCanvas};
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::scale::ScaleKind;
pub use hydroxy_charts::scatter::{Group, Point};

const MIN_SPAN: f64 = 0.02;

/// Dots on two numeric axes. Past a few thousand visible dots they draw on a canvas, so
/// hundreds of thousands of points stay smooth to hover and zoom. Hover or touch to read the nearest dot; focus the plot and
/// use the arrow keys to step through dots from left to right, Home and End to jump,
/// Escape to clear.
#[component]
pub fn ScatterChart(
    /// The groups, one colour each in categorical order. Up to six read well.
    /// Give points a size to draw bubbles.
    groups: ReadSignal<Vec<Group>>,
    /// What the chart shows, such as "Interviews against satisfaction". Used as the accessible name.
    label: String,
    /// Caption for the x axis.
    x_label: Option<String>,
    /// Caption for the y axis.
    y_label: Option<String>,
    /// Name of the bubble size measure, shown in the tooltip and table.
    #[props(default = "Size".to_string())]
    size_label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[props(default = 280)]
    height: u16,
    /// Formats x values for the axis, tooltip and table.
    format_x: Option<Callback<f64, String>>,
    /// Formats y values for the axis, tooltip and table.
    format_y: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units on both axes.
    /// The y axis grows to fit them.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Linear (default), Log or Symlog x axis.
    #[props(default)]
    x_scale: ScaleKind,
    /// Linear (default), Log or Symlog y axis.
    #[props(default)]
    y_scale: ScaleKind,
    /// Wheel or pinch to zoom around the pointer, drag to pan, and `+`/`-`/Shift+arrows on
    /// the focused plot.
    #[props(default)]
    zoom: bool,
    /// How far from a dot, in px, the pointer still picks it. Default: anywhere in the
    /// plot, so the nearest dot is always picked however small it is.
    #[props(default = f64::INFINITY)]
    hover_radius: f64,
    /// Outline the region of the plot that picks each dot (its Voronoi cell). A debugging
    /// aid for small data; ignored once dots draw on a canvas.
    #[props(default)]
    show_voronoi: bool,
    /// The visible window, to control it from outside (link charts by sharing one signal).
    /// Default: the chart keeps its own.
    #[props(default)]
    view: ReadSignal<Option<View>>,
    /// Called once a zoom, pan or reset has settled, with the new view.
    on_view_change: Option<EventHandler<View>>,
    /// Called when the dot under the pointer or keyboard changes, and with `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on the active dot.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fx = move |v: f64| format_x.map_or_else(|| format_value(v), |f| f.call(v));
    let fy = move |v: f64| format_y.map_or_else(|| format_value(v), |f| f.call(v));
    let controlled = view;
    let mut view = use_signal(|| (*controlled.peek()).unwrap_or(View::FULL));
    // Once zoomed, dots stop replaying their entrance every time the view moves.
    let mut settled = use_signal(|| false);
    let mut set_view = move |v: View| {
        settled.set(true);
        view.set(v);
    };
    link_view(
        controlled,
        on_view_change,
        move || *view.read(),
        move || *view.peek(),
        move |v| {
            settled.set(true);
            view.set(v);
        },
    );
    let gesture = use_signal(Gesture::default);
    // Memos, so a hover never re-lays out or clones the data.
    // Laid out once at full extent; zooming only reframes it.
    let full = use_memo(use_reactive((&x_scale, &y_scale), move |kinds| {
        scatter::layout_scaled(&groups.read(), 6, kinds, View::FULL, annotate::y_extent(&annotations.read()))
    }));
    let layout = use_memo(move || scatter::reframe(&full.read(), view(), 6));
    // Decided on the whole cloud, so zooming in never swaps canvas for elements mid-gesture.
    let big_cloud = use_memo(move || full.read().dots.len() > raster::DOM_LIMIT);
    // Big clouds draw on a canvas and find the hovered dot through a grid.
    let index = use_memo(move || big_cloud().then(|| scatter::Index::new(&layout.read().dots)));
    let has_sizes = use_memo(move || groups.read().iter().flat_map(|g| &g.points).any(|p| p.size.is_some()));
    let x_head = x_label.clone().unwrap_or_else(|| "X".into());
    let y_head = y_label.clone().unwrap_or_else(|| "Y".into());
    let size_head = size_label.clone();
    use_export(move |()| {
        let size_head = has_sizes().then_some(size_head.as_str());
        scatter::table_rows(&groups.read(), &x_head, &y_head, size_head, &fx, &fy)
    });
    let draw = use_callback(move |f: Frame| {
        raster::scatter_in(&full.read().dots, f.view, &f.colours, f.width, f.height, f.dpr)
    });
    let groups_v = groups.read();
    let notes = annotations.read();
    let layout_v = layout.read();
    let big = big_cloud();
    let colours: Vec<String> = (0..groups_v.len().max(1)).map(colour).collect();
    let placed = annotate::place(
        &notes,
        |x| layout_v.view.project_x(layout_v.x_scale.map(x)),
        |y| layout_v.view.project_y(layout_v.y_scale.map(y)),
    );
    let mut active = use_signal(|| None::<usize>);
    let mut size = use_signal(|| (0.0_f64, 0.0_f64));
    let current = active().filter(|&i| i < layout_v.dots.len());
    let now = current.and_then(|i| {
        let d = layout_v.dots[i];
        EventDatum::from_point(&groups_v, d.group, d.index)
    });
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let point_of = |d: &scatter::Dot| groups_v.get(d.group).and_then(|g| g.points.get(d.index).copied());
    let has_sizes = has_sizes();
    let total: usize = groups_v.iter().map(|g| g.points.len()).sum();
    let announcement = current.and_then(|i| {
        let d = layout_v.dots[i];
        let p = point_of(&d)?;
        let mut text = format!("{}: {} by {}", groups_v[d.group].name, fx(p.x), fy(p.y));
        if let Some(s) = p.size {
            text = format!("{text}, {size_label} {}", format_value(s));
        }
        Some(text)
    });
    let announcement = announcement.unwrap_or_default();
    let move_to = move |e: Event<PointerData>| {
        if zoom && gesture.read().is_active() {
            if let Some(next) = gesture_move(gesture, &e, size(), view(), Axes::XY, MIN_SPAN) {
                active.set(None);
                set_view(next);
            }
            return;
        }
        if let Some(at) = plot_fractions(e.data().element_coordinates(), size()) {
            let l = layout.read();
            let near = scatter::pick(&l.dots, index.read().as_ref(), at, size(), hover_radius);
            if active() != near {
                active.set(near);
            }
        }
    };
    let mut move_to_down = move_to;
    let release = move |e: Event<PointerData>| {
        let mut g = gesture;
        g.write().up(e.data().pointer_id());
    };
    let (w, h) = size();
    let cells =
        if show_voronoi && !big && w > 0.0 && h > 0.0 { scatter::cells(&layout_v.dots, (w, h)) } else { Vec::new() };
    let tip = current.and_then(|i| {
        let d = layout_v.dots[i];
        point_of(&d).map(|p| (d, p))
    });

    rsx! {
        figure {
            "data-scope": "scatter-chart",
            "data-part": "root",
            "data-zoom": zoom.then_some(""),
            "data-settled": settled().then_some(""),
            style: crate::line_chart::gutter(&layout_v.y_ticks.iter().map(|g| fy(g.value)).collect::<Vec<_>>(), 40.0),
            if let Some(t) = y_label.clone() {
                div { "data-part": "axis-title", "data-axis": "y", "aria-hidden": "true", "{t}" }
            }
            div { "data-part": "plot", style: "height: {height}px;",
                div { "data-part": "grid", "aria-hidden": "true",
                    for g in layout_v.y_ticks.iter() {
                        div { "data-part": "gridline", "data-axis": "y", style: "bottom: {pct(g.fraction)};",
                            span { "data-part": "tick", {fy(g.value)} }
                        }
                    }
                    for g in layout_v.x_ticks.iter() {
                        div { "data-part": "gridline", "data-axis": "x", style: "left: {pct(g.fraction)};",
                            span { "data-part": "tick", {fx(g.value)} }
                        }
                    }
                }
                div { "data-part": "annotations", "aria-hidden": "true", {annotation_nodes(&placed)} }
                div {
                    "data-part": "overlay",
                    tabindex: "0",
                    role: "group",
                    "aria-label": if zoom {
                        format!("{label}. Use the arrow keys to step through points, plus and minus to zoom, Shift and the arrow keys to pan.")
                    } else {
                        hinted(&label, HINT_POINTS)
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
                    onclick: move |_| crate::callbacks::select_active(Some(datum), on_select),
                    onpointerup: release,
                    onpointercancel: release,
                    onpointerleave: move |e| {
                        release(e);
                        active.set(None);
                    },
                    onwheel: move |e| {
                        if let Some(next) = zoom.then(|| wheel_zoom(&e, size(), view(), Axes::XY, MIN_SPAN)).flatten() {
                            set_view(next);
                        }
                    },
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if e.key() == dioxus::html::Key::Enter {
                            crate::callbacks::select_active(Some(datum), on_select);
                        }
                        let key = e.key().to_string();
                        if let Some(next) = zoom.then(|| view().key(&key, e.modifiers().shift(), Axes::XY, MIN_SPAN)).flatten() {
                            e.prevent_default();
                            set_view(next);
                            return;
                        }
                        let next = scatter::step_key(&key, active(), &scatter::order(&layout.read().dots));
                        if let Some(next) = next {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    if big {
                        RasterCanvas { colours, draw, view: Some(view.into()) }
                    }
                    if !cells.is_empty() {
                        svg {
                            "data-part": "voronoi",
                            view_box: "0 0 {w:.0} {h:.0}",
                            "preserveAspectRatio": "none",
                            "aria-hidden": "true",
                            for (i , d) in cells.iter().enumerate() {
                                path { key: "{i}", "data-part": "cell", "data-active": (current == Some(i)).then_some(""), d: "{d}" }
                            }
                        }
                    }
                    for (i , d) in layout_v.dots.iter().enumerate().filter(|_| !big) {
                        span {
                            key: "{d.group}-{d.index}",
                            "data-part": "dot",
                            "data-group": "{d.group}",
                            "data-bubble": d.size.map(|_| ""),
                            "data-active": (current == Some(i)).then_some(""),
                            style: "left: {pct(d.x)}; bottom: {pct(d.y)}; --d: {d.size.map_or(10.0, |s| 8.0 + s * 32.0):.1}px; --i: {i}; --ox-n: {layout_v.dots.len()}; background: {colour(d.group)};",
                        }
                    }
                    if let Some((d , p)) = tip {
                        // Canvas dots have no element of their own, so the active one gets a marker.
                        if big {
                            span {
                                "data-part": "dot",
                                "data-active": "",
                                style: "left: {pct(d.x)}; bottom: {pct(d.y)}; --d: {d.size.map_or(10.0, |s| 8.0 + s * 32.0):.1}px; background: {colour(d.group)}; animation: none;",
                            }
                        }
                        div { "data-part": "crosshair", "data-axis": "x", style: "left: {pct(d.x)};" }
                        div { "data-part": "crosshair", "data-axis": "y", style: "bottom: {pct(d.y)};" }
                        ChartTip {
                            tip: {
                                let mut rows = vec![
                                    TipRow::new(None, x_label.clone().unwrap_or_else(|| "x".into()), fx(p.x)),
                                    TipRow::new(None, y_label.unwrap_or_else(|| "y".into()), fy(p.y)),
                                ];
                                if let Some(s) = p.size {
                                    rows.push(TipRow::new(None, size_label.clone(), format_value(s)));
                                }
                                Tip { x: d.x, y: d.y, title: groups_v[d.group].name.clone(), rows }
                            },
                            swatch: colour(d.group),
                            style: format!("left: {}; bottom: {};", pct(d.x), pct(d.y)),
                        }
                    }
                }
                if zoom && !view().is_full() {
                    button { "data-part": "zoom-reset", r#type: "button", onclick: move |_| view.set(View::FULL), "Reset zoom" }
                }
            }
            if let Some(t) = x_label {
                div { "data-part": "axis-title", "data-axis": "x", "aria-hidden": "true", "{t}" }
            }
            if groups_v.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i , g) in groups_v.iter().enumerate() {
                        li { "data-part": "legend-item", "data-index": "{i}",
                            span { "data-part": "swatch", style: "background: {colour(i)};" }
                            "{g.name}"
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
                        th { scope: "col", "Group" }
                        th { scope: "col", "X" }
                        th { scope: "col", "Y" }
                        if has_sizes {
                            th { scope: "col", "{size_label}" }
                        }
                    }
                }
                tbody {
                    for (g , p) in groups_v.iter().flat_map(|g| g.points.iter().map(move |p| (g, p))).take(raster::TABLE_LIMIT) {
                            tr {
                                th { scope: "row", "{g.name}" }
                                td { {fx(p.x)} }
                                td { {fy(p.y)} }
                                if has_sizes {
                                    td { "{p.size.map(format_value).unwrap_or_default()}" }
                                }
                            }
                    }
                }
            }
        }
    }
}
