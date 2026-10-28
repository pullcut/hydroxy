//! `<TreeChart>`: a tidy node-link tree, top-down or left-to-right, with collapsible
//! branches. Layout comes from `hydroxy_charts::tidy`.

use leptos::prelude::*;
use hydroxy_charts::tidy::{self, TreeKey};
use hydroxy_charts::{format_value, partition};

pub use hydroxy_charts::Orientation;
pub use hydroxy_charts::hierarchy::Tree;

fn colour(slot: Option<usize>) -> String {
    slot.map_or_else(|| "var(--chart-muted)".to_string(), |s| format!("var(--chart-{})", s + 1))
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

const WIDTH: f64 = 800.0;

/// A tidy tree. Click a node with children to collapse or expand it. Focus the chart
/// and use the arrow keys as in a tree view: Up and Down step through visible nodes,
/// Right expands or enters a branch, Left collapses or climbs to the parent, Home and
/// End jump, Enter or Space toggles, Escape clears.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn TreeChart(
    /// The tree. Branch values come from their children.
    #[prop(into)]
    root: Signal<Tree>,
    /// What the chart shows, such as "Reporting lines". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Vertical (default) puts the root at the top; Horizontal puts it on the left.
    #[prop(optional)]
    orientation: Orientation,
    /// Plot height in px at an 800px drawing width.
    #[prop(default = 360)]
    height: u16,
    /// Formats values for the tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let plot_h = f64::from(height);
    let collapsed = RwSignal::new(Vec::<Vec<usize>>::new());
    let active = RwSignal::new(None::<usize>);
    let nodes = Memo::new(move |_| root.with(|r| collapsed.with(|c| tidy::layout(r, c))));
    let toggle = move |i: usize| {
        let Some(path) = nodes.with_untracked(|n| n.get(i).filter(|n| n.has_children).map(|n| n.path.clone())) else { return };
        collapsed.update(|c| {
            if let Some(k) = c.iter().position(|p| *p == path) {
                c.remove(k);
            } else {
                c.push(path);
            }
        });
    };
    let child_count = move |path: &[usize]| root.with(|r| r.node_at(path).map_or(0, |n| n.children.len()));
    let describe = move |i: usize| {
        let n = nodes.with(|n| n.get(i).cloned())?;
        let state = match (n.has_children, n.collapsed) {
            (false, _) => String::new(),
            (true, true) => format!(", collapsed, {} children hidden", child_count(&n.path)),
            (true, false) => format!(", expanded, {} children", child_count(&n.path)),
        };
        Some(format!("{}: {}, level {}{state}", n.name, fmt(n.value), n.depth + 1))
    };
    let legend = Memo::new(move |_| root.with(partition::legend));
    let caption = label.clone();

    view! {
        <figure data-scope="tree-chart" data-part="root" data-orientation=orientation.attr()>
            <div data-part="plot">
                <svg
                    data-part="marks"
                    viewBox=format!("0 0 {WIDTH} {height}")
                    tabindex="0"
                    role="group"
                    aria-label=format!("{label}. Use the arrow keys to move through the tree, Enter to collapse or expand a branch.")
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        let Some(action) = nodes.with_untracked(|n| tidy::key(&ev.key(), active.get_untracked(), n)) else { return };
                        ev.prevent_default();
                        match action {
                            TreeKey::Focus(next) => active.set(next),
                            TreeKey::Toggle(i) => toggle(i),
                        }
                    }
                >
                    {move || {
                        let n = nodes.get();
                        n.iter()
                            .filter_map(|c| {
                                let p = &n[c.parent?];
                                let d = tidy::link(p.point(orientation, WIDTH, plot_h), c.point(orientation, WIDTH, plot_h), orientation);
                                Some(view! { <path data-part="link" pathLength="1" d=d style=format!("--i: {};", c.depth.min(20))></path> })
                            })
                            .collect_view()
                    }}
                    {move || {
                        nodes
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, n)| {
                                let (x, y) = n.point(orientation, WIDTH, plot_h);
                                let (lx, ly, anchor) = match orientation {
                                    Orientation::Vertical => (x, y + 18.0, "middle"),
                                    // Parents label above the node, clear of their outgoing links; leaves label to the right.
                                    Orientation::Horizontal if n.has_children => (x, y - 14.0, "middle"),
                                    Orientation::Horizontal => (x + 10.0, y, "start"),
                                };
                                view! {
                                    <g
                                        data-part="node"
                                        data-collapsed=n.collapsed.then_some("")
                                        data-leaf=(!n.has_children).then_some("")
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        style=format!("--i: {};", n.depth.min(20))
                                        on:pointerenter=move |_| active.set(Some(i))
                                        on:pointerdown=move |_| active.set(Some(i))
                                        on:click=move |_| toggle(i)
                                    >
                                        <circle cx=format!("{x:.2}") cy=format!("{y:.2}") r=if n.collapsed { "6.5" } else { "5" } style=format!("--c: {};", colour(n.colour))></circle>
                                        <text data-part="label" x=format!("{lx:.2}") y=format!("{ly:.2}") text-anchor=anchor dominant-baseline="central">{n.name}</text>
                                    </g>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                {move || {
                    let n = nodes.with(|n| n.get(active.get()?).cloned())?;
                    let (x, y) = n.point(orientation, WIDTH, plot_h);
                    let (fx, fy) = (x / WIDTH, y / plot_h);
                    let kids = n.has_children.then(|| {
                        let state = if n.collapsed { "hidden" } else { "shown" };
                        view! {
                            <li data-part="tooltip-row">
                                <span data-part="tooltip-name">"Children"</span>
                                <span data-part="tooltip-value">{format!("{} {state}", child_count(&n.path))}</span>
                            </li>
                        }
                    });
                    Some(view! {
                        <div
                            data-part="tooltip"
                            data-side={if fx > 0.5 { "left" } else { "right" }}
                            data-vertical={if fy < 0.25 { "below" } else { "above" }}
                            style=format!("left: {}; top: {};", pct(fx), pct(fy))
                            aria-hidden="true"
                        >
                            <div data-part="tooltip-label">
                                <span data-part="swatch" style=format!("background: {};", colour(n.colour))></span>
                                {n.name}
                            </div>
                            <ul>
                                <li data-part="tooltip-row"><span data-part="tooltip-name">"Value"</span><span data-part="tooltip-value">{fmt(n.value)}</span></li>
                                {kids}
                            </ul>
                        </div>
                    })
                }}
            </div>
            <Show when=move || legend.with(|l| l.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        legend
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, name)| view! { <li data-part="legend-item"><span data-part="swatch" style=format!("background: {};", colour(Some(i)))></span>{name}</li> })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr><th scope="col">"Node"</th><th scope="col">"Value"</th></tr>
                </thead>
                <tbody>
                    {move || {
                        root.with(|r| partition::rows(r, false))
                            .into_iter()
                            .map(|(name, v, _)| view! { <tr><th scope="row">{name}</th><td>{fmt(v)}</td></tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
