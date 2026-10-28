//! Range slider connector. Anatomy: root, header, label, value, track, range
//! (the filled band) and two thumbs. Every part carries
//! `data-scope="range-slider"` and its `data-part`.
//!
//! Each thumb is a `role="slider"` whose `aria-valuemin` / `aria-valuemax`
//! stop at the other thumb, so assistive tech announces the real limits.

pub use hydroxy_machines::range_slider::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "range-slider";

/// Attributes for one thumb.
#[derive(Debug, Clone, PartialEq)]
pub struct ThumbAttrs {
    /// Always `"slider"`.
    pub role: &'static str,
    /// `0`, or `-1` while disabled.
    pub tabindex: i32,
    /// Lowest value this thumb can take now.
    pub aria_valuemin: f64,
    /// Highest value this thumb can take now.
    pub aria_valuemax: f64,
    /// The thumb's value.
    pub aria_valuenow: f64,
    /// `aria-disabled`.
    pub aria_disabled: bool,
    /// `data-thumb`: `"min"` or `"max"`.
    pub data_thumb: &'static str,
    /// `data-dragging` while this thumb is being dragged.
    pub dragging: bool,
    /// Position along the track, `0..=100`.
    pub percent: f64,
}

/// How far `v` is from `min` to `max`, as a percentage.
#[must_use]
pub fn percent(v: f64, min: f64, max: f64) -> f64 {
    if max > min { ((v - min) / (max - min) * 100.0).clamp(0.0, 100.0) } else { 0.0 }
}

/// Attributes for thumb `thumb` (`0` low, `1` high).
#[must_use]
pub fn thumb_attrs(state: &RangeSliderState, input: &RangeSliderInput, thumb: Thumb) -> ThumbAttrs {
    let (lo, hi) = input.bounds(state.value, thumb);
    ThumbAttrs {
        role: "slider",
        tabindex: if input.disabled { -1 } else { 0 },
        aria_valuemin: lo,
        aria_valuemax: hi,
        aria_valuenow: state.value[thumb],
        aria_disabled: input.disabled,
        data_thumb: if thumb == 0 { "min" } else { "max" },
        dragging: state.dragging == Some(thumb),
        percent: percent(state.value[thumb], input.min, input.max),
    }
}

/// `left` / `width` of the filled band, as an inline style.
#[must_use]
pub fn range_style(state: &RangeSliderState, input: &RangeSliderInput) -> String {
    let a = percent(state.value[0], input.min, input.max);
    let b = percent(state.value[1], input.min, input.max);
    format!("left: {a}%; width: {}%;", b - a)
}

/// The value under a pointer at client x `x`, for a track at `left` with `width`.
#[must_use]
pub fn value_from_pointer(x: f64, left: f64, width: f64, input: &RangeSliderInput) -> f64 {
    let ratio = if width > 0.0 { ((x - left) / width).clamp(0.0, 1.0) } else { 0.0 };
    input.min + ratio * (input.max - input.min)
}

/// Map a `KeyboardEvent.key` on thumb `thumb` to an event. `Some` means: prevent default.
#[must_use]
pub fn keydown(key: &str, thumb: Thumb, state: &RangeSliderState, input: &RangeSliderInput) -> Option<RangeSliderEvent> {
    let v = state.value[thumb];
    let page = input.step * 10.0;
    let target = match key {
        "ArrowRight" | "ArrowUp" => v + input.step,
        "ArrowLeft" | "ArrowDown" => v - input.step,
        "PageUp" => v + page,
        "PageDown" => v - page,
        "Home" => input.min,
        "End" => input.max,
        _ => return None,
    };
    Some(RangeSliderEvent::Set(thumb, target, ChangeReason::Keyboard))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbs_report_bounds_from_the_other_thumb() {
        let input = RangeSliderInput { min_gap: 5.0, ..RangeSliderInput::default() };
        let state = RangeSliderState { value: [20.0, 60.0], dragging: None, controlled: false };
        let low = thumb_attrs(&state, &input, 0);
        assert_eq!((low.aria_valuemin, low.aria_valuemax, low.data_thumb), (0.0, 55.0, "min"));
        let high = thumb_attrs(&state, &input, 1);
        assert_eq!((high.aria_valuemin, high.aria_valuemax, high.data_thumb), (25.0, 100.0, "max"));
        assert_eq!(range_style(&state, &input), "left: 20%; width: 40%;");
    }

    #[test]
    fn pointer_maps_into_range() {
        let input = RangeSliderInput { min: 10.0, max: 20.0, ..RangeSliderInput::default() };
        assert!((value_from_pointer(150.0, 100.0, 100.0, &input) - 15.0).abs() < 1e-9);
        assert!((value_from_pointer(0.0, 100.0, 100.0, &input) - 10.0).abs() < 1e-9);
        assert!((value_from_pointer(500.0, 100.0, 100.0, &input) - 20.0).abs() < 1e-9);
    }

    #[test]
    fn keys_map_to_targets() {
        let input = RangeSliderInput::default();
        let state = RangeSliderState { value: [20.0, 60.0], dragging: None, controlled: false };
        assert_eq!(keydown("ArrowUp", 1, &state, &input), Some(RangeSliderEvent::Set(1, 61.0, ChangeReason::Keyboard)));
        assert_eq!(keydown("PageDown", 0, &state, &input), Some(RangeSliderEvent::Set(0, 10.0, ChangeReason::Keyboard)));
        assert_eq!(keydown("End", 0, &state, &input), Some(RangeSliderEvent::Set(0, 100.0, ChangeReason::Keyboard)));
        assert_eq!(keydown("Enter", 0, &state, &input), None);
    }
}
