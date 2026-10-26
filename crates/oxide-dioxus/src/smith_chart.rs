//! `SmithChart`: impedance samples on the Smith chart, for RF work. Geometry comes
//! from `oxide_charts::smith`. Every trace is focusable; hovering one picks it out.

use dioxus::prelude::*;
use oxide_charts::smith;

pub use oxide_charts::smith::Sample;

fn impedance(s: &Sample) -> String {
    format!("{} {} j{}", s.real, if s.imaginary < 0.0 { "−" } else { "+" }, s.imaginary.abs())
}

/// Impedance sweeps inside the unit disc, with the resistance and reactance grid.
/// Samples that share a `trace` name form one swept trace; their first and last labels name its ends.
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
) -> Element {
    let items = samples();
    let l = smith::layout(&items, z0);
    let view_box = format!("0 0 {} {}", l.width, l.height);

    rsx! {
        figure { "data-scope": "smith-chart", "data-part": "root",
            svg { "data-part": "marks", view_box: "{view_box}", role: "group", "aria-label": "{label}",
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
                        style: "--c: var(--chart-{i % 6 + 1});",
                        tabindex: "0",
                        role: "img",
                        "aria-label": {
                            let parts: Vec<String> = t.samples.iter().map(|s| format!("{}: {}", s.label.clone().unwrap_or_default(), impedance(s))).collect();
                            format!("{}: {}", t.name, parts.join("; "))
                        },
                        path { "data-part": "line", d: "{t.path}" }
                        for (k , &(x , y)) in t.points.iter().enumerate() {
                            circle {
                                "data-part": "point",
                                "data-end": (k == 0 || k + 1 == t.points.len()).then_some(""),
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
            if l.traces.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i , t) in l.traces.iter().enumerate() {
                        li { "data-part": "legend-item", "data-index": "{i}",
                            span { "data-part": "swatch", style: "background: var(--chart-{i % 6 + 1});" }
                            "{t.name}"
                        }
                    }
                }
            }
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
                            td { {impedance(s)} }
                        }
                    }
                }
            }
        }
    }
}
