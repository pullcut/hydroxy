//! Tour connector. Anatomy: spotlight (the dimmed page with a rounded cut-out
//! around the target), popover (`role="dialog"`), title, body, progress
//! ("2 of 5"), and the skip, prev and next buttons. Every part carries
//! `data-scope="tour"`.
//!
//! Positioning reuses the overlay groundwork: the adapter names the step's
//! target as a CSS anchor, so both the spotlight and the popover follow it
//! through scrolling and resizing without any measuring. A step whose target is
//! missing shows the popover centred over a fully dimmed page.

pub use hydroxy_machines::tour::*;

use crate::overlay::{self, Align, Placement, Side};

/// The `data-scope` value on every part.
pub const SCOPE: &str = "tour";

/// Below the target, centred on it.
pub const DEFAULT_PLACEMENT: Placement = Placement::new(Side::Bottom, Align::Center);

/// Pixels between the spotlight's edge and the popover.
pub const GAP_PX: u16 = 14;

/// One coachmark.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TourStep {
    /// The element to spotlight: an element id, or any CSS selector. `None` centres the step.
    pub target: Option<String>,
    /// Heading.
    pub title: String,
    /// Explanation.
    pub body: String,
    /// Where the popover sits. Default below, centred.
    pub placement: Placement,
}

impl TourStep {
    /// A step pointing at `target` (an id or a selector).
    #[must_use]
    pub fn new(target: impl Into<String>, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self { target: Some(target.into()), title: title.into(), body: body.into(), placement: DEFAULT_PLACEMENT }
    }

    /// A centred step with no target, for a welcome or wrap-up.
    #[must_use]
    pub fn centred(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self { target: None, title: title.into(), body: body.into(), placement: DEFAULT_PLACEMENT }
    }

    /// Place the popover on `side` of the target.
    #[must_use]
    pub fn side(mut self, side: Side) -> Self {
        self.placement = Placement::new(side, Align::Center);
        self
    }
}

/// Element ids for one tour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The popover.
    pub popover: String,
    /// The spotlight.
    pub spotlight: String,
    /// The title, which labels the popover.
    pub title: String,
    /// The body, which describes the popover.
    pub body: String,
    /// The CSS anchor name given to the target, without the leading `--`.
    pub anchor: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids {
        popover: format!("{root}-popover"),
        spotlight: format!("{root}-spotlight"),
        title: format!("{root}-title"),
        body: format!("{root}-body"),
        anchor: format!("{root}-target"),
    }
}

/// "Step 2 of 5".
#[must_use]
pub fn progress(index: usize, count: usize) -> String {
    format!("Step {} of {count}", index + 1)
}

/// The next button's text: "Done" on the last step.
#[must_use]
pub fn next_label(state: &TourState, input: &TourInput) -> &'static str {
    if state.is_last(input) { "Done" } else { "Next" }
}

/// Inline style for the popover: anchored to the target, or centred when it is missing.
#[must_use]
pub fn popover_style(ids: &Ids, placement: Placement, found: bool) -> String {
    if found {
        overlay::floating_style(&ids.anchor, placement, GAP_PX)
    } else {
        "position: fixed; inset: 0; margin: auto; block-size: fit-content;".to_string()
    }
}

/// Inline style for the spotlight: names the anchor it wraps.
#[must_use]
pub fn spotlight_style(ids: &Ids) -> String {
    format!("position-anchor: --{};", ids.anchor)
}

/// Keys in the popover. `Some` means: prevent default.
#[must_use]
pub fn keydown(key: &str) -> Option<TourEvent> {
    match key {
        "Escape" => Some(TourEvent::Skip),
        "ArrowRight" => Some(TourEvent::Next),
        "ArrowLeft" => Some(TourEvent::Prev),
        _ => None,
    }
}

/// Points the anchor at a step's target and scrolls it into view. A JS function
/// body with parameters `root` (the tour id), `target` (an id, a selector, or
/// empty), and `anchor` (the anchor name without `--`). Remembers the focused
/// element the first time it runs for `root`. Returns whether the target exists.
pub const SHOW_STEP_JS: &str = r#"
const store = (window.__oxTour ??= {});
const self = (store[root] ??= { focus: document.activeElement, el: null, prev: "" });
if (self.el) { self.el.style.anchorName = self.prev; self.el = null; }
let el = null;
if (target) {
  el = document.getElementById(target);
  if (!el) { try { el = document.querySelector(target); } catch (_) { el = null; } }
}
if (!el || !el.getClientRects().length) return false;
self.el = el;
self.prev = el.style.anchorName;
el.style.anchorName = self.prev && self.prev !== "none" ? `${self.prev}, --${anchor}` : `--${anchor}`;
const smooth = !matchMedia("(prefers-reduced-motion: reduce)").matches;
el.scrollIntoView({ block: "center", inline: "nearest", behavior: smooth ? "smooth" : "auto" });
return true;
"#;

/// Ends the tour: clears the anchor and gives focus back to whatever had it
/// when the tour started. A JS function body with the parameter `root`.
pub const END_JS: &str = r#"
const store = window.__oxTour || {};
const self = store[root];
if (!self) return;
if (self.el) self.el.style.anchorName = self.prev;
delete store[root];
if (self.focus && self.focus.isConnected && typeof self.focus.focus === "function") self.focus.focus();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popover_anchors_to_the_target_or_centres() {
        let ids = ids("t");
        assert!(popover_style(&ids, DEFAULT_PLACEMENT, true).contains("position-anchor: --t-target;"));
        assert!(popover_style(&ids, DEFAULT_PLACEMENT, false).contains("inset: 0"));
        assert_eq!(spotlight_style(&ids), "position-anchor: --t-target;");
    }

    #[test]
    fn keys_navigate_and_escape_skips() {
        assert_eq!(keydown("Escape"), Some(TourEvent::Skip));
        assert_eq!(keydown("ArrowRight"), Some(TourEvent::Next));
        assert_eq!(keydown("ArrowLeft"), Some(TourEvent::Prev));
        assert_eq!(keydown("Enter"), None);
    }

    #[test]
    fn progress_and_labels() {
        assert_eq!(progress(1, 5), "Step 2 of 5");
        let input = TourInput { count: 2, ..Default::default() };
        assert_eq!(next_label(&TourState { open: true, index: 1, controlled: false }, &input), "Done");
        assert_eq!(next_label(&TourState::default(), &input), "Next");
    }
}
