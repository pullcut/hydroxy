//! Tabs connector. Anatomy: root, list, trigger, content. Every part carries
//! `data-scope="tabs"` and its `data-part`; the styles key off those.

pub use oxide_machines::tabs::*;

pub use crate::Orientation;

/// The `data-scope` value on every tabs part.
pub const SCOPE: &str = "tabs";

/// Attributes for a tab trigger (`<button>`).
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerAttrs {
    /// `id`, referenced by the content's `aria-labelledby`.
    pub id: String,
    /// Always `"tab"`.
    pub role: &'static str,
    /// `aria-selected`.
    pub aria_selected: bool,
    /// `aria-controls`: the content's id.
    pub aria_controls: String,
    /// Roving tabindex: only the selected tab (or first enabled) is 0.
    pub tabindex: i32,
    /// `disabled`.
    pub disabled: bool,
    /// `data-state`: `"active"` or `"inactive"`.
    pub data_state: &'static str,
}

/// Attributes for a tab panel.
#[derive(Debug, Clone, PartialEq)]
pub struct ContentAttrs {
    /// `id`, referenced by the trigger's `aria-controls`.
    pub id: String,
    /// Always `"tabpanel"`.
    pub role: &'static str,
    /// `aria-labelledby`: the trigger's id.
    pub aria_labelledby: String,
    /// Always 0, so the panel is reachable by Tab when it has no focusable content.
    pub tabindex: i32,
    /// `hidden` when not selected.
    pub hidden: bool,
    /// `data-state`: `"active"` or `"inactive"`.
    pub data_state: &'static str,
}

/// DOM id of the trigger for `value` inside the root `root`.
#[must_use]
pub fn trigger_id(root: &str, value: &str) -> String {
    format!("{root}-trigger-{value}")
}

/// DOM id of the content for `value` inside the root `root`.
#[must_use]
pub fn content_id(root: &str, value: &str) -> String {
    format!("{root}-content-{value}")
}

fn data_state(active: bool) -> &'static str {
    if active { "active" } else { "inactive" }
}

/// The tab that takes Tab-key focus: the selected one if enabled, else the first enabled.
fn tabbable<'a>(state: &'a TabsState, input: &'a TabsInput) -> Option<&'a str> {
    let enabled = |v: &str| input.tabs.iter().any(|t| t.value == v && !t.disabled);
    state
        .value
        .as_deref()
        .filter(|v| enabled(v))
        .or_else(|| input.tabs.iter().find(|t| !t.disabled).map(|t| t.value.as_str()))
}

/// Attributes for the trigger of `value`.
#[must_use]
pub fn trigger_attrs(root: &str, state: &TabsState, input: &TabsInput, value: &str) -> TriggerAttrs {
    let selected = state.value.as_deref() == Some(value);
    TriggerAttrs {
        id: trigger_id(root, value),
        role: "tab",
        aria_selected: selected,
        aria_controls: content_id(root, value),
        tabindex: if tabbable(state, input) == Some(value) { 0 } else { -1 },
        disabled: input.tabs.iter().any(|t| t.value == value && t.disabled),
        data_state: data_state(selected),
    }
}

/// Attributes for the content of `value`.
#[must_use]
pub fn content_attrs(root: &str, state: &TabsState, value: &str) -> ContentAttrs {
    let selected = state.value.as_deref() == Some(value);
    ContentAttrs {
        id: content_id(root, value),
        role: "tabpanel",
        aria_labelledby: trigger_id(root, value),
        tabindex: 0,
        hidden: !selected,
        data_state: data_state(selected),
    }
}

/// Map a `KeyboardEvent.key` to a machine event. `Some` means: prevent default.
// no RTL flip yet; add a `dir` arg when an RTL consumer exists.
#[must_use]
pub fn keydown(key: &str, orientation: Orientation) -> Option<TabsEvent> {
    let (prev, next) = match orientation {
        Orientation::Horizontal => ("ArrowLeft", "ArrowRight"),
        Orientation::Vertical => ("ArrowUp", "ArrowDown"),
    };
    match key {
        k if k == prev => Some(TabsEvent::Move(Direction::Prev)),
        k if k == next => Some(TabsEvent::Move(Direction::Next)),
        "Home" => Some(TabsEvent::Home),
        "End" => Some(TabsEvent::End),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roving_tabindex_falls_back_to_first_enabled() {
        let input = TabsInput::new(vec![TabsItem::new("a").disabled(), TabsItem::new("b")]);
        let state = TabsState::default();
        assert_eq!(trigger_attrs("t", &state, &input, "a").tabindex, -1);
        assert_eq!(trigger_attrs("t", &state, &input, "b").tabindex, 0);
    }

    #[test]
    fn keys_follow_orientation() {
        assert_eq!(keydown("ArrowDown", Orientation::Horizontal), None);
        assert_eq!(keydown("ArrowDown", Orientation::Vertical), Some(TabsEvent::Move(Direction::Next)));
    }
}
