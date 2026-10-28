//! KPI maths: the change against a previous value and progress towards a goal.

/// Which way a value moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    /// It went up.
    Up,
    /// It went down.
    Down,
    /// It held.
    Flat,
}

impl Trend {
    /// The `data-trend` value.
    #[must_use]
    pub const fn attr(self) -> &'static str {
        match self {
            Trend::Up => "up",
            Trend::Down => "down",
            Trend::Flat => "flat",
        }
    }
}

/// A change between two readings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Delta {
    /// `value - previous`.
    pub change: f64,
    /// The change as a share of `|previous|`, when previous is non-zero.
    pub ratio: Option<f64>,
    /// Which way it moved.
    pub trend: Trend,
    /// The `data-tone` value: "good", "bad" or "neutral".
    pub tone: &'static str,
}

/// The change from `previous` to `value`. `up_is_good` says which direction is good news.
#[must_use]
pub fn delta(value: f64, previous: f64, up_is_good: bool) -> Delta {
    let change = value - previous;
    let trend = if change.abs() < 1e-12 || !change.is_finite() {
        Trend::Flat
    } else if change > 0.0 {
        Trend::Up
    } else {
        Trend::Down
    };
    let tone = match (trend, up_is_good) {
        (Trend::Flat, _) => "neutral",
        (Trend::Up, true) | (Trend::Down, false) => "good",
        _ => "bad",
    };
    let ratio = (previous != 0.0 && previous.is_finite()).then(|| change / previous.abs());
    Delta { change, ratio, trend, tone }
}

/// Progress towards `goal`, 0..=1 for drawing. Zero for a missing or non-positive goal.
#[must_use]
pub fn progress(value: f64, goal: f64) -> f64 {
    if goal > 0.0 && value.is_finite() { (value / goal).clamp(0.0, 1.0) } else { 0.0 }
}

/// A signed percentage such as "+12.5%" or "−3%" (with a true minus sign).
#[must_use]
pub fn signed_percent(ratio: f64) -> String {
    let p = ratio * 100.0;
    let body = crate::format_value(p.abs());
    if p > 0.0 {
        format!("+{body}%")
    } else if p < 0.0 {
        format!("\u{2212}{body}%")
    } else {
        format!("{body}%")
    }
}

#[allow(clippy::cast_precision_loss, reason = "point counts are small")]
/// Where point `i` of a [`crate::sparkline::layout`] with the same box sits, so a
/// hover marker lands on the line. `None` for a missing or non-finite value.
#[must_use]
pub fn spark_point(values: &[f64], i: usize, width: f64, height: f64, pad: f64) -> Option<(f64, f64)> {
    let value = values.get(i).copied().filter(|v| v.is_finite())?;
    let finite = || values.iter().copied().filter(|v| v.is_finite());
    let (lo, hi) = (finite().fold(f64::INFINITY, f64::min), finite().fold(f64::NEG_INFINITY, f64::max));
    let (lo, hi) = if hi > lo { (lo, hi) } else { (lo - 1.0, lo + 1.0) };
    let len = values.len();
    let x = if len > 1 { pad + (width - 2.0 * pad) * i as f64 / (len - 1) as f64 } else { width / 2.0 };
    Some((x, crate::scale::Linear::new((lo, hi), (height - pad, pad)).map(value)))
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped to the point count"
)]
/// The point nearest a pointer `fraction` (0..=1) across a sparkline of `n` points.
#[must_use]
pub fn spark_index(fraction: f64, n: usize, width: f64, pad: f64) -> Option<usize> {
    let last = n.checked_sub(1)?;
    let inner = (width - 2.0 * pad).max(1.0);
    let t = ((fraction * width - pad) / inner).clamp(0.0, 1.0);
    Some(((t * last as f64).round() as usize).min(last))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spark_points_land_on_the_sparkline() {
        let values = [3.0, 7.0, 5.0];
        let line = crate::sparkline::layout(&values, 120.0, 32.0, 3.0, crate::line::Curve::Linear);
        assert_eq!(spark_point(&values, 2, 120.0, 32.0, 3.0), line.last);
        assert_eq!(spark_point(&values, 1, 120.0, 32.0, 3.0), Some((60.0, 3.0)));
        assert_eq!(spark_index(0.0, 3, 120.0, 3.0), Some(0));
        assert_eq!(spark_index(0.52, 3, 120.0, 3.0), Some(1));
        assert_eq!(spark_index(1.0, 0, 120.0, 3.0), None);
    }

    #[test]
    fn deltas_carry_direction_and_tone() {
        let d = delta(120.0, 100.0, true);
        assert_eq!((d.trend, d.tone), (Trend::Up, "good"));
        assert!((d.ratio.unwrap_or_default() - 0.2).abs() < 1e-9);
        assert_eq!(delta(80.0, 100.0, false).tone, "good");
        assert_eq!(delta(80.0, 100.0, true).tone, "bad");
        assert_eq!(delta(5.0, 5.0, true).trend, Trend::Flat);
        assert_eq!(delta(5.0, 0.0, true).ratio, None);
    }

    #[test]
    fn progress_clamps_and_percent_signs() {
        assert_eq!(progress(150.0, 100.0), 1.0);
        assert_eq!(progress(25.0, 100.0), 0.25);
        assert_eq!(progress(25.0, 0.0), 0.0);
        assert_eq!(signed_percent(0.125), "+12.5%");
        assert_eq!(signed_percent(-0.03), "\u{2212}3%");
    }
}
