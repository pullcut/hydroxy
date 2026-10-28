//! `StripPlotChart`: one row per group with every value as a dot, spread by a fixed
//! jitter so repeats stay visible. Geometry comes from `hydroxy_charts::swarm`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::probe;
use hydroxy_charts::swarm::{self, Arrange};

use crate::plot_shell::{PlotShell, colour, pct};
use crate::sparkline::use_datum;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::boxplot::Group;
pub use hydroxy_charts::scale::ScaleKind;

/// Every value of each group as a dot along a shared value axis. Hover or touch to read
/// a dot; focus the plot and use the arrow keys to step through them, Home and End to
/// jump, Escape to clear.
#[component]
pub fn StripPlotChart(
    /// The groups, one row each from the top.
    groups: ReadSignal<Vec<Group>>,
    /// What the chart shows, such as "Response time by region". Used as the accessible name.
    label: String,
    /// How much of each row the dots spread over, 0 to 1.
    #[props(default = 0.6)]
    jitter: f64,
    /// Value axis mapping.
    #[props(default)]
    scale: ScaleKind,
    /// Reference lines, bands and labelled points. X values are data values, y values are row indices.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[props(default = 240)]
    height: u16,
    /// Formats values for the axis, tooltip and table.
    format: Option<Callback<f64, String>>,
    /// Called when the hovered or focused dot changes; `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a dot.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let g = groups();
    let layout = swarm::layout(&g, scale, Arrange::Strip(jitter), &annotations());
    let active = use_signal(|| None::<usize>);
    // Stops run group by group over each group's sorted finite values.
    let datum = use_datum(active().and_then(|mut i| {
        g.iter().enumerate().find_map(|(gi, group)| {
            let mut values: Vec<f64> = group.values.iter().copied().filter(|v| v.is_finite()).collect();
            values.sort_by(f64::total_cmp);
            if i < values.len() {
                Some(EventDatum {
                    series: gi,
                    index: i,
                    label: Some(group.label.clone()),
                    value: values[i],
                    node: None,
                })
            } else {
                i -= values.len();
                None
            }
        })
    }));
    rsx! {
        PlotShell { scope: "strip-plot-chart", label, hint: hydroxy_charts::nav::HINT_STEP_VALUES, height, readout: layout.readout, format, active, datum, on_hover, on_select,
            {dots(&layout.dots, g.len())}
        }
    }
}

/// The dots of a strip plot or beeswarm.
pub(crate) fn dots(dots: &[swarm::Dot], groups: usize) -> Element {
    rsx! {
        div { "data-part": "marks", "aria-hidden": "true",
            for (i , d) in dots.iter().enumerate() {
                span {
                    key: "{i}",
                    "data-part": "dot",
                    style: "left: {pct(d.x)}; bottom: {pct(d.y)}; --i: {i.min(40)}; background: {colour(probe::slot(d.group, groups))};",
                }
            }
        }
    }
}
