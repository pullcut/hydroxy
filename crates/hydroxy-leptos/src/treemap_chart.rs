//! `<TreemapChart>`: a tree as nested rectangles sized by value, with click-to-zoom and
//! a breadcrumb. Layout comes from `hydroxy_charts::treemap`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, treemap};
use leptos::prelude::*;

use crate::hierarchy_kit::{Breadcrumb, drill};
use crate::plot_kit::{ChartTip, colour, pct};

pub use hydroxy_charts::hierarchy::Tree;

/// A treemap. Cells are sized by value and the children of big cells are drawn inside
/// them. Hover or touch a cell to read it; click one with parts to zoom into it and use
/// the breadcrumb to climb back out. Focus the plot and use the arrow keys to step
/// through cells, Enter to zoom in, Backspace to zoom out, Escape to clear.
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
    /// Formats values for cells, the tooltip and the table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the hovered or focused cell changes; `None` when it clears.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a cell.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (w, h) = (800.0, f64::from(height));
    let zoom = RwSignal::new(Vec::<usize>::new());
    let active = RwSignal::new(None::<usize>);
    let notice = RwSignal::new(String::new());
    let plot = NodeRef::<leptos::html::Div>::new();
    let cells = Memo::new(move |_| root.with(|r| zoom.with(|z| treemap::layout(r, z, w, h))));
    let crumbs = Memo::new(move |_| root.with(|r| zoom.with(|z| r.crumbs(z))));
    let level_total =
        Memo::new(move |_| root.with(|r| zoom.with(|z| r.node_at(&r.valid_path(z)).map_or(0.0, Tree::total))));
    let share = move |v: f64| {
        let t = level_total.get();
        if t > 0.0 { v / t } else { 0.0 }
    };
    let go = move |path: Vec<usize>| {
        let from = zoom.get_untracked();
        zoom.set(path);
        // Climbing out picks the node we left; zooming in starts fresh.
        let back = cells.with_untracked(|c| c.iter().position(|x| x.path == from));
        active.set(back);
        let (name, parts) = crumbs.with_untracked(|c| {
            (c[c.len() - 1].0.clone(), cells.with_untracked(|c| c.iter().filter(|x| x.depth == 1).count()))
        });
        notice.set(format!("Showing {name}, {parts} parts"));
        // The cells are rebuilt, so park focus on the plot rather than losing it.
        if let Some(el) = plot.get_untracked() {
            let _ = el.focus();
        }
    };
    let describe = move |i: usize| {
        cells.with(|c| {
            c.get(i).map(|x| {
                let hint = if x.zoomable { ", press Enter to zoom in" } else { "" };
                format!("{}: {}, {:.0}%{hint}", x.name, fmt(x.value), share(x.value) * 100.0)
            })
        })
    };
    let pick = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };

    let datum = Signal::derive(move || {
        let i = active.get()?;
        root.with(|r| cells.with(|l| EventDatum::from_node(r, &l.get(i)?.path)))
    });
    crate::callbacks::watch_hover(Some(datum), on_hover);

    let group_label =
        format!("{label}. Use the arrow keys to step through cells, Enter to zoom in, Backspace to zoom out.");
    let caption = label.clone();
    view! {
        <figure data-scope="treemap-chart" data-part="root" aria-label=label data-active=move || active.get().map(|_| "")>
            <Breadcrumb crumbs=crumbs go=Callback::new(go) />
            <div
                data-part="plot"
                node_ref=plot
                tabindex="0"
                role="group"
                aria-label=group_label
                style=format!("aspect-ratio: {} / {};", 800, height)
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                        crate::callbacks::select_active(Some(datum), on_select);
                    }
                    let zoomable: Vec<bool> = cells.with_untracked(|c| c.iter().map(|x| x.zoomable).collect());
                    let at_top = crumbs.with_untracked(|c| c.len() < 2);
                    if !drill(
                        &ev.key(),
                        active,
                        &zoomable,
                        at_top,
                        |i| go(cells.with_untracked(|c| c[i].path.clone())),
                        || go(crumbs.with_untracked(|c| c[c.len() - 2].1.clone())),
                    ) {
                        return;
                    }
                    ev.prevent_default();
                }
            >
                {move || {
                    let cells = cells.get();
                    let n = cells.len();
                    cells
                        .into_iter()
                        .enumerate()
                        .map(|(i, c)| {
                            let path = c.path.clone();
                            let zoomable = c.zoomable;
                            view! {
                                <div
                                    data-part="cell"
                                    data-depth=c.depth
                                    data-nested=c.nested.then_some("")
                                    data-leaf=(!c.zoomable).then_some("")
                                    data-active=move || (active.get() == Some(i)).then_some("")
                                    aria-hidden="true"
                                    style=format!(
                                        "left: {}; top: {}; width: {}; height: {}; --c: {}; --i: {i}; --ox-n: {n};",
                                        pct(c.x / w),
                                        pct(c.y / h),
                                        pct(c.w / w),
                                        pct(c.h / h),
                                        colour(c.colour),
                                    )
                                    on:pointerenter=move |_| pick(i)
                                    on:pointerdown=move |_| pick(i)
                                    on:click=move |_| {
                                        crate::callbacks::select_active(Some(datum), on_select);
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
                                </div>
                            }
                        })
                        .collect_view()
                }}
                {move || {
                    let i = active.get()?;
                    let cell = cells.with(|c| c.get(i).cloned())?;
                    let (ax, ay) = ((cell.x + cell.w / 2.0) / w, (cell.y + cell.h / 2.0) / h);
                    let level = crumbs.with(|c| c[c.len() - 1].0.clone());
                    let tip = Tip {
                        x: ax,
                        y: ay,
                        title: cell.name.clone(),
                        rows: vec![
                            TipRow::new(None, "Value", fmt(cell.value)),
                            TipRow::new(None, format!("Share of {level}"), format!("{:.0}%", share(cell.value) * 100.0)),
                        ],
                    };
                    let side = if ax > 0.5 { "left" } else { "right" };
                    let style = format!("left: {:.3}%; top: {:.3}%;", ax * 100.0, ay * 100.0);
                    let swatch = colour(cell.colour);
                    Some(view! { <ChartTip tip swatch anchored=true style side /> })
                }}
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_else(|| notice.get())}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Item"</th>
                        <th scope="col">"Value"</th>
                        <th scope="col">"Share"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let total = root.with(Tree::total);
                        root.with(Tree::flatten)
                            .into_iter()
                            .map(|(name, v)| {
                                let s = if total > 0.0 { v / total * 100.0 } else { 0.0 };
                                view! { <tr><th scope="row">{name}</th><td>{fmt(v)}</td><td>{format!("{s:.1}%")}</td></tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
