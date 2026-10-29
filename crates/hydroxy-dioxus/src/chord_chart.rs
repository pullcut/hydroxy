//! `ChordChart`: flows between groups from a square matrix. Each group's arc is sized
//! by what it sends; ribbons join pairs, as wide at each end as the flow that way.
//! Geometry comes from `hydroxy_charts::chord`.

use dioxus::prelude::*;
use hydroxy_charts::chord::{self, Hit};
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::Tip;
use hydroxy_charts::{format_value, nav};

use crate::plot_kit::ChartTip;

/// A chord diagram. Hover or focus a group to see only its chords; hover a ribbon to
/// read the flow both ways. Focus the chart and use the arrow keys to step through
/// groups, Home and End to jump, Escape to clear.
#[component]
pub fn ChordChart(
    /// One name per group, in matrix order.
    names: ReadSignal<Vec<String>>,
    /// Square matrix: `matrix[i][j]` is the flow from group `i` to group `j`.
    matrix: ReadSignal<Vec<Vec<f64>>>,
    /// What the chart shows, such as "Customers switching plan". Used as the accessible name.
    label: String,
    /// Gap between groups, in radians.
    #[props(default = 0.04)]
    pad: f64,
    /// Formats values for the tooltip, readout and table.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let names_v = names();
    let matrix_v = matrix();
    let lay = chord::layout(&matrix_v, pad);
    let mut active = use_signal(|| None::<Hit>);
    let count = lay.arcs.len();
    let name = |i: usize| names_v.get(i).cloned().unwrap_or_else(|| format!("Group {}", i + 1));
    let current = active().filter(|a| match *a {
        Hit::Group(i) => i < count,
        Hit::Ribbon(k) => k < lay.ribbons.len(),
    });
    let cur = if on_hover.is_some() || on_select.is_some() {
        current.map(|hit| match hit {
            // A group reports what flows out of it; a ribbon its source series and target index.
            Hit::Group(i) => EventDatum::category(i, name(i), lay.arcs[i].out),
            Hit::Ribbon(k) => {
                let r = &lay.ribbons[k];
                EventDatum {
                    series: r.source,
                    ..EventDatum::category(r.target, format!("{} → {}", name(r.source), name(r.target)), r.value)
                }
            }
        })
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let group_of = match current {
        Some(Hit::Group(i)) => Some(i),
        _ => None,
    };
    let dim_ribbon = |k: usize, r: &chord::Ribbon| match current {
        Some(Hit::Group(g)) => g != r.source && g != r.target,
        Some(Hit::Ribbon(on)) => on != k,
        None => false,
    };
    // The style places the tooltip; `y` is then only the `data-vertical` flag (1.0 = below).
    let tip = current.and_then(|hit| {
        let tip = chord::tooltip(&lay, hit, name, &fmt)?;
        let style = format!("left: {:.2}%; top: {:.2}%;", tip.x * 100.0, (1.0 - tip.y) * 100.0);
        let y = if tip.y > 0.75 { 1.0 } else { 0.0 };
        let announce = tip.announce();
        Some((Tip { y, ..tip }, chord::swatch(&lay, hit), style, announce))
    });
    let announcement = tip.as_ref().map(|t| t.3.clone()).unwrap_or_default();
    let legend = (0..count.min(if count > 6 { 5 } else { 6 }))
        .map(|i| (chord::colour(i, count), name(i)))
        .chain((count > 6).then(|| (chord::colour(count, count), "Other".to_string())));
    let cell = |i: usize, j: usize| fmt(matrix_v.get(i).and_then(|r| r.get(j)).copied().unwrap_or(0.0));

    rsx! {
        figure { "data-scope": "chord-chart", "data-part": "root", "data-active": current.map(|_| ""),
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    view_box: "0 0 {chord::SIZE} {chord::SIZE}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{nav::hinted(&label, nav::HINT_GROUPS)}",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if e.key() == dioxus::html::Key::Enter {
                            crate::callbacks::select_active(Some(datum), on_select);
                        }
                        if let Some(next) = nav::list_key(&e.key().to_string(), group_of, count) {
                            e.prevent_default();
                            active.set(next.map(Hit::Group));
                        }
                    },
                    g { "data-part": "ribbons",
                        for (k , r) in lay.ribbons.iter().enumerate() {
                            path {
                                key: "{r.source}-{r.target}",
                                "data-part": "ribbon",
                                "data-dim": dim_ribbon(k, r).then_some(""),
                                "data-active": (current == Some(Hit::Ribbon(k))).then_some(""),
                                d: "{r.path}",
                                fill: "{chord::colour(r.colour, count)}",
                                style: "--i: {k.min(40)};",
                                onpointerenter: move |_| active.set(Some(Hit::Ribbon(k))),
                                onpointerdown: move |_| active.set(Some(Hit::Ribbon(k))),
                                onclick: move |_| {
                                    active.set(Some(Hit::Ribbon(k)));
                                    crate::callbacks::select_active(Some(datum), on_select);
                                },
                            }
                        }
                    }
                    g { "data-part": "groups",
                        for (i , a) in lay.arcs.iter().enumerate() {
                            path {
                                key: "{i}",
                                "data-part": "group",
                                "data-active": (current == Some(Hit::Group(i))).then_some(""),
                                "data-dim": matches!(current, Some(Hit::Group(g)) if g != i).then_some(""),
                                d: "{a.path}",
                                fill: "{chord::colour(i, count)}",
                                style: "--i: {i};",
                                onpointerenter: move |_| active.set(Some(Hit::Group(i))),
                                onpointerdown: move |_| active.set(Some(Hit::Group(i))),
                                onclick: move |_| {
                                    active.set(Some(Hit::Group(i)));
                                    crate::callbacks::select_active(Some(datum), on_select);
                                },
                            }
                            text {
                                "data-part": "label",
                                x: "{a.label.0:.1}",
                                y: "{a.label.1:.1}",
                                text_anchor: a.anchor,
                                dominant_baseline: "central",
                                "{name(i)}"
                            }
                        }
                    }
                }
                if let Some((tip, swatch, style, _)) = tip {
                    ChartTip { tip, swatch, style }
                }
            }
            if count > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (c , t) in legend {
                        li { "data-part": "legend-item",
                            span { "data-part": "swatch", style: "background: {c};" }
                            "{t}"
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "From → To" }
                        for j in 0..count {
                            th { scope: "col", "{name(j)}" }
                        }
                    }
                }
                tbody {
                    for i in 0..count {
                        tr {
                            th { scope: "row", "{name(i)}" }
                            for j in 0..count {
                                td { "{cell(i, j)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
