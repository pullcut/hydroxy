//! Date picker connector. Anatomy: root, label, field, input, trigger, content
//! (holding a calendar). Every part carries `data-scope="date-picker"` and its
//! `data-part`.
//!
//! The content is a native `popover="auto"` anchored to the field, so the top
//! layer and outside-click dismissal come from the browser. The trigger names it
//! with `popovertarget` so clicking the trigger never counts as an outside click.

pub use oxide_machines::date_picker::*;

use crate::overlay::{self, Align, Placement, Side};

/// The `data-scope` value on every date picker part.
pub const SCOPE: &str = "date-picker";

/// Below the field, lined up with its start edge.
pub const PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Start);

/// Pixels between field and popover.
pub const GAP_PX: u16 = 6;

/// Placeholder for the text field.
pub const PLACEHOLDER: &str = "YYYY-MM-DD";

/// What opening focuses inside the content: the calendar's focused day.
pub const FOCUS_ON_OPEN: &str = "[data-part=\"day\"][tabindex=\"0\"]";

/// Ids for every part, from one root id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The text input, named by the label.
    pub input: String,
    /// The calendar button.
    pub trigger: String,
    /// The popover.
    pub content: String,
}

/// Ids for every part.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { input: format!("{root}-input"), trigger: format!("{root}-trigger"), content: format!("{root}-content") }
}

/// `style` for the field: names the CSS anchor.
#[must_use]
pub fn field_style(root: &str) -> String {
    overlay::anchor_style(root)
}

/// `style` for the content: anchored below the field, flipping when short of room.
#[must_use]
pub fn content_style(root: &str) -> String {
    overlay::floating_style(root, PLACEMENT, GAP_PX)
}

/// `data-state` for the trigger and content.
#[must_use]
pub fn data_state(state: &DatePickerState) -> &'static str {
    if state.open { "open" } else { "closed" }
}

/// Keys in the text field. Enter tidies the text without submitting the form.
#[must_use]
pub fn input_keydown(key: &str) -> Option<DatePickerEvent> {
    (key == "Enter").then_some(DatePickerEvent::Commit)
}

/// Keys inside the open popover. Escape is handled here (not left to the
/// browser) so focus goes back to the trigger.
#[must_use]
pub fn content_keydown(key: &str) -> Option<DatePickerEvent> {
    (key == "Escape").then_some(DatePickerEvent::Close { restore_focus: true })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts_share_the_root_anchor() {
        assert!(field_style("p").contains("anchor-name: --p;"));
        assert!(content_style("p").contains("position-anchor: --p;"));
        assert_eq!(ids("p").content, "p-content");
    }

    #[test]
    fn keys() {
        assert_eq!(input_keydown("Enter"), Some(DatePickerEvent::Commit));
        assert_eq!(content_keydown("Escape"), Some(DatePickerEvent::Close { restore_focus: true }));
        assert_eq!(content_keydown("a"), None);
    }
}
