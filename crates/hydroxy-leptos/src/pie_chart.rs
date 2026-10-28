//! `<PieChart>`: parts of a whole as a full pie. Slices come from `hydroxy_charts::pie`,
//! which reuses the donut arcs with a ring as thick as the radius. More than six
//! parts fold into "Other".

use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{fold_other, format_value, funnel, nav, pie};
use leptos::prelude::*;

use crate::plot_kit::{ChartTip, colour, colour_or_muted, pct};

pub use hydroxy_charts::Datum;
pub use hydroxy_charts::pie::Labels as PieLabels;

/// A full pie. Hover, touch or focus a slice to pull it out and read it; focus the pie
/// and use the arrow keys to step through slices, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn PieChart(
    /// The parts, in display order.
    #[prop(into)]
    data: Signal<Vec<Datum>>,
    /// What the chart shows, such as "Revenue by region". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Pie diameter in px.
    #[prop(default = 200)]
    size: u16,
    /// Where slice labels go. Default inside, as shares.
    #[prop(optional)]
    labels: PieLabels,
    /// Formats values for the tooltip, legend and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let parts = Memo::new(move |_| data.with(|d| fold_other(d, 6)));
    let folded = Memo::new(move |_| data.with(Vec::len) > 6);
    let layout = Memo::new(move |_| {
        parts.with(|p| pie::layout(&p.iter().map(|d| d.value).collect::<Vec<_>>(), f64::from(size), 1.5, labels))
    });
    let active = RwSignal::new(None::<usize>);
    let slot_colour = move |i: usize| {
        if folded.get() { colour_or_muted(i, parts.with(Vec::len).saturating_sub(1)) } else { colour(i) }
    };
    let share = |f: f64| format!("{:.0}%", f * 100.0);
    let describe = move |i: usize| {
        let d = parts.with(|p| p.get(i).cloned())?;
        let f = layout.with(|l| l.slices.get(i).map(|s| s.fraction))?;
        Some(format!("{}: {}, {}", d.label, fmt(d.value), share(f)))
    };
    let (box_w, box_h, pad) =
        (layout.with_untracked(|l| l.width), layout.with_untracked(|l| l.height), layout.with_untracked(|l| l.pad));
    let stroke = layout.with_untracked(|l| l.stroke_width);
    // Slices are keyed by label: a changed share reshapes its own arc; new parts sweep in.
    let rows = move || parts.get().into_iter().enumerate().collect::<Vec<_>>();

    view! {
        <figure data-scope="pie-chart" data-part="root" style=move || format!("--ox-segments: {};", parts.with(Vec::len))>
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=format!("{label}. Use the arrow keys to step through slices.")
                style=format!("width: {box_w}px; aspect-ratio: {box_w} / {box_h};")
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    let key = ev.key();
                    if let Some(next) = nav::list_key(funnel::arrow_alias(&key), active.get_untracked(), parts.with(Vec::len)) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                <svg viewBox=format!("0 0 {box_w} {box_h}") aria-hidden="true">
                    <g transform=format!("translate({} {})", pad.0, pad.1)>
                        <For
                            each=rows
                            key=|(_, d)| d.label.clone()
                            children=move |(i, d)| {
                                let name = d.label;
                                let pos = Memo::new(move |_| parts.with(|p| p.iter().position(|x| x.label == name)).unwrap_or(i));
                                let slice = move || layout.with(|l| l.slices.get(pos.get()).cloned());
                                view! {
                                    <path
                                        data-part="slice"
                                        data-index=move || pos.get()
                                        data-active=move || (active.get() == Some(pos.get())).then_some("")
                                        style=move || {
                                            let (dx, dy) = slice().map_or((0.0, 0.0), |s| s.out);
                                            format!("--i: {i}; --dx: {:.2}px; --dy: {:.2}px;", dx * 8.0, dy * 8.0)
                                        }
                                        d=move || slice().map(|s| s.path).unwrap_or_default()
                                        stroke=move || slot_colour(pos.get())
                                        stroke-width=stroke
                                        pathLength="1"
                                        on:pointerenter=move |_| active.set(Some(pos.get_untracked()))
                                        on:pointerdown=move |_| active.set(Some(pos.get_untracked()))
                                    ></path>
                                }
                            }
                        />
                    </g>
                    {move || {
                        layout
                            .get()
                            .slices
                            .into_iter()
                            .enumerate()
                            .filter(|(_, s)| s.fraction > 0.0)
                            .filter_map(|(i, s)| s.leader.map(|l| (i, l)))
                            .map(|(i, l)| {
                                let name = parts.with(|p| p.get(i).map(|d| d.label.clone())).unwrap_or_default();
                                let share_text = share(layout.with(|ly| ly.slices[i].fraction));
                                view! {
                                    <g data-part="leader" data-index=i style=format!("--i: {i};")>
                                        <path d=l.path></path>
                                        <text x=l.x y=l.y text-anchor={if l.end { "end" } else { "start" }} dominant-baseline="central">
                                            {name}
                                            <tspan data-part="leader-share">{format!(" {share_text}")}</tspan>
                                        </text>
                                    </g>
                                }
                            })
                            .collect_view()
                    }}
                </svg>
                {move || {
                    (labels == PieLabels::Inside)
                        .then(|| {
                            layout
                                .get()
                                .slices
                                .into_iter()
                                .enumerate()
                                .filter(|(_, s)| s.fraction >= pie::INSIDE_MIN)
                                .map(|(i, s)| {
                                    view! {
                                        <span
                                            data-part="slice-label"
                                            data-index=i
                                            aria-hidden="true"
                                            style=format!("left: {}; top: {}; --i: {i};", pct(s.centroid.0), pct(s.centroid.1))
                                        >
                                            {share(s.fraction)}
                                        </span>
                                    }
                                })
                                .collect_view()
                        })
                }}
                {move || {
                    let i = active.get()?;
                    let d = parts.with(|p| p.get(i).cloned())?;
                    let s = layout.with(|l| l.slices.get(i).cloned())?;
                    let tip = Tip {
                        x: s.centroid.0,
                        y: 1.0 - s.centroid.1,
                        title: d.label,
                        rows: vec![TipRow::new(None, "Value", fmt(d.value)), TipRow::new(None, "Share", share(s.fraction))],
                    };
                    Some(view! { <ChartTip tip swatch=slot_colour(i) /> })
                }}
            </div>
            <Show when=move || parts.with(|p| p.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        parts
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, d)| {
                                view! {
                                    <li
                                        data-part="legend-item"
                                        data-index=i
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        on:pointerenter=move |_| active.set(Some(i))
                                        on:pointerleave=move |_| active.set(None)
                                    >
                                        <span data-part="swatch" style=format!("background: {};", slot_colour(i))></span>
                                        {d.label}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label.clone()}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Part"</th>
                        <th scope="col">"Value"</th>
                        <th scope="col">"Share"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let l = layout.get();
                        parts
                            .get()
                            .into_iter()
                            .zip(l.slices)
                            .map(|(d, s)| {
                                view! {
                                    <tr>
                                        <th scope="row">{d.label}</th>
                                        <td>{fmt(d.value)}</td>
                                        <td>{share(s.fraction)}</td>
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
