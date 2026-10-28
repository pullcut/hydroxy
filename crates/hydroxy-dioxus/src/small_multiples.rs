//! `SmallMultiples`: a responsive grid with one chart per facet, every facet on the same
//! value axis. Domain sharing comes from `hydroxy_charts::small_multiples`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::small_multiples::{self, FacetKind};
use hydroxy_charts::{Datum, plot};

pub use hydroxy_charts::line::Series;
pub use hydroxy_charts::small_multiples::{Facet, FacetKind as Kind};

use crate::bar_chart::{BarChart, Highlight};
use crate::chart_frame::{ExportRows, use_export};
use crate::line_chart::LineChart;
use crate::stacked_area_chart::StackedAreaChart;

/// One chart per facet in a grid that wraps to the container, sharing one value axis so
/// heights compare across panels. The series keep their colours in every facet, and one
/// legend above the grid names them. Panels enter one after another.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SmallMultiples(
    /// The panels, in reading order. Each facet's series share names and order.
    facets: ReadSignal<Vec<Facet>>,
    /// One label per x position, shared by every facet.
    labels: ReadSignal<Vec<String>>,
    /// What the grid shows, such as "Weekly signups by region". Each panel's accessible
    /// name is this plus its title.
    label: String,
    /// `Line` (default), `Area`, `StackedArea` or `Bar`. Bar draws each facet's first series.
    #[props(default)]
    kind: FacetKind,
    /// Plot height of each panel in px.
    #[props(default = 140)]
    height: u16,
    /// Narrowest a panel gets before the grid wraps to fewer columns, in px.
    #[props(default = 240)]
    min_width: u16,
    /// Give every facet its own value axis instead of the shared one.
    #[props(default)]
    independent: bool,
    /// Formats values for the axes, tooltips and tables.
    format: Option<Callback<f64, String>>,
    /// Called with the mark under the pointer or keyboard focus in any panel, and `None` when
    /// it clears. `series` indexes the series within that panel.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called with the mark when it is clicked, tapped or Enter is pressed on it in any panel.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| hydroxy_charts::format_value(v), |f| f.call(v));
    let all = facets();
    let domain = if independent { None } else { small_multiples::shared_domain(&all, kind) };
    let names: Vec<String> = all.first().map(|f| f.series.iter().map(|s| s.name.clone()).collect()).unwrap_or_default();
    let xs = labels();
    // The frame's CSV gets every facet; each panel's own export goes to a slot nobody reads.
    use_export(move |()| {
        let all = facets.read();
        let head = ["Facet".to_string(), String::new()]
            .into_iter()
            .chain(all.first().into_iter().flat_map(|f| f.series.iter().map(|s| s.name.clone())))
            .collect::<Vec<_>>();
        let body = all.iter().flat_map(|facet| {
            labels
                .read()
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    [facet.title.clone(), x.clone()]
                        .into_iter()
                        .chain(facet.series.iter().map(|s| plot::cell(s.values.get(i).copied(), fmt)))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        });
        std::iter::once(head).chain(body).collect()
    });
    use_context_provider(|| ExportRows(CopyValue::new(None)));

    rsx! {
        div { "data-scope": "small-multiples", "data-part": "root", "data-kind": kind.attr(), role: "group", "aria-label": "{label}",
            if names.len() > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i , n) in names.iter().enumerate() {
                        li { "data-part": "legend-item",
                            span { "data-part": "swatch", style: "background: {plot::colour(i)};" }
                            "{n}"
                        }
                    }
                }
            }
            div { "data-part": "grid", style: "--min: {min_width}px;",
                for (i , facet) in all.into_iter().enumerate() {
                    section { key: "{i}-{facet.title}", "data-part": "facet", style: "--facet: {i};",
                        h4 { "data-part": "facet-title", "{facet.title}" }
                        match kind {
                            FacetKind::Line | FacetKind::Area => rsx! {
                                LineChart {
                                    series: facet.series.clone(),
                                    labels: xs.clone(),
                                    x: None,
                                    label: "{label}: {facet.title}",
                                    height,
                                    area: kind == FacetKind::Area,
                                    format,
                                    y_domain: domain,
                                    on_hover,
                                    on_select,
                                }
                            },
                            FacetKind::StackedArea => rsx! {
                                StackedAreaChart {
                                    series: facet.series.clone(),
                                    labels: xs.clone(),
                                    label: "{label}: {facet.title}",
                                    height,
                                    format,
                                    y_domain: domain,
                                    on_hover,
                                    on_select,
                                }
                            },
                            FacetKind::Bar => rsx! {
                                BarChart {
                                    data: xs.iter().zip(facet.series.first().map(|s| s.values.clone()).unwrap_or_default()).map(|(l, v)| Datum::new(l.clone(), v)).collect::<Vec<_>>(),
                                    label: "{label}: {facet.title}",
                                    height,
                                    highlight: Highlight::Off,
                                    format,
                                    y_domain: domain,
                                    on_hover,
                                    on_select,
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}
