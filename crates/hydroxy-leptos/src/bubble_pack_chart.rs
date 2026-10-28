//! `<BubblePackChart>`: a tree as nested circles, each leaf's area its value. Click a
//! circle with children to zoom into it. Layout comes from `hydroxy_charts::pack`.

use hydroxy_charts::{format_value, nav, pack, partition};
use leptos::either::Either;
use leptos::prelude::*;

pub use hydroxy_charts::hierarchy::Tree;

fn colour(slot: Option<usize>) -> String {
    slot.map_or_else(|| "var(--chart-muted)".to_string(), |s| format!("var(--chart-{})", s + 1))
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Circle packing. Hover, touch or focus the chart and use the arrow keys to read each
/// circle; click one with children, or press Enter on it, to zoom in. Escape, the
/// outer circle or the breadcrumb zoom back out.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn BubblePackChart(
    /// The tree. Branch values come from their children.
    #[prop(into)]
    root: Signal<Tree>,
    /// What the chart shows, such as "Disk usage by folder". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Diameter in px; the chart scales down to fit its container.
    #[prop(default = 480)]
    size: u16,
    /// Formats values for the tooltip, labels and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let px = f64::from(size);
    let zoom = RwSignal::new(Vec::<usize>::new());
    let active = RwSignal::new(None::<usize>);
    let bubbles = Memo::new(move |_| root.with(|r| zoom.with(|z| pack::layout(r, z, px, 3.0))));
    let crumbs = Memo::new(move |_| root.with(|r| zoom.with(|z| r.crumbs(z))));
    let marks = NodeRef::<leptos::svg::Svg>::new();
    let go = move |path: Vec<usize>| {
        active.set(None);
        zoom.set(path);
        if let Some(el) = marks.get_untracked() {
            let _ = el.focus();
        }
    };
    let up = move || {
        let c = crumbs.get_untracked();
        if c.len() > 1 {
            go(c[c.len() - 2].1.clone());
        }
    };
    // Activating a circle zooms into it; activating the outer circle climbs out.
    let activate = move |i: usize| {
        let Some(b) = bubbles.with_untracked(|l| l.get(i).cloned()) else { return };
        if b.depth == 0 {
            up();
        } else if b.zoomable {
            go(b.path);
        }
    };
    let describe = move |i: usize| {
        bubbles.with(|l| {
            let b = l.get(i)?;
            let whole = l[0].value;
            let share = if whole > 0.0 { b.value / whole * 100.0 } else { 0.0 };
            let hint = if b.depth == 0 && l[0].path.is_empty() {
                ""
            } else if b.depth == 0 {
                ", press Enter to zoom out"
            } else if b.zoomable {
                ", press Enter to zoom in"
            } else {
                ""
            };
            Some(format!("{}: {}, {share:.0}% of {}{hint}", b.name, fmt(b.value), l[0].name))
        })
    };
    let legend = Memo::new(move |_| {
        root.with(|r| zoom.with(|z| r.node_at(&r.valid_path(z)).map(partition::legend).unwrap_or_default()))
    });
    let caption = label.clone();

    view! {
        <figure data-scope="bubble-pack-chart" data-part="root" style=format!("max-width: {size}px;")>
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
                                        <li><button type="button" data-part="crumb" on:click=move |_| go(path.clone())>{name}</button></li>
                                    })
                                }
                            })
                            .collect_view()
                    }}
                </ol>
            </nav>
            <div data-part="plot">
                <svg
                    data-part="marks"
                    node_ref=marks
                    viewBox=format!("0 0 {size} {size}")
                    tabindex="0"
                    role="group"
                    aria-label=format!("{label}. Use the arrow keys to move between circles, Enter to zoom in, Escape to zoom out.")
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        let key = ev.key();
                        let current = active.get_untracked();
                        if key == "Enter" || key == " " {
                            if let Some(i) = current {
                                ev.prevent_default();
                                activate(i);
                            }
                            return;
                        }
                        if key == "Escape" && current.is_none() && zoom.with_untracked(|z| !z.is_empty()) {
                            ev.prevent_default();
                            up();
                            return;
                        }
                        let key = match key.as_str() {
                            "ArrowDown" => "ArrowRight",
                            "ArrowUp" => "ArrowLeft",
                            k => k,
                        }
                        .to_string();
                        if let Some(next) = nav::list_key(&key, current, bubbles.with_untracked(Vec::len)) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        bubbles
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, b)| {
                                view! {
                                    <circle
                                        data-part="bubble"
                                        data-depth=b.depth
                                        data-leaf=(!b.zoomable).then_some("")
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        cx=format!("{:.2}", b.x)
                                        cy=format!("{:.2}", b.y)
                                        r=format!("{:.2}", b.r)
                                        fill=colour(b.colour)
                                        style=format!("--i: {};", i.min(40))
                                        on:pointerenter=move |_| active.set(Some(i))
                                        on:pointerdown=move |_| active.set(Some(i))
                                        on:click=move |_| activate(i)
                                    ></circle>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        bubbles
                            .get()
                            .into_iter()
                            .filter(|b| b.depth > 0 && b.r > 22.0 && (!b.zoomable || b.depth == 1))
                            .filter_map(|b| {
                                let text = partition::fit_label(&b.name, b.r * 1.8, 6.5)?;
                                Some(view! {
                                    <text data-part="label" data-leaf=(!b.zoomable).then_some("") x=format!("{:.2}", b.x) y=format!("{:.2}", if b.zoomable { b.y - b.r + 14.0 } else { b.y }) text-anchor="middle" dominant-baseline="central">
                                        {text}
                                    </text>
                                })
                            })
                            .collect_view()
                    }}
                </svg>
                {move || {
                    let i = active.get()?;
                    let (b, whole) = bubbles.with(|l| l.get(i).cloned().map(|b| (b, l[0].value)))?;
                    let share = if whole > 0.0 { b.value / whole * 100.0 } else { 0.0 };
                    let (x, y) = (b.x / px, (b.y - b.r).max(0.0) / px);
                    Some(view! {
                        <div
                            data-part="tooltip"
                            data-side={if x > 0.5 { "left" } else { "right" }}
                            data-vertical={if y < 0.25 { "below" } else { "above" }}
                            style=format!("left: {}; top: {};", pct(x), pct(y))
                            aria-hidden="true"
                        >
                            <div data-part="tooltip-label">
                                <span data-part="swatch" style=format!("background: {};", colour(b.colour))></span>
                                {b.name}
                            </div>
                            <ul>
                                <li data-part="tooltip-row"><span data-part="tooltip-name">"Value"</span><span data-part="tooltip-value">{fmt(b.value)}</span></li>
                                <li data-part="tooltip-row"><span data-part="tooltip-name">"Share"</span><span data-part="tooltip-value">{format!("{share:.1}%")}</span></li>
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
