//! `<DependencyGraphChart>`: a directed graph in layers, drawn left to right. The
//! layered layout comes from `hydroxy_charts::dag`. Hover or focus a node to trace
//! everything upstream and downstream of it.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{dag, format_value, nav};
use leptos::prelude::*;

use crate::plot_kit::{ChartTip, colour_or_muted};
use crate::rank_kit::{GroupLegend, distinct};

pub use hydroxy_charts::network::{Link, Node};

fn colour(group: usize) -> String {
    colour_or_muted(group, 6)
}

/// A layered dependency graph. Focus the chart and use the arrow keys to step through
/// nodes column by column, Home and End to jump, Escape to clear. The active node's
/// upstream and downstream stay lit and the rest fade.
#[component]
pub fn DependencyGraphChart(
    /// The nodes. Ids must be unique; they are also the labels. Groups set the colour, up to six.
    #[prop(into)]
    nodes: Signal<Vec<Node>>,
    /// The links, source → target: the source feeds the target. Unknown ids and self-links are ignored.
    #[prop(into)]
    links: Signal<Vec<Link>>,
    /// What the chart shows, such as "Build pipeline". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Drawing width in px; the chart scales down to fit its container.
    #[prop(default = 720)]
    width: u16,
    /// Drawing height in px.
    #[prop(default = 360)]
    height: u16,
    /// Legend names for the groups, by group number. Missing names read "Group N".
    #[prop(optional, into)]
    group_names: Vec<String>,
    /// Width of each node box in px.
    #[prop(default = 112)]
    node_width: u16,
    /// Formats link weights in the tooltip and readout.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a node, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a node.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (w, h, nw) = (f64::from(width), f64::from(height), f64::from(node_width));
    let layout = Memo::new(move |_| nodes.with(|n| links.with(|l| dag::layout(n, l, w, h, nw))));
    let active = RwSignal::new(None::<usize>);
    let reach = Memo::new(move |_| active.get().map(|i| layout.with(|l| dag::reach(l, i))));
    let position = move || active.get().and_then(|i| layout.with(|l| l.order.iter().position(|&o| o == i)));
    let node_lit =
        move |i: usize| reach.with(|r| r.as_ref().map(|(up, down)| active.get() == Some(i) || up[i] || down[i]));
    let edge_lit = move |s: usize, t: usize| {
        let a = active.get();
        reach.with(|r| {
            r.as_ref().map(|(up, down)| (down[t] && (a == Some(s) || down[s])) || (up[s] && (a == Some(t) || up[t])))
        })
    };
    let flows = move |i: usize| {
        layout.with(|l| {
            l.edges.iter().fold((0.0, 0.0), |(inn, out), e| {
                (inn + if e.target == i { e.value } else { 0.0 }, out + if e.source == i { e.value } else { 0.0 })
            })
        })
    };
    let counts = move || {
        reach.with(|r| {
            r.as_ref().map(|(up, down)| (up.iter().filter(|x| **x).count(), down.iter().filter(|x| **x).count()))
        })
    };
    let describe = move |i: usize| {
        let (up, down) = counts().unwrap_or_default();
        layout.with(|l| {
            l.nodes.get(i).map(|n| format!("{}: layer {}, {up} upstream, {down} downstream", n.id, n.layer + 1))
        })
    };
    let datum = Signal::derive(move || {
        let i = active.get()?;
        let (inn, out) = flows(i);
        layout.with(|l| {
            let n = l.nodes.get(i)?;
            Some(EventDatum { series: n.group, index: i, label: Some(n.id.clone()), value: inn.max(out), node: None })
        })
    });
    crate::callbacks::watch_hover(Some(datum), on_hover);
    let groups = Memo::new(move |_| nodes.with(|n| distinct(n.iter().map(|n| n.group.min(6)))));
    let group_names = StoredValue::new(group_names);
    let group_label = Callback::new(move |g: usize| {
        if g < 6 {
            group_names.with_value(|n| n.get(g).cloned()).unwrap_or_else(|| format!("Group {}", g + 1))
        } else {
            "Other".to_string()
        }
    });
    let caption = label.clone();
    let half = nw / 2.0;

    view! {
        <figure data-scope="dependency-graph-chart" data-part="root" data-active=move || active.get().map(|_| "") style=format!("max-width: {width}px;")>
            <div data-part="plot">
                <svg
                    data-part="marks"
                    viewBox=format!("0 0 {width} {height}")
                    tabindex="0"
                    role="group"
                    aria-label=format!("{label}. Use the arrow keys to step through nodes.")
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:click=move |_| crate::callbacks::select_active(Some(datum), on_select)
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            crate::callbacks::select_active(Some(datum), on_select);
                        }
                        let order = layout.with(|l| l.order.clone());
                        if let Some(next) = nav::list_key(&ev.key(), position(), order.len()) {
                            ev.prevent_default();
                            active.set(next.map(|k| order[k]));
                        }
                    }
                >
                    <g data-part="links" style=move || format!("--ox-n: {};", layout.with(|l| l.edges.len()))>
                        {move || {
                            layout
                                .get()
                                .edges
                                .into_iter()
                                .enumerate()
                                .map(|(k, e)| {
                                    let (s, t) = (e.source, e.target);
                                    view! {
                                        <path
                                            data-part="link"
                                            data-reversed=e.reversed.then_some("")
                                            pathLength=(!e.reversed).then_some("1")
                                            data-lit=move || (edge_lit(s, t) == Some(true)).then_some("")
                                            data-dim=move || (edge_lit(s, t) == Some(false)).then_some("")
                                            d=e.path
                                            style=format!("--i: {k};")
                                        ></path>
                                    }
                                })
                                .collect_view()
                        }}
                    </g>
                    <g data-part="nodes" style=move || format!("--ox-n: {};", layout.with(|l| l.layers))>
                        {move || {
                            layout
                                .get()
                                .nodes
                                .into_iter()
                                .enumerate()
                                .map(|(i, n)| {
                                    view! {
                                        <g
                                            data-part="node"
                                            data-active=move || (active.get() == Some(i)).then_some("")
                                            data-lit=move || (node_lit(i) == Some(true)).then_some("")
                                            data-dim=move || (node_lit(i) == Some(false)).then_some("")
                                            style=format!("--c: {}; --i: {};", colour(n.group), n.layer)
                                            on:pointerenter=move |_| active.set(Some(i))
                                            on:pointerdown=move |_| active.set(Some(i))
                                        >
                                            <rect x=format!("{:.1}", n.x - half) y=format!("{:.1}", n.y - 14.0) width=nw height="28" rx="6"></rect>
                                            <text data-part="label" x=format!("{:.1}", n.x) y=format!("{:.1}", n.y) text-anchor="middle" dominant-baseline="central">
                                                {n.id.clone()}
                                            </text>
                                        </g>
                                    }
                                })
                                .collect_view()
                        }}
                    </g>
                </svg>
                {move || {
                    let i = active.get()?;
                    let n = layout.with(|l| l.nodes.get(i).cloned())?;
                    let (up, down) = counts().unwrap_or_default();
                    let (inn, out) = flows(i);
                    let (left, top) = (n.x / w * 100.0, (n.y - 14.0) / h * 100.0);
                    // The tooltip opens below the node near the top edge (top < 25%), above it elsewhere.
                    let tip = Tip {
                        x: left / 100.0,
                        y: if top < 25.0 { 1.0 } else { 0.0 },
                        title: n.id.clone(),
                        rows: vec![
                            TipRow::new(None, "Layer", (n.layer + 1).to_string()),
                            TipRow::new(None, "Upstream", up.to_string()),
                            TipRow::new(None, "Downstream", down.to_string()),
                            TipRow::new(None, "Flow in", fmt(inn)),
                            TipRow::new(None, "Flow out", fmt(out)),
                        ],
                    };
                    let swatch = colour(n.group);
                    let style = format!("left: {left:.2}%; top: {top:.2}%;");
                    Some(view! { <ChartTip tip swatch style /> })
                }}
            </div>
            <GroupLegend groups=groups name=group_label colour=colour />
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Node"</th>
                        <th scope="col">"Layer"</th>
                        <th scope="col">"Comes from"</th>
                        <th scope="col">"Feeds into"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let l = layout.get();
                        let names = |pick: &dyn Fn(&dag::DagEdge) -> Option<usize>| {
                            l.edges.iter().filter_map(pick).map(|j| l.nodes[j].id.clone()).collect::<Vec<_>>().join(", ")
                        };
                        l.nodes
                            .iter()
                            .enumerate()
                            .map(|(i, n)| {
                                let from = names(&|e| (e.target == i).then_some(e.source));
                                let into = names(&|e| (e.source == i).then_some(e.target));
                                view! { <tr><th scope="row">{n.id.clone()}</th><td>{n.layer + 1}</td><td>{from}</td><td>{into}</td></tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
