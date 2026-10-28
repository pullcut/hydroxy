//! Accordion connector. Anatomy: root, item, heading, trigger, content. Every
//! part carries `data-scope="accordion"` and its `data-part`; item, trigger and
//! content carry `data-state="open" | "closed"`.

pub use hydroxy_machines::accordion::*;

use crate::{
    Orientation,
    attrs::{data_state, nav_key},
};
use hydroxy_virtual::Nav;

/// The `data-scope` value on every accordion part.
pub const SCOPE: &str = "accordion";

/// Attributes for an item wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemAttrs {
    /// `data-state`: `"open"` or `"closed"`.
    pub data_state: &'static str,
    /// Emit `data-disabled` when true.
    pub disabled: bool,
}

/// Attributes for a trigger (`<button>` inside the heading).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerAttrs {
    /// `id`, referenced by the content's `aria-labelledby`.
    pub id: String,
    /// `aria-expanded`.
    pub aria_expanded: bool,
    /// `aria-controls`: the content's id.
    pub aria_controls: String,
    /// Native `disabled`, plus `data-disabled`.
    pub disabled: bool,
    /// `data-state`.
    pub data_state: &'static str,
}

/// Attributes for a content region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAttrs {
    /// `id`, referenced by the trigger's `aria-controls`.
    pub id: String,
    /// Always `"region"`.
    pub role: &'static str,
    /// `aria-labelledby`: the trigger's id.
    pub aria_labelledby: String,
    /// `hidden` while closed. Closed content stays in the DOM.
    pub hidden: bool,
    /// `data-state`.
    pub data_state: &'static str,
}

/// The trigger's id for item `value`.
#[must_use]
pub fn trigger_id(root: &str, value: &str) -> String {
    format!("{root}-trigger-{value}")
}

/// The content's id for item `value`.
#[must_use]
pub fn content_id(root: &str, value: &str) -> String {
    format!("{root}-content-{value}")
}

/// Whether item `value` is open.
#[must_use]
pub fn is_open(state: &AccordionState, value: &str) -> bool {
    state.value.value.iter().any(|v| v == value)
}

fn is_disabled(input: &AccordionInput, value: &str) -> bool {
    input.items.iter().any(|i| i.value == value && i.disabled)
}

/// Attributes for the item wrapper of `value`.
#[must_use]
pub fn item_attrs(state: &AccordionState, input: &AccordionInput, value: &str) -> ItemAttrs {
    ItemAttrs { data_state: data_state(is_open(state, value)), disabled: is_disabled(input, value) }
}

/// Attributes for the trigger of `value`.
#[must_use]
pub fn trigger_attrs(root: &str, state: &AccordionState, input: &AccordionInput, value: &str) -> TriggerAttrs {
    let open = is_open(state, value);
    TriggerAttrs {
        id: trigger_id(root, value),
        aria_expanded: open,
        aria_controls: content_id(root, value),
        disabled: is_disabled(input, value),
        data_state: data_state(open),
    }
}

/// Attributes for the content of `value`.
#[must_use]
pub fn content_attrs(root: &str, state: &AccordionState, value: &str) -> ContentAttrs {
    let open = is_open(state, value);
    ContentAttrs {
        id: content_id(root, value),
        role: "region",
        aria_labelledby: trigger_id(root, value),
        hidden: !open,
        data_state: data_state(open),
    }
}

/// Map a `KeyboardEvent.key` on a trigger to a machine event. `Some` means:
/// prevent default. Enter and Space stay native button clicks.
#[must_use]
pub fn keydown(key: &str) -> Option<AccordionEvent> {
    match nav_key(key, Some(Orientation::Vertical))? {
        Nav::Next => Some(AccordionEvent::Move(Direction::Next)),
        Nav::Prev => Some(AccordionEvent::Move(Direction::Prev)),
        Nav::First => Some(AccordionEvent::Home),
        Nav::Last => Some(AccordionEvent::End),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::Controllable;

    #[test]
    fn closed_content_is_hidden_and_linked() {
        let state = AccordionState { value: Controllable::new(vec!["a".into()], false), ..Default::default() };
        assert!(!content_attrs("r", &state, "a").hidden);
        let closed = content_attrs("r", &state, "b");
        assert!(closed.hidden);
        assert_eq!(closed.aria_labelledby, trigger_id("r", "b"));
    }

    #[test]
    fn keys_are_vertical() {
        assert_eq!(keydown("ArrowRight"), None);
        assert_eq!(keydown("ArrowDown"), Some(AccordionEvent::Move(Direction::Next)));
    }
}
