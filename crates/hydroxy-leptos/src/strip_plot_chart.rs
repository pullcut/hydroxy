//! `<StripPlotChart>`: one row per group with every value as a dot, spread by a fixed
//! jitter so repeats stay visible. Geometry comes from `hydroxy_charts::swarm`.

use hydroxy_charts::probe;
use hydroxy_charts::swarm::{self, Arrange};
use leptos::prelude::*;

use crate::plot_shell::{PlotShell, colour, pct};

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::boxplot::Group;
pub use hydroxy_charts::scale::ScaleKind;

/// Every value of each group as a dot along a shared value axis. Hover or touch to read
/// a dot; focus the plot and use the arrow keys to step through them, Home and End to
/// jump, Escape to clear.
#[component]
pub fn StripPlotChart(
    /// The groups, one row each from the top.
    #[prop(into)]
    groups: Signal<Vec<Group>>,
    /// What the chart shows, such as "Response time by region". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// How much of each row the dots spread over, 0 to 1.
    #[prop(default = 0.6)]
    jitter: f64,
    /// Value axis mapping.
    #[prop(optional)]
    scale: ScaleKind,
    /// Reference lines, bands and labelled points. X values are data values, y values are row indices.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[prop(default = 240)]
    height: u16,
    /// Formats values for the axis, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let layout =
        Memo::new(move |_| groups.with(|g| annotations.with(|a| swarm::layout(g, scale, Arrange::Strip(jitter), a))));
    let readout = Signal::derive(move || layout.with(|l| l.readout.clone()));
    let active = RwSignal::new(None::<usize>);

    view! {
        <PlotShell scope="strip-plot-chart" label hint=hydroxy_charts::nav::HINT_STEP_VALUES height readout format active>
            <div data-part="marks" aria-hidden="true">
                {move || dots(&layout.with(|l| l.dots.clone()), groups.with(Vec::len))}
            </div>
        </PlotShell>
    }
}

/// The dots of a strip plot or beeswarm.
pub(crate) fn dots(dots: &[swarm::Dot], groups: usize) -> impl IntoView + use<> {
    dots.iter()
        .enumerate()
        .map(|(i, d)| {
            let style = format!(
                "left: {}; bottom: {}; --i: {}; background: {};",
                pct(d.x),
                pct(d.y),
                i.min(40),
                colour(probe::slot(d.group, groups))
            );
            view! { <span data-part="dot" style=style></span> }
        })
        .collect_view()
}
