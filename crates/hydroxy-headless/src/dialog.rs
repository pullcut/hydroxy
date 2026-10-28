//! Dialog connector. Anatomy: trigger, content (the native `<dialog>`, full
//! viewport), panel, title, description, actions, close, close-icon. Every part
//! carries `data-scope="dialog"`.
//!
//! The adapters open the content with `showModal()`, so the browser supplies
//! the focus trap, the inert page behind, the top layer and focus return to
//! the trigger on close.

pub use hydroxy_machines::dialog::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "dialog";

/// Attributes for the trigger `<button>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerAttrs {
    /// Always `dialog`.
    pub aria_haspopup: &'static str,
    /// `aria-expanded`.
    pub aria_expanded: bool,
    /// `aria-controls`: the content id.
    pub aria_controls: String,
    /// `data-state`.
    pub data_state: &'static str,
}

/// Attributes for the content `<dialog>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAttrs {
    /// Element id. The adapters open and close the dialog by it.
    pub id: String,
    /// `dialog`, or `alertdialog` for an alert.
    pub role: &'static str,
    /// `aria-labelledby`: the title id.
    pub aria_labelledby: String,
    /// `aria-describedby`: the description id.
    pub aria_describedby: String,
    /// `data-state`.
    pub data_state: &'static str,
}

/// `data-state` for an open flag.
#[must_use]
pub fn data_state(open: bool) -> &'static str {
    if open { "open" } else { "closed" }
}

/// Id of the `<dialog>` for the instance with id prefix `root`.
#[must_use]
pub fn content_id(root: &str) -> String {
    format!("{root}-content")
}

/// Id of the title, for the instance with id prefix `root`.
#[must_use]
pub fn title_id(root: &str) -> String {
    format!("{root}-title")
}

/// Id of the description, for the instance with id prefix `root`.
#[must_use]
pub fn description_id(root: &str) -> String {
    format!("{root}-description")
}

/// Trigger attributes for the instance with id prefix `root`.
#[must_use]
pub fn trigger_attrs(root: &str, state: &DialogState) -> TriggerAttrs {
    TriggerAttrs {
        aria_haspopup: "dialog",
        aria_expanded: state.open,
        aria_controls: content_id(root),
        data_state: data_state(state.open),
    }
}

/// Content attributes for the instance with id prefix `root`.
// describedby always points at the description id, even without a description part; AT ignores a missing id.
#[must_use]
pub fn content_attrs(root: &str, state: &DialogState, input: &DialogInput) -> ContentAttrs {
    ContentAttrs {
        id: content_id(root),
        role: if input.alert { "alertdialog" } else { "dialog" },
        aria_labelledby: title_id(root),
        aria_describedby: description_id(root),
        data_state: data_state(state.open),
    }
}

/// Map a `KeyboardEvent.key` on the content to a machine event. `Some` means:
/// prevent default, which also stops the browser's own Escape close.
#[must_use]
pub fn keydown(key: &str) -> Option<DialogEvent> {
    (key == "Escape").then_some(DialogEvent::Close(CloseReason::Escape))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts_reference_each_other() {
        let state = DialogState { open: true, controlled: false };
        let t = trigger_attrs("d", &state);
        let c = content_attrs("d", &state, &DialogInput::default());
        assert_eq!(t.aria_controls, c.id);
        assert_eq!(c.aria_labelledby, title_id("d"));
        assert_eq!(c.aria_describedby, description_id("d"));
        assert_eq!((c.role, c.data_state), ("dialog", "open"));
    }

    #[test]
    fn alert_role_and_escape_mapping() {
        let input = DialogInput { alert: true, ..Default::default() };
        assert_eq!(content_attrs("d", &DialogState::default(), &input).role, "alertdialog");
        assert_eq!(keydown("Escape"), Some(DialogEvent::Close(CloseReason::Escape)));
        assert_eq!(keydown("Enter"), None);
    }
}
