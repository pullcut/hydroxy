//! `BeeswarmChart`: every value as a dot, packed so none overlap, optionally one row
//! per group. Geometry comes from `hydroxy_charts::swarm`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav;
use hydroxy_charts::swarm::{self, Arrange};

use crate::density_chart::stop_datum;
use crate::plot_kit::formatter;
use crate::plot_shell::PlotShell;
use crate::strip_plot_chart::dots;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::boxplot::Group;
pub use hydroxy_charts::scale::ScaleKind;

/// Dot diameter in px, matching the CSS.
const DOT: f64 = 8.0;
// packing assumes a 480px-wide plot, so narrow plots overlap a little; measure the plot if that matters.
const NOMINAL_WIDTH: f64 = 480.0;

/// Values stacked outwards from each row's centre line where they crowd, so the shape of
/// the distribution shows and every dot stays visible. Hover or touch to read a dot;
/// focus the plot and use the arrow keys to step through them, Home and End to jump,
/// Escape to clear.
#[component]
pub fn BeeswarmChart(
    /// The groups, one row each from the top. Pass one group for a single swarm.
    groups: ReadSignal<Vec<Group>>,
    /// What the chart shows, such as "Session length by plan". Used as the accessible name.
    label: String,
    /// Value axis mapping.
    #[props(default)]
    scale: ScaleKind,
    /// Reference lines, bands and labelled points. X values are data values, y values are row indices.
    #[props(default)]
    annotations: ReadSignal<Vec<Annotation>>,
    /// Plot height in px, excluding the axis and legend.
    #[props(default = 260)]
    height: u16,
    /// Formats values for the axis, tooltip and table.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let g = groups();
    let arrange = Arrange::Swarm { dx: DOT / NOMINAL_WIDTH, dy: DOT / f64::from(height.max(1)) };
    let layout = swarm::layout(&g, scale, arrange, &annotations());
    let active = use_signal(|| None::<usize>);
    let fmt = formatter(format);
    let cur = if on_hover.is_some() || on_select.is_some() {
        active().and_then(|i| {
            let group = layout.dots.get(i)?.group;
            let index = layout.dots[..i].iter().filter(|d| d.group == group).count();
            stop_datum(&layout.readout, i, group, index, fmt)
        })
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();
    rsx! {
        PlotShell { scope: "beeswarm-chart", label, hint: nav::HINT_STEP_VALUES, height, readout: layout.readout, format, active, datum: Some(datum), on_hover, on_select,
            {dots(&layout.dots, g.len())}
        }
    }
}
