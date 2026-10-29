//! `<DivergingBarChart>`: survey (Likert) answers as stacked bars split around a centre
//! line. Geometry comes from `hydroxy_charts::diverging`.

use hydroxy_charts::diverging;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::grid_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow};
use leptos::prelude::*;

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
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::cast_precision_loss, reason = "row counts are small")]
#[component]
pub fn DivergingBarChart(
    /// One label per row (question).
    #[prop(into)]
    rows: Signal<Vec<String>>,
    /// One series per answer level, from most negative to most positive, each with a count
    /// per row. Two to seven levels read well; an odd count makes the middle one neutral.
    #[prop(into)]
    levels: Signal<Vec<Series>>,
    /// What the chart shows, such as "Staff survey answers". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px. Default: 40px per row.
    #[prop(optional)]
    height: Option<u16>,
    /// Called when the pointer or keyboard moves to a segment, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a segment.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let count = move || rows.with(Vec::len);
    let height =
        height.unwrap_or_else(|| u16::try_from(rows.with_untracked(Vec::len) * 40).unwrap_or(u16::MAX).max(120));
    let layout = Memo::new(move |_| levels.with(|l| diverging::layout(l, count())));
    let level_count = move || levels.with(Vec::len);
    // (row, segment), matching nav::grid_key's (row, column).
    let active = RwSignal::new(None::<(usize, usize)>);
    let datum = Signal::derive(move || {
        let (r, s) = active.get()?;
        let level = layout.with(|l| l.rows.get(r)?.segments.get(s).map(|g| g.level))?;
        rows.with(|n| levels.with(|all| EventDatum::from_series(all, level, r, n.get(r).map(String::as_str))))
    });
    let tip = Signal::derive(move || {
        let (r, s) = active.get()?;
        let title = rows.with(|n| n.get(r).cloned())?;
        let (row, levels_n) = (layout.with(|l| l.rows.get(r).cloned())?, level_count());
        let seg = row.segments.get(s)?;
        let rows = levels.with(|all| {
            all.iter()
                .enumerate()
                .map(|(k, x)| {
                    let share = row.segments.iter().find(|g| g.level == k).map_or(0.0, |g| g.share);
                    TipRow {
                        active: k == seg.level,
                        ..TipRow::new(Some(diverging::level_colour(k, levels_n)), x.name.clone(), percent(share))
                    }
                })
                .collect()
        });
        Some(Tip { x: f64::midpoint(seg.a, seg.b), y: 1.0 - plot::band_centre(r, count()), title, rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, y)| {
            let r = plot::band_at(1.0 - y, count())?;
            layout.with(|l| l.rows.get(r).and_then(|row| diverging::segment_at(row, x))).map(|s| (r, s))
        });
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        let current = active.get_untracked();
        let width =
            current.map_or(level_count(), |(r, _)| layout.with(|l| l.rows.get(r).map_or(0, |row| row.segments.len())));
        grid_key(&key, current, count(), width.max(1))
            .map(|next| {
                // Rows can have fewer segments than levels; keep the column inside the new row.
                let next = next.map(|(r, s)| {
                    (r, s.min(layout.with(|l| l.rows.get(r).map_or(0, |row| row.segments.len().saturating_sub(1)))))
                });
                active.set(next);
            })
            .is_some()
    });
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(&|v| percent(v.abs()))));
    let labels = Signal::derive(move || rows.with(|n| plot::category_ticks(n, usize::MAX)));
    let legend = Signal::derive(move || {
        levels.with(|l| {
            l.iter()
                .enumerate()
                .map(|(k, s)| Key::new(s.name.clone(), diverging::level_colour(k, l.len())))
                .collect::<Vec<_>>()
        })
    });
    let head = Signal::derive(move || {
        std::iter::once(String::new())
            .chain(levels.with(|l| l.iter().map(|s| s.name.clone()).collect::<Vec<_>>()))
            .collect()
    });
    let table =
        Signal::derive(move || {
            rows.with(|names| {
                layout.with(|l| {
                    names
                        .iter()
                        .zip(&l.rows)
                        .map(|(name, row)| {
                            std::iter::once(name.clone())
                                .chain((0..level_count()).map(|k| {
                                    percent(row.segments.iter().find(|g| g.level == k).map_or(0.0, |g| g.share))
                                }))
                                .collect()
                        })
                        .collect()
                })
            })
        });
    let marks = move || {
        let l = layout.get();
        let (n, levels_n) = (l.rows.len(), level_count());
        let has_neutral = diverging::neutral(levels_n).is_some();
        let w = 0.62 / n.max(1) as f64;
        let bars = l
            .rows
            .iter()
            .enumerate()
            .map(|(r, row)| {
                let c = plot::band_centre(r, n);
                let segments = row
                    .segments
                    .iter()
                    .enumerate()
                    .map(|(s, g)| {
                        let style = format!(
                            "--c: {c:.4}; --w: {w:.4}; --a: {:.4}; --b: {:.4}; --o: {:.2}%; --i: {r}; background: {};",
                            g.a,
                            g.b,
                            g.origin * 100.0,
                            diverging::level_colour(g.level, levels_n)
                        );
                        view! {
                            <span
                                data-part="bar"
                                data-index=g.level
                                data-edge=(!g.edge.is_empty()).then_some(g.edge)
                                data-active=move || (active.get() == Some((r, s))).then_some("")
                                style=style
                            ></span>
                        }
                    })
                    .collect_view();
                let totals = (!row.segments.is_empty()).then(|| {
                    let (neg, pos) = (percent(row.negative), percent(row.positive));
                    view! {
                        <span data-part="total" data-side="start" style=format!("--c: {c:.4}; --v: {:.4}; --i: {r};", row.start)>{neg}</span>
                        <span data-part="total" data-side="end" style=format!("--c: {c:.4}; --v: {:.4}; --i: {r};", row.end)>{pos}</span>
                    }
                });
                view! { {segments}{totals} }
            })
            .collect_view();
        view! {
            <div data-part="bars" data-neutral=has_neutral.then_some("")>
                <span data-part="centre" style=format!("--p: {};", pct(l.axis.at(0.0)))></span>
                {bars}
            </div>
        }
    };

    view! {
        <Plot
            scope="diverging-bar-chart"
            label
            hint="Use the up and down arrow keys to move between rows, left and right between answers."
            height
            horizontal=true
            ticks
            labels
            annotations=Signal::derive(Vec::new)
            legend
            tip
            head
            rows=table
            on_key
            on_point
            marks
            datum=Some(datum)
            on_hover
            on_select
        />
    }
}
