//! `HorizonChart`: many time series as thin rows, each folded into colour bands so a
//! tall line fits a short row. Geometry comes from `hydroxy_charts::horizon`.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::grid_key;
use hydroxy_charts::plot::{self, Tip, TipRow};
use hydroxy_charts::scale::date_time;
use hydroxy_charts::{horizon, line, raster};

pub use hydroxy_charts::line::Series;

use crate::chart_frame::use_export;
use crate::plot_kit::{Plot, formatter, pct};

/// Columns each row is thinned to before drawing; about one per pixel on a wide screen.
const BUCKETS: usize = 600;

/// The table rows for the first `limit` moments: the time, then each series' value.
fn table(times: &[f64], series: &[Series], limit: usize, fmt: impl Fn(f64) -> String + Copy) -> Vec<Vec<String>> {
    times
        .iter()
        .take(limit)
        .enumerate()
        .map(|(i, &at)| {
            std::iter::once(date_time(at))
                .chain(series.iter().map(|s| plot::cell(s.values.get(i).copied(), &fmt)))
                .collect()
        })
        .collect()
}

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
    series: ReadSignal<Vec<Series>>,
    /// When each value was taken: UTC seconds since the Unix epoch, ascending, shared by every series.
    times: ReadSignal<Vec<f64>>,
    /// What the chart shows, such as "CPU load by host". Used as the accessible name.
    label: String,
    /// Colour bands per row, 2 to 4. More bands fit more range into a row but are harder to tell apart.
    #[props(default = 3)]
    bands: usize,
    /// Height of each row in px.
    #[props(default = 36)]
    row_height: u16,
    /// Formats values for the legend, tooltip and table. Default: whole numbers, else one decimal place.
    format: Option<Callback<f64, String>>,
    /// Called when the pointer or keyboard moves to a row and time, and with `None` when it leaves.
    on_hover: Option<EventHandler<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a row and time.
    on_select: Option<EventHandler<EventDatum>>,
) -> Element {
    let fmt = formatter(format);
    let axis = use_memo(move || horizon::time_axis(&times.read()));
    let layout = use_memo(move || {
        let values: Vec<Vec<f64>> = series.read().iter().map(|s| s.values.clone()).collect();
        horizon::layout(&values, &axis.read().0, bands, BUCKETS)
    });
    use_export(move |()| {
        let s = series.read();
        let head = std::iter::once("Time".to_string()).chain(s.iter().map(|s| s.name.clone())).collect();
        std::iter::once(head).chain(table(&times.read(), &s, usize::MAX, fmt)).collect()
    });
    let all = series();
    let ts = times.read();
    let (xs, ticks) = &*axis.read();
    let l = layout.read();
    let (n, len) = (all.len(), ts.len());
    let height = u16::try_from(n.max(1) * usize::from(row_height)).unwrap_or(u16::MAX);
    let k = bands.clamp(1, 4);
    // (row, time index), matching nav::grid_key's (row, column).
    let mut active = use_signal(|| None::<(usize, usize)>);
    let current = active().filter(|&(r, i)| r < n && i < len);
    let now = current.and_then(|(r, i)| EventDatum::from_series(&all, r, i, Some(&date_time(ts[i]))));
    let datum: ReadSignal<Option<EventDatum>> = use_memo(use_reactive((&now,), |(d,)| d)).into();
    let tip = current.map(|(r, i)| {
        let rows = all
            .iter()
            .enumerate()
            .map(|(k, s)| TipRow {
                active: k == r,
                ..TipRow::new(None, s.name.clone(), plot::cell(s.values.get(i).copied(), &fmt))
            })
            .collect();
        Tip { x: xs[i], y: 1.0 - plot::band_centre(r, n), title: date_time(ts[i]), rows }
    });
    let head: Vec<String> = std::iter::once("Time".to_string()).chain(all.iter().map(|s| s.name.clone())).collect();
    let body = table(&ts, &all, raster::table_rows(len, n + 1), fmt);
    let xs_for_pointer = xs.clone();
    let names: Vec<String> = all.iter().map(|s| s.name.clone()).collect();

    rsx! {
        Plot {
            scope: "horizon-chart",
            label,
            hint: "Use the left and right arrow keys to move through time, up and down between rows.",
            height,
            horizontal: true,
            ticks: ticks.clone(),
            labels: plot::category_ticks(&names, usize::MAX),
            annotations: Vec::new(),
            legend: horizon::legend(&l, bands, &fmt),
            tip,
            datum: Some(datum),
            on_hover,
            on_select,
            head,
            rows: body,
            on_key: move |key: String| grid_key(&key, active(), n, len).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, y)| Some((plot::band_at(1.0 - y, n)?, line::nearest_index(&xs_for_pointer, x)?)));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                div { "data-part": "rows",
                    for (r , row) in l.rows.iter().enumerate() {
                        svg {
                            "data-part": "row",
                            view_box: "0 0 100 100",
                            preserve_aspect_ratio: "none",
                            "aria-hidden": "true",
                            style: "top: {pct(r as f64 / n.max(1) as f64)}; height: {pct(1.0 / n.max(1) as f64)}; --i: {r}; --ox-n: {n};",
                            for (negative , list) in [(false, &row.positive), (true, &row.negative)] {
                                for (b , d) in list.iter().enumerate().filter(|(_, d)| !d.is_empty()) {
                                    path {
                                        "data-part": "fold",
                                        "data-negative": negative.then_some(""),
                                        style: "fill: {horizon::shade(negative, b, k)};",
                                        d: "{d}",
                                    }
                                }
                            }
                        }
                    }
                }
            },
            overlay: current.map(|(_, i)| rsx! {
                div { "data-part": "crosshair", style: "--p: {pct(xs[i])};" }
            }),
        }
    }
}
