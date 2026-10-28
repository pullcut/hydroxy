//! `<SmallMultiples>`: a responsive grid with one chart per facet, every facet on the same
//! value axis. Domain sharing comes from `hydroxy_charts::small_multiples`.

use leptos::prelude::*;
use hydroxy_charts::small_multiples::{self, FacetKind};
use hydroxy_charts::{Datum, plot};

pub use hydroxy_charts::line::Series;
pub use hydroxy_charts::small_multiples::{Facet, FacetKind as Kind};

use crate::bar_chart::{BarChart, Highlight};
use crate::chart_frame::{ExportRows, provide_export};
use crate::line_chart::LineChart;
use crate::stacked_area_chart::StackedAreaChart;

/// One chart per facet in a grid that wraps to the container, sharing one value axis so
/// heights compare across panels. The series keep their colours in every facet, and one
/// legend above the grid names them. Panels enter one after another.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SmallMultiples(
    /// The panels, in reading order. Each facet's series share names and order.
    #[prop(into)]
    facets: Signal<Vec<Facet>>,
    /// One label per x position, shared by every facet.
    #[prop(into)]
    labels: Signal<Vec<String>>,
    /// What the grid shows, such as "Weekly signups by region". Each panel's accessible
    /// name is this plus its title.
    #[prop(into)]
    label: String,
    /// `Line` (default), `Area`, `StackedArea` or `Bar`. Bar draws each facet's first series.
    #[prop(optional)]
    kind: FacetKind,
    /// Plot height of each panel in px.
    #[prop(default = 140)]
    height: u16,
    /// Narrowest a panel gets before the grid wraps to fewer columns, in px.
    #[prop(default = 240)]
    min_width: u16,
    /// Give every facet its own value axis instead of the shared one.
    #[prop(optional)]
    independent: bool,
    /// Formats values for the axes, tooltips and tables.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let domain =
        Memo::new(move |_| if independent { None } else { facets.with(|f| small_multiples::shared_domain(f, kind)) });
    let names = Memo::new(move |_| {
        facets.with(|f| {
            f.first().map(|f| f.series.iter().map(|s| s.name.clone()).collect::<Vec<_>>()).unwrap_or_default()
        })
    });
    let fmt = move |v: f64| format.map_or_else(|| hydroxy_charts::format_value(v), |f| f.run(v));
    // The frame's CSV gets every facet; each panel's own export goes to a slot nobody reads.
    provide_export(move || {
        let head = ["Facet".to_string(), String::new()].into_iter().chain(names.get_untracked()).collect::<Vec<_>>();
        let body = facets.with_untracked(|f| {
            labels.with_untracked(|l| {
                f.iter()
                    .flat_map(|facet| {
                        l.iter().enumerate().map(move |(i, x)| {
                            [facet.title.clone(), x.clone()]
                                .into_iter()
                                .chain(facet.series.iter().map(|s| plot::cell(s.values.get(i).copied(), fmt)))
                                .collect::<Vec<_>>()
                        })
                    })
                    .collect::<Vec<_>>()
            })
        });
        std::iter::once(head).chain(body).collect()
    });
    provide_context(ExportRows(StoredValue::new(None)));
    let name = StoredValue::new(label);
    let panel = move |i: usize, facet: Facet| {
        let title = facet.title.clone();
        let label = format!("{}: {}", name.get_value(), facet.title);
        let y_domain = domain.get_untracked();
        let chart = match kind {
            FacetKind::Line | FacetKind::Area => {
                view! { <LineChart series=facet.series labels=labels label height area=kind == FacetKind::Area format=format.unwrap_or_else(|| Callback::new(hydroxy_charts::format_value)) y_domain /> }
                    .into_any()
            }
            FacetKind::StackedArea => {
                view! { <StackedAreaChart series=facet.series labels=labels label height format=format.unwrap_or_else(|| Callback::new(hydroxy_charts::format_value)) y_domain /> }
                    .into_any()
            }
            FacetKind::Bar => {
                let values = facet.series.into_iter().next().map(|s| s.values).unwrap_or_default();
                let data = Signal::derive(move || labels.with(|l| l.iter().zip(&values).map(|(l, &v)| Datum::new(l.clone(), v)).collect::<Vec<_>>()));
                view! { <BarChart data label height highlight=Highlight::Off format=format.unwrap_or_else(|| Callback::new(hydroxy_charts::format_value)) y_domain /> }
                    .into_any()
            }
        };
        view! {
            <section data-part="facet" style=format!("--facet: {i};")>
                <h4 data-part="facet-title">{title}</h4>
                {chart}
            </section>
        }
    };

    view! {
        <div data-scope="small-multiples" data-part="root" data-kind=kind.attr() role="group" aria-label=name.get_value()>
            <Show when=move || names.with(|n| n.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        names
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, n)| {
                                view! {
                                    <li data-part="legend-item">
                                        <span data-part="swatch" style=format!("background: {};", plot::colour(i))></span>
                                        {n}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <div data-part="grid" style=format!("--min: {min_width}px;")>
                {move || {
                    // Re-read the domain so new data re-lays out every panel on it.
                    domain.track();
                    facets.get().into_iter().enumerate().map(|(i, f)| panel(i, f)).collect_view()
                }}
            </div>
        </div>
    }
}
