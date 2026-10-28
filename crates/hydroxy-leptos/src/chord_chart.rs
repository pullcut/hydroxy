//! `<ChordChart>`: flows between groups from a square matrix. Each group's arc is sized
//! by what it sends; ribbons join pairs, as wide at each end as the flow that way.
//! Geometry comes from `hydroxy_charts::chord`.

use hydroxy_charts::chord::{self, Hit};
use hydroxy_charts::format_value;
use hydroxy_charts::nav::{self, HINT_GROUPS, hinted};
use leptos::prelude::*;

use crate::plot_kit::ChartTip;

/// A chord diagram. Hover or focus a group to see only its chords; hover a ribbon to
/// read the flow both ways. Focus the chart and use the arrow keys to step through
/// groups, Home and End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn ChordChart(
    /// One name per group, in matrix order.
    #[prop(into)]
    names: Signal<Vec<String>>,
    /// Square matrix: `matrix[i][j]` is the flow from group `i` to group `j`.
    #[prop(into)]
    matrix: Signal<Vec<Vec<f64>>>,
    /// What the chart shows, such as "Customers switching plan". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Gap between groups, in radians.
    #[prop(default = 0.04)]
    pad: f64,
    /// Formats values for the tooltip, readout and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| matrix.with(|m| chord::layout(m, pad)));
    let active = RwSignal::new(None::<Hit>);
    let count = move || layout.with(|l| l.arcs.len());
    let name = move |i: usize| names.with(|n| n.get(i).cloned().unwrap_or_else(|| format!("Group {}", i + 1)));
    let group_of = move || match active.get() {
        Some(Hit::Group(i)) => Some(i),
        _ => None,
    };
    let describe = move |hit: Hit| layout.with(|lay| chord::tooltip(lay, hit, name, fmt)).map(|tip| tip.announce());
    let dim_ribbon = move |k: usize| {
        let (s, t) = layout.with(|l| l.ribbons.get(k).map_or((usize::MAX, usize::MAX), |r| (r.source, r.target)));
        match active.get() {
            Some(Hit::Group(g)) => g != s && g != t,
            Some(Hit::Ribbon(r)) => r != k,
            None => false,
        }
    };
    let caption = label.clone();

    view! {
        <figure data-scope="chord-chart" data-part="root" data-active=move || active.get().map(|_| "")>
            <div data-part="plot">
                <svg
                    data-part="marks"
                    viewBox=format!("0 0 {0} {0}", chord::SIZE)
                    tabindex="0"
                    role="group"
                    aria-label=hinted(&label, HINT_GROUPS)
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if let Some(next) = nav::list_key(&ev.key(), group_of(), count()) {
                            ev.prevent_default();
                            active.set(next.map(Hit::Group));
                        }
                    }
                >
                    <g data-part="ribbons">
                        {move || {
                            let n = count();
                            layout
                                .get()
                                .ribbons
                                .into_iter()
                                .enumerate()
                                .map(|(k, r)| {
                                    view! {
                                        <path
                                            data-part="ribbon"
                                            data-dim=move || dim_ribbon(k).then_some("")
                                            data-active=move || (active.get() == Some(Hit::Ribbon(k))).then_some("")
                                            d=r.path
                                            fill=chord::colour(r.colour, n)
                                            style=format!("--i: {};", k.min(40))
                                            on:pointerenter=move |_| active.set(Some(Hit::Ribbon(k)))
                                            on:pointerdown=move |_| active.set(Some(Hit::Ribbon(k)))
                                        ></path>
                                    }
                                })
                                .collect_view()
                        }}
                    </g>
                    <g data-part="groups">
                        {move || {
                            let n = count();
                            layout
                                .get()
                                .arcs
                                .into_iter()
                                .enumerate()
                                .map(|(i, a)| {
                                    view! {
                                        <path
                                            data-part="group"
                                            data-active=move || (active.get() == Some(Hit::Group(i))).then_some("")
                                            data-dim=move || matches!(active.get(), Some(Hit::Group(g)) if g != i).then_some("")
                                            d=a.path
                                            fill=chord::colour(i, n)
                                            style=format!("--i: {i};")
                                            on:pointerenter=move |_| active.set(Some(Hit::Group(i)))
                                            on:pointerdown=move |_| active.set(Some(Hit::Group(i)))
                                        ></path>
                                        <text data-part="label" x=format!("{:.1}", a.label.0) y=format!("{:.1}", a.label.1) text-anchor=a.anchor dominant-baseline="central">
                                            {name(i)}
                                        </text>
                                    }
                                })
                                .collect_view()
                        }}
                    </g>
                </svg>
                {move || {
                    let hit = active.get()?;
                    let (mut tip, swatch) = layout.with(|lay| Some((chord::tooltip(lay, hit, name, fmt)?, chord::swatch(lay, hit)?)))?;
                    let style = format!("left: {:.2}%; top: {:.2}%;", tip.x * 100.0, (1.0 - tip.y) * 100.0);
                    // Open below the anchor in the top quarter; the anchor only picks the side and
                    // vertical placement because `style` positions it.
                    tip.y = if tip.y > 0.75 { 1.0 } else { 0.0 };
                    Some(view! { <ChartTip tip swatch style /> })
                }}
            </div>
            <Show when=move || { count() > 1 }>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        let n = count();
                        let mut items: Vec<(String, String)> = (0..n.min(if n > 6 { 5 } else { 6 })).map(|i| (chord::colour(i, n), name(i))).collect();
                        if n > 6 {
                            items.push(("var(--chart-muted)".to_string(), "Other".to_string()));
                        }
                        items
                            .into_iter()
                            .map(|(c, t)| view! {
                                <li data-part="legend-item">
                                    <span data-part="swatch" style=format!("background: {c};")></span>
                                    {t}
                                </li>
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"From → To"</th>
                        {move || (0..count()).map(|j| view! { <th scope="col">{name(j)}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let n = count();
                        matrix.with(|m| {
                            (0..n)
                                .map(|i| {
                                    let row: Vec<String> = (0..n).map(|j| fmt(m.get(i).and_then(|r| r.get(j)).copied().unwrap_or(0.0))).collect();
                                    view! {
                                        <tr>
                                            <th scope="row">{name(i)}</th>
                                            {row.into_iter().map(|v| view! { <td>{v}</td> }).collect_view()}
                                        </tr>
                                    }
                                })
                                .collect_view()
                        })
                    }}
                </tbody>
            </table>
        </figure>
    }
}
