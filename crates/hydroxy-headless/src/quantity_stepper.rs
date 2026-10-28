//! Quantity stepper connector. Anatomy: root, decrement, control (the
//! `role="spinbutton"` text input), increment, hidden input. It is the
//! [`number_field`](crate::number_field) behaviour in a compact − / value / +
//! layout, so the machine, ids, ARIA and keys are all that module's; only the
//! `data-scope` differs.

pub use crate::number_field::{ControlAttrs, Ids, can_step, control_attrs, form_value, ids, keydown};
pub use hydroxy_machines::number_field::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "quantity-stepper";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_stop_at_the_bounds() {
        let input = NumberFieldInput { min: Some(1.0), max: Some(3.0), ..Default::default() };
        let state = NumberFieldState { value: Some(3.0), draft: "3".into(), controlled: false };
        assert!(!can_step(&state, &input, true));
        assert!(can_step(&state, &input, false));
        assert_eq!(control_attrs(&state, &input).aria_valuenow, Some(3.0));
        assert_eq!(ids("q").control, "q-control");
    }
}
