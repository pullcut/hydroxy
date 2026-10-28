//! Shared parts for the set and part-of-whole charts (`UpSet`, Venn, Voronoi treemap,
//! nested donut and alluvial): the anchored tooltip.

use hydroxy_charts::plot::Tip;
use leptos::prelude::*;

/// The tooltip for `tip`, pinned at its anchor in % of the plot.
#[component]
pub(crate) fn SetsTip(tip: Tip) -> impl IntoView {
    let style = format!("left: {:.2}%; top: {:.2}%;", tip.x * 100.0, (1.0 - tip.y) * 100.0);
    let (side, vertical) = (tip.side(), tip.vertical());
    view! {
        <div data-part="tooltip" data-side=side data-vertical=vertical style=style aria-hidden="true">
            <div data-part="tooltip-label">{tip.title.clone()}</div>
            <ul>
                {tip
                    .rows
                    .into_iter()
                    .map(|r| {
                        view! {
                            <li data-part="tooltip-row">
                                <span data-part="tooltip-name">
                                    {r.colour.map(|c| view! { <span data-part="swatch" style=format!("background: {c};")></span> })}
                                    {r.name}
                                </span>
                                <span data-part="tooltip-value">{r.value}</span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </div>
    }
}
