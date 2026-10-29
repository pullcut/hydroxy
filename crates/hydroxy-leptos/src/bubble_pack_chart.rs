//! `<BubblePackChart>`: a tree as nested circles, each leaf's area its value. Click a
//! circle with children to zoom into it. Layout comes from `hydroxy_charts::pack`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, pack, partition};
use leptos::prelude::*;

use crate::callbacks::{select_active, watch_hover};
use crate::hierarchy_kit::{Breadcrumb, PartitionLegend, ZoomKey, zoom_key};
use crate::plot_kit::{ChartTip, colour, pct};

pub use hydroxy_charts::hierarchy::Tree;

/// Circle packing. Hover, touch or focus the chart and use the arrow keys to read each
/// circle; click one with children, or press Enter on it, to zoom in. Escape, the
/// outer circle or the breadcrumb zoom back out.
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
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
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
    let datum = (on_hover.is_some() || on_select.is_some()).then(|| {
        Signal::derive(move || {
            let path = bubbles.with(|l| l.get(active.get()?).map(|b| b.path.clone()))?;
            root.with(|r| EventDatum::from_node(r, &path))
        })
    });
    watch_hover(datum, on_hover);
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
            <Breadcrumb crumbs=crumbs go=Callback::new(go) />
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
                        let zoomed = zoom.with_untracked(|z| !z.is_empty());
                        match zoom_key(&ev.key(), active.get_untracked(), bubbles.with_untracked(Vec::len), zoomed) {
                            Some(ZoomKey::Activate(i)) => {
                                ev.prevent_default();
                                select_active(datum, on_select);
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
                                        fill=b.colour.map_or_else(|| "var(--chart-muted)".to_string(), colour)
                                        style=format!("--i: {};", i.min(40))
                                        on:pointerenter=move |_| active.set(Some(i))
                                        on:pointerdown=move |_| active.set(Some(i))
                                        on:click=move |_| {
                                            active.set(Some(i));
                                            select_active(datum, on_select);
                                            activate(i);
                                        }
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
                    let fill = b.colour.map_or_else(|| "var(--chart-muted)".to_string(), colour);
                    // The anchor only picks the side and vertical placement; `style` positions it.
                    let tip = Tip {
                        x,
                        y: if y < 0.25 { 1.0 } else { 0.0 },
                        title: b.name,
                        rows: vec![
                            TipRow::new(None, "Value", fmt(b.value)),
                            TipRow::new(None, "Share", format!("{share:.1}%")),
                        ],
                    };
                    Some(view! { <ChartTip tip swatch=fill style=format!("left: {}; top: {};", pct(x), pct(y)) /> })
                }}
            </div>
            <PartitionLegend items=legend />
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
