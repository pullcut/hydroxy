//! `SmithChart`: impedance samples on the Smith chart, for RF work. Geometry comes
//! from `hydroxy_charts::smith`. Hover a sample or a legend row to pick out its trace.

use dioxus::prelude::*;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{nav, smith};

use crate::plot_kit::{ChartTip, colour};

pub use hydroxy_charts::smith::Sample;

/// Pointer reach for picking a sample, in drawing units.
const REACH: f64 = 24.0;

fn impedance(s: &Sample, fmt: impl Fn(f64) -> String) -> String {
    format!("{} {} j{}", fmt(s.real), if s.imaginary < 0.0 { "−" } else { "+" }, fmt(s.imaginary.abs()))
}

/// Impedance sweeps inside the unit disc, with the resistance and reactance grid.
/// Samples that share a `trace` name form one swept trace; their first and last labels name its ends.
/// Hover or touch to read the nearest sample; focus the chart and use Left and Right to
/// step along a trace, Up and Down to switch trace, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SmithChart(
    /// The samples, in sweep order within each trace.
    samples: ReadSignal<Vec<Sample>>,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Reference impedance the samples are measured against. 1 means already normalised;
    /// pass 50 when the samples are raw ohms.
    #[props(default = 1.0)]
    z0: f64,
    /// Formats resistance and reactance in the tooltip and table. Default: two decimal places.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format!("{v:.2}"), |f| f.call(v));
    let items = samples();
    let l = smith::layout(&items, z0);
    let view_box = format!("0 0 {} {}", l.width, l.height);
    let mut active = use_signal(|| None::<(usize, usize)>);
    let mut hovered_trace = use_signal(|| None::<usize>);
    let mut plot_width = use_signal(|| 1.0_f64);
    let current = active().filter(|&(t, k)| l.traces.get(t).is_some_and(|tr| k < tr.points.len()));
    let highlight = hovered_trace().or(current.map(|(t, _)| t));
    let rows = l.traces.len();
    let lens: Vec<usize> = l.traces.iter().map(|t| t.points.len()).collect();
    let cols = lens.iter().copied().max().unwrap_or(0);
    let picker = l.clone();
    let pick = move |e: Event<PointerData>| {
        let c = e.data().element_coordinates();
        let scale = picker.width / plot_width().max(1.0);
        let near = smith::nearest(&picker, c.x * scale, c.y * scale, REACH);
        if active() != near {
            active.set(near);
        }
    };
    let pick_down = pick.clone();
    let announcement = current.map(|(t, k)| {
        let s = &l.traces[t].samples[k];
        match &s.label {
            Some(lab) => format!("{}, {lab}: {}", l.traces[t].name, impedance(s, fmt)),
            None => format!("{}: {}", l.traces[t].name, impedance(s, fmt)),
        }
    });
    let reference = if z0.is_finite() && z0 > 0.0 { z0 } else { 1.0 };

    rsx! {
        figure { "data-scope": "smith-chart", "data-part": "root", "data-highlight": highlight.map(|_| ""),
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through samples; Up and Down switch trace.",
                onresize: move |e| {
                    if let Ok(s) = e.data().get_border_box_size() {
                        plot_width.set(s.width);
                    }
                },
                onpointermove: pick,
                onpointerdown: pick_down,
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    let next = nav::grid_key(&e.key().to_string(), active(), rows, cols)
                        .map(|n| n.map(|(t, k)| (t, k.min(lens[t].saturating_sub(1)))));
                    if let Some(next) = next {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { "data-part": "marks", view_box: "{view_box}", "aria-hidden": "true",
                    line {
                        "data-part": "axis",
                        x1: "{l.centre.0 - l.radius}",
                        y1: "{l.centre.1}",
                        x2: "{l.centre.0 + l.radius}",
                        y2: "{l.centre.1}",
                    }
                    for (x , y , r) in l.resistance.iter() {
                        circle { "data-part": "grid", cx: "{x}", cy: "{y}", r: "{r}" }
                    }
                    for d in l.reactance.iter() {
                        path { "data-part": "grid", d: "{d}" }
                    }
                    circle { "data-part": "boundary", cx: "{l.centre.0}", cy: "{l.centre.1}", r: "{l.radius}" }
                    for (i , t) in l.traces.iter().enumerate() {
                        g {
                            key: "{t.name}",
                            "data-part": "series",
                            "data-index": "{i}",
                            "data-active": (highlight == Some(i)).then_some(""),
                            style: "--c: {colour(i)};",
                            path { "data-part": "line", "pathLength": "1", d: "{t.path}" }
                            for (k , &(x , y)) in t.points.iter().enumerate() {
                                circle {
                                    "data-part": "point",
                                    "data-end": (k == 0 || k + 1 == t.points.len()).then_some(""),
                                    "data-active": (current == Some((i, k))).then_some(""),
                                    cx: "{x}",
                                    cy: "{y}",
                                    r: if k == 0 || k + 1 == t.points.len() { "4.5" } else { "3" },
                                }
                            }
                            if t.points.len() > 1 {
                                for k in [0, t.points.len() - 1] {
                                    if let Some(text) = t.samples[k].label.clone() {
                                        {
                                            let (x, y) = t.points[k];
                                            let right = x >= l.centre.0;
                                            let tx = (x + if right { 10.0 } else { -10.0 }).clamp(6.0, l.width - 6.0);
                                            let ty = (y - 9.0).clamp(12.0, l.height - 6.0);
                                            rsx! {
                                                text {
                                                    "data-part": "end-label",
                                                    x: "{tx}",
                                                    y: "{ty}",
                                                    "text-anchor": if right { "start" } else { "end" },
                                                    "{text}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some((t, k)) = current {
                    {
                        let s = &l.traces[t].samples[k];
                        let (x, y) = l.traces[t].points[k];
                        let (gr, gi) = smith::gamma(s.real, s.imaginary, reference);
                        let mut rows = Vec::new();
                        if let Some(lab) = s.label.clone() {
                            rows.push(TipRow::new(None, "Sample", lab));
                        }
                        rows.push(TipRow::new(None, "Impedance", impedance(s, fmt)));
                        rows.push(TipRow::new(None, "|Γ|", format!("{:.2}", gr.hypot(gi))));
                        rsx! {
                            ChartTip {
                                tip: Tip { x: x / l.width, y: 1.0 - y / l.height, title: l.traces[t].name.clone(), rows },
                                swatch: colour(t),
                                anchored: true,
                                side: if x > l.width / 2.0 { "left" } else { "right" },
                            }
                        }
                    }
                }
            }
            if l.traces.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i , t) in l.traces.iter().enumerate() {
                        li {
                            "data-part": "legend-item",
                            "data-index": "{i}",
                            "data-active": (highlight == Some(i)).then_some(""),
                            onpointerenter: move |_| hovered_trace.set(Some(i)),
                            onpointerleave: move |_| hovered_trace.set(None),
                            span { "data-part": "swatch", style: "background: {colour(i)};" }
                            "{t.name}"
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Trace" }
                        th { scope: "col", "Sample" }
                        th { scope: "col", "Impedance" }
                    }
                }
                tbody {
                    for s in items.iter() {
                        tr {
                            th { scope: "row", "{s.trace}" }
                            td { "{s.label.clone().unwrap_or_default()}" }
                            td { {impedance(s, fmt)} }
                        }
                    }
                }
            }
        }
    }
}
