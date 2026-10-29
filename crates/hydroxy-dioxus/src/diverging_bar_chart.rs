//! `DivergingBarChart`: survey (Likert) answers as stacked bars split around a centre
//! line. Geometry comes from `hydroxy_charts::diverging`.

use dioxus::prelude::*;
use hydroxy_charts::diverging;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::grid_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow};

pub use hydroxy_charts::line::Series;

use crate::plot_kit::{Plot, pct};

fn percent(share: f64) -> String {
    format!("{:.0}%", share * 100.0)
}

/// One row per question, its answers stacked outwards from a centre line: levels below the
/// middle run left, levels above it run right, and with an odd number of levels the
/// neutral middle one straddles the centre. Values are counts (or shares) and each row is
/// shown as shares of its own total, on a percentage axis that reads outwards both ways.
/// Totals either side sit at the bar ends. Hover or touch a segment to read the row; focus
/// the plot and use Up/Down to move between rows, Left/Right between answers, Escape to clear.
#[allow(clippy::cast_precision_loss, reason = "row counts are small")]
#[component]
pub fn DivergingBarChart(
    /// One label per row (question).
    rows: ReadSignal<Vec<String>>,
    /// One series per answer level, from most negative to most positive, each with a count
    /// per row. Two to seven levels read well; an odd count makes the middle one neutral.
    levels: ReadSignal<Vec<Series>>,
    /// What the chart shows, such as "Staff survey answers". Used as the accessible name.
    label: String,
    /// Plot height in px. Default: 40px per row.
    height: Option<u16>,
    /// Called when the pointer or keyboard moves to a segment, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a segment.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let names = rows();
    let levels = levels();
    let (n, levels_n) = (names.len(), levels.len());
    let layout = diverging::layout(&levels, n);
    let height = height.unwrap_or_else(|| u16::try_from(n * 40).unwrap_or(u16::MAX).max(120));
    let widths: Vec<usize> = layout.rows.iter().map(|r| r.segments.len()).collect();
    // (row, segment), matching nav::grid_key's (row, column).
    let mut active = use_signal(|| None::<(usize, usize)>);
    let current = active().filter(|&(r, s)| r < n && s < widths[r]);
    let now = current.and_then(|(r, s)| {
        let level = layout.rows.get(r)?.segments.get(s)?.level;
        EventDatum::from_series(&levels, level, r, names.get(r).map(String::as_str))
    });
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    let share_of =
        |row: &diverging::DivergingRow, k: usize| row.segments.iter().find(|g| g.level == k).map_or(0.0, |g| g.share);
    let tip = current.map(|(r, s)| {
        let row = &layout.rows[r];
        let seg = &row.segments[s];
        let rows = levels
            .iter()
            .enumerate()
            .map(|(k, x)| TipRow {
                active: k == seg.level,
                ..TipRow::new(Some(diverging::level_colour(k, levels_n)), x.name.clone(), percent(share_of(row, k)))
            })
            .collect();
        Tip { x: f64::midpoint(seg.a, seg.b), y: 1.0 - plot::band_centre(r, n), title: names[r].clone(), rows }
    });
    let legend: Vec<Key> = levels
        .iter()
        .enumerate()
        .map(|(k, s)| Key::new(s.name.clone(), diverging::level_colour(k, levels_n)))
        .collect();
    let head: Vec<String> = std::iter::once(String::new()).chain(levels.iter().map(|s| s.name.clone())).collect();
    let table: Vec<Vec<String>> = names
        .iter()
        .zip(&layout.rows)
        .map(|(name, row)| {
            std::iter::once(name.clone()).chain((0..levels_n).map(|k| percent(share_of(row, k)))).collect()
        })
        .collect();
    let w = 0.62 / n.max(1) as f64;
    let centre = layout.axis.at(0.0);
    let rows_for_pointer = layout.rows.clone();
    let widths_for_key = widths;

    rsx! {
        Plot {
            scope: "diverging-bar-chart",
            label,
            hint: "Use the up and down arrow keys to move between rows, left and right between answers.",
            height,
            horizontal: true,
            ticks: layout.axis.ticks(&|v| percent(v.abs())),
            labels: plot::category_ticks(&names, usize::MAX),
            annotations: Vec::new(),
            legend,
            tip,
            datum: Some(datum),
            on_hover,
            on_select,
            head,
            rows: table,
            on_key: move |key: String| {
                let now = active();
                let width = now.map_or(levels_n, |(r, _)| widths_for_key.get(r).copied().unwrap_or(0));
                grid_key(&key, now, n, width.max(1))
                    .map(|next| {
                        // Rows can have fewer segments than levels; keep the column inside the new row.
                        active.set(next.map(|(r, s)| (r, s.min(widths_for_key.get(r).copied().unwrap_or(1).saturating_sub(1)))));
                    })
                    .is_some()
            },
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, y)| {
                    let r = plot::band_at(1.0 - y, n)?;
                    diverging::segment_at(rows_for_pointer.get(r)?, x).map(|s| (r, s))
                });
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                div { "data-part": "bars", "data-neutral": diverging::neutral(levels_n).map(|_| ""),
                    span { "data-part": "centre", style: "--p: {pct(centre)};" }
                    for (r , row) in layout.rows.iter().enumerate() {
                        for (s , g) in row.segments.iter().enumerate() {
                            span {
                                "data-part": "bar",
                                "data-index": "{g.level}",
                                "data-edge": (!g.edge.is_empty()).then_some(g.edge),
                                "data-active": (current == Some((r, s))).then_some(""),
                                style: "--c: {plot::band_centre(r, n):.4}; --w: {w:.4}; --a: {g.a:.4}; --b: {g.b:.4}; --o: {g.origin * 100.0:.2}%; --i: {r}; background: {diverging::level_colour(g.level, levels_n)};",
                            }
                        }
                        if !row.segments.is_empty() {
                            span {
                                "data-part": "total",
                                "data-side": "start",
                                style: "--c: {plot::band_centre(r, n):.4}; --v: {row.start:.4}; --i: {r};",
                                {percent(row.negative)}
                            }
                            span {
                                "data-part": "total",
                                "data-side": "end",
                                style: "--c: {plot::band_centre(r, n):.4}; --v: {row.end:.4}; --i: {r};",
                                {percent(row.positive)}
                            }
                        }
                    }
                }
            },
        }
    }
}
