//! Sheet connector: a modal panel that slides in from an edge of the viewport.
//! Anatomy: trigger, content (the native `<dialog>`), panel, header, title,
//! description, body, footer, close, close-icon. Every part carries
//! `data-scope="sheet"`.
//!
//! Behaviour is the dialog's: the same machine, opened with `showModal()`, so
//! the browser supplies the focus trap, the inert page and focus return. Only
//! the side and the scope differ.

pub use crate::dialog::{
    CloseReason, ContentAttrs, DialogCommand as SheetCommand, DialogEvent as SheetEvent, DialogInput as SheetInput,
    DialogMachine as SheetMachine, DialogState as SheetState, TriggerAttrs, content_id, data_state, description_id,
    keydown, title_id, trigger_attrs,
};

/// The `data-scope` value on every part.
pub const SCOPE: &str = "sheet";

/// The viewport edge the sheet slides in from. Logical, so it follows `dir`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SheetSide {
    /// The inline start edge (left in LTR).
    Start,
    /// The inline end edge (right in LTR).
    #[default]
    End,
    /// The top edge.
    Top,
    /// The bottom edge.
    Bottom,
}

impl SheetSide {
    /// The `data-side` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            SheetSide::Start => "start",
            SheetSide::End => "end",
            SheetSide::Top => "top",
            SheetSide::Bottom => "bottom",
        }
    }
}

/// Content attributes for the instance with id prefix `root`. Always `role="dialog"`.
#[must_use]
pub fn content_attrs(root: &str, state: &SheetState) -> ContentAttrs {
    crate::dialog::content_attrs(root, state, &SheetInput::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::Controllable;

    #[test]
    fn reuses_dialog_wiring_with_its_own_sides() {
        let state = SheetState { open: Controllable::new(true, false) };
        let t = trigger_attrs("s", &state);
        let c = content_attrs("s", &state);
        assert_eq!(t.aria_controls, c.id);
        assert_eq!((c.role, c.data_state), ("dialog", "open"));
        assert_eq!(c.aria_labelledby, title_id("s"));
        assert_eq!(SheetSide::default().as_str(), "end");
    }
}
