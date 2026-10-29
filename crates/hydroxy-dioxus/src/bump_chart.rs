//! `BumpChart`: how each series ranks at every step. Geometry comes from
//! `hydroxy_charts::rank`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, nav, rank};

use crate::plot_kit::{ChartTip, colour};
use crate::rank_kit::{Dim, SeriesLegend, series_style};

pub use hydroxy_charts::line::Series;

/// Rank over time. Values decide the order at each column; the best rank is on top.
/// Hover or touch a marker to read it; focus the chart and use the arrow keys
/// (Left/Right along a line, Up/Down between lines), Home and End to jump, Escape to clear.
/// Hovering a legend item picks its series out.
#[component]
pub fn BumpChart(
    /// The series, drawn in categorical colour order. Up to six read well.
    series: ReadSignal<Vec<Series>>,
    /// One label per column, such as a season or month.
    labels: ReadSignal<Vec<String>>,
    /// What the chart shows. Used as the accessible name.
    label: String,
    /// Rank the highest value first (default). Turn off for measures like lap time.
    #[props(default = true)]
    higher_is_better: bool,
    /// Formats values in the tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a mark, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a mark.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let items = series();
    let heads = labels();
    let l = rank::bump(&items.iter().map(|s| s.values.clone()).collect::<Vec<_>>(), higher_is_better);
    let value_at = move |s: &Series, i: usize| {
        s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt)
    };
    let view_box = format!("0 0 {} {}", l.width, l.height);
    let (left, right) = (l.edges.0 - 14.0, l.edges.1 + 14.0);
    let (rows, cols) = (l.lines.len(), l.columns.len());
    let mut active = use_signal(|| None::<(usize, usize)>);
    let dim = Dim::new(active);
    let current = active().filter(|&(s, c)| s < rows && c < cols);
    let column = |ci: usize| heads.get(ci).cloned().unwrap_or_default();
    let cur = if on_hover.is_some() || on_select.is_some() {
        current.and_then(|(si, ci)| EventDatum::from_series(&items, si, ci, heads.get(ci).map(String::as_str)))
    } else {
        None
    };
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&cur,), move |(cur,)| cur)).into();
    crate::callbacks::use_hover(Some(datum), on_hover);
    let announcement = current.map(|(si, ci)| {
        format!("{}, {}: rank {} ({})", items[si].name, column(ci), l.lines[si].ranks[ci] + 1, value_at(&items[si], ci))
    });
    let tip = current.map(|(si, ci)| {
        let (x, y) = l.lines[si].points[ci];
        let (fx, fy) = (x / l.width, y / l.height);
        let rows = vec![
            TipRow::new(None, column(ci), format!("#{}", l.lines[si].ranks[ci] + 1)),
            TipRow::new(None, "Value", value_at(&items[si], ci)),
        ];
        let tip = Tip { x: fx, y: 1.0 - fy, title: items[si].name.clone(), rows };
        (tip, colour(si), format!("left: {:.3}%; top: {:.3}%;", fx * 100.0, fy * 100.0))
    });

    rsx! {
        figure { "data-scope": "bump-chart", "data-part": "root",
            div {
                "data-part": "plot",
                tabindex: "0",
                role: "group",
                "aria-label": "{label}. Use the arrow keys to step through the ranks.",
                onpointerleave: move |_| active.set(None),
                onblur: move |_| active.set(None),
                onkeydown: move |e| {
                    if e.key() == dioxus::html::Key::Enter {
                        crate::callbacks::select_active(Some(datum), on_select);
                    }
                    if let Some(next) = nav::grid_key(&e.key().to_string(), active(), rows, cols) {
                        e.prevent_default();
                        active.set(next);
                    }
                },
                svg { "data-part": "marks", view_box: "{view_box}", "aria-hidden": "true",
                    for (t , x) in heads.iter().zip(l.columns.iter()) {
                        text { "data-part": "column-label", x: "{x}", y: "14", "text-anchor": "middle", "{t}" }
                    }
                    for (si , line) in l.lines.iter().enumerate() {
                        g {
                            key: "{si}",
                            "data-part": "series",
                            "data-index": "{si}",
                            "data-dim": dim.of(si),
                            style: series_style(si),
                            path { "data-part": "line", d: "{line.path}" }
                            for (ci , (x , y)) in line.points.iter().enumerate() {
                                circle { "data-part": "point", "data-active": (current == Some((si, ci))).then_some(""), cx: "{x}", cy: "{y}", r: "5" }
                                circle {
                                    "data-part": "hit",
                                    cx: "{x}",
                                    cy: "{y}",
                                    r: "16",
                                    onpointerenter: move |_| active.set(Some((si, ci))),
                                    onpointerdown: move |_| active.set(Some((si, ci))),
                                    onclick: move |_| {
                                        active.set(Some((si, ci)));
                                        crate::callbacks::select_active(Some(datum), on_select);
                                    },
                                }
                            }
                            if let (Some(first), Some(last)) = (line.points.first(), line.points.last()) {
                                text { "data-part": "name", x: "{left}", y: "{first.1}", "text-anchor": "end", "{items[si].name}" }
                                text { "data-part": "name", x: "{right}", y: "{last.1}", "text-anchor": "start", "{items[si].name}" }
                            }
                        }
                    }
                }
                if let Some((tip, swatch, style)) = tip {
                    ChartTip { tip, swatch, anchored: true, style }
                }
            }
            SeriesLegend { series: items.clone(), dim }
            span { "data-part": "sr-live", "aria-live": "polite", {announcement.unwrap_or_default()} }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Series" }
                        for t in heads.iter() {
                            th { scope: "col", "{t}" }
                        }
                    }
                }
                tbody {
                    for (si , s) in items.iter().enumerate() {
                        tr {
                            th { scope: "row", "{s.name}" }
                            for ci in 0..heads.len() {
                                td {
                                    {
                                        l.lines[si]
                                            .ranks
                                            .get(ci)
                                            .map_or_else(|| "–".to_string(), |r| format!("{} ({})", r + 1, value_at(s, ci)))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
