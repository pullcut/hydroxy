//! `<ChordChart>`: flows between groups from a square matrix. Each group's arc is sized
//! by what it sends; ribbons join pairs, as wide at each end as the flow that way.
//! Geometry comes from `hydroxy_charts::chord`.

use leptos::prelude::*;
use hydroxy_charts::{chord, format_value, nav};

/// What the pointer or keyboard is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Active {
    Group(usize),
    Ribbon(usize),
}

fn pct(px: f64) -> f64 {
    px / chord::SIZE * 100.0
}

type Tip = (String, String, Vec<(String, String)>, (f64, f64));

/// Tooltip title, swatch colour, rows and anchor (in % of the plot) for the active mark.
fn tooltip(lay: &chord::Chord, hit: Active, name: &dyn Fn(usize) -> String, fmt: &dyn Fn(f64) -> String) -> Option<Tip> {
    let groups = lay.arcs.len();
    let (title, swatch, rows, at) = match hit {
        Active::Group(i) => {
            let arc = lay.arcs.get(i)?;
            (name(i), chord::colour(i, groups), vec![("Sends".to_string(), fmt(arc.out)), ("Receives".to_string(), fmt(arc.incoming))], arc.label)
        }
        Active::Ribbon(k) => {
            let rib = lay.ribbons.get(k)?;
            let (from, to) = (name(rib.source), name(rib.target));
            let mut rows = vec![(format!("{from} → {to}"), fmt(rib.value))];
            if rib.source != rib.target {
                rows.push((format!("{to} → {from}"), fmt(rib.back)));
            }
            let title = if rib.source == rib.target { from } else { format!("{from} and {to}") };
            (title, chord::colour(rib.colour, groups), rows, rib.anchor)
        }
    };
    Some((title, swatch, rows, (pct(at.0), pct(at.1))))
}

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
    let active = RwSignal::new(None::<Active>);
    let count = move || layout.with(|l| l.arcs.len());
    let name = move |i: usize| names.with(|n| n.get(i).cloned().unwrap_or_else(|| format!("Group {}", i + 1)));
    let group_of = move || match active.get() {
        Some(Active::Group(i)) => Some(i),
        _ => None,
    };
    let describe = move |hit: Active| {
        let (title, _, rows, _) = layout.with(|lay| tooltip(lay, hit, &name, &fmt))?;
        let rows: Vec<String> = rows.iter().map(|(k, v)| format!("{k} {v}")).collect();
        Some(format!("{title}: {}", rows.join(", ")))
    };
    let dim_ribbon = move |k: usize| {
        let (s, t) = layout.with(|l| l.ribbons.get(k).map_or((usize::MAX, usize::MAX), |r| (r.source, r.target)));
        match active.get() {
            Some(Active::Group(g)) => g != s && g != t,
            Some(Active::Ribbon(r)) => r != k,
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
                    aria-label=format!("{label}. Use the arrow keys to step through groups.")
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if let Some(next) = nav::list_key(&ev.key(), group_of(), count()) {
                            ev.prevent_default();
                            active.set(next.map(Active::Group));
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
                                            data-active=move || (active.get() == Some(Active::Ribbon(k))).then_some("")
                                            d=r.path
                                            fill=chord::colour(r.colour, n)
                                            style=format!("--i: {};", k.min(40))
                                            on:pointerenter=move |_| active.set(Some(Active::Ribbon(k)))
                                            on:pointerdown=move |_| active.set(Some(Active::Ribbon(k)))
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
                                            data-active=move || (active.get() == Some(Active::Group(i))).then_some("")
                                            data-dim=move || matches!(active.get(), Some(Active::Group(g)) if g != i).then_some("")
                                            d=a.path
                                            fill=chord::colour(i, n)
                                            style=format!("--i: {i};")
                                            on:pointerenter=move |_| active.set(Some(Active::Group(i)))
                                            on:pointerdown=move |_| active.set(Some(Active::Group(i)))
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
                    let (title, swatch, rows, (left, top)) = layout.with(|lay| tooltip(lay, hit, &name, &fmt))?;
                    Some(view! {
                        <div
                            data-part="tooltip"
                            data-side={if left > 50.0 { "left" } else { "right" }}
                            data-vertical={if top < 25.0 { "below" } else { "above" }}
                            style=format!("left: {left:.2}%; top: {top:.2}%;")
                            aria-hidden="true"
                        >
                            <div data-part="tooltip-label">
                                <span data-part="swatch" style=format!("background: {swatch};")></span>
                                {title}
                            </div>
                            <ul>
                                {rows
                                    .into_iter()
                                    .map(|(k, v)| view! {
                                        <li data-part="tooltip-row">
                                            <span data-part="tooltip-name">{k}</span>
                                            <span data-part="tooltip-value">{v}</span>
                                        </li>
                                    })
                                    .collect_view()}
                            </ul>
                        </div>
                    })
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
