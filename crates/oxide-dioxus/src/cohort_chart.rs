//! `CohortChart`: a retention triangle. Rows are cohorts, columns periods since they
//! started, cells shaded on a sequential ramp. Geometry comes from `oxide_charts::cohort`.

use dioxus::prelude::*;
use oxide_charts::cohort::{self, percent};
use oxide_charts::format_value;

pub use oxide_charts::cohort::Cohort;

use crate::chart_frame::use_export;

/// One row per cohort with its size, then a cell per period it has been through, shaded
/// by the share still active and labelled as a percentage. Hover or touch a cell for the
/// counts; focus the grid and use the arrow keys, Home and End to move, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[allow(clippy::cast_precision_loss, reason = "grid sizes are small")]
#[component]
pub fn CohortChart(
    /// The cohorts, oldest first. Each has a size and a count per period so far.
    cohorts: ReadSignal<Vec<Cohort>>,
    /// One label per period, such as "Month 0", "Month 1"…
    periods: ReadSignal<Vec<String>>,
    /// What the chart shows, such as "Monthly retention by signup month". Used as the accessible name.
    label: String,
    /// What a count is of, for the tooltip, such as "customers". Default "people".
    #[props(default = "people".to_string())]
    noun: String,
    /// Cell height in px.
    #[props(default = 32)]
    cell_height: u16,
    /// Formats counts in the size column, tooltip and table.
    format: Option<Callback<f64, String>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    // A memo, so a hover never re-lays out the grid.
    let layout = use_memo(move || cohort::layout(&cohorts.read()));
    let mut active = use_signal(|| None::<(usize, usize)>);
    use_export(move |()| {
        let head = ["Cohort".to_string(), "Size".to_string()]
            .into_iter()
            .chain(periods.read().iter().cloned())
            .collect::<Vec<_>>();
        let body = cohorts
            .read()
            .iter()
            .map(|c| {
                [c.name.clone(), fmt(c.size)]
                    .into_iter()
                    .chain(c.retained.iter().map(|&v| if v.is_finite() { fmt(v) } else { String::new() }))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        std::iter::once(head).chain(body).collect()
    });
    let all = cohorts.read();
    let cols_v = periods.read();
    let l = layout.read();
    let (rows, cols) = (l.rows.max(1), l.cols.max(1));
    let current = active().filter(|&(r, c)| l.at(r, c).is_some());
    let describe = |(r, c): (usize, usize)| {
        let cell = l.at(r, c)?;
        let co = all.get(r)?;
        let period = cols_v.get(c).cloned().unwrap_or_default();
        Some((
            format!("{} · {period}", co.name),
            format!("{} of {} {noun} ({})", fmt(cell.count), fmt(co.size), percent(cell.rate)),
        ))
    };
    let tip = current.and_then(|(r, c)| {
        let (title, value) = describe((r, c))?;
        let below = r == 0;
        let edge = if (c as f64) < cols as f64 / 3.0 {
            "start"
        } else if (c as f64) >= cols as f64 * 2.0 / 3.0 {
            "end"
        } else {
            ""
        };
        let style = format!(
            "left: {:.3}%; top: {:.3}%;",
            (c as f64 + 0.5) / cols as f64 * 100.0,
            (r as f64 + if below { 1.0 } else { 0.0 }) / rows as f64 * 100.0
        );
        Some((title, value, if below { "below" } else { "above" }, edge, style))
    });
    let announcement = current.and_then(describe).map(|(t, v)| format!("{t}: {v}")).unwrap_or_default();
    let top = percent(l.cells.iter().filter(|c| c.col > 0).map(|c| c.rate).fold(0.0, f64::max));

    rsx! {
        figure { "data-scope": "cohort-chart", "data-part": "root",
            div { "data-part": "frame", style: "--cols: {cols}; --rows: {rows}; --cell: {cell_height}px;",
                div { "data-part": "corner", "aria-hidden": "true",
                    span { "data-part": "label", "Cohort" }
                    span { "data-part": "label", "data-size": "", "Size" }
                }
                div { "data-part": "col-labels", "aria-hidden": "true",
                    for p in cols_v.iter() {
                        span { "data-part": "label", title: "{p}", "{p}" }
                    }
                }
                div { "data-part": "row-labels", "aria-hidden": "true",
                    for c in all.iter() {
                        span { "data-part": "row-label", title: "{c.name}",
                            span { "data-part": "label", "{c.name}" }
                            span { "data-part": "size", {fmt(c.size)} }
                        }
                    }
                }
                div {
                    "data-part": "cells",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to move between cells.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onkeydown: move |e| {
                        if let Some(next) = cohort::key(&layout.read(), &e.key().to_string(), active()) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    for cell in l.cells.iter().copied() {
                        span {
                            key: "{cell.row}-{cell.col}",
                            "data-part": "cell",
                            "data-strong": (cell.level > 0.55).then_some(""),
                            "data-active": (current == Some((cell.row, cell.col))).then_some(""),
                            style: format!("grid-row: {}; grid-column: {}; --level: {:.3}; --i: {};", cell.row + 1, cell.col + 1, cell.level, cell.row + cell.col),
                            onpointerenter: move |_| active.set(Some((cell.row, cell.col))),
                            onpointerdown: move |_| active.set(Some((cell.row, cell.col))),
                            {percent(cell.rate)}
                        }
                    }
                    if let Some((title, value, side, edge, style)) = tip {
                        div { "data-part": "tooltip", "data-side": side, "data-edge": edge, style, "aria-hidden": "true",
                            div { "data-part": "tooltip-label", "{title}" }
                            div { "data-part": "tooltip-value", "{value}" }
                        }
                    }
                }
            }
            div { "data-part": "scale", "aria-hidden": "true",
                span { "data-part": "scale-value", "0%" }
                span { "data-part": "ramp" }
                span { "data-part": "scale-value", "{top}+" }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Cohort" }
                        th { scope: "col", "Size" }
                        for p in cols_v.iter() {
                            th { scope: "col", "{p}" }
                        }
                    }
                }
                tbody {
                    for (r , c) in all.iter().enumerate() {
                        tr {
                            th { scope: "row", "{c.name}" }
                            td { {fmt(c.size)} }
                            for col in 0..l.cols {
                                td { {l.at(r, col).map_or_else(|| "–".to_string(), |x| percent(x.rate))} }
                            }
                        }
                    }
                }
            }
        }
    }
}
