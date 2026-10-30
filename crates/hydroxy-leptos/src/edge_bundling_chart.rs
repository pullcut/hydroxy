//! `<EdgeBundlingChart>`: hierarchical edge bundling. The leaves of a tree sit round a
//! circle and each link follows the tree between them, so links between the same
//! branches gather into bundles. Geometry comes from `hydroxy_charts::bundle`.

use hydroxy_charts::bundle::{self, Role};
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav;
use leptos::prelude::*;

pub use hydroxy_charts::bundle::{Link, Tree};

/// Links out of and into leaf `i`, as names.
fn ends(b: &bundle::Bundle, i: usize) -> (Vec<String>, Vec<String>) {
    let out = b.edges.iter().filter(|e| e.source == i).map(|e| b.leaves[e.target].name.clone()).collect();
    let inn = b.edges.iter().filter(|e| e.target == i).map(|e| b.leaves[e.source].name.clone()).collect();
    (out, inn)
}

/// An edge-bundling diagram. Hover, touch or focus a leaf to see what it links to in one
/// colour and what links to it in another. Focus the chart and use the arrow keys to go
/// round the circle, Home and End to jump, Escape to clear.
#[component]
pub fn EdgeBundlingChart(
    /// The hierarchy. Its leaves go round the circle in order; its top-level branches are
    /// set apart by a gap. Leaf values are ignored.
    #[prop(into)]
    tree: Signal<Tree>,
    /// Directed links between leaves, by name. Values are ignored.
    #[prop(into)]
    links: Signal<Vec<Link>>,
    /// What the chart shows, such as "Module imports". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Bundling strength from 0 (straight lines) to 1 (links follow the tree exactly).
    #[prop(default = 0.85)]
    beta: f64,
    /// Drawing width and height in px; the chart scales down to fit its container.
    #[prop(default = 600)]
    size: u16,
    /// What a link means read forwards, for the legend and tooltip.
    #[prop(default = "Links to".into(), into)]
    out_label: String,
    /// What a link means read backwards, for the legend and tooltip.
    #[prop(default = "Linked from".into(), into)]
    in_label: String,
    /// Called when the pointer or keyboard moves to a leaf, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a leaf.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let layout = Memo::new(move |_| tree.with(|t| links.with(|l| bundle::layout(t, l, f64::from(size), beta))));
    let active = RwSignal::new(None::<usize>);
    let datum = Signal::derive(move || {
        let i = active.get()?;
        layout.with(|b| {
            let leaf = b.leaves.get(i)?;
            let (out, inn) = ends(b, i);
            #[allow(clippy::cast_precision_loss, reason = "link counts are small")]
            let links = (out.len() + inn.len()) as f64;
            Some(EventDatum { series: leaf.group, index: i, label: Some(leaf.name.clone()), value: links, node: None })
        })
    });
    crate::callbacks::watch_hover(Some(datum), on_hover);
    let roles = Memo::new(move |_| active.get().map(|i| layout.with(|b| bundle::roles(b, i))));
    let role_of = move |j: usize| roles.with(|r| r.as_ref().and_then(|r| r.get(j).copied().flatten()).map(Role::attr));
    let pick = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };
    let (out_name, in_name) = (StoredValue::new(out_label), StoredValue::new(in_label));
    let describe = move |i: usize| {
        layout.with(|b| {
            let leaf = b.leaves.get(i)?;
            let (out, inn) = ends(b, i);
            Some(format!(
                "{}, {}: {} {}, {} {}",
                leaf.name,
                b.groups[leaf.group],
                out_name.get_value().to_lowercase(),
                out.len(),
                in_name.get_value().to_lowercase(),
                inn.len()
            ))
        })
    };
    let edge_role = move |s: usize, t: usize| match active.get() {
        Some(i) if i == s => Some("out"),
        Some(i) if i == t => Some("in"),
        _ => None,
    };

    let group_label = format!("{label}. Use the arrow keys to go round the circle.");
    let caption = label;
    view! {
        <figure data-scope="edge-bundling-chart" data-part="root" data-active=move || active.get().map(|_| "") style=format!("max-width: {size}px;")>
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=group_label
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:click=move |_| crate::callbacks::select_active(Some(datum), on_select)
                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                        crate::callbacks::select_active(Some(datum), on_select);
                    }
                    if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), layout.with(|b| b.leaves.len())) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg viewBox=format!("0 0 {size} {size}") aria-hidden="true">
                    {move || {
                        let b = layout.get();
                        let edges = b
                            .edges
                            .iter()
                            .enumerate()
                            .map(|(k, e)| {
                                let (s, t) = (e.source, e.target);
                                view! {
                                    <path
                                        data-part="link"
                                        pathLength="1"
                                        data-role=move || edge_role(s, t)
                                        d=e.path.clone()
                                        style=format!("--i: {k}; --ox-n: {};", b.edges.len())
                                    ></path>
                                }
                            })
                            .collect_view();
                        let leaves = b
                            .leaves
                            .iter()
                            .enumerate()
                            .map(|(i, leaf)| {
                                view! {
                                    <g
                                        data-part="leaf"
                                        data-group=leaf.group
                                        data-role=move || role_of(i)
                                        data-selected=move || (active.get() == Some(i)).then_some("")
                                        on:pointerenter=move |_| pick(i)
                                        on:pointerdown=move |_| pick(i)
                                    >
                                        <circle data-part="hit" cx=leaf.x cy=leaf.y r="8"></circle>
                                        <circle data-part="dot" cx=leaf.x cy=leaf.y r="2.5"></circle>
                                        <text data-part="leaf-label" transform=leaf.transform.clone() text-anchor=leaf.anchor dominant-baseline="central">
                                            {leaf.name.clone()}
                                        </text>
                                    </g>
                                }
                            })
                            .collect_view();
                        view! {
                            <g data-part="links">{edges}</g>
                            <g data-part="leaves">{leaves}</g>
                        }
                    }}
                </svg>
                {move || {
                    let i = active.get()?;
                    let (leaf, group, (out, inn)) = layout.with(|b| Some((b.leaves.get(i)?.clone(), b.groups[b.leaves[i].group].clone(), ends(b, i))))?;
                    let (fx, fy) = (leaf.x / f64::from(size), leaf.y / f64::from(size));
                    let row = |name: String, items: Vec<String>, role: &'static str| {
                        view! {
                            <li data-part="tooltip-row" data-role=role>
                                <span data-part="tooltip-name">
                                    <span data-part="swatch"></span>
                                    {name}
                                </span>
                                <span data-part="tooltip-value">{items.len()}</span>
                            </li>
                        }
                    };
                    Some(
                        view! {
                            <div
                                data-part="tooltip"
                                data-anchored=""
                                data-side={if fx > 0.5 { "left" } else { "right" }}
                                style=format!("left: {:.3}%; top: {:.3}%;", fx * 100.0, fy * 100.0)
                                aria-hidden="true"
                            >
                                <div data-part="tooltip-label">{leaf.name}</div>
                                <ul>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Group"</span>
                                        <span data-part="tooltip-value">{group}</span>
                                    </li>
                                    {row(out_name.get_value(), out, "out")}
                                    {row(in_name.get_value(), inn, "in")}
                                </ul>
                            </div>
                        },
                    )
                }}
            </div>
            <ul data-part="legend" aria-hidden="true">
                <li data-part="legend-item" data-role="out">
                    <span data-part="swatch"></span>
                    {out_name.get_value()}
                </li>
                <li data-part="legend-item" data-role="in">
                    <span data-part="swatch"></span>
                    {in_name.get_value()}
                </li>
            </ul>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Leaf"</th>
                        <th scope="col">"Group"</th>
                        <th scope="col">{out_name.get_value()}</th>
                        <th scope="col">{in_name.get_value()}</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let b = layout.get();
                        b.leaves
                            .iter()
                            .enumerate()
                            .map(|(i, leaf)| {
                                let (out, inn) = ends(&b, i);
                                view! {
                                    <tr>
                                        <th scope="row">{leaf.name.clone()}</th>
                                        <td>{b.groups[leaf.group].clone()}</td>
                                        <td>{out.join(", ")}</td>
                                        <td>{inn.join(", ")}</td>
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
