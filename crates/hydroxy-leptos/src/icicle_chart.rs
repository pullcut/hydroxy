//! `<IcicleChart>`: a tree as rows of rectangles, one row per level, each cell as wide
//! as its share of the zoomed node. Click a cell to drill into it. Layout comes from
//! `hydroxy_charts::partition`, which the flame graph shares.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, partition};
use leptos::prelude::*;

use crate::hierarchy_kit::{Breadcrumb, PartitionLegend, ZoomKey, zoom_key};
use crate::plot_kit::{ChartTip, colour, colour_or_muted, pct};

pub use hydroxy_charts::hierarchy::Tree;

const WIDTH: f64 = 800.0;

/// A cell's colour slot as a CSS colour, muted when it has none.
fn slot_colour(slot: Option<usize>) -> String {
    slot.map_or_else(|| colour_or_muted(0, 0), colour) // n = 0: no slot is in range, so muted
}

/// An icicle chart. Hover, touch or focus the chart and use the arrow keys to read each
/// cell; click a cell with children, or press Enter on it, to drill in. Escape, the top
/// row or the breadcrumb climb back out.
#[component]
pub fn IcicleChart(
    /// The tree. Branch values come from their children.
    #[prop(into)]
    root: Signal<Tree>,
    /// What the chart shows, such as "Bundle size by module". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px at an 800px drawing width.
    #[prop(default = 320)]
    height: u16,
    /// Formats values for the tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a cell, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a cell.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    partition_chart(root, label, height, format, None, on_hover, on_select)
}

/// The icicle and flame graph body. `highlight` is `Some` for a flame graph: rows run
/// bottom-up, branches keep self time, cells colour by name and matches stand out.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[allow(clippy::many_single_char_names, reason = "x, y, w for geometry read best short")]
pub(crate) fn partition_chart(
    root: Signal<Tree>,
    label: String,
    height: u16,
    format: Option<Callback<f64, String>>,
    highlight: Option<Signal<String>>,
    on_hover: Option<Callback<Option<EventDatum>>>,
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let flame = highlight.is_some();
    let query = move || highlight.map(|h| h.get()).unwrap_or_default();
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let plot_h = f64::from(height);
    let zoom = RwSignal::new(Vec::<usize>::new());
    let active = RwSignal::new(None::<usize>);
    let layout = Memo::new(move |_| root.with(|r| zoom.with(|z| partition::layout(r, z, flame))));
    let datum = Signal::derive(move || {
        let i = active.get()?;
        root.with(|r| layout.with(|l| EventDatum::from_node(r, &l.cells.get(i)?.path)))
    });
    crate::callbacks::watch_hover(Some(datum), on_hover);
    let crumbs = Memo::new(move |_| root.with(|r| zoom.with(|z| r.crumbs(z))));
    let marks = NodeRef::<leptos::svg::Svg>::new();
    let fill = move |c: &partition::Cell| {
        if flame { slot_colour(Some(partition::name_slot(&c.name))) } else { slot_colour(c.colour) }
    };
    let rect = move |c: &partition::Cell, levels: usize| c.rect(levels, WIDTH, plot_h, flame);
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
    let activate = move |i: usize| {
        let Some(c) = layout.with_untracked(|l| l.cells.get(i).cloned()) else { return };
        if c.depth == 0 {
            up();
        } else if c.zoomable {
            go(c.path);
        }
    };
    let share =
        move |l: &partition::Partition, v: f64| if l.cells[0].value > 0.0 { v / l.cells[0].value * 100.0 } else { 0.0 };
    let describe = move |i: usize| {
        layout.with(|l| {
            let c = l.cells.get(i)?;
            let own = if flame { format!(", self {}", fmt(c.self_value)) } else { String::new() };
            let hint = match (c.depth, c.zoomable) {
                (0, _) if !c.path.is_empty() => ", press Enter to zoom out",
                (0, _) => "",
                (_, true) => ", press Enter to zoom in",
                _ => "",
            };
            Some(format!("{}: {}{own}, {:.0}% of {}{hint}", c.name, fmt(c.value), share(l, c.value), l.cells[0].name))
        })
    };
    let match_count = Memo::new(move |_| {
        let q = query();
        layout.with(|l| l.cells.iter().filter(|c| partition::matches(&c.name, &q)).count())
    });
    let announce = move || {
        active.get().and_then(describe).unwrap_or_else(|| {
            let q = query();
            if q.trim().is_empty() {
                String::new()
            } else {
                let n = match_count.get();
                format!("{n} {} match “{}”", if n == 1 { "frame" } else { "frames" }, q.trim())
            }
        })
    };
    let legend = Memo::new(move |_| {
        if flame {
            return Vec::new();
        }
        root.with(|r| zoom.with(|z| r.node_at(&r.valid_path(z)).map(partition::legend).unwrap_or_default()))
    });
    let scope = if flame { "flame-graph-chart" } else { "icicle-chart" };
    let noun = if flame { "frames" } else { "cells" };
    let caption = label.clone();

    view! {
        <figure data-scope=scope data-part="root" data-searching=move || (!query().trim().is_empty()).then_some("")>
            <Breadcrumb crumbs=crumbs go=Callback::new(go) />
            <div data-part="plot">
                <svg
                    data-part="marks"
                    node_ref=marks
                    viewBox=format!("0 0 {WIDTH} {height}")
                    tabindex="0"
                    role="group"
                    aria-label=format!("{label}. Use the arrow keys to move between {noun}, Enter to zoom in, Escape to zoom out.")
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        let zoomed = zoom.with_untracked(|z| !z.is_empty());
                        let len = layout.with_untracked(|l| l.cells.len());
                        match zoom_key(&ev.key(), active.get_untracked(), len, zoomed) {
                            Some(ZoomKey::Activate(i)) => {
                                ev.prevent_default();
                                crate::callbacks::select_active(Some(datum), on_select);
                                activate(i);
                            }
                            Some(ZoomKey::Up) => {
                                ev.prevent_default();
                                up();
                            }
                            Some(ZoomKey::Move(next)) => {
                                ev.prevent_default();
                                active.set(next);
                            }
                            None => {}
                        }
                    }
                >
                    {move || {
                        let l = layout.get();
                        let q = query();
                        l.cells
                            .iter()
                            .enumerate()
                            .map(|(i, c)| {
                                let (x, y, w, rh) = rect(c, l.levels);
                                let text = (rh >= 16.0).then(|| partition::fit_label(&c.name, w, 6.5)).flatten();
                                view! {
                                    <g
                                        data-part="cell"
                                        data-depth=c.depth
                                        data-leaf=(!c.zoomable).then_some("")
                                        data-match=partition::matches(&c.name, &q).then_some("")
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        style=format!("--i: {};", c.depth.min(20))
                                        on:pointerenter=move |_| active.set(Some(i))
                                        on:pointerdown=move |_| active.set(Some(i))
                                        on:click=move |_| {
                                            crate::callbacks::select_active(Some(datum), on_select);
                                            activate(i);
                                        }
                                    >
                                        <rect x=format!("{x:.2}") y=format!("{:.2}", y + 0.5) width=format!("{w:.2}") height=format!("{:.2}", (rh - 1.0).max(0.5)) rx="2" fill=fill(c)></rect>
                                        {text.map(|t| view! { <text data-part="label" x=format!("{:.2}", x + 4.0) y=format!("{:.2}", y + rh / 2.0) dominant-baseline="central">{t}</text> })}
                                    </g>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                {move || {
                    let l = layout.get();
                    let c = l.cells.get(active.get()?)?;
                    let (x, y, w, rh) = rect(c, l.levels);
                    let (fx, fy) = ((x + w / 2.0) / WIDTH, if flame { y / plot_h } else { (y + rh).min(plot_h) / plot_h });
                    let mut rows = vec![TipRow::new(None, if flame { "Total" } else { "Value" }, fmt(c.value))];
                    if flame {
                        rows.push(TipRow::new(None, "Self", fmt(c.self_value)));
                    }
                    rows.push(TipRow::new(None, "Share", format!("{:.1}%", share(&l, c.value))));
                    // The tooltip opens below the cell near the top edge (fy < 0.25), above it elsewhere.
                    let tip = Tip { x: fx, y: if fy < 0.25 { 1.0 } else { 0.0 }, title: c.name.clone(), rows };
                    let (swatch, style) = (fill(c), format!("left: {}; top: {};", pct(fx), pct(fy)));
                    Some(view! { <ChartTip tip swatch style /> })
                }}
            </div>
            <PartitionLegend items=legend />
            <span data-part="sr-live" aria-live="polite">{announce}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">{if flame { "Frame" } else { "Node" }}</th>
                        <th scope="col">{if flame { "Total" } else { "Value" }}</th>
                        {flame.then(|| view! { <th scope="col">"Self"</th> })}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        root.with(|r| partition::rows(r, flame))
                            .into_iter()
                            .map(|(name, v, own)| view! { <tr><th scope="row">{name}</th><td>{fmt(v)}</td>{flame.then(|| view! { <td>{fmt(own)}</td> })}</tr> })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
