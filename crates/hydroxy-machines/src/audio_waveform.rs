//! Audio waveform: the rules for drawing and steering a peak display.
//!
//! The host supplies peak bins and the playhead. Nothing here reads the display
//! width or a clock: the seconds-per-bin scale comes from the peaks themselves.

/// Host-supplied peak bins. `samples_per_peak / sample_rate` is the seconds per bin.
#[derive(Debug, Clone, PartialEq)]
pub struct Peaks {
    /// One amplitude per bin, 0 to 1.
    pub values: Vec<f32>,
    /// Audio samples summarised by each bin.
    pub samples_per_peak: f64,
    /// Samples per second of the source.
    pub sample_rate: f64,
}

impl Peaks {
    /// Peaks at the given scale.
    #[must_use]
    pub fn new(values: Vec<f32>, samples_per_peak: f64, sample_rate: f64) -> Self {
        Self { values, samples_per_peak, sample_rate }
    }

    /// Seconds covered by one bin, or `None` for an unusable scale.
    #[must_use]
    pub fn seconds_per_peak(&self) -> Option<f64> {
        let ok = self.sample_rate.is_finite()
            && self.sample_rate > 0.0
            && self.samples_per_peak.is_finite()
            && self.samples_per_peak > 0.0;
        ok.then(|| self.samples_per_peak / self.sample_rate)
    }

    /// The duration the bins add up to.
    #[must_use]
    pub fn implied_duration(&self) -> Option<f64> {
        #[allow(clippy::cast_precision_loss, reason = "bin counts are far below 2^52")]
        self.seconds_per_peak().filter(|_| !self.values.is_empty()).map(|s| self.values.len() as f64 * s)
    }
}

/// A selected span of the timeline, in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range {
    /// Start, seconds.
    pub start: f64,
    /// End, seconds.
    pub end: f64,
}

/// Something wrong with what the host supplied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issue {
    /// No peaks.
    MissingPeaks,
    /// Peaks with a bad scale or a non-finite value.
    InvalidPeaks,
    /// The peaks add up to a different length than the duration.
    PeakTimeMismatch,
    /// The duration is unknown or not positive.
    InvalidDuration,
    /// The selection is reversed or not finite.
    InvalidRange,
}

impl Issue {
    /// The text used in `data-issue` and the status line.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Issue::MissingPeaks => "missing-peaks",
            Issue::InvalidPeaks => "invalid-peaks",
            Issue::PeakTimeMismatch => "peak-time-mismatch",
            Issue::InvalidDuration => "invalid-duration",
            Issue::InvalidRange => "invalid-range",
        }
    }
}

/// The visible slice of the timeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// First visible second.
    pub start: f64,
    /// Last visible second.
    pub end: f64,
    /// Zoom factor, 1 or more.
    pub zoom: f64,
}

/// One drawn bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bar {
    /// Index into the peaks.
    pub index: usize,
    /// Left edge as a fraction of the viewport, 0 to 1.
    pub left: f64,
    /// Width as a fraction of the viewport.
    pub width: f64,
    /// Height as a fraction of the track, 0 to 1.
    pub amplitude: f32,
}

/// Everything the renderer needs.
#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    /// Known duration.
    pub duration: Option<f64>,
    /// Playhead, clamped.
    pub current: f64,
    /// Playhead as a fraction of the viewport, if inside it.
    pub playhead: Option<f64>,
    /// The selection, if valid.
    pub range: Option<Range>,
    /// Selection edges as viewport fractions, clamped into view.
    pub range_fractions: Option<(f64, f64)>,
    /// The visible slice.
    pub viewport: Viewport,
    /// Bars in view.
    pub bars: Vec<Bar>,
    /// Problems with the input.
    pub issues: Vec<Issue>,
}

fn known(duration: Option<f64>) -> Option<f64> {
    duration.filter(|d| d.is_finite() && *d > 0.0)
}

/// Clamp a time into `0..=duration`.
#[must_use]
pub fn clamp_time(time: f64, duration: Option<f64>) -> f64 {
    let t = if time.is_finite() { time.max(0.0) } else { 0.0 };
    known(duration).map_or(t, |d| t.min(d))
}

/// The visible slice for a zoom and a left edge. The edge is clamped so the window stays inside the audio.
#[must_use]
pub fn viewport(duration: Option<f64>, zoom: f64, start: f64) -> Viewport {
    let zoom = if zoom.is_finite() && zoom >= 1.0 { zoom } else { 1.0 };
    let Some(d) = known(duration) else { return Viewport { start: 0.0, end: 0.0, zoom } };
    let window = d / zoom;
    let start = clamp_time(start, Some((d - window).max(0.0)));
    Viewport { start, end: (start + window).min(d), zoom }
}

/// The time at a fraction (0 to 1) across the viewport.
#[must_use]
pub fn time_at(fraction: f64, view: &Viewport, duration: Option<f64>) -> f64 {
    let f = if fraction.is_finite() { fraction.clamp(0.0, 1.0) } else { 0.0 };
    clamp_time(view.start + f * (view.end - view.start), duration)
}

/// Where a time sits in the viewport, or `None` when it is outside.
#[must_use]
pub fn fraction_of(seconds: f64, view: &Viewport) -> Option<f64> {
    let span = view.end - view.start;
    if !seconds.is_finite() || span <= 0.0 {
        return None;
    }
    let f = (seconds - view.start) / span;
    (0.0..=1.0).contains(&f).then_some(f)
}

/// Validate a selection against the duration, clamping it into range.
///
/// # Errors
///
/// [`Issue::InvalidRange`] when an edge is not finite or the span is reversed.
pub fn resolve_range(range: Option<Range>, duration: Option<f64>) -> Result<Option<Range>, Issue> {
    let Some(r) = range else { return Ok(None) };
    if !r.start.is_finite() || !r.end.is_finite() {
        return Err(Issue::InvalidRange);
    }
    let known_duration = known(duration);
    let (start, end) = (clamp_time(r.start, known_duration), clamp_time(r.end, known_duration));
    if start > end { Err(Issue::InvalidRange) } else { Ok(Some(Range { start, end })) }
}

/// Move the playhead by `delta` seconds, clamped.
#[must_use]
pub fn step_time(seconds: f64, delta: f64, duration: Option<f64>) -> f64 {
    clamp_time(
        (if seconds.is_finite() { seconds } else { 0.0 }) + (if delta.is_finite() { delta } else { 0.0 }),
        duration,
    )
}

/// Which edge of a selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// The start.
    Start,
    /// The end.
    End,
}

/// Move one selection edge by `delta` seconds without crossing the other.
#[must_use]
pub fn step_edge(range: Range, edge: Edge, delta: f64, duration: Option<f64>) -> Range {
    let Ok(Some(r)) = resolve_range(Some(range), duration) else { return range };
    let shift = if delta.is_finite() { delta } else { 0.0 };
    match edge {
        Edge::Start => Range { start: clamp_time(r.start + shift, duration).min(r.end), end: r.end },
        Edge::End => {
            let upper = known(duration).unwrap_or(f64::INFINITY);
            Range { start: r.start, end: (r.end + shift).min(upper).max(r.start) }
        }
    }
}

/// Everything wrong with the input.
#[must_use]
pub fn diagnose(peaks: Option<&Peaks>, duration: Option<f64>, range: Option<Range>) -> Vec<Issue> {
    let mut issues = Vec::new();
    if known(duration).is_none() {
        issues.push(Issue::InvalidDuration);
    }
    match peaks {
        None => issues.push(Issue::MissingPeaks),
        Some(p) if p.values.is_empty() => issues.push(Issue::MissingPeaks),
        Some(p) => {
            if p.seconds_per_peak().is_none() || p.values.iter().any(|v| !v.is_finite()) {
                issues.push(Issue::InvalidPeaks);
            } else if let (Some(implied), Some(per), Some(d)) =
                (p.implied_duration(), p.seconds_per_peak(), known(duration))
                && (implied - d).abs() > per
            {
                issues.push(Issue::PeakTimeMismatch);
            }
        }
    }
    if resolve_range(range, duration).is_err() {
        issues.push(Issue::InvalidRange);
    }
    issues
}

/// Work out what to draw.
#[must_use]
pub fn project(
    peaks: Option<&Peaks>,
    duration: Option<f64>,
    current: f64,
    range: Option<Range>,
    zoom: f64,
    start: f64,
) -> Projection {
    let duration = known(duration);
    let current = clamp_time(current, duration);
    let view = viewport(duration, zoom, start);
    let issues = diagnose(peaks, duration, range);
    let range = resolve_range(range, duration).ok().flatten();
    let span = view.end - view.start;
    let bars = match peaks {
        Some(p) if !issues.contains(&Issue::InvalidPeaks) && span > 0.0 => p
            .seconds_per_peak()
            .map(|per| {
                p.values
                    .iter()
                    .enumerate()
                    .filter_map(|(index, v)| {
                        #[allow(clippy::cast_precision_loss, reason = "bin counts are far below 2^52")]
                        let (from, to) = (index as f64 * per, (index + 1) as f64 * per);
                        (to > view.start && from < view.end).then(|| Bar {
                            index,
                            left: ((from - view.start) / span).max(0.0),
                            width: ((to.min(view.end) - from.max(view.start)) / span).max(0.0),
                            amplitude: v.abs().clamp(0.0, 1.0),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    let edge = |t: f64| if span > 0.0 { ((t - view.start) / span).clamp(0.0, 1.0) } else { 0.0 };
    Projection {
        duration,
        current,
        playhead: fraction_of(current, &view),
        range,
        range_fractions: range.map(|r| (edge(r.start), edge(r.end))),
        viewport: view,
        bars,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peaks() -> Peaks {
        Peaks::new(vec![0.1, 0.5, 1.0, 0.5], 1000.0, 1000.0)
    }

    #[test]
    fn scale_comes_from_the_peaks() {
        let p = peaks();
        assert_eq!(p.seconds_per_peak(), Some(1.0));
        assert_eq!(p.implied_duration(), Some(4.0));
        assert_eq!(Peaks::new(vec![1.0], 0.0, 44100.0).seconds_per_peak(), None);
    }

    #[test]
    fn viewport_clamps_inside_the_audio() {
        let v = viewport(Some(10.0), 2.0, 9.0);
        assert_eq!((v.start, v.end), (5.0, 10.0));
        assert_eq!(viewport(None, 2.0, 0.0).end, 0.0);
        assert!((time_at(0.5, &v, Some(10.0)) - 7.5).abs() < 1e-9);
        assert_eq!(fraction_of(2.0, &v), None);
    }

    #[test]
    fn range_edges_never_cross() {
        let r = Range { start: 2.0, end: 3.0 };
        assert_eq!(step_edge(r, Edge::Start, 5.0, Some(10.0)).start, 3.0);
        assert_eq!(step_edge(r, Edge::End, -5.0, Some(10.0)).end, 2.0);
        assert_eq!(step_edge(r, Edge::End, 99.0, Some(10.0)).end, 10.0);
        assert!(resolve_range(Some(Range { start: 5.0, end: 1.0 }), Some(10.0)).is_err());
    }

    #[test]
    fn diagnoses_bad_input() {
        let p = peaks();
        assert!(diagnose(Some(&p), Some(4.0), None).is_empty());
        assert_eq!(diagnose(Some(&p), Some(30.0), None), [Issue::PeakTimeMismatch]);
        assert_eq!(diagnose(None, None, None), [Issue::InvalidDuration, Issue::MissingPeaks]);
    }

    #[test]
    fn projects_bars_in_view() {
        let p = peaks();
        let all = project(Some(&p), Some(4.0), 1.0, None, 1.0, 0.0);
        assert_eq!(all.bars.len(), 4);
        assert_eq!(all.playhead, Some(0.25));
        let zoomed = project(Some(&p), Some(4.0), 1.0, Some(Range { start: 1.0, end: 2.0 }), 2.0, 2.0);
        assert_eq!(zoomed.bars.iter().map(|b| b.index).collect::<Vec<_>>(), [2, 3]);
        assert_eq!(zoomed.playhead, None);
        assert_eq!(zoomed.range_fractions, Some((0.0, 0.0)));
    }
}
