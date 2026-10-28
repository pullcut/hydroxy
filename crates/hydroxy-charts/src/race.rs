//! Bar chart races: the top entries ranked at each frame, interpolated between frames so
//! bars can glide past each other.

use crate::line::Series;
use crate::scale;

/// One entry's bar at a moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaceBar {
    /// The value, interpolated between frames. Gaps count as zero.
    pub value: f64,
    /// Position in the ranking, 0 = largest. Ties keep input order.
    pub rank: usize,
    /// Bar length as a share of the axis, 0..=1.
    pub fraction: f64,
}

/// Everything a bar race needs to draw at one moment.
#[derive(Debug, Clone, PartialEq)]
pub struct RaceFrame {
    /// One per series, in input order (so keyed markup keeps its elements and can glide).
    pub bars: Vec<RaceBar>,
    /// The axis maximum: the leader's value, so the leader always spans the plot.
    pub max: f64,
    /// Axis ticks as `(value, fraction)`.
    pub ticks: Vec<(f64, f64)>,
    /// The frame whose label to show: the nearest whole frame.
    pub frame: usize,
}

/// Each series' value at `at`, a position in frames (2.5 is halfway between frames 2 and 3).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "frame positions are small and clamped")]
#[must_use]
pub fn values_at(series: &[Series], frames: usize, at: f64) -> Vec<f64> {
    let last = frames.saturating_sub(1);
    #[allow(clippy::cast_precision_loss, reason = "frame counts are small")]
    let at = at.clamp(0.0, last as f64);
    let (i, t) = (at.floor() as usize, at.fract());
    let j = (i + 1).min(last);
    let get = |s: &Series, k: usize| s.values.get(k).copied().filter(|v| v.is_finite()).unwrap_or(0.0);
    series.iter().map(|s| get(s, i) + (get(s, j) - get(s, i)) * t).collect()
}

/// The race at `at` frames in, with about `tick_count` axis ticks.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss, reason = "frame positions are small and clamped")]
#[must_use]
pub fn race(series: &[Series], frames: usize, at: f64, tick_count: usize) -> RaceFrame {
    let values = values_at(series, frames, at);
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|&a, &b| values[b].total_cmp(&values[a]).then(a.cmp(&b)));
    let mut ranks = vec![0; values.len()];
    for (rank, &i) in order.iter().enumerate() {
        ranks[i] = rank;
    }
    let lead = order.first().map_or(0.0, |&i| values[i]);
    let max = if lead > 0.0 { lead } else { 1.0 };
    let bars = values.iter().zip(&ranks).map(|(&value, &rank)| RaceBar { value, rank, fraction: (value / max).clamp(0.0, 1.0) }).collect();
    let ticks = scale::ticks(0.0, max, tick_count).into_iter().filter(|&t| t <= max * (1.0 + 1e-9)).map(|t| (t, t / max)).collect();
    let frame = (at.clamp(0.0, frames.saturating_sub(1) as f64).round() as usize).min(frames.saturating_sub(1));
    RaceFrame { bars, max, ticks, frame }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series() -> Vec<Series> {
        vec![Series::new("A", [10.0, 20.0, 30.0]), Series::new("B", [30.0, 20.0, 10.0]), Series::new("C", [5.0, f64::NAN, 40.0])]
    }

    #[test]
    fn interpolates_between_frames() {
        let v = values_at(&series(), 3, 0.5);
        assert_eq!(v, vec![15.0, 25.0, 2.5]);
        assert_eq!(values_at(&series(), 3, 9.0), vec![30.0, 10.0, 40.0], "clamped to the last frame");
    }

    #[test]
    fn ranks_and_scales_to_the_leader() {
        let r = race(&series(), 3, 0.0, 4);
        assert_eq!(r.bars.iter().map(|b| b.rank).collect::<Vec<_>>(), vec![1, 0, 2]);
        assert!((r.max - 30.0).abs() < 1e-12);
        assert!((r.bars[1].fraction - 1.0).abs() < 1e-12);
        assert!(r.ticks.iter().all(|&(v, f)| v <= 30.0 && (0.0..=1.0).contains(&f)));
        let mid = race(&series(), 3, 1.0, 4);
        assert_eq!(mid.bars[0].rank, 0, "ties keep input order");
        assert_eq!(mid.bars[1].rank, 1);
        assert_eq!(race(&series(), 3, 1.6, 4).frame, 2);
    }

    #[test]
    fn empty_races_do_not_panic() {
        let r = race(&[], 0, 0.0, 4);
        assert_eq!(r.bars, Vec::new());
        assert_eq!(r.frame, 0);
    }
}
