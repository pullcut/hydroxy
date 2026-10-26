//! `<SankeyChart>`: flows between stages, link width by value. Layout comes from
//! `oxide_charts::sankey`. Hover or focus a node or link to light its whole path.

use leptos::prelude::*;
use oxide_charts::{format_value, sankey};

pub use oxide_charts::sankey::Link;

fn colour(i: usize) -> String {
    format!("var(--chart-{})", i % 6 + 1)
}

#[derive(Clone, Copy, PartialEq)]
enum Active {
    Node(usize),
    Link(usize),
}

/// A flow diagram. Nodes sit in columns by longest path and link widths follow their
/// values. Tab through the nodes to light up everything upstream and downstream.
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
    /// Formats values for labels and the table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (w, h) = (f64::from(width), f64::from(height));
    let layout = Memo::new(move |_| links.with(|l| sankey::layout(l, w, h, 12.0, 14.0, 6)));
    let active = RwSignal::new(None::<Active>);
    let lit = Memo::new(move |_| {
        active.get().map(|a| {
            layout.with(|l| match a {
                Active::Node(i) => sankey::highlight_node(l, i),
                Active::Link(k) => sankey::highlight_link(l, k),
            })
        })
    });
    let node_lit = move |i: usize| lit.with(|x| x.as_ref().is_some_and(|x| x.nodes.get(i).copied().unwrap_or(false))).then_some("");
    let link_lit = move |k: usize| lit.with(|x| x.as_ref().is_some_and(|x| x.links.get(k).copied().unwrap_or(false))).then_some("");

    let caption = label.clone();
    view! {
        <figure data-scope="sankey-chart" data-part="root" data-active=move || active.get().map(|_| "") style=format!("max-width: {width}px;")>
            <svg viewBox=format!("0 0 {width} {height}") role="group" aria-label=label>
                {move || {
                    let l = layout.get();
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
                                    d=r.path.clone()
                                    stroke=colour(r.source)
                                    stroke-width=r.width
                                    on:pointerenter=move |_| active.set(Some(Active::Link(k)))
                                    on:pointerleave=move |_| active.set(None)
                                >
                                    <title>{format!("{} to {}: {}", l.nodes[r.source].name, l.nodes[r.target].name, fmt(r.value))}</title>
                                </path>
                            }
                        })
                        .collect_view();
                    let nodes = l
                        .nodes
                        .iter()
                        .enumerate()
                        .map(|(i, n)| {
                            let at_end = n.column == last && last > 0;
                            let (tx, anchor) = if at_end { (n.x - 6.0, "end") } else { (n.x + l.node_width + 6.0, "start") };
                            view! {
                                <g
                                    data-part="node"
                                    data-highlighted=move || node_lit(i)
                                    tabindex="0"
                                    role="img"
                                    aria-label=format!("{}: {}", n.name, fmt(n.value))
                                    on:pointerenter=move |_| active.set(Some(Active::Node(i)))
                                    on:pointerleave=move |_| active.set(None)
                                    on:focus=move |_| active.set(Some(Active::Node(i)))
                                    on:blur=move |_| active.set(None)
                                >
                                    <rect x=n.x y=n.y width=l.node_width height=n.height.max(1.0) rx="2" fill=colour(i)></rect>
                                    <text data-part="node-label" x=tx y=n.y + n.height / 2.0 text-anchor=anchor dominant-baseline="middle">
                                        {n.name.clone()}
                                        <tspan data-part="node-value" dx="6">{fmt(n.value)}</tspan>
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
