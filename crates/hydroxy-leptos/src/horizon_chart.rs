//! `<HorizonChart>`: many time series as thin rows, each folded into colour bands so a
//! tall line fits a short row. Geometry comes from `hydroxy_charts::horizon`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::grid_key;
use hydroxy_charts::plot::{self, Tip, TipRow};
use hydroxy_charts::scale::date_time;
use hydroxy_charts::{horizon, line, raster};
use leptos::prelude::*;

pub use hydroxy_charts::line::Series;

use crate::chart_frame::provide_export;
use crate::plot_kit::{Plot, formatter, pct};

/// Columns each row is thinned to before drawing; about one per pixel on a wide screen.
const BUCKETS: usize = 600;

/// One row per series over a shared time axis. Each row's area is cut into equal value
/// bands that fold down on top of each other, darker for higher values; values below zero
/// fold up in a second hue. Every row uses the same band size, so rows compare. Long
/// series are thinned to the screen, keeping peaks. Hover or touch to read every series at
/// that moment; focus the plot and use Left/Right to step through time, Up/Down between
/// rows, Home/End to jump, Escape to clear.
#[allow(clippy::cast_precision_loss, reason = "row counts are small")]
#[component]
pub fn HorizonChart(
    /// The series, one row each, top to bottom. One value per entry of `times`; NaN leaves a gap.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// When each value was taken: UTC seconds since the Unix epoch, ascending, shared by every series.
    #[prop(into)]
    times: Signal<Vec<f64>>,
    /// What the chart shows, such as "CPU load by host". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Colour bands per row, 2 to 4. More bands fit more range into a row but are harder to tell apart.
    #[prop(default = 3)]
    bands: usize,
    /// Height of each row in px.
    #[prop(default = 36)]
    row_height: u16,
    /// Formats values for the legend, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a row and time, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a row and time.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let axis = Memo::new(move |_| times.with(|t| horizon::time_axis(t)));
    let layout = Memo::new(move |_| {
        let values: Vec<Vec<f64>> = series.with(|s| s.iter().map(|s| s.values.clone()).collect());
        axis.with(|(xs, _)| horizon::layout(&values, xs, bands, BUCKETS))
    });
    let rows = move || series.with(Vec::len);
    let len = move || times.with(Vec::len);
    let height = u16::try_from(series.with_untracked(Vec::len).max(1) * usize::from(row_height)).unwrap_or(u16::MAX);
    // (row, time index), matching nav::grid_key's (row, column).
    let active = RwSignal::new(None::<(usize, usize)>);
    let datum = Signal::derive(move || {
        let (r, i) = active.get()?;
        let at = times.with(|t| t.get(i).map(|&t| date_time(t)));
        series.with(|all| EventDatum::from_series(all, r, i, at.as_deref()))
    });
    let tip = Signal::derive(move || {
        let (r, i) = active.get()?;
        let at = times.with(|t| t.get(i).copied())?;
        let x = axis.with(|(xs, _)| xs.get(i).copied())?;
        let rows_tip = series.with(|all| {
            all.iter()
                .enumerate()
                .map(|(k, s)| TipRow {
                    active: k == r,
                    ..TipRow::new(None, s.name.clone(), plot::cell(s.values.get(i).copied(), &fmt))
                })
                .collect()
        });
        Some(Tip { x, y: 1.0 - plot::band_centre(r, rows()), title: date_time(at), rows: rows_tip })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, y)| {
            Some((plot::band_at(1.0 - y, rows())?, axis.with(|(xs, _)| line::nearest_index(xs, x))?))
        });
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        grid_key(&key, active.get_untracked(), rows(), len()).map(|next| active.set(next)).is_some()
    });
    let ticks = Signal::derive(move || axis.with(|(_, t)| t.clone()));
    let labels = Signal::derive(move || {
        series.with(|s| plot::category_ticks(&s.iter().map(|s| s.name.clone()).collect::<Vec<_>>(), usize::MAX))
    });
    let legend = Signal::derive(move || layout.with(|l| horizon::legend(l, bands, &fmt)));
    let head = Signal::derive(move || {
        std::iter::once("Time".to_string())
            .chain(series.with(|s| s.iter().map(|s| s.name.clone()).collect::<Vec<_>>()))
            .collect()
    });
    let table = move |limit: usize| {
        times.with(|t| {
            series.with(|s| {
                t.iter()
                    .take(limit)
                    .enumerate()
                    .map(|(i, &at)| {
                        std::iter::once(date_time(at))
                            .chain(s.iter().map(|x| plot::cell(x.values.get(i).copied(), &fmt)))
                            .collect()
                    })
                    .collect::<Vec<Vec<String>>>()
            })
        })
    };
    let body = Signal::derive(move || table(raster::table_rows(len(), rows() + 1)));
    provide_export(move || std::iter::once(head.get_untracked()).chain(table(usize::MAX)).collect());
    let marks = move || {
        let l = layout.get();
        let n = l.rows.len();
        let k = bands.clamp(1, 4);
        let rows_view = l
            .rows
            .into_iter()
            .enumerate()
            .map(|(r, row)| {
                let style = format!("top: {}; height: {}; --i: {r}; --ox-n: {n};", pct(r as f64 / n.max(1) as f64), pct(1.0 / n.max(1) as f64));
                let paths = |list: Vec<String>, negative: bool| {
                    list.into_iter()
                        .enumerate()
                        .filter(|(_, d)| !d.is_empty())
                        .map(|(b, d)| {
                            view! { <path data-part="fold" data-negative=negative.then_some("") style=format!("fill: {};", horizon::shade(negative, b, k)) d=d></path> }
                        })
                        .collect_view()
                };
                view! {
                    <svg data-part="row" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true" style=style>
                        {paths(row.positive, false)}
                        {paths(row.negative, true)}
                    </svg>
                }
            })
            .collect_view();
        view! { <div data-part="rows">{rows_view}</div> }
    };
    let overlay = move || {
        let (_, i) = active.get()?;
        let x = axis.with(|(xs, _)| xs.get(i).copied())?;
        Some(view! { <div data-part="crosshair" style=format!("--p: {};", pct(x))></div> })
    };

    view! {
        <Plot
            scope="horizon-chart"
            label
            hint="Use the left and right arrow keys to move through time, up and down between rows."
            height
            horizontal=true
            ticks
            labels
            annotations=Signal::derive(Vec::new)
            legend
            tip
            head
            rows=body
            on_key
            on_point
            marks
            datum=Some(datum)
            on_hover
            on_select
            overlay
        />
    }
}
