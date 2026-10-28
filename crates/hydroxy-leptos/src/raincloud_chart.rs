//! `<RaincloudChart>`: per group a half violin, a slim box plot and the raw observations
//! jittered beside it. Geometry comes from `hydroxy_charts::raincloud`.

use hydroxy_charts::{format_value, probe, raincloud, raster};
use leptos::prelude::*;

use crate::chart_frame::provide_export;
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
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn RaincloudChart(
    /// The groups, each with its raw values, in display order.
    #[prop(into)]
    groups: Signal<Vec<Group>>,
    /// What the chart shows, such as "Page load time by region". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Groups along the bottom with values rising (default), or stacked rows with values running across.
    #[prop(optional)]
    orientation: Orientation,
    /// Kernel bandwidth in data units. Default: Silverman's rule per group.
    #[prop(optional, into)]
    bandwidth: Option<f64>,
    /// Reference lines, bands and labelled points, in data values along the value axis and
    /// group indices across it.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[prop(default = 280)]
    height: u16,
    /// Formats values for the axis, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let layout =
        Memo::new(move |_| groups.with(|g| annotations.with(|a| raincloud::raincloud(g, orientation, bandwidth, a))));
    let big = Memo::new(move |_| layout.with(|l| l.dots.len() > raster::DOM_LIMIT));
    // The table lists what a screen reader can cope with; CSV export still gets every row.
    let readout = Signal::derive(move || {
        layout.with(|l| {
            let mut r = l.readout.clone();
            r.rows.truncate(raster::table_rows(r.rows.len(), r.head.len()));
            r
        })
    });
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    provide_export(move || {
        layout.with(|l| {
            std::iter::once(l.readout.head.clone())
                .chain(l.readout.rows.iter().map(|r| r.iter().map(|c| c.show(fmt)).collect()))
                .collect()
        })
    });
    let active = RwSignal::new(None::<usize>);
    let colours = Signal::derive(move || (0..6).map(colour).collect::<Vec<_>>());
    let draw =
        Callback::new(move |f: Frame| layout.with(|l| raster::scatter(&l.dots, &f.colours, f.width, f.height, f.dpr)));
    let picked = move |g: usize| move || (active.get() == Some(g)).then_some("");

    view! {
        <PlotShell scope="raincloud-chart" label hint="Use the arrow keys to move between groups." height readout format active>
            <div data-part="rain" data-orientation=orientation.attr() data-active=move || active.get().map(|_| "")>
                <svg data-part="clouds" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                    {move || {
                        layout
                            .get()
                            .clouds
                            .into_iter()
                            .map(|c| {
                                let fill = colour(c.colour);
                                view! {
                                    <g data-part="cloud" data-active=picked(c.group) style=format!("--i: {};", c.group.min(12))>
                                        <path data-part="area" d=c.area fill=fill.clone()></path>
                                        <path data-part="line" d=c.line stroke=fill></path>
                                    </g>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                <div data-part="drops" aria-hidden="true">
                    <Show
                        when=move || big.get()
                        fallback=move || {
                            layout
                                .get()
                                .dots
                                .into_iter()
                                .map(|d| {
                                    let style = format!("left: {}; bottom: {}; background: {}; --i: {};", pct(d.x), pct(d.y), colour(d.group), d.index.min(40));
                                    view! { <span data-part="dot" style=style></span> }
                                })
                                .collect_view()
                        }
                    >
                        <RasterCanvas colours draw />
                    </Show>
                </div>
                <div data-part="boxes" aria-hidden="true">
                    {move || {
                        let l = layout.get();
                        let n = l.boxes.len();
                        l.boxes
                            .into_iter()
                            .enumerate()
                            .filter_map(|(g, b)| b.map(|b| (g, b)))
                            .map(|(g, b)| {
                                let o = l.orientation;
                                view! {
                                    <div data-part="box-plot" data-active=picked(g) style=format!("--i: {}; --c: {};", g.min(12), colour(probe::slot(g, n)))>
                                        <span data-part="whisker" style=b.span(o, b.low, b.high)></span>
                                        <span data-part="box" style=b.span(o, b.q1, b.q3)></span>
                                        <span data-part="median" style=b.span(o, b.median, b.median)></span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </div>
        </PlotShell>
    }
}
