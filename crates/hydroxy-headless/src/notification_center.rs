//! Notification center connector. Anatomy: trigger (the bell), badge, content
//! (the popover), header, title, mark-all, list, item, item-title,
//! item-description, item-time, dismiss, empty. Every part carries
//! `data-scope="notification-center"` and its `data-part`.
//!
//! The content is a `popover="auto"`: Esc and outside presses are native. The
//! trigger names it with `popovertarget`, so the browser also handles the toggle.

pub use hydroxy_machines::notification_center::*;

use crate::overlay::{self, Align, Placement, Side};

/// The `data-scope` value on every part.
pub const SCOPE: &str = "notification-center";

/// Below the trigger, lined up with its end edge (the bell sits at the right).
pub const DEFAULT_PLACEMENT: Placement = Placement::new(Side::Bottom, Align::End);

/// Pixels between trigger and popover.
pub const GAP_PX: u16 = 8;

/// Attributes for the bell trigger (`<button>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerAttrs {
    /// `id`, referenced by the content's `aria-labelledby`.
    pub id: String,
    /// `aria-label`: names the bell and says how many are unread.
    pub aria_label: String,
    /// `aria-controls` and `popovertarget`: the content's id.
    pub aria_controls: String,
    /// `style`: names the CSS anchor.
    pub style: String,
    /// `data-unread`, emitted when something is unread.
    pub has_unread: bool,
}

/// Attributes for the content (`popover="auto"`, `role="dialog"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAttrs {
    /// `id`, referenced by the trigger.
    pub id: String,
    /// `aria-labelledby`: the title's id.
    pub aria_labelledby: String,
    /// `style`: anchored placement with flip.
    pub style: String,
    /// `data-side`, for side-aware motion.
    pub data_side: &'static str,
}

/// Id of the trigger.
#[must_use]
pub fn trigger_id(root: &str) -> String {
    format!("{root}-trigger")
}

/// Id of the content.
#[must_use]
pub fn content_id(root: &str) -> String {
    format!("{root}-content")
}

/// Id of the heading.
#[must_use]
pub fn title_id(root: &str) -> String {
    format!("{root}-title")
}

/// The badge text: the count, capped at `99+`.
#[must_use]
pub fn badge_text(unread: usize) -> String {
    if unread > 99 { "99+".to_string() } else { unread.to_string() }
}

/// Trigger attributes for the instance with id prefix `root`, given the items' unread count.
#[must_use]
pub fn trigger_attrs(root: &str, label: &str, unread: usize) -> TriggerAttrs {
    TriggerAttrs {
        id: trigger_id(root),
        aria_label: match unread {
            0 => label.to_string(),
            n => format!("{label}, {n} unread"),
        },
        aria_controls: content_id(root),
        style: overlay::anchor_style(root),
        has_unread: unread > 0,
    }
}

/// Content attributes for the instance with id prefix `root`.
#[must_use]
pub fn content_attrs(root: &str, placement: Placement) -> ContentAttrs {
    ContentAttrs {
        id: content_id(root),
        aria_labelledby: title_id(root),
        style: overlay::floating_style(root, placement, GAP_PX),
        data_side: placement.side_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_and_content_reference_each_other() {
        let t = trigger_attrs("nc", "Notifications", 3);
        let c = content_attrs("nc", DEFAULT_PLACEMENT);
        assert_eq!(t.aria_controls, c.id);
        assert_eq!(c.aria_labelledby, title_id("nc"));
        assert_eq!(t.aria_label, "Notifications, 3 unread");
        assert!(t.has_unread);
        assert!(!trigger_attrs("nc", "Notifications", 0).has_unread);
    }

    #[test]
    fn badge_caps_at_99() {
        assert_eq!(badge_text(7), "7");
        assert_eq!(badge_text(100), "99+");
    }
}
