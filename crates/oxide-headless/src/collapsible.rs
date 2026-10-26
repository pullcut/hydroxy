//! Collapsible connector. Anatomy: root, trigger, content. Every part carries
//! `data-scope="collapsible"`, its `data-part` and `data-state="open|closed"`.

pub use oxide_machines::collapsible::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "collapsible";

/// Attributes for the trigger `<button>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerAttrs {
    /// Element id, referenced by the content's `aria-labelledby`.
    pub id: String,
    /// `aria-expanded`.
    pub aria_expanded: bool,
    /// `aria-controls`: the content id.
    pub aria_controls: String,
    /// Native `disabled`.
    pub disabled: bool,
    /// `data-state`.
    pub data_state: &'static str,
}

/// Attributes for the content region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAttrs {
    /// Element id, referenced by the trigger's `aria-controls`.
    pub id: String,
    /// Always `region`.
    pub role: &'static str,
    /// `aria-labelledby`: the trigger id.
    pub aria_labelledby: String,
    /// Closed content stays mounted but hidden.
    pub hidden: bool,
    /// `data-state`.
    pub data_state: &'static str,
}

/// `data-state` for an open flag.
#[must_use]
pub fn data_state(open: bool) -> &'static str {
    if open { "open" } else { "closed" }
}

/// Trigger attributes for the instance with id prefix `root`.
#[must_use]
pub fn trigger_attrs(root: &str, state: &CollapsibleState, input: &CollapsibleInput) -> TriggerAttrs {
    TriggerAttrs {
        id: format!("{root}-trigger"),
        aria_expanded: state.open,
        aria_controls: format!("{root}-content"),
        disabled: input.disabled,
        data_state: data_state(state.open),
    }
}

/// Content attributes for the instance with id prefix `root`.
#[must_use]
pub fn content_attrs(root: &str, state: &CollapsibleState) -> ContentAttrs {
    ContentAttrs {
        id: format!("{root}-content"),
        role: "region",
        aria_labelledby: format!("{root}-trigger"),
        hidden: !state.open,
        data_state: data_state(state.open),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_and_content_reference_each_other() {
        let state = CollapsibleState { open: false, controlled: false };
        let t = trigger_attrs("c", &state, &CollapsibleInput::default());
        let c = content_attrs("c", &state);
        assert_eq!(t.aria_controls, c.id);
        assert_eq!(c.aria_labelledby, t.id);
        assert!(c.hidden);
        assert_eq!(t.data_state, "closed");
    }
}
