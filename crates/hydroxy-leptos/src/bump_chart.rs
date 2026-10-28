//! `<BumpChart>`: how each series ranks at every step. Geometry comes from
//! `hydroxy_charts::rank`.

use hydroxy_charts::nav::{self, hinted};
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{format_value, rank};
use leptos::prelude::*;

use crate::plot_kit::{ChartTip, colour};
use crate::rank_kit::{Dim, SeriesLegend, series_style};

pub use hydroxy_charts::line::Series;

/// Rank over time. Values decide the order at each column; the best rank is on top.
/// Hover or touch a marker to read it; focus the chart and use the arrow keys
/// (Left/Right along a line, Up/Down between lines), Home and End to jump, Escape to clear.
/// Hovering a legend item picks its series out.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn BumpChart(
    /// The series, drawn in categorical colour order. Up to six read well.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// One label per column, such as a season or month.
    #[prop(into)]
    labels: Signal<Vec<String>>,
    /// What the chart shows. Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Rank the highest value first (default). Turn off for measures like lap time.
    #[prop(default = true)]
    higher_is_better: bool,
    /// Formats values in the tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| {
        series.with(|s| rank::bump(&s.iter().map(|s| s.values.clone()).collect::<Vec<_>>(), higher_is_better))
    });
    let value_at = move |s: &Series, i: usize| {
        s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt)
    };
    let active = RwSignal::new(None::<(usize, usize)>);
    let dim = Dim::new(active);
    let column = move |ci: usize| labels.with(|l| l.get(ci).cloned().unwrap_or_default());
    let describe = move |(si, ci): (usize, usize)| {
        let rank = layout.with(|l| l.lines.get(si).and_then(|line| line.ranks.get(ci).copied()))?;
        series
            .with(|s| s.get(si).map(|s| format!("{}, {}: rank {} ({})", s.name, column(ci), rank + 1, value_at(s, ci))))
    };
    let group_label = hinted(&label, "Use the arrow keys to step through the ranks.");

    view! {
        <figure data-scope="bump-chart" data-part="root">
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=group_label
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    let (rows, cols) = layout.with(|l| (l.lines.len(), l.columns.len()));
                    if let Some(next) = nav::grid_key(&ev.key(), active.get_untracked(), rows, cols) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                {move || {
                    let l = layout.get();
                    let names: Vec<String> = series.with(|s| s.iter().map(|s| s.name.clone()).collect());
                    let heads = labels
                        .get()
                        .into_iter()
                        .zip(l.columns.iter().copied())
                        .map(|(t, x)| view! { <text data-part="column-label" x=x y="14" text-anchor="middle">{t}</text> })
                        .collect_view();
                    let lines = l
                        .lines
                        .iter()
                        .enumerate()
                        .map(|(si, line)| {
                            let first = line.points.first().copied().unwrap_or_default();
                            let last = line.points.last().copied().unwrap_or_default();
                            let dots = line
                                .points
                                .iter()
                                .enumerate()
                                .map(|(ci, &(x, y))| {
                                    view! {
                                        <circle data-part="point" data-active=move || (active.get() == Some((si, ci))).then_some("") cx=x cy=y r="5"></circle>
                                        <circle
                                            data-part="hit"
                                            cx=x
                                            cy=y
                                            r="16"
                                            on:pointerenter=move |_| active.set(Some((si, ci)))
                                            on:pointerdown=move |_| active.set(Some((si, ci)))
                                        ></circle>
                                    }
                                })
                                .collect_view();
                            view! {
                                <g data-part="series" data-index=si data-dim=move || dim.of(si) style=series_style(si)>
                                    <path data-part="line" d=line.path.clone()></path>
                                    {dots}
                                    <text data-part="name" x=l.edges.0 - 14.0 y=first.1 text-anchor="end">{names[si].clone()}</text>
                                    <text data-part="name" x=l.edges.1 + 14.0 y=last.1 text-anchor="start">{names[si].clone()}</text>
                                </g>
                            }
                        })
                        .collect_view();
                    view! {
                        <svg data-part="marks" viewBox=format!("0 0 {} {}", l.width, l.height) aria-hidden="true">
                            {heads}
                            {lines}
                        </svg>
                    }
                }}
                {move || {
                    let (si, ci) = active.get()?;
                    let l = layout.get();
                    let line = l.lines.get(si)?;
                    let (x, y) = *line.points.get(ci)?;
                    let s = series.with(|s| s.get(si).cloned())?;
                    let tip = Tip {
                        x: x / l.width,
                        y: 1.0 - y / l.height,
                        title: s.name.clone(),
                        rows: vec![
                            TipRow::new(None, column(ci), format!("#{}", line.ranks[ci] + 1)),
                            TipRow::new(None, "Value", value_at(&s, ci)),
                        ],
                    };
                    Some(view! {
                        <ChartTip
                            tip
                            swatch=colour(si)
                            anchored=true
                            style=format!("left: {:.3}%; top: {:.3}%;", x / l.width * 100.0, y / l.height * 100.0)
                        />
                    })
                }}
            </div>
            <SeriesLegend series dim />
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Series"</th>
                        {move || labels.get().into_iter().map(|l| view! { <th scope="col">{l}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let n = labels.with(Vec::len);
                        series
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(si, s)| {
                                let cells = (0..n)
                                    .map(|ci| {
                                        let rank = layout.with(|l| l.lines.get(si).and_then(|line| line.ranks.get(ci).copied()));
                                        let text = rank.map_or_else(|| "–".to_string(), |r| format!("{} ({})", r + 1, value_at(&s, ci)));
                                        view! { <td>{text}</td> }
                                    })
                                    .collect_view();
                                view! { <tr><th scope="row">{s.name.clone()}</th>{cells}</tr> }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
