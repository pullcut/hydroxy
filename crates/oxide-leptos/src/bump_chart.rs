//! `<BumpChart>`: how each series ranks at every step. Geometry comes from
//! `oxide_charts::rank`. Every line is focusable; hovering or focusing one picks it out.

use leptos::prelude::*;
use oxide_charts::{format_value, rank};

pub use oxide_charts::line::Series;

fn colour(i: usize) -> String {
    format!("--c: var(--chart-{});", i % 6 + 1)
}

/// Rank over time. Values decide the order at each column; the best rank is on top.
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
    /// Formats values in the tooltips and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| series.with(|s| rank::bump(&s.iter().map(|s| s.values.clone()).collect::<Vec<_>>(), higher_is_better)));
    let value_at = move |s: &Series, i: usize| s.values.get(i).copied().filter(|v| v.is_finite()).map_or_else(|| "–".to_string(), fmt);
    let aria = label.clone();
    let describe = move |si: usize, ci: usize| {
        series.with(|s| {
            let l = layout.with(|l| l.lines[si].ranks[ci]);
            format!("{}, {}: rank {} ({})", s[si].name, labels.with(|l| l.get(ci).cloned().unwrap_or_default()), l + 1, value_at(&s[si], ci))
        })
    };

    view! {
        <figure data-scope="bump-chart" data-part="root">
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
                        let summary = (0..line.ranks.len()).map(|ci| describe(si, ci)).collect::<Vec<_>>().join("; ");
                        let dots = line
                            .points
                            .iter()
                            .enumerate()
                            .map(|(ci, &(x, y))| view! { <circle data-part="point" cx=x cy=y r="5"><title>{describe(si, ci)}</title></circle> })
                            .collect_view();
                        view! {
                            <g data-part="series" data-index=si style=colour(si) tabindex="0" role="img" aria-label=summary>
                                <path data-part="line" d=line.path.clone()></path>
                                {dots}
                                <text data-part="name" x=l.edges.0 - 14.0 y=first.1 text-anchor="end">{names[si].clone()}</text>
                                <text data-part="name" x=l.edges.1 + 14.0 y=last.1 text-anchor="start">{names[si].clone()}</text>
                            </g>
                        }
                    })
                    .collect_view();
                view! {
                    <svg data-part="marks" viewBox=format!("0 0 {} {}", l.width, l.height) role="group" aria-label=aria.clone()>
                        {heads}
                        {lines}
                    </svg>
                }
            }}
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <td></td>
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
                                        let rank = layout.with(|l| l.lines[si].ranks[ci]) + 1;
                                        view! { <td>{format!("{rank} ({})", value_at(&s, ci))}</td> }
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
