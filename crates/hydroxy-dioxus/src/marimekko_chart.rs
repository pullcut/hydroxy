//! `MarimekkoChart`: variable-width 100% stacks. Each column is as wide as its
//! category's share of the grand total; its segments split it by series. Geometry
//! comes from `hydroxy_charts::marimekko`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, marimekko, nav};

use crate::callbacks::{select_active, use_hover};
use crate::plot_kit::{ChartTip, colour_or_muted, pct};

pub use hydroxy_charts::line::Series;

fn percent(f: f64) -> String {
    format!("{}%", format_value(f * 100.0))
}

/// A Marimekko chart. Hover, touch or focus a segment to read it; focus the chart and
/// use Left and Right to move between categories, Up and Down between series, Home
/// and End to jump, Escape to clear.
#[component]
pub fn MarimekkoChart(
    /// The series, stacked from the bottom, each with one value per category.
    series: ReadSignal<Vec<Series>>,
    /// Category names, one per column, left to right.
    labels: ReadSignal<Vec<String>>,
    /// What the chart shows, such as "Market by region and segment". Used as the accessible name.
    label: String,
    /// Plot height in px.
    #[props(default = 280)]
    height: u16,
    /// Formats values for the column totals, tooltip and table.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a segment, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a segment.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let series_v = series();
    let labels_v = labels();
    let (n_series, n_cols) = (series_v.len(), labels_v.len());
    let columns = marimekko::layout(&series_v.iter().map(|s| s.values.clone()).collect::<Vec<_>>(), n_cols);
    let grand: f64 = columns.iter().map(|c| c.total).sum();
    let value = |s: usize, c: usize| series_v.get(s).and_then(|x| x.values.get(c).copied()).unwrap_or(0.0);
    // Grid rows run top to bottom, so row 0 is the last (top) series.
    let mut active = use_signal(|| None::<(usize, usize)>);
    let pick = active().filter(|&(r, c)| r < n_series && c < n_cols).map(|(r, c)| (n_series - r - 1, c));
    let now = pick.and_then(|(s, c)| EventDatum::from_series(&series_v, s, c, labels_v.get(c).map(String::as_str)));
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    use_hover(Some(datum), on_hover);
    let announcement = pick
        .map(|(s, c)| {
            let cat = &labels_v[c];
            format!(
                "{cat}, {}: {}, {} of {cat}",
                series_v[s].name,
                fmt(value(s, c)),
                percent(columns[c].segments[s].share)
            )
        })
        .unwrap_or_default();

    rsx! {
        figure { "data-scope": "marimekko-chart", "data-part": "root",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to move between segments.",
                style: "height: {height}px;",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onclick: move |_| select_active(Some(datum), on_select),
                onkeydown: move |e| {
                    if e.key() == dioxus::html::Key::Enter {
                        select_active(Some(datum), on_select);
                    }
                    if let Some(next) = nav::grid_key(&e.key().to_string(), active(), n_series, n_cols) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                div { "data-part": "grid", "aria-hidden": "true",
                    for f in [0.0, 0.25, 0.5, 0.75, 1.0] {
                        div { "data-part": "gridline", style: "bottom: {pct(f)};",
                            span { "data-part": "tick", {percent(f)} }
                        }
                    }
                }
                for (c , col) in columns.iter().enumerate() {
                    div { key: "{c}", "data-part": "column", style: "left: {pct(col.x0)}; width: {pct(col.x1 - col.x0)}; --i: {c};",
                        for (s , g) in col.segments.iter().enumerate() {
                            span {
                                "data-part": "segment",
                                "data-series": "{s}",
                                "data-active": (pick == Some((s, c))).then_some(""),
                                style: "bottom: {pct(g.y0)}; height: {pct(g.y1 - g.y0)}; background: {colour_or_muted(s, 6)};",
                                onpointerenter: move |_| active.set(Some((n_series - s - 1, c))),
                                onpointerdown: move |_| active.set(Some((n_series - s - 1, c))),
                            }
                        }
                    }
                }
                if let Some((s , c)) = pick {
                    {
                        let col = &columns[c];
                        let seg = col.segments[s];
                        let x = f64::midpoint(col.x0, col.x1);
                        let share_of_total = if grand > 0.0 { value(s, c).max(0.0) / grand } else { 0.0 };
                        let tip = Tip {
                            x,
                            y: f64::midpoint(seg.y0, seg.y1),
                            title: format!("{}, {}", labels_v[c], series_v[s].name),
                            rows: vec![
                                TipRow::new(None, "Value", fmt(value(s, c))),
                                TipRow::new(None, "Of column", percent(seg.share)),
                                TipRow::new(None, "Of total", percent(share_of_total)),
                            ],
                        };
                        rsx! {
                            ChartTip { tip, swatch: colour_or_muted(s, 6) }
                        }
                    }
                }
            }
            div { "data-part": "x-labels", "aria-hidden": "true",
                for (col , name) in columns.iter().zip(labels_v.iter()) {
                    span { "data-part": "x-label", style: "left: {pct(f64::midpoint(col.x0, col.x1))}; max-width: {pct(col.x1 - col.x0)};",
                        span { "data-part": "x-name", "{name}" }
                        span { "data-part": "x-total", {fmt(col.total)} }
                    }
                }
            }
            if n_series > 1 {
                ul { "data-part": "legend", "aria-hidden": "true",
                    for (i , s) in series_v.iter().enumerate() {
                        li { "data-part": "legend-item", "data-active": (pick.map(|p| p.0) == Some(i)).then_some(""),
                            span { "data-part": "swatch", style: "background: {colour_or_muted(i, 6)};" }
                            "{s.name}"
                        }
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Category" }
                        for s in series_v.iter() {
                            th { scope: "col", "{s.name}" }
                        }
                        th { scope: "col", "Total" }
                    }
                }
                tbody {
                    for (c , (name , col)) in labels_v.iter().zip(columns.iter()).enumerate() {
                        tr {
                            th { scope: "row", "{name}" }
                            for s in 0..n_series {
                                td { {fmt(value(s, c))} }
                            }
                            td { {fmt(col.total)} }
                        }
                    }
                }
            }
        }
    }
}
