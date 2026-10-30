//! `UpsetChart`: how sets overlap, for more sets than a Venn diagram can hold. Bars on
//! top count the members of each exclusive intersection, the dot matrix below shows which
//! sets it combines, and bars on the left give each set's size. Layout comes from
//! `hydroxy_charts::upset`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::{format_value, nav, upset};

use crate::callbacks::{select_active, use_hover};
use crate::sets_kit::SetsTip;
use crate::sparkline::use_datum;

pub use hydroxy_charts::upset::Intersection;

/// An `UpSet` plot. Intersections sort largest first, sets by size. Hover or touch a column
/// to light its sets, or a set to light every intersection that includes it; focus the
/// chart and use the arrow keys to step through the columns and then the sets, Home and
/// End to jump, Escape to clear.
#[component]
pub fn UpsetChart(
    /// Members per exclusive intersection: each entry counts the members in exactly those
    /// sets. Sets are created from the names.
    data: ReadSignal<Vec<Intersection>>,
    /// What the chart shows, such as "Notification channels customers use". Used as the accessible name.
    label: String,
    /// Drawing width in px; the chart scales down to fit its container. The height follows the set count.
    #[props(default = 720)]
    width: u16,
    /// Most intersections to draw; smaller ones are left out of the plot but stay in the table.
    #[props(default = 16)]
    max_columns: usize,
    /// Formats counts for labels, the tooltip and the table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Called when the hovered or focused column or set changes; `None` when it clears.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a column or set.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.call(v));
    let d = data();
    let lay = upset::layout(&d, f64::from(width), max_columns);
    let (names, mut combos) = upset::index(&d);
    let total: f64 = combos.iter().map(|c| c.1).sum();
    combos.sort_by(|a, b| b.1.total_cmp(&a.1));
    // Marks are the columns first, then the sets.
    let mut active = use_signal(|| None::<usize>);
    let n = lay.columns.len();
    let marks = n + lay.rows.len();
    let current = active().filter(|&m| m < marks);
    // Columns are series 0 (value: members); sets are series 1 (value: set size).
    let datum = use_datum(current.and_then(|m| {
        if let Some(col) = lay.columns.get(m) {
            let names: Vec<&str> = col.sets.iter().filter_map(|&r| lay.rows.get(r)).map(|r| r.name.as_str()).collect();
            Some(EventDatum {
                series: 0,
                index: m,
                label: Some(upset::combo_name(&names)),
                value: col.count,
                node: None,
            })
        } else {
            let r = m - n;
            let row = lay.rows.get(r)?;
            Some(EventDatum { series: 1, index: r, label: Some(row.name.clone()), value: row.size, node: None })
        }
    }));
    use_hover(datum, on_hover);
    let lit = current.map(|m| lay.lit(m));
    let col_lit = |k: usize| lit.as_ref().map(|(c, _)| c.get(k).copied().unwrap_or(false));
    let row_lit = |k: usize| lit.as_ref().map(|(_, r)| r.get(k).copied().unwrap_or(false));
    let flag = |b: Option<bool>, want: bool| (b == Some(want)).then_some("");
    let tip = current.and_then(|m| lay.tip(m, total, &fmt));
    let announcement = tip.as_ref().map(hydroxy_charts::plot::Tip::announce).unwrap_or_default();
    let half = lay.column_width / 2.0;
    let bh = (lay.row_height * 0.5).min(12.0);
    let height = lay.height;
    let hidden = lay.hidden;

    rsx! {
        figure {
            "data-scope": "upset-chart",
            "data-part": "root",
            "data-active": current.map(|_| ""),
            style: "max-width: {width}px;",
            div { "data-part": "plot",
                svg {
                    "data-part": "marks",
                    view_box: "0 0 {width} {height:.0}",
                    tabindex: "0",
                    role: "group",
                    "aria-label": "{label}. Use the arrow keys to step through the intersections and then the sets.",
                    onpointerleave: move |_| active.set(None),
                    onblur: move |_| active.set(None),
                    onclick: move |_| select_active(datum, on_select),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            select_active(datum, on_select);
                        }
                        if let Some(next) = nav::list_key(&e.key().to_string(), active(), marks) {
                            e.prevent_default();
                            active.set(next);
                        }
                    },
                    g { "data-part": "stripes",
                        for (r , row) in lay.rows.iter().enumerate().filter(|(r, _)| r % 2 == 0) {
                            rect {
                                key: "{r}",
                                "data-part": "stripe",
                                x: "{lay.label_x}",
                                y: "{row.y - lay.row_height / 2.0}",
                                width: "{lay.width - lay.label_x}",
                                height: "{lay.row_height}",
                                rx: "3",
                            }
                        }
                    }
                    g { "data-part": "grid",
                        for &(v , y) in lay.ticks.iter() {
                            line {
                                "data-part": "gridline",
                                "data-baseline": (v <= 0.0).then_some(""),
                                x1: "{lay.matrix_x}",
                                x2: "{lay.width}",
                                y1: "{y}",
                                y2: "{y}",
                            }
                            text {
                                "data-part": "tick",
                                x: "{lay.matrix_x - 6.0}",
                                y: "{y}",
                                text_anchor: "end",
                                dominant_baseline: "middle",
                                {fmt(v)}
                            }
                        }
                    }
                    text {
                        "data-part": "axis-title",
                        x: "{lay.set_right}",
                        y: "{lay.baseline - 4.0}",
                        text_anchor: "end",
                        "Set size"
                    }
                    g { "data-part": "sets",
                        for (r , row) in lay.rows.iter().enumerate() {
                            g {
                                key: "{r}",
                                "data-part": "set",
                                "data-highlighted": flag(row_lit(r), true),
                                "data-dim": flag(row_lit(r), false),
                                "data-selected": (current == Some(n + r)).then_some(""),
                                style: "--i: {r}; --ox-n: {lay.rows.len()};",
                                onpointerenter: move |_| active.set(Some(n + r)),
                                onpointerdown: move |_| active.set(Some(n + r)),
                                rect {
                                    "data-part": "band",
                                    x: "0",
                                    y: "{row.y - lay.row_height / 2.0}",
                                    width: "{lay.matrix_x}",
                                    height: "{lay.row_height}",
                                }
                                path {
                                    "data-part": "set-bar",
                                    d: upset::bar_path(row.bar_x, row.y - bh / 2.0, lay.set_right - row.bar_x, bh, 3.0, true),
                                }
                                text {
                                    "data-part": "set-value",
                                    x: "{row.bar_x - 6.0}",
                                    y: "{row.y}",
                                    text_anchor: "end",
                                    dominant_baseline: "middle",
                                    {fmt(row.size)}
                                }
                                text {
                                    "data-part": "set-name",
                                    x: "{lay.label_x + 8.0}",
                                    y: "{row.y}",
                                    dominant_baseline: "middle",
                                    "{row.name}"
                                }
                            }
                        }
                    }
                    g { "data-part": "columns",
                        for (c , col) in lay.columns.iter().enumerate() {
                            g {
                                key: "{c}",
                                "data-part": "column",
                                "data-highlighted": flag(col_lit(c), true),
                                "data-dim": flag(col_lit(c), false),
                                "data-selected": (current == Some(c)).then_some(""),
                                style: "--i: {c}; --ox-n: {lay.columns.len()};",
                                onpointerenter: move |_| active.set(Some(c)),
                                onpointerdown: move |_| active.set(Some(c)),
                                rect {
                                    "data-part": "band",
                                    x: "{col.x - half}",
                                    y: "0",
                                    width: "{lay.column_width}",
                                    height: "{lay.height}",
                                    rx: "4",
                                }
                                path {
                                    "data-part": "bar",
                                    d: upset::bar_path(col.x - lay.bar_width / 2.0, col.bar_y, lay.bar_width, lay.baseline - col.bar_y, 3.0, false),
                                }
                                if lay.column_width >= 20.0 {
                                    text {
                                        "data-part": "bar-value",
                                        x: "{col.x}",
                                        y: "{col.bar_y - 5.0}",
                                        text_anchor: "middle",
                                        {fmt(col.count)}
                                    }
                                }
                                if col.sets.len() > 1 {
                                    line {
                                        "data-part": "connector",
                                        x1: "{col.x}",
                                        x2: "{col.x}",
                                        y1: "{lay.rows[col.sets[0]].y}",
                                        y2: "{lay.rows[col.sets[col.sets.len() - 1]].y}",
                                    }
                                }
                                g { "data-part": "dots",
                                    for (r , row) in lay.rows.iter().enumerate() {
                                        circle {
                                            "data-part": "dot",
                                            "data-member": col.sets.contains(&r).then_some(""),
                                            cx: "{col.x}",
                                            cy: "{row.y}",
                                            r: "{lay.dot}",
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(tip) = tip {
                    SetsTip { tip }
                }
            }
            if hidden > 0 {
                p { "data-part": "note",
                    if hidden == 1 {
                        "1 smaller intersection not shown"
                    } else {
                        "{hidden} smaller intersections not shown"
                    }
                }
            }
            span { "data-part": "sr-live", "aria-live": "polite", "{announcement}" }
            table { "data-part": "sr-table",
                caption { "{label}" }
                thead {
                    tr {
                        th { scope: "col", "Intersection" }
                        th { scope: "col", "Members" }
                    }
                }
                tbody {
                    for (sets , count) in combos.iter() {
                        tr {
                            th { scope: "row",
                                {upset::combo_name(&sets.iter().map(|&s| names[s].as_str()).collect::<Vec<_>>())}
                            }
                            td { {fmt(*count)} }
                        }
                    }
                }
            }
        }
    }
}
