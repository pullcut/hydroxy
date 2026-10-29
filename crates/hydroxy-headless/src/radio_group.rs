//! Radio group connector. Anatomy: root, item, control, label, description.
//! Every part carries `data-scope="radio-group"` and its `data-part`; the
//! styles key off those.

pub use crate::Orientation;
pub use hydroxy_machines::radio_group::*;

use crate::attrs::{nav_key, roving};
use hydroxy_virtual::Nav;

/// The `data-scope` value on every radio group part.
pub const SCOPE: &str = "radio-group";

/// Attributes for one option (`<button role="radio">`).
#[derive(Debug, Clone, PartialEq)]
pub struct ItemAttrs {
    /// `id`, used to move focus.
    pub id: String,
    /// Always `"radio"`.
    pub role: &'static str,
    /// `aria-checked`.
    pub aria_checked: bool,
    /// `aria-labelledby`: the label's id.
    pub aria_labelledby: String,
    /// `aria-describedby`: the description's id.
    pub aria_describedby: String,
    /// Roving tabindex: only the selected option (or first enabled) is 0.
    pub tabindex: i32,
    /// The option or the whole group is disabled.
    pub disabled: bool,
    /// `data-state`: `"checked"` or `"unchecked"`.
    pub data_state: &'static str,
}

/// The option's `id`.
#[must_use]
pub fn item_id(root: &str, value: &str) -> String {
    format!("{root}-item-{value}")
}

/// The option label's `id`.
#[must_use]
pub fn label_id(root: &str, value: &str) -> String {
    format!("{root}-label-{value}")
}

/// The option description's `id`.
#[must_use]
pub fn description_id(root: &str, value: &str) -> String {
    format!("{root}-description-{value}")
}

/// The option that takes Tab-key focus: the selected one if enabled, else the first enabled.
fn tabbable<'a>(state: &'a RadioGroupState, input: &'a RadioGroupInput) -> Option<&'a str> {
    state
        .value
        .value
        .as_deref()
        .filter(|v| input.is_enabled(v))
        .or_else(|| input.items.iter().map(|i| i.value.as_str()).find(|v| input.is_enabled(v)))
}

/// Attributes for the option `value`.
#[must_use]
pub fn item_attrs(root: &str, state: &RadioGroupState, input: &RadioGroupInput, value: &str) -> ItemAttrs {
    let checked = state.value.value.as_deref() == Some(value);
    ItemAttrs {
        id: item_id(root, value),
        role: "radio",
        aria_checked: checked,
        aria_labelledby: label_id(root, value),
        aria_describedby: description_id(root, value),
        tabindex: roving(tabbable(state, input) == Some(value)),
        disabled: !input.is_enabled(value),
        data_state: if checked { "checked" } else { "unchecked" },
    }
}

/// Map a `KeyboardEvent.key` to a machine event. `Some` means: prevent default.
/// Both axes move focus, whatever the orientation (WAI-ARIA radio group pattern).
#[must_use]
pub fn keydown(key: &str) -> Option<RadioGroupEvent> {
    match nav_key(key, None) {
        Some(Nav::Prev) => Some(RadioGroupEvent::Move(Direction::Prev)),
        Some(Nav::Next) => Some(RadioGroupEvent::Move(Direction::Next)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roving_tabindex_falls_back_to_first_enabled() {
        let input = RadioGroupInput::new(vec![RadioItem::new("a").disabled(), RadioItem::new("b")]);
        let state = RadioGroupState::default();
        assert_eq!(item_attrs("r", &state, &input, "a").tabindex, -1);
        assert_eq!(item_attrs("r", &state, &input, "b").tabindex, 0);
    }

    #[test]
    fn disabled_group_has_no_tab_stop() {
        let mut input = RadioGroupInput::new(vec![RadioItem::new("a")]);
        input.disabled = true;
        let attrs = item_attrs("r", &RadioGroupState::default(), &input, "a");
        assert!(attrs.disabled);
        assert_eq!(attrs.tabindex, -1);
    }

    #[test]
    fn both_axes_move() {
        assert_eq!(keydown("ArrowRight"), Some(RadioGroupEvent::Move(Direction::Next)));
        assert_eq!(keydown("ArrowUp"), Some(RadioGroupEvent::Move(Direction::Prev)));
        assert_eq!(keydown("Enter"), None);
    }
}
