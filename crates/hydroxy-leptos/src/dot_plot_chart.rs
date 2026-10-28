//! `<DotPlotChart>`: one row per category with a dot per series; two or more series draw
//! a dumbbell bar between them. Geometry comes from `hydroxy_charts::compare`.

use hydroxy_charts::compare;
use hydroxy_charts::nav::grid_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};
use leptos::prelude::*;

pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::line::Series;
pub use hydroxy_charts::scale::ScaleKind;

use crate::plot_kit::{Plot, formatter, placed};

/// Categories down the left, values along the bottom, a dot per series. With two or more
/// series a bar joins each row's lowest and highest dot (a dumbbell), so the gap reads at a
/// glance. Hover or touch a dot to read its row; focus the plot and use Up/Down to move
/// between rows, Left/Right between series, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn DotPlotChart(
    /// One label per row.
    #[prop(into)]
    rows: Signal<Vec<String>>,
    /// The series, one value per row, in categorical colour order. Past six, the smallest
    /// fold into "Other". NaN leaves the dot out.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// What the chart shows, such as "Median salary by role, 2024 and 2025". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px. Default: 36px per row.
    #[prop(optional)]
    height: Option<u16>,
    /// Linear (default), Log or Symlog value axis. The axis fits the data rather than starting at zero.
    #[prop(optional)]
    scale: ScaleKind,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is the value, y a row index).
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let names = rows;
    let series = Memo::new(move |_| series.with(|s| plot::fold_series(s, 6)));
    let count = move || names.with(Vec::len);
    let height =
        height.unwrap_or_else(|| u16::try_from(names.with_untracked(Vec::len) * 36).unwrap_or(u16::MAX).max(108));
    let layout = Memo::new(move |_| series.with(|s| annotations.with(|a| compare::dots(s, count(), scale, a))));
    // (row, series), matching nav::grid_key's (row, column).
    let active = RwSignal::new(None::<(usize, usize)>);
    let tip = Signal::derive(move || {
        let (r, s) = active.get()?;
        let title = names.with(|n| n.get(r).cloned())?;
        let x = layout.with(|l| {
            l.dots.get(r).and_then(|row| row.get(s).copied().flatten().or_else(|| l.spans[r].map(|sp| sp.1)))
        })?;
        let rows = series.with(|all| {
            all.iter()
                .enumerate()
                .map(|(i, x)| TipRow {
                    active: i == s,
                    ..TipRow::new(Some(colour(i)), x.name.clone(), plot::cell(x.values.get(r).copied(), fmt))
                })
                .collect()
        });
        Some(Tip { x, y: 1.0 - plot::band_centre(r, count()), title, rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, y)| {
            let r = plot::band_at(1.0 - y, count())?;
            layout.with(|l| compare::nearest_dot(&l.dots[r], x)).map(|s| (r, s))
        });
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        grid_key(&key, active.get_untracked(), count(), series.with(Vec::len)).map(|next| active.set(next)).is_some()
    });
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(fmt)));
    let labels = Signal::derive(move || names.with(|n| plot::category_ticks(n, usize::MAX)));
    let notes = placed(
        move |a| layout.with(|l| plot::place(a, true, |x| plot::category_at(x, count()), |v| l.axis.scale.map(v))),
        annotations,
    );
    let legend = Signal::derive(move || {
        series.with(|s| {
            if s.len() > 1 {
                s.iter().enumerate().map(|(i, s)| Key::new(s.name.clone(), colour(i))).collect()
            } else {
                Vec::new()
            }
        })
    });
    let head = Signal::derive(move || {
        std::iter::once(String::new())
            .chain(series.with(|s| s.iter().map(|s| s.name.clone()).collect::<Vec<_>>()))
            .collect()
    });
    let table = Signal::derive(move || {
        names.with(|n| {
            series.with(|s| {
                n.iter()
                    .enumerate()
                    .map(|(r, name)| {
                        std::iter::once(name.clone())
                            .chain(s.iter().map(|x| plot::cell(x.values.get(r).copied(), fmt)))
                            .collect()
                    })
                    .collect()
            })
        })
    });
    let marks = move || {
        let l = layout.get();
        let n = l.dots.len();
        let rows = l
            .dots
            .iter()
            .zip(&l.spans)
            .enumerate()
            .map(|(r, (row, span))| {
                let c = plot::band_centre(r, n);
                let stick = span.map(|(lo, hi)| {
                    let style = format!("--c: {c:.4}; --w: 0; --a: {lo:.4}; --b: {hi:.4}; --i: {r};");
                    view! { <span data-part="stick" style=style></span> }
                });
                let dots = row
                    .iter()
                    .enumerate()
                    .filter_map(|(s, v)| {
                        v.map(|v| {
                            let style = format!("--c: {c:.4}; --v: {v:.4}; --i: {r}; background: {};", colour(s));
                            view! { <span data-part="dot" data-index=s data-active=move || (active.get() == Some((r, s))).then_some("") style=style></span> }
                        })
                    })
                    .collect_view();
                view! { {stick}{dots} }
            })
            .collect_view();
        view! { <div data-part="bars">{rows}</div> }
    };

    view! {
        <Plot
            scope="dot-plot-chart"
            label
            hint="Use the up and down arrow keys to move between rows, left and right between series."
            height
            horizontal=true
            ticks
            labels
            annotations=notes
            legend
            tip
            head
            rows=table
            on_key
            on_point
            marks
        />
    }
}
