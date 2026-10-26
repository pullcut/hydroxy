//! Progress bar connector. Anatomy: root, header, label, value, track, indicator.
//! `track` carries `role="progressbar"` and the value attributes; a `None` value
//! means indeterminate (work is happening, amount unknown).

/// The `data-scope` value on every progress bar part.
pub const SCOPE: &str = "progress-bar";

/// Attributes for the `progressbar` element, plus the indicator width.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressAttrs {
    /// Always `"progressbar"`.
    pub role: &'static str,
    /// `aria-valuemax`. The `max` passed in, or 100 if it was not a positive number.
    pub aria_valuemax: f64,
    /// `aria-valuenow`, clamped to `0..=max`. `None` when indeterminate.
    pub aria_valuenow: Option<f64>,
    /// `aria-valuetext`, also shown as the visible value: the caller's text, or a rounded percentage.
    pub aria_valuetext: Option<String>,
    /// Indicator width in percent. `None` when indeterminate.
    pub percent: Option<f64>,
    /// `data-state`: `indeterminate`, `loading` or `complete`.
    pub data_state: &'static str,
}

/// Attributes for a progress bar at `value` out of `max`.
#[must_use]
pub fn progress_attrs(value: Option<f64>, max: f64, value_text: Option<&str>) -> ProgressAttrs {
    let max = if max.is_finite() && max > 0.0 { max } else { 100.0 };
    let now = value.filter(|v| v.is_finite()).map(|v| v.clamp(0.0, max));
    let percent = now.map(|v| v / max * 100.0);
    let aria_valuetext = match (value_text, percent) {
        (Some(text), _) => Some(text.to_string()),
        (None, Some(p)) => Some(format!("{p:.0}%")),
        (None, None) => None,
    };
    let data_state = match now {
        None => "indeterminate",
        Some(v) if v >= max => "complete",
        Some(_) => "loading",
    };
    ProgressAttrs { role: "progressbar", aria_valuemax: max, aria_valuenow: now, aria_valuetext, percent, data_state }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_and_reports_percent() {
        let a = progress_attrs(Some(30.0), 24.0, None);
        assert_eq!(a.aria_valuenow, Some(24.0));
        assert_eq!(a.percent, Some(100.0));
        assert_eq!(a.data_state, "complete");
        assert_eq!(progress_attrs(Some(-5.0), 100.0, None).aria_valuenow, Some(0.0));
    }

    #[test]
    fn default_text_is_rounded_percent_and_caller_text_wins() {
        assert_eq!(progress_attrs(Some(2.0), 3.0, None).aria_valuetext.as_deref(), Some("67%"));
        assert_eq!(progress_attrs(Some(22.0), 24.0, Some("22 of 24")).aria_valuetext.as_deref(), Some("22 of 24"));
    }

    #[test]
    fn none_or_nan_is_indeterminate() {
        for value in [None, Some(f64::NAN)] {
            let a = progress_attrs(value, 100.0, None);
            assert_eq!(a.data_state, "indeterminate");
            assert_eq!(a.percent, None);
            assert_eq!(a.aria_valuetext, None);
        }
    }

    #[test]
    fn bad_max_falls_back_to_100() {
        assert!((progress_attrs(Some(50.0), 0.0, None).aria_valuemax - 100.0).abs() < f64::EPSILON);
        assert_eq!(progress_attrs(Some(50.0), f64::NAN, None).percent, Some(50.0));
    }
}
