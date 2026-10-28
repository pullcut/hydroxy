//! Kaplan–Meier survival curves: the share of subjects still event-free over time, from
//! (time, event or censored) observations, with Greenwood confidence bands and the
//! number at risk.
#![allow(clippy::many_single_char_names, clippy::cast_precision_loss, reason = "short names follow the maths; group sizes are far below 2^52")]

use std::fmt::Write as _;

use crate::plot::ValueAxis;
use crate::qq::normal_quantile;
use crate::scale::ScaleKind;

/// One subject: when they were last seen, and whether the event happened then (`true`) or
/// they were censored, still event-free (`false`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Observation {
    /// Follow-up time.
    pub time: f64,
    /// The event happened at `time`; `false` means censored.
    pub event: bool,
}

impl Observation {
    /// An event at `time`.
    #[must_use]
    pub const fn event(time: f64) -> Self {
        Self { time, event: true }
    }

    /// Censored (still event-free) at `time`.
    #[must_use]
    pub const fn censored(time: f64) -> Self {
        Self { time, event: false }
    }
}

/// A named group of subjects, such as one treatment arm.
#[derive(Debug, Clone, PartialEq)]
pub struct SurvivalGroup {
    /// The group's name, for the legend.
    pub name: String,
    /// The subjects, in any order.
    pub observations: Vec<Observation>,
}

impl SurvivalGroup {
    /// A group from `(time, event)` pairs; `event` is `false` for censored subjects.
    #[must_use]
    pub fn new(name: impl Into<String>, observations: impl IntoIterator<Item = (f64, bool)>) -> Self {
        Self { name: name.into(), observations: observations.into_iter().map(|(time, event)| Observation { time, event }).collect() }
    }
}

/// The estimate just after one distinct time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    /// The time.
    pub time: f64,
    /// Subjects at risk just before `time`.
    pub at_risk: usize,
    /// Events at `time`.
    pub events: usize,
    /// Subjects censored at `time`.
    pub censored: usize,
    /// The Kaplan–Meier survival estimate after `time`.
    pub survival: f64,
    /// Greenwood standard error of `survival`.
    pub se: f64,
    /// 95% interval of `survival` (log–log transformed, so it stays within 0..=1).
    pub ci: (f64, f64),
}

/// The Kaplan–Meier estimate for one group: one [`Step`] per distinct time, ascending.
/// Observations with a non-finite or negative time are dropped.
#[must_use]
pub fn kaplan_meier(observations: &[Observation]) -> Vec<Step> {
    let mut obs: Vec<Observation> = observations.iter().copied().filter(|o| o.time.is_finite() && o.time >= 0.0).collect();
    obs.sort_by(|a, b| a.time.total_cmp(&b.time));
    let z = normal_quantile(0.975);
    let (mut at_risk, mut survival, mut greenwood) = (obs.len(), 1.0_f64, 0.0_f64);
    let mut steps = Vec::new();
    let mut i = 0;
    while i < obs.len() {
        let time = obs[i].time;
        let same = obs[i..].iter().take_while(|o| o.time.total_cmp(&time).is_eq()).count();
        let events = obs[i..i + same].iter().filter(|o| o.event).count();
        let (n, d) = (at_risk as f64, events as f64);
        if events > 0 {
            survival *= 1.0 - d / n;
            if at_risk > events {
                greenwood += d / (n * (n - d));
            }
        }
        let se = survival * greenwood.sqrt();
        let ci = if survival <= 0.0 {
            (0.0, 0.0)
        } else if survival >= 1.0 || greenwood == 0.0 {
            (survival, survival)
        } else {
            // Kalbfleisch–Prentice: a normal interval for log(−log S), mapped back.
            let spread = z * greenwood.sqrt() / survival.ln().abs();
            (survival.powf(spread.exp()), survival.powf((-spread).exp()))
        };
        steps.push(Step { time, at_risk, events, censored: same - events, survival, se, ci });
        at_risk -= same;
        i += same;
    }
    steps
}

/// The survival estimate in force at `time` (1 before the first step).
#[must_use]
pub fn survival_at(steps: &[Step], time: f64) -> Option<&Step> {
    steps.iter().take_while(|s| s.time <= time).last()
}

/// How many subjects are still at risk at `time`: those whose follow-up reaches it.
#[must_use]
pub fn at_risk(observations: &[Observation], time: f64) -> usize {
    observations.iter().filter(|o| o.time.is_finite() && o.time >= time).count()
}

/// One group's curve, ready to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct Curve {
    /// The group's name.
    pub name: String,
    /// The estimate at each distinct time.
    pub steps: Vec<Step>,
    /// SVG `d` of the step curve, in the 100×100 box.
    pub line: String,
    /// SVG `d` of the confidence band, in the 100×100 box. Empty when there are no events.
    pub band: String,
    /// Censoring marks as (x, y) plot fractions, y from the bottom.
    pub censors: Vec<(f64, f64)>,
    /// Median survival time: the first time the estimate reaches 0.5 or below.
    pub median: Option<f64>,
}

/// Everything a survival chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct SurvivalLayout {
    /// One curve per group, in input order.
    pub curves: Vec<Curve>,
    /// The time (x) axis, from zero.
    pub x_axis: ValueAxis,
    /// The survival (y) axis, 0..=1.
    pub axis: ValueAxis,
    /// Every distinct time across groups, ascending, for the crosshair.
    pub times: Vec<f64>,
    /// At-risk counts per group (outer) at each x tick (inner).
    pub risk: Vec<Vec<usize>>,
}

/// Lay out a Kaplan–Meier curve per group (the first six), with time running from zero to the latest
/// observation.
#[must_use]
pub fn layout(groups: &[SurvivalGroup]) -> SurvivalLayout {
    let all = groups.iter().flat_map(|g| g.observations.iter().map(|o| o.time)).filter(|t| t.is_finite() && *t >= 0.0);
    let x_axis = ValueAxis::new(ScaleKind::Linear, all.clone().chain([0.0]), &[], true);
    let axis = ValueAxis::over(ScaleKind::Linear, 0.0, 1.0);
    let end = x_axis.scale.domain.1.max(all.clone().fold(0.0, f64::max));
    let px = |t: f64| x_axis.scale.map(t) * 100.0;
    let py = |s: f64| (1.0 - axis.scale.map(s)) * 100.0;
    let curves = groups
        .iter()
        .take(6)
        .map(|g| {
            let steps = kaplan_meier(&g.observations);
            let last = steps.last().map_or(0.0, |s| s.time);
            let mut line = format!("M{:.3} {:.3}", px(0.0), py(1.0));
            let (mut upper, mut lower) = (vec![(0.0, 1.0)], vec![(0.0, 1.0)]);
            let mut ci_prev = (1.0, 1.0);
            for s in &steps {
                if s.events > 0 {
                    let _ = write!(line, "H{:.3}V{:.3}", px(s.time), py(s.survival));
                    upper.extend([(s.time, ci_prev.1), (s.time, s.ci.1)]);
                    lower.extend([(s.time, ci_prev.0), (s.time, s.ci.0)]);
                    ci_prev = s.ci;
                }
            }
            let _ = write!(line, "H{:.3}", px(last));
            upper.push((last, ci_prev.1));
            lower.push((last, ci_prev.0));
            let mut band = String::new();
            if steps.iter().any(|s| s.events > 0) {
                for (k, &(t, v)) in upper.iter().enumerate() {
                    let _ = write!(band, "{}{:.3} {:.3}", if k == 0 { 'M' } else { 'L' }, px(t), py(v));
                }
                for &(t, v) in lower.iter().rev() {
                    let _ = write!(band, "L{:.3} {:.3}", px(t), py(v));
                }
                band.push('Z');
            }
            let censors = steps.iter().filter(|s| s.censored > 0).map(|s| (x_axis.scale.map(s.time), axis.scale.map(s.survival))).collect();
            let median = steps.iter().find(|s| s.survival <= 0.5).map(|s| s.time);
            Curve { name: g.name.clone(), steps, line, band, censors, median }
        })
        .collect();
    let mut times: Vec<f64> = groups.iter().flat_map(|g| g.observations.iter().map(|o| o.time)).filter(|t| t.is_finite() && *t >= 0.0).chain([0.0]).collect();
    times.sort_by(f64::total_cmp);
    times.dedup();
    times.retain(|&t| t <= end);
    let risk = groups.iter().take(6).map(|g| x_axis.gridlines.iter().map(|tick| at_risk(&g.observations, tick.value)).collect()).collect();
    SurvivalLayout { curves, x_axis, axis, times, risk }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 6-MP arm of Freireich et al. (1963), the remission-time data in Gehan (1965)
    /// and most survival-analysis textbooks; censored times are negative here.
    const SIX_MP: [f64; 21] = [6.0, 6.0, 6.0, -6.0, 7.0, -9.0, 10.0, -10.0, -11.0, 13.0, 16.0, -17.0, -19.0, -20.0, 22.0, 23.0, -25.0, -32.0, -32.0, -34.0, -35.0];

    fn six_mp() -> Vec<Observation> {
        SIX_MP.iter().map(|&t| Observation { time: t.abs(), event: t > 0.0 }).collect()
    }

    #[test]
    fn kaplan_meier_matches_the_textbook_table() {
        let steps = kaplan_meier(&six_mp());
        let events: Vec<&Step> = steps.iter().filter(|s| s.events > 0).collect();
        // (time, at risk, events, S(t), Greenwood SE) as tabulated in Klein & Moeschberger, Table 4.1A.
        let expected = [
            (6.0, 21, 3, 0.857, 0.076),
            (7.0, 17, 1, 0.807, 0.087),
            (10.0, 15, 1, 0.753, 0.096),
            (13.0, 12, 1, 0.690, 0.107),
            (16.0, 11, 1, 0.627, 0.114),
            (22.0, 7, 1, 0.538, 0.128),
            (23.0, 6, 1, 0.448, 0.135),
        ];
        assert_eq!(events.len(), expected.len());
        for (s, &(t, n, d, surv, se)) in events.iter().zip(&expected) {
            assert_eq!((s.time, s.at_risk, s.events), (t, n, d));
            assert!((s.survival - surv).abs() < 5e-4, "S({t}) = {}", s.survival);
            assert!((s.se - se).abs() < 5e-4, "SE({t}) = {}", s.se);
            assert!(s.ci.0 < s.survival && s.survival < s.ci.1);
        }
    }

    #[test]
    fn censoring_and_risk_counts() {
        let obs = six_mp();
        let steps = kaplan_meier(&obs);
        assert_eq!(steps[0].censored, 1, "one censored at 6");
        assert_eq!(at_risk(&obs, 0.0), 21);
        assert_eq!(at_risk(&obs, 10.0), 15);
        assert!((survival_at(&steps, 12.0).unwrap().survival - 0.753).abs() < 5e-4);
        assert!(survival_at(&steps, 1.0).is_none());
    }

    #[test]
    fn layout_draws_steps_from_one() {
        let l = layout(&[SurvivalGroup { name: "6-MP".into(), observations: six_mp() }, SurvivalGroup::new("Empty", [])]);
        let c = &l.curves[0];
        assert!(c.line.starts_with("M0.000 0.000H"));
        assert!(c.band.ends_with('Z'));
        assert_eq!(c.censors.len(), 11, "distinct censoring times");
        assert_eq!(c.median, Some(23.0));
        assert_eq!(l.curves[1].band, "");
        assert_eq!(l.risk[0][0], 21);
        assert_eq!(l.times[0], 0.0);
        assert_eq!(l.x_axis.scale.domain.0, 0.0);
    }

    #[test]
    fn everyone_dying_reaches_zero() {
        let s = kaplan_meier(&[Observation::event(1.0), Observation::event(2.0)]);
        assert!((s[0].survival - 0.5).abs() < 1e-12);
        assert_eq!(s[1].survival, 0.0);
        assert_eq!(s[1].ci, (0.0, 0.0));
    }
}
