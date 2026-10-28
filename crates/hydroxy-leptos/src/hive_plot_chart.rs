//! `<HivePlotChart>`: two to four axes radiating from the centre, one per node type. A
//! node sits on its type's axis, as far out as its value, so the picture depends only on
//! the data and never on a layout's whims. Geometry comes from `hydroxy_charts::hive`.

use leptos::prelude::*;
use hydroxy_charts::arc_diagram::colour;
use hydroxy_charts::{format_value, hive, nav};

pub use hydroxy_charts::hive::{Link, Node};

fn axis_name(names: &[String], axis: usize) -> String {
    names.get(axis).cloned().unwrap_or_else(|| format!("Axis {}", axis + 1))
}

/// How many axes to draw: one per name, or one per axis the nodes use, 2 to 4.
fn axis_count(names: &[String], nodes: &[Node]) -> usize {
    if names.is_empty() { nodes.iter().map(|n| n.axis + 1).max().unwrap_or(2) } else { names.len() }
}

fn pct(px: f64) -> f64 {
    px / hive::SIZE * 100.0
}

/// A hive plot. Hover, touch or focus a node to light its links and the nodes at their
/// other ends. Focus the chart and use the arrow keys to step through the nodes axis by
/// axis, centre outwards; Home and End jump, Escape clears.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn HivePlotChart(
    /// The nodes: an id, the axis for its type, and a value that sets how far out it sits.
    #[prop(into)]
    nodes: Signal<Vec<Node>>,
    /// The links, by node id. Links within one axis and to unknown ids are ignored.
    #[prop(into)]
    links: Signal<Vec<Link>>,
    /// What the chart shows, such as "Service calls by tier". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Axis names, by axis number. Two to four; their count sets the number of axes.
    #[prop(optional, into)]
    axes: Vec<String>,
    /// Name of the value that places nodes along an axis, for the tooltip and table.
    #[prop(default = "Value".into(), into)]
    value_label: String,
    /// Formats node values for the tooltip, axis ends and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let names = StoredValue::new(axes);
    let layout = Memo::new(move |_| {
        nodes.with(|n| links.with(|l| names.with_value(|a| hive::layout(n, l, axis_count(a, n)))))
    });
    let aname = move |a: usize| names.with_value(|n| axis_name(n, a));
    let order = Memo::new(move |_| layout.with(hive::order));
    let active = RwSignal::new(None::<usize>);
    let near = Memo::new(move |_| active.get().map(|i| layout.with(|l| hive::neighbours(l, i))));
    let node_lit = move |i: usize| near.with(|n| n.as_ref().is_some_and(|n| n.get(i).copied().unwrap_or(false))).then_some("");
    let pick = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };
    let value_name = StoredValue::new(value_label);
    let describe = move |i: usize| {
        layout.with(|l| {
            let node = l.nodes.get(i)?;
            Some(format!("{}, {}: {} {}, {} links", node.id, aname(node.axis), value_name.get_value(), fmt(node.value), node.degree))
        })
    };
    let step = move |key: &str| {
        order.with(|o| {
            let pos = active.get_untracked().and_then(|a| o.iter().position(|&x| x == a));
            nav::list_key(key, pos, o.len()).map(|next| next.map(|p| o[p]))
        })
    };
    let linked = move |l: &hive::Hive, i: usize| {
        let near = hive::neighbours(l, i);
        l.nodes.iter().enumerate().filter(|(j, _)| *j != i && near[*j]).map(|(_, n)| n.id.clone()).collect::<Vec<_>>()
    };

    let group_label = format!("{label}. Use the arrow keys to step through the nodes.");
    let caption = label;
    view! {
        <figure data-scope="hive-plot-chart" data-part="root" data-active=move || active.get().map(|_| "")>
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=group_label
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if let Some(next) = step(&ev.key()) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg viewBox=format!("0 0 {0} {0}", hive::SIZE) aria-hidden="true">
                    {move || {
                        let l = layout.get();
                        let axes = l
                            .axes
                            .iter()
                            .enumerate()
                            .map(|(k, a)| {
                                view! {
                                    <line data-part="axis" pathLength="1" x1=a.from.0 y1=a.from.1 x2=a.to.0 y2=a.to.1 style=format!("--i: {k};")></line>
                                    <text data-part="axis-label" x=a.label.0 y=a.label.1 text-anchor=a.anchor dominant-baseline="central">
                                        {aname(k)}
                                    </text>
                                }
                            })
                            .collect_view();
                        let edges = l
                            .edges
                            .iter()
                            .enumerate()
                            .map(|(k, e)| {
                                let (src, dst) = (e.source, e.target);
                                view! {
                                    <path
                                        data-part="link"
                                        pathLength="1"
                                        data-highlighted=move || (active.get() == Some(src) || active.get() == Some(dst)).then_some("")
                                        d=e.path.clone()
                                        stroke=colour(l.nodes[src].axis)
                                        style=format!("--i: {};", k.min(40))
                                    ></path>
                                }
                            })
                            .collect_view();
                        let dots = l
                            .nodes
                            .iter()
                            .enumerate()
                            .map(|(i, node)| {
                                view! {
                                    <g
                                        data-part="node"
                                        data-axis=node.axis
                                        data-highlighted=move || node_lit(i)
                                        data-selected=move || (active.get() == Some(i)).then_some("")
                                        style=format!("--i: {};", i.min(40))
                                        on:pointerenter=move |_| pick(i)
                                        on:pointerdown=move |_| pick(i)
                                    >
                                        <circle data-part="hit" cx=node.x cy=node.y r="10"></circle>
                                        <circle data-part="dot" cx=node.x cy=node.y r="4.5" fill=colour(node.axis)></circle>
                                    </g>
                                }
                            })
                            .collect_view();
                        view! {
                            <g data-part="axes">{axes}</g>
                            <g data-part="links">{edges}</g>
                            <g data-part="nodes">{dots}</g>
                        }
                    }}
                </svg>
                {move || {
                    let i = active.get()?;
                    let node = layout.with(|l| l.nodes.get(i).cloned())?;
                    let (fx, fy) = (pct(node.x), pct(node.y));
                    Some(
                        view! {
                            <div
                                data-part="tooltip"
                                data-anchored=""
                                data-side={if fx > 50.0 { "left" } else { "right" }}
                                style=format!("left: {fx:.3}%; top: {fy:.3}%;")
                                aria-hidden="true"
                            >
                                <div data-part="tooltip-label">
                                    <span data-part="swatch" style=format!("background: {};", colour(node.axis))></span>
                                    {node.id}
                                </div>
                                <ul>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Type"</span>
                                        <span data-part="tooltip-value">{aname(node.axis)}</span>
                                    </li>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">{value_name.get_value()}</span>
                                        <span data-part="tooltip-value">{fmt(node.value)}</span>
                                    </li>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Links"</span>
                                        <span data-part="tooltip-value">{node.degree}</span>
                                    </li>
                                </ul>
                            </div>
                        },
                    )
                }}
            </div>
            <p data-part="scale-note" aria-hidden="true">
                {move || {
                    let (lo, hi) = layout.with(|l| l.domain);
                    format!("{}: {} at the centre to {} at the tips", value_name.get_value(), fmt(lo), fmt(hi))
                }}
            </p>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Node"</th>
                        <th scope="col">"Type"</th>
                        <th scope="col">{move || value_name.get_value()}</th>
                        <th scope="col">"Links"</th>
                        <th scope="col">"Linked to"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let l = layout.get();
                        order
                            .get()
                            .into_iter()
                            .map(|i| {
                                let node = &l.nodes[i];
                                view! {
                                    <tr>
                                        <th scope="row">{node.id.clone()}</th>
                                        <td>{aname(node.axis)}</td>
                                        <td>{fmt(node.value)}</td>
                                        <td>{node.degree}</td>
                                        <td>{linked(&l, i).join(", ")}</td>
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
