//! Checkbox connector. Anatomy: root (`<label>`), control (`role="checkbox"`),
//! indicator, label. Every part carries `data-scope="checkbox"`, its
//! `data-part` and `data-state` (`checked`, `unchecked` or `indeterminate`).

pub use oxide_machines::checkbox::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "checkbox";

/// Attributes for the control (the box).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlAttrs {
    /// Always `checkbox`.
    pub role: &'static str,
    /// `true`, `false` or `mixed`.
    pub aria_checked: &'static str,
    /// Native `disabled`.
    pub disabled: bool,
    /// `checked`, `unchecked` or `indeterminate`.
    pub data_state: &'static str,
}

/// `data-state` for a checkbox state.
#[must_use]
pub fn data_state(checked: CheckedState) -> &'static str {
    match checked {
        CheckedState::Checked => "checked",
        CheckedState::Unchecked => "unchecked",
        CheckedState::Indeterminate => "indeterminate",
    }
}

/// Attributes for the control.
#[must_use]
pub fn control_attrs(state: &CheckboxState, input: &CheckboxInput) -> ControlAttrs {
    ControlAttrs {
        role: "checkbox",
        aria_checked: match state.checked {
            CheckedState::Checked => "true",
            CheckedState::Unchecked => "false",
            CheckedState::Indeterminate => "mixed",
        },
        disabled: input.disabled,
        data_state: data_state(state.checked),
    }
}

/// Whether a form submits this checkbox's value. Like a native checkbox,
/// only when checked.
#[must_use]
pub fn submits(state: &CheckboxState) -> bool {
    state.checked == CheckedState::Checked
}

/// Keys to `preventDefault` on the control. Space toggles through the native
/// button click; Enter must not, as on a native checkbox.
#[must_use]
pub fn blocks_key(key: &str) -> bool {
    key == "Enter"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indeterminate_is_mixed() {
        let state = CheckboxState { checked: CheckedState::Indeterminate, controlled: false };
        let attrs = control_attrs(&state, &CheckboxInput::default());
        assert_eq!((attrs.aria_checked, attrs.data_state), ("mixed", "indeterminate"));
        assert!(!submits(&state));
    }

    #[test]
    fn enter_is_blocked_space_is_not() {
        assert!(blocks_key("Enter"));
        assert!(!blocks_key(" "));
    }
}
