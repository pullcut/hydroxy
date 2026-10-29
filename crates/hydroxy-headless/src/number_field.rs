//! Number field connector. Anatomy: root, label, field, decrement, control
//! (the `role="spinbutton"` text input), increment, hidden input. Every part
//! carries `data-scope="number-field"` and its `data-part`.
//!
//! The control is a text input, not `type="number"`: that one hides its value
//! from the page while it is invalid, changes on scroll, and can't be formatted.

use hydroxy_machines::list::Direction;
pub use hydroxy_machines::number_field::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "number-field";

/// Element ids for one number field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The text input; the label's `for` and the buttons' `aria-controls`.
    pub control: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { control: format!("{root}-control") }
}

/// ARIA for the spinbutton.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlAttrs {
    /// Always `"spinbutton"`.
    pub role: &'static str,
    /// The committed value, absent when empty.
    pub aria_valuenow: Option<f64>,
    /// The lower bound.
    pub aria_valuemin: Option<f64>,
    /// The upper bound.
    pub aria_valuemax: Option<f64>,
}

/// Attributes for the control.
#[must_use]
pub fn control_attrs(state: &NumberFieldState, input: &NumberFieldInput) -> ControlAttrs {
    ControlAttrs { role: "spinbutton", aria_valuenow: state.value, aria_valuemin: input.min, aria_valuemax: input.max }
}

/// Whether a step button would do anything. Disabled at a bound so it can't be pressed in vain.
#[must_use]
pub fn can_step(state: &NumberFieldState, input: &NumberFieldInput, direction: Direction) -> bool {
    if input.disabled || input.read_only {
        return false;
    }
    match (state.value, direction == Direction::Next) {
        (Some(v), true) => input.max.is_none_or(|max| v < max),
        (Some(v), false) => input.min.is_none_or(|min| v > min),
        (None, _) => true,
    }
}

/// The hidden input's value: the committed number, empty when there is none.
#[must_use]
pub fn form_value(state: &NumberFieldState) -> String {
    state.value.map(|v| (v + 0.0).to_string()).unwrap_or_default()
}

/// Map a `KeyboardEvent.key` on the control to a machine event.
/// `Some` means: prevent the default. Home and End only act when that bound exists,
/// so otherwise they still move the caret.
#[must_use]
pub fn keydown(key: &str, input: &NumberFieldInput) -> Option<NumberFieldEvent> {
    match key {
        "ArrowUp" => Some(NumberFieldEvent::Step { direction: Direction::Next, large: false }),
        "ArrowDown" => Some(NumberFieldEvent::Step { direction: Direction::Prev, large: false }),
        "PageUp" => Some(NumberFieldEvent::Step { direction: Direction::Next, large: true }),
        "PageDown" => Some(NumberFieldEvent::Step { direction: Direction::Prev, large: true }),
        "Home" if input.min.is_some() => Some(NumberFieldEvent::ToMin),
        "End" if input.max.is_some() => Some(NumberFieldEvent::ToMax),
        "Enter" => Some(NumberFieldEvent::Commit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_end_only_with_bounds() {
        let free = NumberFieldInput::default();
        assert_eq!(keydown("Home", &free), None);
        let bounded = NumberFieldInput { min: Some(0.0), max: Some(9.0), ..free };
        assert_eq!(keydown("Home", &bounded), Some(NumberFieldEvent::ToMin));
        assert_eq!(keydown("End", &bounded), Some(NumberFieldEvent::ToMax));
        assert_eq!(keydown("a", &bounded), None);
    }

    #[test]
    fn buttons_disable_at_bounds() {
        let input = NumberFieldInput { min: Some(0.0), max: Some(2.0), ..NumberFieldInput::default() };
        let at_max = NumberFieldState { value: Some(2.0), draft: "2".into(), controlled: false };
        assert!(!can_step(&at_max, &input, Direction::Next));
        assert!(can_step(&at_max, &input, Direction::Prev));
        assert_eq!(form_value(&at_max), "2");
        assert_eq!(form_value(&NumberFieldState::default()), "");
    }
}
