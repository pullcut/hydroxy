//! Inline edit connector. Anatomy: root, preview, field. Every part carries
//! `data-scope="inline-edit"` and its `data-part`. The preview is a
//! `role="button"`; the field is a native `<input>` or `<textarea>`.

pub use hydroxy_machines::inline_edit::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "inline-edit";

/// Element ids for one inline edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The preview button.
    pub preview: String,
    /// The input or textarea.
    pub field: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { preview: format!("{root}-preview"), field: format!("{root}-field") }
}

/// The preview's accessible name: "Edit Project name".
#[must_use]
pub fn preview_label(label: &str) -> String {
    format!("Edit {label}")
}

/// Map a key on the preview to an event. `Some` means: prevent default.
#[must_use]
pub fn preview_keydown(key: &str) -> Option<InlineEditEvent> {
    matches!(key, "Enter" | "F2").then_some(InlineEditEvent::Edit)
}

/// Map a key in the field to an event. A single-line field commits on Enter; a
/// multi-line one commits on Ctrl/⌘+Enter, so Enter can add a line. `Some` means: prevent default.
#[must_use]
pub fn field_keydown(key: &str, multiline: bool, modifier: bool) -> Option<InlineEditEvent> {
    match key {
        "Escape" => Some(InlineEditEvent::Cancel),
        "Enter" if !multiline || modifier => Some(InlineEditEvent::Commit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_and_f2_start_editing() {
        assert_eq!(preview_keydown("Enter"), Some(InlineEditEvent::Edit));
        assert_eq!(preview_keydown("F2"), Some(InlineEditEvent::Edit));
        assert_eq!(preview_keydown("a"), None);
    }

    #[test]
    fn multiline_needs_a_modifier_to_commit() {
        assert_eq!(field_keydown("Enter", false, false), Some(InlineEditEvent::Commit));
        assert_eq!(field_keydown("Enter", true, false), None);
        assert_eq!(field_keydown("Enter", true, true), Some(InlineEditEvent::Commit));
        assert_eq!(field_keydown("Escape", true, false), Some(InlineEditEvent::Cancel));
    }
}
