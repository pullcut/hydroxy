//! `<SpectrogramChart>`: how a signal's frequency content changes over time. Time runs
//! along the bottom, frequency up the side, and each cell's shade is its power in decibels.
//! It takes raw samples (and runs a short-time Fourier transform) or a precomputed matrix,
//! and always draws on a canvas. Geometry comes from `hydroxy_charts::spectrogram`.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::nav::grid_key;
use hydroxy_charts::plot::{Tick, Tip, TipRow};
use hydroxy_charts::{format_value, raster, spectrogram};
use leptos::prelude::*;

pub use hydroxy_charts::spectrogram::Spectrum;
use hydroxy_charts::spectrogram::{hz, ramp_colour};

use crate::chart_frame::provide_export;
use crate::plot_kit::{Plot, pct};
use crate::raster_canvas::{Frame, RasterCanvas};

/// Steps in the colour ramp.
const RAMP_STEPS: usize = 64;

/// A spectrogram: time along the bottom, frequency (linear or log) up the side, power in dB
/// as the shade. Pass `samples` and `sample_rate` to transform raw audio or sensor data, or
/// `spectrum` for a matrix you computed. Hover or touch to read time, frequency and level;
/// focus the plot and use the arrow keys to move cell by cell, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::cast_precision_loss, reason = "ramp steps are small")]
#[component]
pub fn SpectrogramChart(
    /// What the chart shows, such as "Spectrogram of the doorbell recording". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Raw samples, transformed with a Hann-windowed STFT. Ignored when `spectrum` is set.
    #[prop(optional, into)]
    samples: Signal<Vec<f64>>,
    /// Samples per second.
    #[prop(default = 44_100.0)]
    sample_rate: f64,
    /// Samples per frame, rounded up to a power of two. Longer windows resolve frequency more finely, time less.
    #[prop(default = 512)]
    window: usize,
    /// Samples between frame starts; 0 for a quarter window.
    #[prop(optional)]
    hop: usize,
    /// A precomputed spectrum (`Spectrum::new(times, freqs, db)`), used instead of `samples`.
    #[prop(optional, into)]
    spectrum: Option<Signal<Spectrum>>,
    /// Space frequencies logarithmically, so each octave gets the same height.
    #[prop(optional)]
    log_frequency: bool,
    /// How many dB below the loudest cell the colour scale reaches; quieter cells clip to the background.
    #[prop(default = 80.0)]
    range: f64,
    /// Plot height in px, excluding the axis labels and colour scale.
    #[prop(default = 260)]
    height: u16,
    /// Called when the hovered or focused cell changes; `None` when it clears.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a cell.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let layout = Memo::new(move |_| {
        let s =
            spectrum.map_or_else(|| samples.with(|x| Spectrum::from_samples(x, sample_rate, window, hop)), |s| s.get());
        spectrogram::layout(s, log_frequency, range)
    });
    let shape = move || layout.with(|l| (l.spectrum.freqs.len(), l.spectrum.times.len()));
    // (bin from the top, frame), so Up and Down move the way they look.
    let active = RwSignal::new(None::<(usize, usize)>);
    let cell = move |(top, frame): (usize, usize)| {
        layout.with(|l| {
            let bin = l.spectrum.freqs.len().checked_sub(top + 1)?;
            let db = l.spectrum.at(frame, bin)?;
            Some((frame, bin, db, l.position(frame, bin), l.spectrum.times[frame], l.spectrum.freqs[bin]))
        })
    };

    let tip = Signal::derive(move || {
        let (_, _, db, (x, y), t, f) = cell(active.get()?)?;
        let level = layout.with(|l| l.level(db));
        let rows =
            vec![TipRow { active: true, ..TipRow::new(Some(ramp_colour(level)), "Level", format!("{db:.1} dB")) }];
        Some(Tip { x, y, title: format!("{} s · {}", format_value(t), hz(f)), rows })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next = p.and_then(|(x, y)| {
            layout.with(|l| l.cell_at(x, y).map(|(frame, bin)| (l.spectrum.freqs.len() - 1 - bin, frame)))
        });
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        let (rows, cols) = shape();
        grid_key(&key, active.get_untracked(), rows, cols).map(|next| active.set(next)).is_some()
    });
    let datum = Signal::derive(move || {
        let (frame, bin, db, _, t, freq) = cell(active.get()?)?;
        Some(EventDatum {
            series: bin,
            index: frame,
            label: Some(format!("{} s · {}", format_value(t), hz(freq))),
            value: db,
            node: None,
        })
    });
    let ticks = Signal::derive(move || {
        layout.with(|l| {
            l.axis
                .gridlines
                .iter()
                .map(|g| Tick { at: g.fraction, text: hz(g.value), baseline: false })
                .collect::<Vec<_>>()
        })
    });
    let labels = Signal::derive(move || layout.with(|l| l.x_axis.ticks(&|t| format!("{} s", format_value(t)))));
    let head = Signal::derive(|| ["Time", "Peak frequency", "Peak level"].map(String::from).to_vec());
    let rows = Signal::derive(move || {
        layout.with(|l| {
            let s = &l.spectrum;
            (0..raster::table_rows(s.times.len(), 3))
                .map(|i| {
                    let (f, db) = s
                        .peak(i)
                        .map_or_else(|| ("–".to_string(), "–".to_string()), |(f, db)| (hz(f), format!("{db:.1} dB")));
                    vec![format!("{} s", format_value(s.times[i])), f, db]
                })
                .collect()
        })
    });
    provide_export(move || {
        layout.with_untracked(|l| {
            let s = &l.spectrum;
            let head = std::iter::once("Time (s)".to_string())
                .chain(s.freqs.iter().map(|f| format!("{f:.1} Hz")))
                .collect::<Vec<_>>();
            let body = s.times.iter().zip(&s.db).map(|(t, row)| {
                std::iter::once(format!("{t:.4}")).chain(row.iter().map(|v| format!("{v:.1}"))).collect()
            });
            std::iter::once(head).chain(body).collect()
        })
    });
    let ramp =
        Signal::derive(|| (0..RAMP_STEPS).map(|i| ramp_colour(i as f64 / (RAMP_STEPS - 1) as f64)).collect::<Vec<_>>());
    let draw = Callback::new(move |f: Frame| layout.with(|l| spectrogram::paint(l, &f.colours, f.width, f.height)));
    let overlay = move || {
        let (_, _, _, (x, y), _, _) = cell(active.get()?)?;
        Some(view! {
            <div data-part="crosshair" style=format!("--p: {};", pct(x))></div>
            <div data-part="crosshair-y" style=format!("--p: {};", pct(y))></div>
        })
    };

    view! {
        <Plot
            scope="spectrogram-chart"
            label
            hint="Use the arrow keys to move through time and frequency."
            height
            variant=if log_frequency { "log" } else { "linear" }
            ticks
            labels
            annotations=Signal::derive(Vec::new)
            tip
            head
            rows
            on_key
            on_point
            marks=move || view! { <div data-part="image"><RasterCanvas colours=ramp draw /></div> }
            overlay
            datum=Some(datum)
            on_hover
            on_select
        />
        <div data-scope="spectrogram-chart" data-part="scale" aria-hidden="true">
            <span data-part="scale-value">{move || format!("{:.0} dB", layout.with(|l| l.levels.0))}</span>
            <span data-part="ramp"></span>
            <span data-part="scale-value">{move || format!("{:.0} dB", layout.with(|l| l.levels.1))}</span>
        </div>
    }
}
