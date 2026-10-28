//! `<BoxPlot>`: one box per group, with quartiles, whiskers at 1.5 IQR and outliers.
//! Statistics and heights come from `hydroxy_charts::boxplot`; layout is plain CSS.

use hydroxy_charts::nav::{self, HINT_GROUPS, hinted};
use hydroxy_charts::{boxplot, format_value};
use leptos::either::Either;
use leptos::prelude::*;

use crate::plot_kit::pct;

/// Where the median sits down an element spanning `bottom..top`, as a CSS length from its top,
/// so the box opens out from the median.
fn mid(top: f64, bottom: f64, at: f64) -> String {
    let span = top - bottom;
    format!("{:.2}%", if span > 0.0 { ((top - at) / span * 100.0).clamp(0.0, 100.0) } else { 50.0 })
}

pub use hydroxy_charts::boxplot::Group;

/// A box plot of several groups on one shared value axis. Hover or touch a box to read
/// its numbers; focus the plot and use the arrow keys to step through groups, Home and
/// End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn BoxPlot(
    /// The groups, left to right, each with its raw values.
    #[prop(into)]
    groups: Signal<Vec<Group>>,
    /// What the chart shows, such as "Deploy time by team". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the group labels.
    #[prop(default = 240)]
    height: u16,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let layout = Memo::new(move |_| {
        groups.with(|g| boxplot::layout(&g.iter().map(|g| g.values.clone()).collect::<Vec<_>>(), 5))
    });
    let summary = move |name: &str, s: &boxplot::Stats| {
        format!(
            "{name}: median {}, quartiles {} to {}, whiskers {} to {}, {} outliers, {} values",
            fmt(s.median),
            fmt(s.q1),
            fmt(s.q3),
            fmt(s.low),
            fmt(s.high),
            s.outliers.len(),
            s.n
        )
    };

    let active = RwSignal::new(None::<usize>);
    let describe = move |i: usize| {
        let name = groups.with(|g| g.get(i).map(|g| g.label.clone()))?;
        Some(layout.with(|l| match l.boxes.get(i) {
            Some(Some(b)) => summary(&name, &b.stats),
            _ => format!("{name}: no data"),
        }))
    };
    let group_label = hinted(&label, HINT_GROUPS);
    let caption = label;

    view! {
        <figure data-scope="box-plot" data-part="root">
            <div data-part="plot" style=format!("height: {height}px;")>
                <div data-part="grid" aria-hidden="true">
                    {move || {
                        layout
                            .get()
                            .gridlines
                            .into_iter()
                            .map(|g| {
                                view! {
                                    <div data-part="gridline" data-baseline=(g.value == 0.0).then_some("") style=format!("bottom: {};", pct(g.fraction))>
                                        <span data-part="tick">{fmt(g.value)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <div
                    data-part="columns"
                    role="group"
                    tabindex="0"
                    aria-label=group_label
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if let Some(next) = nav::list_key(&ev.key(), active.get_untracked(), groups.with(Vec::len)) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    {move || {
                        let l = layout.get();
                        let names = groups.get();
                        let n = names.len();
                        names
                            .into_iter()
                            .zip(l.boxes)
                            .enumerate()
                            .map(|(i, (g, geom))| {
                                let side = if i * 3 < n { "start" } else if i * 3 >= n * 2 { "end" } else { "" };
                                let name = geom.as_ref().map_or_else(|| format!("{}: no data", g.label), |b| summary(&g.label, &b.stats));
                                let body = geom.map(|b| {
                                    let outliers = b
                                        .outliers
                                        .iter()
                                        .map(|&o| view! { <span data-part="outlier" style=format!("bottom: {};", pct(o))></span> })
                                        .collect_view();
                                    let s = b.stats.clone();
                                    view! {
                                        <div data-part="whisker" style=format!("bottom: {}; height: {}; --ox-mid: {};", pct(b.low), pct(b.high - b.low), mid(b.high, b.low, b.median))></div>
                                        <div data-part="cap" style=format!("bottom: {};", pct(b.low))></div>
                                        <div data-part="cap" style=format!("bottom: {};", pct(b.high))></div>
                                        <div data-part="box" style=format!("bottom: {}; height: {}; --ox-mid: {};", pct(b.q1), pct(b.q3 - b.q1), mid(b.q3, b.q1, b.median))></div>
                                        <div data-part="median" style=format!("bottom: {};", pct(b.median))></div>
                                        {outliers}
                                        <div data-part="tooltip" data-side=side aria-hidden="true">
                                            <div data-part="tooltip-label">{g.label.clone()}</div>
                                            <dl>
                                                <dt>"Upper whisker"</dt><dd>{fmt(s.high)}</dd>
                                                <dt>"Q3"</dt><dd>{fmt(s.q3)}</dd>
                                                <dt>"Median"</dt><dd>{fmt(s.median)}</dd>
                                                <dt>"Q1"</dt><dd>{fmt(s.q1)}</dd>
                                                <dt>"Lower whisker"</dt><dd>{fmt(s.low)}</dd>
                                                <dt>"Outliers"</dt><dd>{s.outliers.len()}</dd>
                                            </dl>
                                        </div>
                                    }
                                });
                                view! {
                                    <div
                                        data-part="column"
                                        role="img"
                                        aria-label=name
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        style=format!("--i: {i};")
                                        on:pointerenter=move |_| active.set(Some(i))
                                        on:pointerdown=move |_| active.set(Some(i))
                                    >
                                        {body}
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </div>
            <div data-part="labels" aria-hidden="true">
                {move || {
                    let names = groups.get();
                    let n = names.len().max(1);
                    names
                        .into_iter()
                        .enumerate()
                        .map(|(i, g)| {
                            #[allow(clippy::cast_precision_loss, reason = "group counts are small")]
                            let at = (i as f64 + 0.5) / n as f64;
                            view! { <span data-part="label" data-slot="" style=format!("left: {};", pct(at))>{g.label}</span> }
                        })
                        .collect_view()
                }}
            </div>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Group"</th>
                        <th scope="col">"Count"</th>
                        <th scope="col">"Lower whisker"</th>
                        <th scope="col">"Q1"</th>
                        <th scope="col">"Median"</th>
                        <th scope="col">"Q3"</th>
                        <th scope="col">"Upper whisker"</th>
                        <th scope="col">"Outliers"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        layout
                            .get()
                            .boxes
                            .into_iter()
                            .zip(groups.get())
                            .map(|(b, g)| match b {
                                Some(b) => {
                                    let s = b.stats;
                                    let outliers = s.outliers.iter().map(|&o| fmt(o)).collect::<Vec<_>>().join(", ");
                                    Either::Left(view! {
                                        <tr>
                                            <th scope="row">{g.label}</th>
                                            <td>{s.n}</td>
                                            <td>{fmt(s.low)}</td>
                                            <td>{fmt(s.q1)}</td>
                                            <td>{fmt(s.median)}</td>
                                            <td>{fmt(s.q3)}</td>
                                            <td>{fmt(s.high)}</td>
                                            <td>{outliers}</td>
                                        </tr>
                                    })
                                }
                                None => Either::Right(view! { <tr><th scope="row">{g.label}</th><td colspan="7">"No data"</td></tr> }),
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
