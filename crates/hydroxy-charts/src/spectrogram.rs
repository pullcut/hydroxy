//! Spectrograms: a short-time Fourier transform (Hann window, radix-2 FFT) of a signal into
//! power in decibels per time frame and frequency bin, and the pixels to draw it.
#![allow(
    clippy::many_single_char_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "short names follow the maths; frame and bin counts are small; fractions are clamped"
)]

use std::f64::consts::PI;

use crate::plot::ValueAxis;
use crate::raster::{Raster, Rgba};
use crate::scale::{ScaleKind, ValueScale};

/// In-place iterative radix-2 FFT of `re` + i·`im`. Lengths must match and be a power of two.
///
/// # Panics
/// When the lengths differ or are not a power of two.
pub fn fft(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
    assert!(n == im.len() && n.is_power_of_two(), "fft needs equal power-of-two lengths");
    // Bit-reversal permutation.
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let angle = -2.0 * PI / len as f64;
        let (wr, wi) = (angle.cos(), angle.sin());
        for start in (0..n).step_by(len) {
            let (mut cr, mut ci) = (1.0, 0.0);
            for k in 0..len / 2 {
                let (a, b) = (start + k, start + k + len / 2);
                let (tr, ti) = (re[b] * cr - im[b] * ci, re[b] * ci + im[b] * cr);
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
                (cr, ci) = (cr * wr - ci * wi, cr * wi + ci * wr);
            }
        }
        len <<= 1;
    }
}

/// A periodic Hann window of `n` points (the form for spectral analysis).
#[must_use]
pub fn hann(n: usize) -> Vec<f64> {
    (0..n).map(|i| 0.5 - 0.5 * (2.0 * PI * i as f64 / n as f64).cos()).collect()
}

/// Power in decibels by time frame and frequency bin.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Spectrum {
    /// Centre time of each frame, in seconds, ascending.
    pub times: Vec<f64>,
    /// Centre frequency of each bin, in Hz, ascending.
    pub freqs: Vec<f64>,
    /// `db[frame][bin]`, in decibels.
    pub db: Vec<Vec<f64>>,
}

impl Spectrum {
    /// A precomputed matrix: `db[frame][bin]` at the given frame times and bin frequencies.
    #[must_use]
    pub fn new(times: impl Into<Vec<f64>>, freqs: impl Into<Vec<f64>>, db: Vec<Vec<f64>>) -> Self {
        Self { times: times.into(), freqs: freqs.into(), db }
    }

    /// The short-time Fourier transform of `samples` at `sample_rate` Hz: frames of `window`
    /// samples (rounded up to a power of two, at least 16) every `hop` samples (`0` for a
    /// quarter window), Hann-windowed, as one-sided power in dB relative to a full-scale sine.
    #[must_use]
    pub fn from_samples(samples: &[f64], sample_rate: f64, window: usize, hop: usize) -> Self {
        let n = window.max(16).next_power_of_two();
        let hop = if hop == 0 { n / 4 } else { hop };
        let w = hann(n);
        // A full-scale sine gives |X| = A·Σw/2; normalise so it reads 0 dB.
        let gain = w.iter().sum::<f64>() / 2.0;
        let bins = n / 2 + 1;
        let rate = if sample_rate > 0.0 { sample_rate } else { 1.0 };
        let frames = if samples.len() >= n { (samples.len() - n) / hop + 1 } else { usize::from(!samples.is_empty()) };
        let (mut re, mut im) = (vec![0.0; n], vec![0.0; n]);
        let mut db = Vec::with_capacity(frames);
        for f in 0..frames {
            let start = f * hop;
            for i in 0..n {
                re[i] = samples.get(start + i).copied().filter(|v| v.is_finite()).unwrap_or(0.0) * w[i];
                im[i] = 0.0;
            }
            fft(&mut re, &mut im);
            db.push((0..bins).map(|k| 20.0 * (re[k].hypot(im[k]) / gain).max(1e-10).log10()).collect());
        }
        Self {
            times: (0..frames).map(|f| (f * hop) as f64 / rate + n as f64 / 2.0 / rate).collect(),
            freqs: (0..bins).map(|k| k as f64 * rate / n as f64).collect(),
            db,
        }
    }

    /// The value at (`frame`, `bin`), if any.
    #[must_use]
    pub fn at(&self, frame: usize, bin: usize) -> Option<f64> {
        self.db.get(frame)?.get(bin).copied().filter(|v| v.is_finite())
    }

    /// The loudest bin of `frame`: (frequency, dB).
    #[must_use]
    pub fn peak(&self, frame: usize) -> Option<(f64, f64)> {
        let row = self.db.get(frame)?;
        let (bin, &db) = row.iter().enumerate().filter(|(_, v)| v.is_finite()).max_by(|a, b| a.1.total_cmp(b.1))?;
        Some((self.freqs.get(bin).copied()?, db))
    }
}

/// Everything a spectrogram chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct SpectrogramLayout {
    /// The spectrum drawn.
    pub spectrum: Spectrum,
    /// The time (x) axis, in seconds.
    pub x_axis: ValueAxis,
    /// The frequency (y) axis, in Hz; linear or log.
    pub axis: ValueAxis,
    /// The colour scale's ends in dB: the loudest value and `range` below it.
    pub levels: (f64, f64),
}

impl SpectrogramLayout {
    /// The frame and bin under plot fraction (`x`, `y`), y from the bottom.
    #[must_use]
    pub fn cell_at(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        let s = &self.spectrum;
        let frame = nearest(&s.times, invert(self.x_axis.scale, x.clamp(0.0, 1.0)))?;
        let bin = nearest(&s.freqs, invert(self.axis.scale, y.clamp(0.0, 1.0)))?;
        Some((frame, bin))
    }

    /// Where (`frame`, `bin`) sits as plot fractions, y from the bottom.
    #[must_use]
    pub fn position(&self, frame: usize, bin: usize) -> (f64, f64) {
        let s = &self.spectrum;
        (
            self.x_axis.at(s.times.get(frame).copied().unwrap_or(0.0)),
            self.axis.at(s.freqs.get(bin).copied().unwrap_or(0.0)),
        )
    }

    /// `db` on the colour scale, 0..=1.
    #[must_use]
    pub fn level(&self, db: f64) -> f64 {
        let (lo, hi) = self.levels;
        if hi > lo { ((db - lo) / (hi - lo)).clamp(0.0, 1.0) } else { 1.0 }
    }
}

/// The data value at fraction `t` along `axis`.
fn invert(scale: ValueScale, t: f64) -> f64 {
    let (a, b) = scale.domain;
    match scale.kind {
        ScaleKind::Log => 10f64.powf(a.log10() + t * (b.log10() - a.log10())),
        _ => a + t * (b - a),
    }
}

/// The index of the value in ascending `xs` nearest `v`.
fn nearest(xs: &[f64], v: f64) -> Option<usize> {
    crate::line::nearest_index(xs, v)
}

/// Lay out `spectrum` with a linear or log (`log`) frequency axis, colouring the top
/// `range` dB (values further below the peak clip to the bottom of the scale).
#[must_use]
pub fn layout(spectrum: Spectrum, log: bool, range: f64) -> SpectrogramLayout {
    let t0 = spectrum.times.first().copied().unwrap_or(0.0);
    let t1 = spectrum.times.last().copied().unwrap_or(1.0);
    // Frames are centred on their times; extend by half a frame step so edge frames show whole.
    let half = if spectrum.times.len() > 1 { (t1 - t0) / (spectrum.times.len() - 1) as f64 / 2.0 } else { 0.5 };
    let x_axis = ValueAxis::new(ScaleKind::Linear, [(t0 - half).max(0.0), t1 + half], &[], true);
    let nyquist = spectrum.freqs.last().copied().unwrap_or(1.0).max(f64::MIN_POSITIVE);
    let axis = if log {
        let low = spectrum.freqs.iter().copied().find(|f| *f > 0.0).unwrap_or(1.0).max(nyquist / 10_000.0);
        let mut a = ValueAxis::new(ScaleKind::Log, [low, nyquist], &[], false);
        // Log domains snap out to whole decades; keep the top at Nyquist so nothing is empty.
        a.scale.domain = (a.scale.domain.0.max(low / 1.5), nyquist);
        let lo = a.scale.domain.0;
        a.gridlines.retain(|g| g.value <= nyquist && g.value >= lo * 0.999);
        for g in &mut a.gridlines {
            g.fraction = a.scale.map(g.value);
        }
        a
    } else {
        ValueAxis::over(ScaleKind::Linear, 0.0, nyquist)
    };
    let hi = spectrum.db.iter().flatten().copied().filter(|v| v.is_finite()).fold(f64::NEG_INFINITY, f64::max);
    let hi = if hi.is_finite() { hi } else { 0.0 };
    SpectrogramLayout { spectrum, x_axis, axis, levels: (hi - range.abs().max(1.0), hi) }
}

/// Draw the spectrogram at `width` × `height` device pixels, nearest-cell, with `ramp`
/// colouring quiet to loud (at least one entry).
#[must_use]
pub fn paint(layout: &SpectrogramLayout, ramp: &[Rgba], width: usize, height: usize) -> Raster {
    let mut out = Raster::new(width, height);
    let s = &layout.spectrum;
    if ramp.is_empty() || s.times.is_empty() || s.freqs.is_empty() {
        return out;
    }
    // Nearest frame per column and bin per row, worked out once.
    let frames: Vec<Option<usize>> =
        (0..width).map(|x| nearest(&s.times, invert(layout.x_axis.scale, (x as f64 + 0.5) / width as f64))).collect();
    let bins: Vec<Option<usize>> = (0..height)
        .map(|y| nearest(&s.freqs, invert(layout.axis.scale, 1.0 - (y as f64 + 0.5) / height as f64)))
        .collect();
    let steps = (ramp.len() - 1) as f64;
    // Columns outside the frames' span stay empty rather than smearing the edge frame.
    let span = (
        s.times[0] - (s.times.get(1).unwrap_or(&s.times[0]) - s.times[0]) / 2.0,
        s.times[s.times.len() - 1] + (s.times[s.times.len() - 1] - s.times[s.times.len().saturating_sub(2)]) / 2.0,
    );
    for (x, frame) in frames.iter().enumerate() {
        let t = invert(layout.x_axis.scale, (x as f64 + 0.5) / width as f64);
        let Some(f) = frame.filter(|_| s.times.len() == 1 || (span.0..=span.1).contains(&t)) else { continue };
        for (y, bin) in bins.iter().enumerate() {
            let Some(db) = bin.and_then(|b| s.at(f, b)) else { continue };
            let c = ramp[((layout.level(db) * steps).round() as usize).min(ramp.len() - 1)];
            let i = (y * width + x) * 4;
            out.rgba[i..i + 4].copy_from_slice(&c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dft(x: &[f64]) -> Vec<(f64, f64)> {
        let n = x.len();
        (0..n)
            .map(|k| {
                x.iter().enumerate().fold((0.0, 0.0), |(r, i), (t, &v)| {
                    let a = -2.0 * PI * (k * t) as f64 / n as f64;
                    (r + v * a.cos(), i + v * a.sin())
                })
            })
            .collect()
    }

    #[test]
    fn fft_matches_a_direct_dft() {
        let x: Vec<f64> = (0..16).map(|i| (f64::from(i * 7 % 5) - 2.0) * 0.3 + (f64::from(i) * 0.4).sin()).collect();
        let (mut re, mut im) = (x.clone(), vec![0.0; 16]);
        fft(&mut re, &mut im);
        for (k, (r, i)) in dft(&x).into_iter().enumerate() {
            assert!((re[k] - r).abs() < 1e-9 && (im[k] - i).abs() < 1e-9, "bin {k}");
        }
    }

    #[test]
    fn hann_is_zero_at_the_edge_and_one_in_the_middle() {
        let w = hann(8);
        assert!(w[0].abs() < 1e-12);
        assert!((w[4] - 1.0).abs() < 1e-12);
        assert!((w[2] - w[6]).abs() < 1e-12);
    }

    #[test]
    fn a_sine_peaks_at_its_frequency_at_zero_db() {
        let rate = 8_000.0;
        let tone = 1_000.0; // bin 32 of a 256-point frame
        let x: Vec<f64> = (0..4_000).map(|i| (2.0 * PI * tone * f64::from(i) / rate).sin()).collect();
        let s = Spectrum::from_samples(&x, rate, 256, 128);
        assert_eq!(s.freqs.len(), 129);
        assert_eq!(s.db.len(), (4_000 - 256) / 128 + 1);
        assert!((s.times[0] - 128.0 / rate).abs() < 1e-12);
        let (f, db) = s.peak(3).unwrap();
        assert!((f - tone).abs() < 1e-9);
        assert!(db.abs() < 0.01, "full-scale sine reads 0 dB, got {db}");
        assert!(s.at(3, 80).unwrap() < -60.0, "far bins are quiet");
    }

    #[test]
    fn windows_round_up_and_short_signals_still_frame() {
        let s = Spectrum::from_samples(&[1.0; 10], 100.0, 300, 0);
        assert_eq!(s.freqs.len(), 257);
        assert_eq!(s.db.len(), 1);
        assert_eq!(Spectrum::from_samples(&[], 100.0, 64, 0).db.len(), 0);
    }

    #[test]
    fn layout_maps_cells_both_ways() {
        let x: Vec<f64> = (0..2_048).map(|i| (f64::from(i) * 0.3).sin()).collect();
        let l = layout(Spectrum::from_samples(&x, 1_000.0, 128, 64), false, 80.0);
        assert!((l.axis.scale.domain.1 - 500.0).abs() < 1e-9);
        let (fx, fy) = l.position(5, 20);
        assert_eq!(l.cell_at(fx, fy), Some((5, 20)));
        assert!((l.levels.1 - l.levels.0 - 80.0).abs() < 1e-9);
        let log = layout(l.spectrum.clone(), true, 60.0);
        assert!(log.axis.scale.domain.0 > 0.0 && (log.axis.scale.domain.1 - 500.0).abs() < 1e-9);
        let (fx, fy) = log.position(5, 20);
        assert_eq!(log.cell_at(fx, fy), Some((5, 20)));
    }

    #[test]
    fn paint_fills_the_image() {
        let l = layout(
            Spectrum::new([0.5, 1.5], [0.0, 10.0, 20.0], vec![vec![0.0, -10.0, -20.0], vec![-5.0, -5.0, -5.0]]),
            false,
            20.0,
        );
        let r = paint(&l, &[[0, 0, 0, 255], [255, 255, 255, 255]], 20, 10);
        assert!(r.rgba.chunks(4).filter(|p| p[3] == 255).count() > 100);
    }
}
