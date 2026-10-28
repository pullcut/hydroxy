//! `<ViolinChart>`: a mirrored density per group with an optional inner box plot.
//! Shapes come from `hydroxy_charts::density::violin`, box stats from `hydroxy_charts::boxplot`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{density, probe};
use leptos::prelude::*;

use crate::density_chart::curves;
use crate::plot_shell::{PlotShell, pct};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::boxplot::Group;

/// Groups side by side, each drawn as its density mirrored about a centre line, with the
/// median marked. Hover or touch a violin to read its numbers; focus the plot and use
/// the arrow keys to move between groups, Home and End to jump, Escape to clear.
#[component]
pub fn ViolinChart(
    /// The groups, left to right, each with its raw values.
    #[prop(into)]
    groups: Signal<Vec<Group>>,
    /// What the chart shows, such as "Deploy time by team". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Draw the quartile box and whiskers inside each violin.
    #[prop(default = true)]
    inner_box: bool,
    /// Kernel bandwidth in data units. Default: Silverman's rule per group.
    #[prop(optional, into)]
    bandwidth: Option<f64>,
    /// Reference lines, bands and labelled points. Y values are data values, x values are group indices.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[prop(default = 260)]
    height: u16,
    /// Formats values for the axis, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the hovered or focused violin changes; `None` when it clears.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a violin.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let layout = Memo::new(move |_| groups.with(|g| annotations.with(|a| density::violin(g, bandwidth, a))));
    let readout = Signal::derive(move || layout.with(|l| l.readout.clone()));
    let active = RwSignal::new(None::<usize>);

    let datum = Signal::derive(move || {
        let g = active.get()?;
        let name = groups.with(|gr| gr.get(g).map(|gr| gr.label.clone()))?;
        let median = layout.with(|l| l.boxes.get(g).and_then(Option::as_ref).map_or(f64::NAN, |b| b.stats.median));
        Some(EventDatum::category(g, name, median))
    });

    view! {
        <PlotShell scope="violin-chart" label hint="Use the left and right arrow keys to move between groups." height readout format active datum=Some(datum) on_hover on_select>
            <svg data-part="marks" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
                {move || curves(layout.with(|l| l.shapes.clone()))}
            </svg>
            <div data-part="boxes" aria-hidden="true">
                {move || {
                    let boxes = layout.with(|l| l.boxes.clone());
                    let n = boxes.len();
                    boxes
                        .into_iter()
                        .enumerate()
                        .filter_map(|(i, b)| b.map(|b| (i, b)))
                        .map(|(i, b)| {
                            let inner = inner_box.then(|| {
                                view! {
                                    <span data-part="whisker" style=format!("bottom: {}; height: {};", pct(b.low), pct(b.high - b.low))></span>
                                    <span data-part="box" style=format!("bottom: {}; height: {};", pct(b.q1), pct(b.q3 - b.q1))></span>
                                }
                            });
                            view! {
                                <div data-part="inner" style=format!("left: {}; --i: {i};", pct(probe::band(i, n)))>
                                    {inner}
                                    <span data-part="median" style=format!("bottom: {};", pct(b.median))></span>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </PlotShell>
    }
}
