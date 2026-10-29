use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::spectrogram_chart::*;
use leptos::prelude::*;

const RATE: f64 = 16_000.0;

/// A two-tone doorbell, "ding" (E5) then "dong" (C5), each with decaying harmonics.
fn doorbell() -> Vec<f64> {
    use std::f64::consts::TAU;
    (0..32_000)
        .map(|i| {
            let t = f64::from(i) / RATE;
            let strike = |at: f64, f0: f64| {
                let s = t - at;
                if s < 0.0 {
                    return 0.0;
                }
                [(1.0, 1.0), (2.0, 0.4), (3.0, 0.2), (4.2, 0.12)]
                    .iter()
                    .map(|(h, a)| a * (TAU * f0 * h * s).sin() * (-s * (1.6 + h)).exp())
                    .sum::<f64>()
                    * 0.45
            };
            strike(0.1, 659.3) + strike(0.9, 523.3)
        })
        .collect()
}

/// A bird-like call: three rising chirps from 2 to 6 kHz over light noise.
fn chirps() -> Vec<f64> {
    use std::f64::consts::TAU;
    let mut seed = 9u32;
    (0..24_000)
        .map(|i| {
            let t = f64::from(i) / RATE;
            let k = (t / 0.5).floor();
            let s = t - k * 0.5 - 0.05;
            let call = if (0.0..0.22).contains(&s) && k < 3.0 {
                // Phase of a linear sweep from 2 kHz up by 18 kHz/s.
                (TAU * (2_000.0 * s + 9_000.0 * s * s)).sin() * (s * 14.3).sin()
            } else {
                0.0
            };
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            call * 0.6 + (f64::from(seed >> 8) / f64::from(1u32 << 24) - 0.5) * 0.004
        })
        .collect()
}

pub fn demo() -> AnyView {
    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Doorbell" description="Two struck tones and their harmonics, on a log frequency scale so each octave gets equal height.">
                <SpectrogramChart samples=doorbell() sample_rate=RATE window=1024 hop=128 log_frequency=true range=70.0 label="Spectrogram of a two-tone doorbell" />
            </ChartFrame>
            <ChartFrame title="Bird call" description="Three rising chirps, each sweeping from 2 to 6 kHz.">
                <SpectrogramChart samples=chirps() sample_rate=RATE window=256 hop=64 height=220 label="Spectrogram of a bird call" />
            </ChartFrame>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "spectrogram-chart",
    name: "Spectrogram chart",
    group: "Time",
    summary: "Show how a signal's frequencies change over time. Raw samples go through a short-time Fourier transform; time runs across, frequency up, and loudness is the shade.",
    demo,
    leptos: include_str!("spectrogram_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/spectrogram_chart.rs"),
    parts: &[
        &[
            "SpectrogramChart",
            "root",
            "figure with a screen-reader table of each frame's loudest frequency; data-variant is linear or log",
        ],
        &["", "image", "The canvas, wiped in left to right"],
        &["", "crosshair", "The hovered time and frequency (crosshair, crosshair-y)"],
        &["", "scale", "Level colour ramp in dB, after the figure"],
        &["", "tooltip", "Time, frequency and level"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next frame"],
        &["ArrowUp / ArrowDown", "Higher or lower frequency"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["samples", "Vec<f64>", "Raw samples to transform"],
        &["sample_rate", "f64", "Samples per second. Default 44100"],
        &["window", "usize", "Samples per frame, rounded up to a power of two. Default 512"],
        &["hop", "usize", "Samples between frames. Default a quarter window"],
        &["spectrum", "Spectrum", "A precomputed matrix, Spectrum::new(times, freqs, db), instead of samples"],
        &["log_frequency", "bool", "Log frequency axis"],
        &["range", "f64", "dB below the loudest cell the colours reach. Default 80"],
        &["height", "u16", "Plot height in px. Default 260"],
    ],
    events: crate::doc::CHART_EVENTS,
};
