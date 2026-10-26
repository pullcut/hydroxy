//! Linear scales and "nice" ticks (1, 2, 5 × 10ⁿ steps), ported from hecto's chart-core.

/// The nearest 1, 2, 5 or 10 × 10ⁿ step to `raw`.
#[must_use]
pub fn nice_step(raw: f64) -> f64 {
    let raw = raw.abs();
    if raw == 0.0 || !raw.is_finite() {
        return 1.0;
    }
    let power = 10f64.powf(raw.log10().floor());
    let error = raw / power;
    let factor = if error >= 7.5 {
        10.0
    } else if error >= 3.5 {
        5.0
    } else if error >= 1.5 {
        2.0
    } else {
        1.0
    };
    factor * power
}

#[allow(clippy::cast_precision_loss, reason = "tick counts are tiny")]
/// Expand `[min, max]` outwards to whole multiples of a nice step, aiming for `count` ticks.
#[must_use]
pub fn nice_domain(min: f64, max: f64, count: usize) -> (f64, f64) {
    let span = if max - min == 0.0 { 1.0 } else { max - min };
    let step = nice_step(span / count.saturating_sub(1).max(1) as f64);
    ((min / step).floor() * step, (max / step).ceil() * step)
}

#[allow(clippy::cast_precision_loss, reason = "tick counts are tiny")]
/// Evenly spaced nice tick values covering `[min, max]`, including both nice ends.
#[must_use]
pub fn ticks(min: f64, max: f64, count: usize) -> Vec<f64> {
    let count = count.max(2);
    let (lo, hi) = nice_domain(min, max, count);
    let step = nice_step((hi - lo).max(f64::EPSILON) / (count - 1) as f64);
    // Round to the step's decimal places, so 3 × 0.2 reads 0.6, not 0.6000000000000001.
    let places = 10f64.powf((-step.log10().floor()).max(0.0));
    let mut out = Vec::new();
    let mut i = 0.0;
    while lo + i * step <= hi + step * 0.5 {
        out.push(((lo + i * step) * places).round() / places);
        i += 1.0;
    }
    out
}

/// Maps a numeric domain onto a numeric range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Linear {
    /// Input interval.
    pub domain: (f64, f64),
    /// Output interval.
    pub range: (f64, f64),
}

impl Linear {
    /// A scale from `domain` to `range`.
    #[must_use]
    pub const fn new(domain: (f64, f64), range: (f64, f64)) -> Self {
        Self { domain, range }
    }

    /// The range position of `value`.
    #[must_use]
    pub fn map(self, value: f64) -> f64 {
        let span = self.domain.1 - self.domain.0;
        let ratio = if span == 0.0 { 0.0 } else { (value - self.domain.0) / span };
        self.range.0 + ratio * (self.range.1 - self.range.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_snap_to_one_two_five() {
        assert!((nice_step(0.23) - 0.2).abs() < 1e-12);
        assert!((nice_step(37.0) - 50.0).abs() < 1e-12);
        assert!((nice_step(8.0) - 10.0).abs() < 1e-12);
    }

    #[test]
    fn ticks_cover_the_data_with_round_numbers() {
        assert_eq!(ticks(0.0, 87.0, 5), vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
        assert_eq!(ticks(0.0, 1.0, 5), vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
    }

    #[test]
    fn linear_maps_and_handles_flat_domains() {
        let s = Linear::new((0.0, 100.0), (0.0, 1.0));
        assert!((s.map(25.0) - 0.25).abs() < 1e-12);
        assert!((Linear::new((5.0, 5.0), (0.0, 1.0)).map(5.0)).abs() < 1e-12);
    }
}
