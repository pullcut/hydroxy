//! `<SunburstChart>`: a tree as concentric rings, each arc's sweep its share of the
//! whole. Click an arc to zoom into it, the centre to climb out. Layout comes from
//! `oxide_charts::sunburst`.

use leptos::either::Either;
use leptos::prelude::*;
use oxide_charts::{format_value, sunburst};

pub use oxide_charts::hierarchy::Tree;

/// A sunburst. Arcs with children can be clicked, or focused and activated with Enter
/// or Space, to zoom in. The centre shows the hovered arc, or the current level, and
/// zooms back out when activated.
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
    /// Formats values for the centre and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let s = f64::from(size);
    // The centre disc matches the layout's hole: 26% of the outer radius.
    let hole_pct = 2.0 * (s / 2.0 - 2.0) * 0.26 / s * 100.0;
    let zoom = RwSignal::new(Vec::<usize>::new());
    let hovered = RwSignal::new(None::<usize>);
    let layout = Memo::new(move |_| root.with(|r| zoom.with(|z| sunburst::layout(r, z, s, usize::from(depth.clamp(1, 4))))));
    let crumbs = Memo::new(move |_| root.with(|r| zoom.with(|z| r.crumbs(z))));
    let plot = NodeRef::<leptos::html::Div>::new();
    let go = move |path: Vec<usize>| {
        hovered.set(None);
        zoom.set(path);
        if let Some(el) = plot.get_untracked() {
            let _ = el.focus();
        }
    };
    let out = move || {
        let c = crumbs.get();
        if c.len() > 1 {
            go(c[c.len() - 2].1.clone());
        }
    };
    // The centre reads the hovered or focused arc, else the zoomed node.
    let readout = move || {
        let l = layout.get();
        let (name, value) = if let Some(a) = hovered.get().and_then(|i| l.arcs.get(i)) {
            (a.name.clone(), a.value)
        } else {
            let c = crumbs.get();
            (c[c.len() - 1].0.clone(), root.with(|r| zoom.with(|z| r.node_at(&r.valid_path(z)).map_or(0.0, Tree::total))))
        };
        (name, fmt(value))
    };

    let caption = label.clone();
    view! {
        <figure data-scope="sunburst-chart" data-part="root" aria-label=label style=format!("max-width: {size}px;")>
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
            <div data-part="plot" node_ref=plot tabindex="-1">
                <svg viewBox=format!("0 0 {size} {size}") role="group" aria-label=caption.clone()>
                    {move || {
                        let l = layout.get();
                        l.arcs
                            .into_iter()
                            .enumerate()
                            .map(|(i, a)| {
                                let path = a.path.clone();
                                let zoomable = a.zoomable;
                                let zoom_path = path.clone();
                                view! {
                                    <path
                                        data-part="arc"
                                        data-depth=a.depth
                                        data-leaf=(!a.zoomable).then_some("")
                                        d=a.path_d
                                        fill=format!("var(--chart-{})", a.colour % 6 + 1)
                                        tabindex="0"
                                        role="button"
                                        aria-disabled=(!a.zoomable).then_some("true")
                                        aria-label=format!(
                                            "{}: {}, {:.0}%{}",
                                            a.name,
                                            fmt(a.value),
                                            a.share * 100.0,
                                            if a.zoomable { ", press Enter to zoom in" } else { "" },
                                        )
                                        on:pointerenter=move |_| hovered.set(Some(i))
                                        on:pointerleave=move |_| hovered.set(None)
                                        on:focus=move |_| hovered.set(Some(i))
                                        on:blur=move |_| hovered.set(None)
                                        on:click=move |_| {
                                            if zoomable {
                                                go(path.clone());
                                            }
                                        }
                                        on:keydown=move |ev| {
                                            if zoomable && (ev.key() == "Enter" || ev.key() == " ") {
                                                ev.prevent_default();
                                                go(zoom_path.clone());
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
                    disabled=move || crumbs.with(|c| c.len() < 2)
                    aria-label=move || crumbs.with(|c| if c.len() < 2 { "Top level".to_string() } else { format!("Zoom out from {}", c[c.len() - 1].0) })
                    style=format!("width: {hole_pct:.3}%;")
                    on:click=move |_| out()
                >
                    <span data-part="center-name">{move || readout().0}</span>
                    <span data-part="center-value">{move || readout().1}</span>
                </button>
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
