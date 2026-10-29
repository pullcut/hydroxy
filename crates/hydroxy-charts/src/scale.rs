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

/// The lowest and highest finite value, or `None` when there are none. Non-finite values are skipped.
#[must_use]
pub fn extent(values: impl IntoIterator<Item = f64>) -> Option<(f64, f64)> {
    values
        .into_iter()
        .filter(|v| v.is_finite())
        .fold(None, |acc, v| Some(acc.map_or((v, v), |(lo, hi): (f64, f64)| (lo.min(v), hi.max(v)))))
}

/// `v` if it is finite and not below zero, else zero: how sizes and flows are cleaned.
#[must_use]
pub fn non_negative(v: f64) -> f64 {
    if v.is_finite() { v.max(0.0) } else { 0.0 }
}

#[allow(clippy::cast_precision_loss, reason = "tick counts are tiny")]
/// Expand `[min, max]` outwards to whole multiples of a nice step, aiming for `count` ticks.
#[must_use]
pub fn nice_domain(min: f64, max: f64, count: usize) -> (f64, f64) {
    let span = if max - min == 0.0 { 1.0 } else { max - min };
    let step = nice_step(span / count.saturating_sub(1).max(1) as f64);
    ((min / step).floor() * step, (max / step).ceil() * step)
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "tick counts are tiny and capped"
)]
/// Evenly spaced nice tick values covering `[min, max]`, including both nice ends.
#[must_use]
pub fn ticks(min: f64, max: f64, count: usize) -> Vec<f64> {
    let count = count.max(2);
    let (lo, hi) = nice_domain(min, max, count);
    let step = nice_step((hi - lo).max(f64::EPSILON) / (count - 1) as f64);
    // Round to the step's decimal places, so 3 × 0.2 reads 0.6, not 0.6000000000000001.
    let places = 10f64.powf((-step.log10().floor()).max(0.0));
    // A flat, sub-ulp or infinite span collapses the step: one tick, not billions.
    let n = ((hi - lo) / step).round();
    if !n.is_finite() || n < 1.0 {
        return if lo.is_finite() { vec![lo] } else { Vec::new() };
    }
    (0..=n.min(1000.0) as usize).map(|i| ((lo + i as f64 * step) * places).round() / places).collect()
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
    fn degenerate_ranges_give_few_ticks_quickly() {
        for (lo, hi) in [(5.0, 5.0), (1.7e9, 1.7e9), (-1e308, 1e308), (1e20, 1e20)] {
            let t = ticks(lo, hi, 5);
            assert!(t.len() <= 4 * 5 + 2, "{lo}..{hi}: {} ticks", t.len());
            assert!(t.iter().all(|v| v.is_finite()));
        }
    }

    #[test]
    fn linear_maps_and_handles_flat_domains() {
        let s = Linear::new((0.0, 100.0), (0.0, 1.0));
        assert!((s.map(25.0) - 0.25).abs() < 1e-12);
        assert!((Linear::new((5.0, 5.0), (0.0, 1.0)).map(5.0)).abs() < 1e-12);
    }
}

/// How values map onto an axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScaleKind {
    /// Even spacing. Starts at zero for bars.
    #[default]
    Linear,
    /// Each power of ten gets the same space. Values at or below zero are clamped to the smallest positive tick.
    Log,
    /// Like log, but linear near zero, so it handles zero and negatives.
    Symlog,
}

impl ScaleKind {
    /// The `data-scale` value.
    #[must_use]
    pub const fn attr(self) -> &'static str {
        match self {
            ScaleKind::Linear => "linear",
            ScaleKind::Log => "log",
            ScaleKind::Symlog => "symlog",
        }
    }
}

fn symlog(v: f64) -> f64 {
    v.signum() * v.abs().ln_1p()
}

/// A value axis of any [`ScaleKind`], mapping its domain onto `0..=1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueScale {
    /// The mapping.
    pub kind: ScaleKind,
    /// The nice domain, as data values.
    pub domain: (f64, f64),
}

impl ValueScale {
    /// A scale over `[min, max]` widened to nice ticks. Log domains snap to powers of ten.
    #[must_use]
    pub fn new(kind: ScaleKind, min: f64, max: f64, tick_count: usize) -> Self {
        let domain = match kind {
            ScaleKind::Log => {
                let lo = if min > 0.0 { min } else { max.clamp(f64::MIN_POSITIVE, 1.0) };
                let hi = max.max(lo * 10.0);
                (10f64.powf(lo.log10().floor()), 10f64.powf(hi.log10().ceil()))
            }
            ScaleKind::Linear | ScaleKind::Symlog => {
                let t = ticks(min, max, tick_count);
                (t.first().copied().unwrap_or(min), t.last().copied().unwrap_or(max))
            }
        };
        Self { kind, domain }
    }

    fn transform(self, v: f64) -> f64 {
        match self.kind {
            ScaleKind::Linear => v,
            ScaleKind::Log => v.max(self.domain.0).log10(),
            ScaleKind::Symlog => symlog(v),
        }
    }

    /// Where `value` sits, 0 at the bottom of the domain and 1 at the top.
    #[must_use]
    pub fn map(self, value: f64) -> f64 {
        let (a, b) = (self.transform(self.domain.0), self.transform(self.domain.1));
        if (b - a).abs() < f64::EPSILON { 0.0 } else { (self.transform(value) - a) / (b - a) }
    }

    /// Tick values for the axis.
    #[allow(clippy::cast_possible_truncation, reason = "decade exponents are small")]
    #[must_use]
    pub fn ticks(self, count: usize) -> Vec<f64> {
        match self.kind {
            // The domain is already nice, but re-deriving the step can overshoot it; keep ticks inside.
            ScaleKind::Linear => {
                let eps = (self.domain.1 - self.domain.0).abs() * 1e-9;
                ticks(self.domain.0, self.domain.1, count)
                    .into_iter()
                    .filter(|t| *t >= self.domain.0 - eps && *t <= self.domain.1 + eps)
                    .collect()
            }
            ScaleKind::Log => {
                let (lo, hi) = (self.domain.0.log10().round() as i32, self.domain.1.log10().round() as i32);
                // Thin out decades so long ranges still get about `count` ticks.
                let every = usize::try_from(hi - lo).unwrap_or(0).div_ceil(count.max(2) - 1).max(1);
                (lo..=hi).step_by(every).map(|p| 10f64.powi(p)).collect()
            }
            ScaleKind::Symlog => {
                // Zero plus ±1, ±10, ±100… within the domain.
                let mut out = vec![0.0];
                let mut p = 1.0;
                while p <= self.domain.1.abs().max(self.domain.0.abs()) * 1.0001 {
                    if p <= self.domain.1 {
                        out.push(p);
                    }
                    if -p >= self.domain.0 {
                        out.insert(0, -p);
                    }
                    p *= 10.0;
                }
                out
            }
        }
    }
}

/// Days since 1970-01-01 → (year, month 1..=12, day 1..=31). Howard Hinnant's civil-from-days.
#[must_use]
pub fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = u32::try_from(doy - (153 * mp + 2) / 5 + 1).unwrap_or(1);
    let m = u32::try_from(if mp < 10 { mp + 3 } else { mp - 9 }).unwrap_or(1);
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// (year, month 1..=12, day) → days since 1970-01-01.
#[must_use]
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = i64::from(if m > 2 { m - 3 } else { m + 9 });
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// A UTC timestamp (seconds) as "3 Oct 2026", with "14:30" or "14:30:05" added when it
/// isn't midnight.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, reason = "epoch seconds fit easily")]
#[must_use]
pub fn date_time(secs: f64) -> String {
    let days = (secs / 86_400.0).floor() as i64;
    let (y, m, d) = civil(days);
    let date = format!("{d} {} {y}", MONTHS[(m - 1) as usize]);
    let t = (secs - days as f64 * 86_400.0).round() as i64;
    match t {
        0 => date,
        _ if t % 60 == 0 => format!("{date} {:02}:{:02}", t / 3600, t / 60 % 60),
        _ => format!("{date} {:02}:{:02}:{:02}", t / 3600, t / 60 % 60, t % 60),
    }
}

/// A tick on a time axis.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeTick {
    /// Seconds since the Unix epoch, UTC.
    pub at: f64,
    /// Short label sized to the interval: "14:30", "3 Oct", "Oct 2026", "2026".
    pub label: String,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, reason = "epoch seconds fit easily")]
/// Calendar-aligned ticks between two UTC timestamps (seconds), aiming for about `count`.
/// Steps run from one second to whole years, always on round boundaries.
#[must_use]
pub fn time_ticks(start: f64, end: f64, count: usize) -> Vec<TimeTick> {
    const DAY: f64 = 86_400.0;
    let (start, end) = if start <= end { (start, end) } else { (end, start) };
    let target = (end - start) / count.max(2) as f64;
    let fixed = [
        1.0,
        5.0,
        15.0,
        30.0,
        60.0,
        300.0,
        900.0,
        1800.0,
        3600.0,
        10_800.0,
        21_600.0,
        43_200.0,
        DAY,
        2.0 * DAY,
        7.0 * DAY,
    ];
    if let Some(&step) = fixed.iter().find(|&&s| s >= target) {
        let mut out = Vec::new();
        // Weeks start on Monday (1970-01-01 was a Thursday, so shift by 4 days).
        let offset = if (step - 7.0 * DAY).abs() < 1.0 { -4.0 * DAY } else { 0.0 };
        let mut t = ((start - offset) / step).ceil() * step + offset;
        while t <= end && out.len() < 500 {
            let days = (t / DAY).floor() as i64;
            let (_, m, d) = civil(days);
            let label = if step < DAY {
                let secs = t.rem_euclid(DAY) as i64;
                if step < 60.0 {
                    format!("{:02}:{:02}:{:02}", secs / 3600, secs / 60 % 60, secs % 60)
                } else {
                    format!("{:02}:{:02}", secs / 3600, secs / 60 % 60)
                }
            } else {
                format!("{d} {}", MONTHS[(m - 1) as usize])
            };
            out.push(TimeTick { at: t, label });
            t += step;
        }
        return out;
    }
    // Months or years: step the calendar, not seconds.
    let months = [1u32, 2, 3, 6, 12, 24, 60, 120, 600];
    let step = months.iter().copied().find(|&s| f64::from(s) * 30.44 * DAY >= target).unwrap_or(1200);
    let (y0, m0, d0) = civil((start / DAY).floor() as i64);
    let mut index = y0 * 12 + i64::from(m0 - 1) + i64::from(d0 > 1 || start.rem_euclid(DAY) > 0.0);
    index = index.div_euclid(i64::from(step)) * i64::from(step)
        + if index % i64::from(step) == 0 { 0 } else { i64::from(step) };
    let mut out = Vec::new();
    loop {
        let (y, m) = (index.div_euclid(12), u32::try_from(index.rem_euclid(12)).unwrap_or(0) + 1);
        let t = days_from_civil(y, m, 1) as f64 * DAY;
        if t > end || out.len() >= 500 {
            break;
        }
        let label = if step >= 12 { y.to_string() } else { format!("{} {y}", MONTHS[(m - 1) as usize]) };
        out.push(TimeTick { at: t, label });
        index += i64::from(step);
    }
    out
}

#[cfg(test)]
mod more_tests {
    use super::*;

    #[test]
    fn extent_skips_non_finite_and_empty_is_none() {
        assert_eq!(extent([3.0, f64::NAN, -1.0, f64::INFINITY, 2.0]), Some((-1.0, 3.0)));
        assert_eq!(extent([f64::NAN]), None);
        assert_eq!(extent(std::iter::empty()), None);
    }

    #[test]
    fn non_negative_clamps_and_zeroes_bad_values() {
        assert!((non_negative(2.5) - 2.5).abs() < 1e-12);
        assert!(non_negative(-1.0).abs() < 1e-12);
        assert!(non_negative(f64::NAN).abs() < 1e-12);
        assert!(non_negative(f64::INFINITY).abs() < 1e-12);
    }

    #[test]
    fn date_times_drop_midnight() {
        assert_eq!(date_time(0.0), "1 Jan 1970");
        assert_eq!(date_time(86_400.0 * 365.0 + 3_600.0 * 14.5), "1 Jan 1971 14:30");
        assert_eq!(date_time(5.0), "1 Jan 1970 00:00:05");
    }

    #[test]
    fn log_scale_spaces_decades_evenly() {
        let s = ValueScale::new(ScaleKind::Log, 3.0, 4000.0, 5);
        assert_eq!(s.domain, (1.0, 10_000.0));
        assert!((s.map(100.0) - 0.5).abs() < 1e-9);
        assert_eq!(s.ticks(5), vec![1.0, 10.0, 100.0, 1000.0, 10_000.0]);
        assert_eq!(s.map(0.0), 0.0, "non-positive values clamp to the bottom");
    }

    #[test]
    fn linear_ticks_stay_inside_the_domain() {
        let s = ValueScale::new(ScaleKind::Linear, 198.0, 224.0, 5);
        let t = s.ticks(5);
        assert!(t.iter().all(|v| *v >= s.domain.0 && *v <= s.domain.1), "{t:?} outside {:?}", s.domain);
    }

    #[test]
    fn symlog_handles_zero_and_negatives() {
        let s = ValueScale::new(ScaleKind::Symlog, -100.0, 1000.0, 5);
        assert!(s.map(0.0) > 0.0 && s.map(0.0) < 1.0);
        assert!(s.map(-100.0) < s.map(0.0));
        assert!(s.ticks(5).contains(&0.0) && s.ticks(5).contains(&-100.0));
    }

    #[test]
    fn civil_round_trips() {
        for days in [-1000, 0, 19_000, 20_754] {
            let (y, m, d) = civil(days);
            assert_eq!(days_from_civil(y, m, d), days);
        }
        assert_eq!(civil(0), (1970, 1, 1));
    }

    #[test]
    fn time_ticks_pick_calendar_steps() {
        let day = 86_400.0;
        let t = time_ticks(0.0, 10.0 * day, 5);
        assert_eq!(t[0].label, "1 Jan");
        assert!((t[1].at - t[0].at - 2.0 * day).abs() < 1.0);
        let y = time_ticks(0.0, 365.0 * 6.0 * day, 6);
        assert_eq!(y[0].label, "1970");
        assert_eq!(y[1].label, "1971");
        let m = time_ticks(0.0, 200.0 * day, 6);
        assert_eq!(m[1].label, "Mar 1970", "about 33 days per tick rounds up to two months");
        let h = time_ticks(0.0, 6.0 * 3600.0, 6);
        assert_eq!(h[1].label, "01:00");
    }
}
