//! `SpectrogramChart`: how a signal's frequency content changes over time. Time runs along
//! the bottom, frequency up the side, and each cell's shade is its power in decibels. It
//! takes raw samples (and runs a short-time Fourier transform) or a precomputed matrix, and
//! always draws on a canvas. Geometry comes from `hydroxy_charts::spectrogram`.

use dioxus::prelude::*;
use hydroxy_charts::nav::grid_key;
use hydroxy_charts::plot::{Tick, Tip, TipRow};
use hydroxy_charts::{format_value, raster, spectrogram};

pub use hydroxy_charts::spectrogram::Spectrum;

use crate::chart_frame::use_export;
use crate::plot_kit::{Plot, colour, pct};
use crate::raster_canvas::{Frame, RasterCanvas};

/// Steps in the colour ramp.
const RAMP_STEPS: usize = 64;

/// A frequency for axes and readouts: "440 Hz", "2.5 kHz".
fn hz(f: f64) -> String {
    if f.abs() >= 1000.0 { format!("{} kHz", format_value(f / 1000.0)) } else { format!("{} Hz", format_value(f)) }
}

/// The level colour at `level` (0..=1): one chart hue rising out of the surface.
fn ramp_colour(level: f64) -> String {
    format!("color-mix(in oklab, {} {:.1}%, var(--surface-sunken))", colour(0), level.clamp(0.0, 1.0).powf(0.8) * 100.0)
}

/// A spectrogram: time along the bottom, frequency (linear or log) up the side, power in dB
/// as the shade. Pass `samples` and `sample_rate` to transform raw audio or sensor data, or
/// `spectrum` for a matrix you computed. Hover or touch to read time, frequency and level;
/// focus the plot and use the arrow keys to move cell by cell, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[allow(clippy::cast_precision_loss, reason = "ramp steps are small")]
#[component]
pub fn SpectrogramChart(
    /// What the chart shows, such as "Spectrogram of the doorbell recording". Used as the accessible name.
    label: String,
    /// Raw samples, transformed with a Hann-windowed STFT. Ignored when `spectrum` is set.
    #[props(default)]
    samples: ReadSignal<Vec<f64>>,
    /// Samples per second.
    #[props(default = 44_100.0)]
    sample_rate: f64,
    /// Samples per frame, rounded up to a power of two. Longer windows resolve frequency more finely, time less.
    #[props(default = 512)]
    window: usize,
    /// Samples between frame starts; 0 for a quarter window.
    #[props(default)]
    hop: usize,
    /// A precomputed spectrum (`Spectrum::new(times, freqs, db)`), used instead of `samples`.
    #[props(default)]
    spectrum: Option<ReadSignal<Spectrum>>,
    /// Space frequencies logarithmically, so each octave gets the same height.
    #[props(default)]
    log_frequency: bool,
    /// How many dB below the loudest cell the colour scale reaches; quieter cells clip to the background.
    #[props(default = 80.0)]
    range: f64,
    /// Plot height in px, excluding the axis labels and colour scale.
    #[props(default = 260)]
    height: u16,
) -> Element {
    let layout = use_memo(move || {
        let s = spectrum.map_or_else(|| Spectrum::from_samples(&samples.read(), sample_rate, window, hop), |s| s());
        spectrogram::layout(s, log_frequency, range)
    });
    use_export(move |()| {
        let l = layout.read();
        let s = &l.spectrum;
        let head = std::iter::once("Time (s)".to_string())
            .chain(s.freqs.iter().map(|f| format!("{f:.1} Hz")))
            .collect::<Vec<_>>();
        let body =
            s.times.iter().zip(&s.db).map(|(t, row)| {
                std::iter::once(format!("{t:.4}")).chain(row.iter().map(|v| format!("{v:.1}"))).collect()
            });
        std::iter::once(head).chain(body).collect()
    });
    let draw = use_callback(move |f: Frame| spectrogram::paint(&layout.read(), &f.colours, f.width, f.height));
    let l = layout.read();
    let s = &l.spectrum;
    let (bins, frames) = (s.freqs.len(), s.times.len());
    // (bin from the top, frame), so Up and Down move the way they look.
    let mut active = use_signal(|| None::<(usize, usize)>);
    let cell = active().filter(|&(r, c)| r < bins && c < frames).and_then(|(top, frame)| {
        let bin = bins - 1 - top;
        Some((s.at(frame, bin)?, l.position(frame, bin), s.times[frame], s.freqs[bin]))
    });
    let tip = cell.map(|(db, (x, y), t, f)| {
        let rows = vec![TipRow {
            active: true,
            ..TipRow::new(Some(ramp_colour(l.level(db))), "Level", format!("{db:.1} dB"))
        }];
        Tip { x, y, title: format!("{} s · {}", format_value(t), hz(f)), rows }
    });
    let ticks: Vec<Tick> =
        l.axis.gridlines.iter().map(|g| Tick { at: g.fraction, text: hz(g.value), baseline: false }).collect();
    let rows: Vec<Vec<String>> = (0..raster::table_rows(frames, 3))
        .map(|i| {
            let (f, db) =
                s.peak(i).map_or_else(|| ("–".to_string(), "–".to_string()), |(f, db)| (hz(f), format!("{db:.1} dB")));
            vec![format!("{} s", format_value(s.times[i])), f, db]
        })
        .collect();
    let ramp: Vec<String> = (0..RAMP_STEPS).map(|i| ramp_colour(i as f64 / (RAMP_STEPS - 1) as f64)).collect();
    let levels = l.levels;

    let variant = if log_frequency { "log" } else { "linear" };

    rsx! {
        Plot {
            scope: "spectrogram-chart",
            label,
            hint: "Use the arrow keys to move through time and frequency.",
            height,
            variant,
            ticks,
            labels: l.x_axis.ticks(|t| format!("{} s", format_value(t))),
            annotations: Vec::new(),
            tip,
            head: ["Time", "Peak frequency", "Peak level"].map(String::from).to_vec(),
            rows,
            on_key: move |key: String| grid_key(&key, active(), bins, frames).map(|next| active.set(next)).is_some(),
            on_point: move |p: Option<(f64, f64)>| {
                let next = p.and_then(|(x, y)| layout.read().cell_at(x, y).map(|(frame, bin)| (bins - 1 - bin, frame)));
                if active() != next {
                    active.set(next);
                }
            },
            marks: rsx! {
                div { "data-part": "image",
                    RasterCanvas { colours: ramp, draw }
                }
            },
            overlay: cell.map(|(_, (x, y), _, _)| rsx! {
                div { "data-part": "crosshair", style: "--p: {pct(x)};" }
                div { "data-part": "crosshair-y", style: "--p: {pct(y)};" }
            }),
        }
        div { "data-scope": "spectrogram-chart", "data-part": "scale", "aria-hidden": "true",
            span { "data-part": "scale-value", "{levels.0:.0} dB" }
            span { "data-part": "ramp" }
            span { "data-part": "scale-value", "{levels.1:.0} dB" }
        }
    }
}
