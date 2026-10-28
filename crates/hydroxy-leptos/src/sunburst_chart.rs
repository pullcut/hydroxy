//! `<SunburstChart>`: a tree as concentric rings, each arc's sweep its share of the
//! whole. Click an arc to zoom into it, the centre to climb out. Layout comes from
//! `hydroxy_charts::sunburst`.

use hydroxy_charts::hierarchy::{Drill, drill_key};
use hydroxy_charts::{format_value, sunburst};
use leptos::either::Either;
use leptos::prelude::*;

pub use hydroxy_charts::hierarchy::Tree;

/// A sunburst. Hover or touch an arc to read it; click one with parts to zoom in, and
/// the centre to zoom back out. Focus the chart and use the arrow keys to step through
/// arcs, Enter to zoom in, Backspace to zoom out, Escape to clear. The centre shows the
/// active arc, or the current level.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SunburstChart(
    /// The tree. Branch values come from their children.
    #[prop(into)]
    root: Signal<Tree>,
    /// What the chart shows, such as "Spend by team". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Diameter in px; the chart scales down to fit its container.
    #[prop(default = 420)]
    size: u16,
    /// How many rings to draw, 1 to 4.
    #[prop(default = 3)]
    depth: u8,
    /// Formats values for the centre, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let s = f64::from(size);
    // The centre disc matches the layout's hole: 26% of the outer radius.
    let hole_pct = 2.0 * (s / 2.0 - 2.0) * 0.26 / s * 100.0;
    let zoom = RwSignal::new(Vec::<usize>::new());
    let active = RwSignal::new(None::<usize>);
    let notice = RwSignal::new(String::new());
    let layout =
        Memo::new(move |_| root.with(|r| zoom.with(|z| sunburst::layout(r, z, s, usize::from(depth.clamp(1, 4))))));
    let crumbs = Memo::new(move |_| root.with(|r| zoom.with(|z| r.crumbs(z))));
    let plot = NodeRef::<leptos::html::Div>::new();
    let go = move |path: Vec<usize>| {
        let from = zoom.get_untracked();
        zoom.set(path);
        // Climbing out picks the node we left; zooming in starts fresh.
        active.set(layout.with_untracked(|l| l.arcs.iter().position(|a| a.path == from)));
        let name = crumbs.with_untracked(|c| c[c.len() - 1].0.clone());
        let parts = layout.with_untracked(|l| l.arcs.iter().filter(|a| a.depth == 1).count());
        notice.set(format!("Showing {name}, {parts} parts"));
        if let Some(el) = plot.get_untracked() {
            let _ = el.focus();
        }
    };
    let out = move || {
        let c = crumbs.get_untracked();
        if c.len() > 1 {
            go(c[c.len() - 2].1.clone());
        }
    };
    let pick = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };
    let describe = move |i: usize| {
        layout.with(|l| {
            l.arcs.get(i).map(|a| {
                let hint = if a.zoomable { ", press Enter to zoom in" } else { "" };
                format!("{}: {}, {:.0}%{hint}", a.name, fmt(a.value), a.share * 100.0)
            })
        })
    };
    // The centre reads the active arc, else the zoomed node.
    let readout = move || {
        let l = layout.get();
        let (name, value) = if let Some(a) = active.get().and_then(|i| l.arcs.get(i)) {
            (a.name.clone(), a.value)
        } else {
            let c = crumbs.get();
            (
                c[c.len() - 1].0.clone(),
                root.with(|r| zoom.with(|z| r.node_at(&r.valid_path(z)).map_or(0.0, Tree::total))),
            )
        };
        (name, fmt(value))
    };
    let top_arcs = move || {
        layout.with(|l| {
            l.arcs
                .iter()
                .enumerate()
                .filter(|(_, a)| a.depth == 1)
                .map(|(i, a)| (i, a.name.clone(), a.colour))
                .collect::<Vec<_>>()
        })
    };

    let group_label =
        format!("{label}. Use the arrow keys to step through arcs, Enter to zoom in, Backspace to zoom out.");
    let caption = label.clone();
    view! {
        <figure
            data-scope="sunburst-chart"
            data-part="root"
            aria-label=label
            data-active=move || active.get().map(|_| "")
            style=format!("max-width: {size}px;")
        >
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
            <div
                data-part="plot"
                node_ref=plot
                tabindex="0"
                role="group"
                aria-label=group_label
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    let zoomable: Vec<bool> = layout.with_untracked(|l| l.arcs.iter().map(|a| a.zoomable).collect());
                    let at_top = crumbs.with_untracked(|c| c.len() < 2);
                    match drill_key(&ev.key(), active.get_untracked(), &zoomable, at_top) {
                        Some(Drill::Move(next)) => active.set(next),
                        Some(Drill::Into(i)) => go(layout.with_untracked(|l| l.arcs[i].path.clone())),
                        Some(Drill::Up) => out(),
                        None => return,
                    }
                    ev.prevent_default();
                }
            >
                <svg viewBox=format!("0 0 {size} {size}") aria-hidden="true">
                    {move || {
                        let l = layout.get();
                        l.arcs
                            .into_iter()
                            .enumerate()
                            .map(|(i, a)| {
                                let path = a.path.clone();
                                let zoomable = a.zoomable;
                                view! {
                                    <path
                                        data-part="arc"
                                        data-depth=a.depth
                                        data-leaf=(!a.zoomable).then_some("")
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        d=a.path_d
                                        fill=format!("var(--chart-{})", a.colour % 6 + 1)
                                        on:pointerenter=move |_| pick(i)
                                        on:pointerdown=move |_| pick(i)
                                        on:click=move |_| {
                                            if zoomable {
                                                go(path.clone());
                                            }
                                        }
                                    ></path>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                <button
                    type="button"
                    data-part="center"
                    tabindex="-1"
                    disabled=move || crumbs.with(|c| c.len() < 2)
                    aria-label=move || crumbs.with(|c| if c.len() < 2 { "Top level".to_string() } else { format!("Zoom out from {}", c[c.len() - 1].0) })
                    style=format!("width: {hole_pct:.3}%;")
                    on:click=move |_| out()
                >
                    <span data-part="center-name">{move || readout().0}</span>
                    <span data-part="center-value">{move || readout().1}</span>
                </button>
                {move || {
                    let i = active.get()?;
                    let l = layout.get();
                    let arc = l.arcs.get(i)?;
                    let (ax, ay) = l.anchor(i)?;
                    let level = crumbs.with(|c| c[c.len() - 1].0.clone());
                    Some(
                        view! {
                            <div
                                data-part="tooltip"
                                data-anchored=""
                                data-side={if ax > 0.5 { "right" } else { "left" }}
                                style=format!("left: {:.3}%; top: {:.3}%;", ax * 100.0, ay * 100.0)
                                aria-hidden="true"
                            >
                                <div data-part="tooltip-label">
                                    <span data-part="swatch" style=format!("background: var(--chart-{});", arc.colour % 6 + 1)></span>
                                    {arc.name.clone()}
                                </div>
                                <ul>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">"Value"</span>
                                        <span data-part="tooltip-value">{fmt(arc.value)}</span>
                                    </li>
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">{format!("Share of {level}")}</span>
                                        <span data-part="tooltip-value">{format!("{:.0}%", arc.share * 100.0)}</span>
                                    </li>
                                </ul>
                            </div>
                        },
                    )
                }}
            </div>
            <Show when=move || layout.with(|l| l.arcs.iter().filter(|a| a.depth == 1).count() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        top_arcs()
                            .into_iter()
                            .map(|(i, name, colour)| {
                                view! {
                                    <li
                                        data-part="legend-item"
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        on:pointerenter=move |_| pick(i)
                                        on:pointerleave=move |_| active.set(None)
                                    >
                                        <span data-part="swatch" style=format!("background: var(--chart-{});", colour % 6 + 1)></span>
                                        {name}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
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
