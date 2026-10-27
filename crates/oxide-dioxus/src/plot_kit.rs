//! The shared shell for the single-value-axis cartesian charts: gridlines and ticks,
//! annotations, a focusable overlay with a tooltip, category labels, legend, live region
//! and screen-reader table. Each chart supplies its marks and keyboard and pointer handling.

use dioxus::prelude::*;
use oxide_charts::annotate::Placed;
use oxide_charts::format_value;
use oxide_charts::plot::{self, Key, Tick, Tip};

pub(crate) fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// The chart's formatter, or the plain default.
pub(crate) fn formatter(format: Option<Callback<f64, String>>) -> impl Fn(f64) -> String + Copy {
    move |v| format.map_or_else(|| format_value(v), |f| f.call(v))
}

#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub(crate) fn Plot(
    scope: &'static str,
    label: String,
    /// How to use the keyboard, appended to the accessible name.
    hint: &'static str,
    height: u16,
    #[props(default)] horizontal: bool,
    /// A chart-specific mode, as `data-variant`.
    #[props(default)]
    variant: &'static str,
    ticks: Vec<Tick>,
    #[props(default)] right_ticks: Vec<Tick>,
    labels: Vec<Tick>,
    annotations: Vec<Placed>,
    #[props(default)] legend: Vec<Key>,
    tip: Option<Tip>,
    head: Vec<String>,
    rows: Vec<Vec<String>>,
    on_key: Callback<String, bool>,
    on_point: EventHandler<Option<(f64, f64)>>,
    marks: Element,
    overlay: Option<Element>,
) -> Element {
    let mut size = use_signal(|| (0.0_f64, 0.0_f64));
    let point = move |e: Event<PointerData>| {
        let (w, h) = size();
        if w > 0.0 && h > 0.0 {
            let c = e.data().element_coordinates();
            on_point.call(Some((c.x / w, 1.0 - c.y / h)));
        }
    };
    let announcement = tip.as_ref().map(Tip::announce).unwrap_or_default();

    rsx! {
        figure {
            "data-scope": scope,
            "data-part": "root",
            "data-plot": "",
            "data-orientation": if horizontal { "horizontal" } else { "vertical" },
            "data-right-axis": (!right_ticks.is_empty()).then_some(""),
            "data-variant": (!variant.is_empty()).then_some(variant),
            style: "--ox-height: {height}px;",
            div { "data-part": "plot", style: "height: {height}px;",
                div { "data-part": "grid", "aria-hidden": "true",
                    for (axis , list) in [("value", &ticks), ("right", &right_ticks)] {
                        for t in list.iter() {
                            div {
                                "data-part": "gridline",
                                "data-axis": axis,
                                "data-baseline": t.baseline.then_some(""),
                                style: "--p: {pct(t.at)};",
                                span { "data-part": "tick", "{t.text}" }
                            }
                        }
                    }
                }
                {marks}
                for a in annotations.iter() {
                    div { "data-part": "annotation", "data-shape": a.shape.attr(), style: a.style(), "aria-hidden": "true",
                        if let Some(l) = a.label.clone() {
                            span { "data-part": "annotation-label", "{l}" }
                        }
                    }
                }
                div {
                    "data-part": "overlay",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. {hint}",
                    onresize: move |e| {
                        if let Ok(s) = e.data().get_border_box_size() {
                            size.set((s.width, s.height));
                        }
                    },
                    onpointermove: point,
                    onpointerdown: point,
                    onpointerleave: move |_| on_point.call(None),
                    onblur: move |_| on_point.call(None),
                    onkeydown: move |e| {
                        if on_key.call(e.key().to_string()) {
                            e.prevent_default();
                        }
                    },
                    if let Some(o) = overlay {
                        {o}
                    }
                    if let Some(t) = tip.clone() {
                        div {
                            "data-part": "tooltip",
                            "data-side": t.side(),
                            "data-vertical": t.vertical(),
                            style: "left: {pct(t.x)}; bottom: {pct(t.y)};",
                            "aria-hidden": "true",
                            div { "data-part": "tooltip-label", "{t.title}" }
                            ul {
                                for r in t.rows.iter() {
                                    li { "data-part": "tooltip-row", "data-active": r.active.then_some(""),
                                        if let Some(c) = r.colour.clone() {
                                            span { "data-part": "swatch", style: "background: {c};" }
                                        }
                                        span { "data-part": "tooltip-name", "{r.name}" }
                                        span { "data-part": "tooltip-value", "{r.value}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { "data-part": "x-labels", "aria-hidden": "true", style: "--room: {pct(plot::tick_room(&labels))};",
                for t in labels.iter() {
                    span { "data-part": "x-label", title: "{t.text}", style: "--p: {pct(t.at)};", "{t.text}" }
                }
            }
            if !legend.is_empty() {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i , k) in legend.iter().enumerate() {
                        li { "data-part": "legend-item", "data-index": "{i}",
                            span { "data-part": "swatch", "data-dashed": k.dashed.then_some(""), style: "--swatch: {k.colour};" }
                            "{k.name}"
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        for h in head.iter() {
                            th { scope: "col", "{h}" }
                        }
                    }
                }
                tbody {
                    for r in rows.iter() {
                        tr {
                            th { scope: "row", {r.first().cloned().unwrap_or_default()} }
                            for c in r.iter().skip(1) {
                                td { "{c}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
