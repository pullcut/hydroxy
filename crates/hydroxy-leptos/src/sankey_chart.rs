//! `<SankeyChart>`: flows between stages, link width by value. Layout comes from
//! `hydroxy_charts::sankey`. Hover, touch or focus a node or link to light its whole path.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{format_value, nav, sankey};
use leptos::prelude::*;

use crate::callbacks::{select_active, watch_hover};
use crate::plot_kit::{ChartTip, colour};

pub use hydroxy_charts::sankey::Link;

/// A flow diagram. Nodes sit in columns by longest path and link widths follow their
/// values. Hover or touch a node or link to light everything upstream and downstream
/// and read its values; focus the chart and use the arrow keys to step through the nodes
/// and then the links, Home and End to jump, Escape to clear.
/// Links that would close a cycle are ignored.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SankeyChart(
    /// The flows. Nodes are created from the names, in order of first appearance.
    #[prop(into)]
    links: Signal<Vec<Link>>,
    /// What the chart shows, such as "Signups to plan". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Drawing width in px; the chart scales down to fit its container.
    #[prop(default = 720)]
    width: u16,
    /// Drawing height in px.
    #[prop(default = 340)]
    height: u16,
    /// Formats values for labels, the tooltip and the table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called with the node or link under the pointer or keyboard focus, and `None` when it clears.
    #[prop(optional)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called with the node or link when it is clicked, tapped or Enter is pressed on it.
    #[prop(optional)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (box_w, box_h) = (f64::from(width), f64::from(height));
    let layout = Memo::new(move |_| links.with(|l| sankey::layout(l, box_w, box_h, 12.0, 14.0, 6)));
    // Marks are the nodes first, then the links.
    let active = RwSignal::new(None::<usize>);
    // Nodes are series 0 and links series 1; `index` counts within each.
    let datum = Signal::derive(move || {
        let m = active.get()?;
        layout.with(|l| {
            if let Some(node) = l.nodes.get(m) {
                return Some(EventDatum {
                    series: 0,
                    index: m,
                    label: Some(node.name.clone()),
                    value: node.value,
                    node: None,
                });
            }
            let k = m - l.nodes.len();
            let link = l.links.get(k)?;
            let name = format!("{} to {}", l.nodes[link.source].name, l.nodes[link.target].name);
            Some(EventDatum { series: 1, index: k, label: Some(name), value: link.value, node: None })
        })
    });
    watch_hover(Some(datum), on_hover);
    let lit = Memo::new(move |_| {
        active.get().map(|m| {
            layout.with(|l| {
                if m < l.nodes.len() {
                    sankey::highlight_node(l, m)
                } else {
                    sankey::highlight_link(l, m - l.nodes.len())
                }
            })
        })
    });
    let node_lit = move |i: usize| {
        lit.with(|x| x.as_ref().is_some_and(|x| x.nodes.get(i).copied().unwrap_or(false))).then_some("")
    };
    let link_lit = move |k: usize| {
        lit.with(|x| x.as_ref().is_some_and(|x| x.links.get(k).copied().unwrap_or(false))).then_some("")
    };
    let pick = move |m: usize| {
        if active.get_untracked() != Some(m) {
            active.set(Some(m));
        }
    };
    let describe = move |m: usize| layout.with(|l| sankey::tooltip(l, m, (box_w, box_h), &fmt).map(|t| t.announce()));

    let group_label = format!("{label}. Use the arrow keys to step through the nodes and links.");
    let caption = label;
    view! {
        <figure data-scope="sankey-chart" data-part="root" data-active=move || active.get().map(|_| "") style=format!("max-width: {width}px;")>
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=group_label
                on:click=move |_| select_active(Some(datum), on_select)
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                        select_active(Some(datum), on_select);
                    }
                    let len = layout.with(|l| l.nodes.len() + l.links.len());
                    if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), len) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg viewBox=format!("0 0 {width} {height}") aria-hidden="true">
                    {move || {
                        let l = layout.get();
                        let n = l.nodes.len();
                        let last = l.nodes.iter().map(|n| n.column).max().unwrap_or(0);
                        let ribbons = l
                            .links
                            .iter()
                            .enumerate()
                            .map(|(k, r)| {
                                view! {
                                    <path
                                        data-part="link"
                                        data-highlighted=move || link_lit(k)
                                        style=format!("--i: {k};")
                                        d=r.path.clone()
                                        stroke=colour(r.source)
                                        stroke-width=r.width
                                        on:pointerenter=move |_| pick(n + k)
                                        on:pointerdown=move |_| pick(n + k)
                                    ></path>
                                }
                            })
                            .collect_view();
                        let nodes = l
                            .nodes
                            .iter()
                            .enumerate()
                            .map(|(i, node)| {
                                let at_end = node.column == last && last > 0;
                                let (tx, anchor) = if at_end { (node.x - 6.0, "end") } else { (node.x + l.node_width + 6.0, "start") };
                                view! {
                                    <g
                                        data-part="node"
                                        data-highlighted=move || node_lit(i)
                                        data-selected=move || (active.get() == Some(i)).then_some("")
                                        on:pointerenter=move |_| pick(i)
                                        on:pointerdown=move |_| pick(i)
                                    >
                                        <rect x=node.x y=node.y width=l.node_width height=node.height.max(1.0) rx="2" fill=colour(i)></rect>
                                        <text data-part="node-label" x=tx y=node.y + node.height / 2.0 text-anchor=anchor dominant-baseline="middle">
                                            {node.name.clone()}
                                            <tspan data-part="node-value" dx="6">{fmt(node.value)}</tspan>
                                        </text>
                                    </g>
                                }
                            })
                            .collect_view();
                        view! {
                            <g data-part="links">{ribbons}</g>
                            <g data-part="nodes">{nodes}</g>
                        }
                    }}
                </svg>
                {move || {
                    let m = active.get()?;
                    let (tip, swatch) =
                        layout.with(|l| Some((sankey::tooltip(l, m, (box_w, box_h), &fmt)?, sankey::swatch(l, m)?)))?;
                    Some(view! { <ChartTip tip swatch anchored=true /> })
                }}
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"From"</th>
                        <th scope="col">"To"</th>
                        <th scope="col">"Value"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let l = layout.get();
                        l.links
                            .iter()
                            .map(|r| {
                                view! {
                                    <tr>
                                        <td>{l.nodes[r.source].name.clone()}</td>
                                        <td>{l.nodes[r.target].name.clone()}</td>
                                        <td>{fmt(r.value)}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
