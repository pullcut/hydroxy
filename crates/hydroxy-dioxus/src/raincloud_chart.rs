//! `RaincloudChart`: per group a half violin, a slim box plot and the raw observations
//! jittered beside it. Geometry comes from `hydroxy_charts::raincloud`.

use dioxus::prelude::*;
use hydroxy_charts::{format_value, probe, raincloud, raster};

use crate::chart_frame::use_export;
use crate::plot_shell::{PlotShell, colour, pct};
use crate::raster_canvas::{Frame, RasterCanvas};

pub use hydroxy_charts::Orientation;
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::boxplot::Group;

/// Each group's distribution three ways at once: its smoothed shape (the cloud), its
/// quartiles (the box) and every observation (the rain, spread across a strip so they
/// don't overlap). Hover or touch a group to read its numbers; focus the plot and use the
/// arrow keys to move between groups, Home and End to jump, Escape to clear. Past a couple
/// of thousand observations the rain draws on a canvas.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn RaincloudChart(
    /// The groups, each with its raw values, in display order.
    groups: ReadSignal<Vec<Group>>,
    /// What the chart shows, such as "Page load time by region". Used as the accessible name.
    label: String,
    /// Groups along the bottom with values rising (default), or stacked rows with values running across.
    #[props(default)]
    orientation: Orientation,
    /// Kernel bandwidth in data units. Default: Silverman's rule per group.
    bandwidth: Option<f64>,
    /// Reference lines, bands and labelled points, in data values along the value axis and
    /// group indices across it.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[props(default = 280)]
    height: u16,
    /// Formats values for the axis, tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let layout = use_memo(use_reactive((&orientation, &bandwidth), move |(o, b)| raincloud::raincloud(&groups.read(), o, b, &annotations.read())));
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    // The table lists what a screen reader can cope with; CSV export still gets every row.
    use_export(move |()| {
        let l = layout.read();
        std::iter::once(l.readout.head.clone()).chain(l.readout.rows.iter().map(|r| r.iter().map(|c| c.show(fmt)).collect())).collect()
    });
    let draw = use_callback(move |f: Frame| raster::scatter(&layout.read().dots, &f.colours, f.width, f.height, f.dpr));
    let active = use_signal(|| None::<usize>);
    let l = layout.read();
    let mut readout = l.readout.clone();
    readout.rows.truncate(raster::table_rows(readout.rows.len(), readout.head.len()));
    let big = l.dots.len() > raster::DOM_LIMIT;
    let colours: Vec<String> = (0..6).map(colour).collect();
    let current = active();
    let n = l.boxes.len();
    let o = l.orientation;

    rsx! {
        PlotShell { scope: "raincloud-chart", label, hint: "Use the arrow keys to move between groups.", height, readout, format, active,
            div { "data-part": "rain", "data-orientation": o.attr(), "data-active": current.map(|_| ""),
                svg { "data-part": "clouds", view_box: "0 0 100 100", preserve_aspect_ratio: "none", "aria-hidden": "true",
                    for c in l.clouds.iter() {
                        g { key: "{c.group}", "data-part": "cloud", "data-active": (current == Some(c.group)).then_some(""), style: "--i: {c.group.min(12)};",
                            path { "data-part": "area", d: "{c.area}", fill: colour(c.colour) }
                            path { "data-part": "line", d: "{c.line}", stroke: colour(c.colour) }
                        }
                    }
                }
                div { "data-part": "drops", "aria-hidden": "true",
                    if big {
                        RasterCanvas { colours, draw }
                    } else {
                        for (k , d) in l.dots.iter().enumerate() {
                            span {
                                key: "{k}",
                                "data-part": "dot",
                                style: "left: {pct(d.x)}; bottom: {pct(d.y)}; background: {colour(d.group)}; --i: {d.index.min(40)};",
                            }
                        }
                    }
                }
                div { "data-part": "boxes", "aria-hidden": "true",
                    for (g , b) in l.boxes.iter().enumerate().filter_map(|(g, b)| b.as_ref().map(|b| (g, b))) {
                        div {
                            key: "{g}",
                            "data-part": "box-plot",
                            "data-active": (current == Some(g)).then_some(""),
                            style: "--i: {g.min(12)}; --c: {colour(probe::slot(g, n))};",
                            span { "data-part": "whisker", style: b.span(o, b.low, b.high) }
                            span { "data-part": "box", style: b.span(o, b.q1, b.q3) }
                            span { "data-part": "median", style: b.span(o, b.median, b.median) }
                        }
                    }
                }
            }
        }
    }
}
