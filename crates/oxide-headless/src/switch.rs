//! Switch connector. Anatomy: root (a `<label>`), control (the `role="switch"`
//! button), thumb, label, and a hidden input for forms. Every part carries
//! `data-scope="switch"`, its `data-part` and `data-state`.
//!
//! The control is a native `<button>`, so Space and Enter toggle it through
//! its click; no key mapping is needed.

pub use oxide_machines::switch::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "switch";

/// Attributes for the `role="switch"` control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlAttrs {
    /// Always `"switch"`.
    pub role: &'static str,
    /// Mirrors the checked state.
    pub aria_checked: bool,
    /// Native `disabled`; also drops the control from the tab order.
    pub disabled: bool,
    /// `"checked"` or `"unchecked"`.
    pub data_state: &'static str,
}

/// Attributes for the hidden form input, present only while checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HiddenInputAttrs {
    /// Form field name.
    pub name: String,
    /// Submitted value.
    pub value: String,
}

/// `data-state` for every part.
#[must_use]
pub fn data_state(state: &SwitchState) -> &'static str {
    if state.checked { "checked" } else { "unchecked" }
}

/// Attributes for the control.
#[must_use]
pub fn control_attrs(state: &SwitchState, input: &SwitchInput) -> ControlAttrs {
    ControlAttrs {
        role: "switch",
        aria_checked: state.checked,
        disabled: input.disabled,
        data_state: data_state(state),
    }
}

/// The hidden input, mirroring native checkbox submission: only a checked,
/// enabled, named switch submits.
#[must_use]
pub fn hidden_input(
    state: &SwitchState,
    input: &SwitchInput,
    name: Option<&str>,
    value: &str,
) -> Option<HiddenInputAttrs> {
    let name = name?;
    (state.checked && !input.disabled).then(|| HiddenInputAttrs { name: name.to_string(), value: value.to_string() })
}

/// Reason for a click: a click with `detail == 0` came from the keyboard.
#[must_use]
pub fn click_reason(detail: i32) -> ChangeReason {
    if detail == 0 { ChangeReason::Keyboard } else { ChangeReason::Pointer }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_input_only_when_checked_named_and_enabled() {
        let on = SwitchState { checked: true, controlled: false };
        let off = SwitchState { checked: false, controlled: false };
        let enabled = SwitchInput::default();
        let disabled = SwitchInput { disabled: true, ..SwitchInput::default() };

        assert_eq!(
            hidden_input(&on, &enabled, Some("alerts"), "on"),
            Some(HiddenInputAttrs { name: "alerts".into(), value: "on".into() })
        );
        assert_eq!(hidden_input(&off, &enabled, Some("alerts"), "on"), None);
        assert_eq!(hidden_input(&on, &disabled, Some("alerts"), "on"), None);
        assert_eq!(hidden_input(&on, &enabled, None, "on"), None);
    }

    #[test]
    fn control_mirrors_state() {
        let attrs = control_attrs(&SwitchState { checked: true, controlled: false }, &SwitchInput::default());
        assert!(attrs.aria_checked);
        assert_eq!(attrs.data_state, "checked");
    }
}
