//! `<TreemapChart>`: a tree as nested rectangles sized by value, with click-to-zoom and
//! a breadcrumb. Layout comes from `oxide_charts::treemap`.

use leptos::either::Either;
use leptos::prelude::*;
use oxide_charts::{format_value, treemap};

pub use oxide_charts::hierarchy::Tree;

fn pct(v: f64, of: f64) -> String {
    format!("{:.3}%", v / of * 100.0)
}

/// A treemap. Cells are sized by value and the children of big cells are drawn inside
/// them. Click, or press Enter on, a cell with children to zoom into it; use the
/// breadcrumb to climb back out.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn TreemapChart(
    /// The tree. Branch values come from their children.
    #[prop(into)]
    root: Signal<Tree>,
    /// What the chart shows, such as "Spend by team". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px at the drawing's native width of 800.
    #[prop(default = 380)]
    height: u16,
    /// Formats values for cells and the table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (w, h) = (800.0, f64::from(height));
    let zoom = RwSignal::new(Vec::<usize>::new());
    let plot = NodeRef::<leptos::html::Div>::new();
    let cells = Memo::new(move |_| root.with(|r| zoom.with(|z| treemap::layout(r, z, w, h))));
    let crumbs = Memo::new(move |_| root.with(|r| zoom.with(|z| r.crumbs(z))));
    let go = move |path: Vec<usize>| {
        zoom.set(path);
        // The cells are rebuilt, so park focus on the plot rather than losing it.
        if let Some(el) = plot.get_untracked() {
            let _ = el.focus();
        }
    };

    let caption = label.clone();
    view! {
        <figure data-scope="treemap-chart" data-part="root" aria-label=label>
            <nav data-part="breadcrumb" aria-label="Zoom level">
                <ol>
                    {move || {
                        let c = crumbs.get();
                        let last = c.len() - 1;
                        c.into_iter()
                            .enumerate()
                            .map(|(i, (name, path))| {
                                if i == last {
                                    Either::Left(view! { <li><span data-part="crumb" aria-current="location">{name}</span></li> })
                                } else {
                                    Either::Right(view! {
                                        <li>
                                            <button type="button" data-part="crumb" on:click=move |_| go(path.clone())>{name}</button>
                                        </li>
                                    })
                                }
                            })
                            .collect_view()
                    }}
                </ol>
            </nav>
            <div data-part="plot" node_ref=plot tabindex="-1" style=format!("aspect-ratio: {} / {};", 800, height)>
                {move || {
                    cells
                        .get()
                        .into_iter()
                        .map(|c| {
                            let path = c.path.clone();
                            let zoomable = c.zoomable;
                            let children = if c.zoomable { ", has parts, press Enter to zoom in" } else { "" };
                            view! {
                                <button
                                    type="button"
                                    data-part="cell"
                                    data-depth=c.depth
                                    data-nested=c.nested.then_some("")
                                    data-leaf=(!c.zoomable).then_some("")
                                    aria-disabled=(!c.zoomable).then_some("true")
                                    aria-label=format!("{}: {}{children}", c.name, fmt(c.value))
                                    style=format!(
                                        "left: {}; top: {}; width: {}; height: {}; --c: var(--chart-{});",
                                        pct(c.x, w),
                                        pct(c.y, h),
                                        pct(c.w, w),
                                        pct(c.h, h),
                                        c.colour % 6 + 1,
                                    )
                                    on:click=move |_| {
                                        if zoomable {
                                            go(path.clone());
                                        }
                                    }
                                >
                                    {c.label
                                        .then(|| {
                                            view! {
                                                <span data-part="cell-name">{c.name.clone()}</span>
                                                <span data-part="cell-value">{fmt(c.value)}</span>
                                            }
                                        })}
                                </button>
                            }
                        })
                        .collect_view()
                }}
            </div>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Item"</th>
                        <th scope="col">"Value"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        root.with(Tree::flatten)
                            .into_iter()
                            .map(|(name, v)| view! { <tr><th scope="row">{name}</th><td>{fmt(v)}</td></tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
