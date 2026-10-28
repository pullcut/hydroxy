//! `ChordChart`: flows between groups from a square matrix. Each group's arc is sized
//! by what it sends; ribbons join pairs, as wide at each end as the flow that way.
//! Geometry comes from `hydroxy_charts::chord`.

use dioxus::prelude::*;
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
fn tooltip(
    lay: &chord::Chord,
    hit: Active,
    name: &dyn Fn(usize) -> String,
    fmt: &dyn Fn(f64) -> String,
) -> Option<Tip> {
    let groups = lay.arcs.len();
    let (title, swatch, rows, at) = match hit {
        Active::Group(i) => {
            let arc = lay.arcs.get(i)?;
            (
                name(i),
                chord::colour(i, groups),
                vec![("Sends".to_string(), fmt(arc.out)), ("Receives".to_string(), fmt(arc.incoming))],
                arc.label,
            )
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
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
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
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let names_v = names();
    let matrix_v = matrix();
    let lay = chord::layout(&matrix_v, pad);
    let mut active = use_signal(|| None::<Active>);
    let count = lay.arcs.len();
    let name = |i: usize| names_v.get(i).cloned().unwrap_or_else(|| format!("Group {}", i + 1));
    let current = active().filter(|a| match *a {
        Active::Group(i) => i < count,
        Active::Ribbon(k) => k < lay.ribbons.len(),
    });
    let group_of = match current {
        Some(Active::Group(i)) => Some(i),
        _ => None,
    };
    let dim_ribbon = |k: usize, r: &chord::Ribbon| match current {
        Some(Active::Group(g)) => g != r.source && g != r.target,
        Some(Active::Ribbon(on)) => on != k,
        None => false,
    };
    let tip = current.and_then(|a| tooltip(&lay, a, &name, &fmt));
    let announcement = tip
        .as_ref()
        .map(|(title, _, rows, _)| {
            let rows: Vec<String> = rows.iter().map(|(k, v)| format!("{k} {v}")).collect();
            format!("{title}: {}", rows.join(", "))
        })
        .unwrap_or_default();
    let mut legend: Vec<(String, String)> =
        (0..count.min(if count > 6 { 5 } else { 6 })).map(|i| (chord::colour(i, count), name(i))).collect();
    if count > 6 {
        legend.push(("var(--chart-muted)".to_string(), "Other".to_string()));
    }
    let cell = |i: usize, j: usize| fmt(matrix_v.get(i).and_then(|r| r.get(j)).copied().unwrap_or(0.0));

    rsx! {
        figure { "data-scope": "chord-chart", "data-part": "root", "data-active": current.map(|_| ""),
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    view_box: "0 0 {chord::SIZE} {chord::SIZE}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to step through groups.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if let Some(next) = nav::list_key(&e.key().to_string(), group_of, count) {
                            e.prevent_default();
                            active.set(next.map(Active::Group));
                        }
                    },
                    g { "data-part": "ribbons",
                        for (k , r) in lay.ribbons.iter().enumerate() {
                            path {
                                key: "{r.source}-{r.target}",
                                "data-part": "ribbon",
                                "data-dim": dim_ribbon(k, r).then_some(""),
                                "data-active": (current == Some(Active::Ribbon(k))).then_some(""),
                                d: "{r.path}",
                                fill: "{chord::colour(r.colour, count)}",
                                style: "--i: {k.min(40)};",
                                onpointerenter: move |_| active.set(Some(Active::Ribbon(k))),
                                onpointerdown: move |_| active.set(Some(Active::Ribbon(k))),
                            }
                        }
                    }
                    g { "data-part": "groups",
                        for (i , a) in lay.arcs.iter().enumerate() {
                            path {
                                key: "{i}",
                                "data-part": "group",
                                "data-active": (current == Some(Active::Group(i))).then_some(""),
                                "data-dim": matches!(current, Some(Active::Group(g)) if g != i).then_some(""),
                                d: "{a.path}",
                                fill: "{chord::colour(i, count)}",
                                style: "--i: {i};",
                                onpointerenter: move |_| active.set(Some(Active::Group(i))),
                                onpointerdown: move |_| active.set(Some(Active::Group(i))),
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
                if let Some((title , swatch , rows , (left , top))) = tip {
                    div {
                        "data-part": "tooltip",
                        "data-side": if left > 50.0 { "left" } else { "right" },
                        "data-vertical": if top < 25.0 { "below" } else { "above" },
                        style: "left: {left:.2}%; top: {top:.2}%;",
                        "aria-hidden": "true",
                        div { "data-part": "tooltip-label",
                            span { "data-part": "swatch", style: "background: {swatch};" }
                            "{title}"
                        }
                        ul {
                            for (k , v) in rows {
                                li { "data-part": "tooltip-row",
                                    span { "data-part": "tooltip-name", "{k}" }
                                    span { "data-part": "tooltip-value", "{v}" }
                                }
                            }
                        }
                    }
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
