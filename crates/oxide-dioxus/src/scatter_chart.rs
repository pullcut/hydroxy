//! `ScatterChart`: numeric x and y, with optional bubble size and one colour per group.
//! Geometry comes from `oxide_charts::scatter`; dots are positioned with CSS percentages.

use dioxus::prelude::*;
use oxide_charts::annotate;
use oxide_charts::zoom::{Axes, Gesture, View};
use oxide_charts::{format_value, scatter};

use crate::line_chart::{annotation_nodes, gesture_down, gesture_move, plot_fractions, wheel_zoom};
pub use oxide_charts::annotate::Annotation;
pub use oxide_charts::scale::ScaleKind;
pub use oxide_charts::scatter::{Group, Point};

const MIN_SPAN: f64 = 0.02;

fn colour(i: usize) -> String {
    format!("var(--chart-{})", i % 6 + 1)
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Dots on two numeric axes. Hover or touch to read the nearest dot; focus the plot and
/// use the arrow keys to step through dots from left to right, Home and End to jump,
/// Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
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
) -> Element {
    let fx = move |v: f64| format_x.map_or_else(|| format_value(v), |f| f.call(v));
    let fy = move |v: f64| format_y.map_or_else(|| format_value(v), |f| f.call(v));
    let mut view = use_signal(|| View::FULL);
    // Once zoomed, dots stop replaying their entrance every time the view moves.
    let mut settled = use_signal(|| false);
    let mut set_view = move |v: View| {
        settled.set(true);
        view.set(v);
    };
    let gesture = use_signal(Gesture::default);
    let groups_v = groups();
    let notes = annotations();
    let layout = scatter::layout_scaled(&groups_v, 6, (x_scale, y_scale), view(), annotate::y_extent(&notes));
    let placed = annotate::place(&notes, |x| layout.view.project_x(layout.x_scale.map(x)), |y| layout.view.project_y(layout.y_scale.map(y)));
    let mut active = use_signal(|| None::<usize>);
    let mut size = use_signal(|| (0.0_f64, 0.0_f64));
    let current = active().filter(|&i| i < layout.dots.len());
    let point_of = |d: &scatter::Dot| groups_v.get(d.group).and_then(|g| g.points.get(d.index).copied());
    let has_sizes = groups_v.iter().flat_map(|g| &g.points).any(|p| p.size.is_some());
    let announcement = current.and_then(|i| {
        let d = layout.dots[i];
        let p = point_of(&d)?;
        let mut text = format!("{}: {} by {}", groups_v[d.group].name, fx(p.x), fy(p.y));
        if let Some(s) = p.size {
            text = format!("{text}, {size_label} {}", format_value(s));
        }
        Some(text)
    });
    let announcement = announcement.unwrap_or_default();
    let dots_for_pointer = layout.dots.clone();
    let move_to = move |e: Event<PointerData>| {
        if zoom && gesture.read().active() {
            if let Some(next) = gesture_move(gesture, &e, size(), view(), Axes::XY, MIN_SPAN) {
                active.set(None);
                set_view(next);
            }
            return;
        }
        if let Some((x, y)) = plot_fractions(e.data().element_coordinates(), size()) {
            let near = scatter::nearest(&dots_for_pointer, x, y, 0.08);
            if active() != near {
                active.set(near);
            }
        }
    };
    let mut move_to_down = move_to.clone();
    let release = move |e: Event<PointerData>| {
        let mut g = gesture;
        g.write().up(e.data().pointer_id());
    };
    let order = layout.order.clone();
    let tip = current.and_then(|i| {
        let d = layout.dots[i];
        point_of(&d).map(|p| (d, p))
    });

    rsx! {
        figure {
            "data-scope": "scatter-chart",
            "data-part": "root",
            "data-zoom": zoom.then_some(""),
            "data-settled": settled().then_some(""),
            if let Some(t) = y_label.clone() {
                div { "data-part": "axis-title", "data-axis": "y", "aria-hidden": "true", "{t}" }
            }
            div { "data-part": "plot", style: "height: {height}px;",
                div { "data-part": "grid", "aria-hidden": "true",
                    for g in layout.y_ticks.iter() {
                        div { "data-part": "gridline", "data-axis": "y", style: "bottom: {pct(g.fraction)};",
                            span { "data-part": "tick", {fy(g.value)} }
                        }
                    }
                    for g in layout.x_ticks.iter() {
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
                        format!("{label}. Use the arrow keys to step through points.")
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
                        if let Some(next) = zoom.then(|| wheel_zoom(&e, size(), view(), Axes::XY, MIN_SPAN)).flatten() {
                            set_view(next);
                        }
                    },
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        let key = e.key().to_string();
                        if let Some(next) = zoom.then(|| view().key(&key, e.modifiers().shift(), Axes::XY, MIN_SPAN)).flatten() {
                            e.prevent_default();
                            set_view(next);
                            return;
                        }
                        if let Some(next) = scatter::step_key(&key, active(), &order) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    for (i , d) in layout.dots.iter().enumerate() {
                        span {
                            key: "{d.group}-{d.index}",
                            "data-part": "dot",
                            "data-group": "{d.group}",
                            "data-bubble": d.size.map(|_| ""),
                            "data-active": (current == Some(i)).then_some(""),
                            style: "left: {pct(d.x)}; bottom: {pct(d.y)}; --d: {d.size.map_or(10.0, |s| 8.0 + s * 32.0):.1}px; --i: {i.min(40)}; background: {colour(d.group)};",
                        }
                    }
                    if let Some((d , p)) = tip {
                        div { "data-part": "crosshair", "data-axis": "x", style: "left: {pct(d.x)};" }
                        div { "data-part": "crosshair", "data-axis": "y", style: "bottom: {pct(d.y)};" }
                        div {
                            "data-part": "tooltip",
                            "data-side": if d.x > 0.5 { "left" } else { "right" },
                            "data-vertical": if d.y > 0.5 { "below" } else { "above" },
                            style: "left: {pct(d.x)}; bottom: {pct(d.y)};",
                            "aria-hidden": "true",
                            div { "data-part": "tooltip-label",
                                span { "data-part": "swatch", style: "background: {colour(d.group)};" }
                                "{groups_v[d.group].name}"
                            }
                            ul {
                                li { "data-part": "tooltip-row",
                                    span { "data-part": "tooltip-name", "{x_label.clone().unwrap_or_else(|| \"x\".into())}" }
                                    span { "data-part": "tooltip-value", {fx(p.x)} }
                                }
                                li { "data-part": "tooltip-row",
                                    span { "data-part": "tooltip-name", "{y_label.clone().unwrap_or_else(|| \"y\".into())}" }
                                    span { "data-part": "tooltip-value", {fy(p.y)} }
                                }
                                if let Some(s) = p.size {
                                    li { "data-part": "tooltip-row",
                                        span { "data-part": "tooltip-name", "{size_label}" }
                                        span { "data-part": "tooltip-value", {format_value(s)} }
                                    }
                                }
                            }
                        }
                    }
                }
                if zoom && !view().is_full() {
                    button { "data-part": "zoom-reset", r#type: "button", onclick: move |_| view.set(View::FULL), "Reset zoom" }
                }
            }
            if let Some(t) = x_label.clone() {
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
                caption { "{label}" }
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
                    for g in groups_v.iter() {
                        for p in g.points.iter() {
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
}
