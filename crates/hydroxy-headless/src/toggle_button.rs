//! Toggle button connector. One part: root, a native `<button>` with
//! `aria-pressed`, carrying `data-scope="toggle-button"`, `data-part="root"`
//! and `data-state`.
//!
//! The behaviour is a switch's (one on/off value, controlled or not), so the
//! switch machine drives it; `checked` there means pressed here. Space and
//! Enter press it through the native button's click.

pub use hydroxy_machines::switch::{
    ChangeReason, SwitchCommand as ToggleButtonCommand, SwitchEvent as ToggleButtonEvent,
    SwitchInput as ToggleButtonInput, SwitchMachine as ToggleButtonMachine, SwitchState as ToggleButtonState,
};

/// `data-scope` for every part.
pub const SCOPE: &str = "toggle-button";

/// Attributes for the button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootAttrs {
    /// `aria-pressed`.
    pub aria_pressed: bool,
    /// Native `disabled`.
    pub disabled: bool,
    /// `"pressed"` or `"unpressed"`.
    pub data_state: &'static str,
}

/// Attributes for the button.
#[must_use]
pub fn root_attrs(state: &ToggleButtonState, input: &ToggleButtonInput) -> RootAttrs {
    RootAttrs {
        aria_pressed: state.checked,
        disabled: input.disabled,
        data_state: if state.checked { "pressed" } else { "unpressed" },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirrors_pressed() {
        let attrs = root_attrs(&ToggleButtonState { checked: true, controlled: false }, &ToggleButtonInput::default());
        assert!(attrs.aria_pressed);
        assert_eq!(attrs.data_state, "pressed");
    }
}
