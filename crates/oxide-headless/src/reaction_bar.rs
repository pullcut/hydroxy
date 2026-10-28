//! Reaction bar connector. Anatomy: root (`role="group"`), chip (a toggle
//! `<button aria-pressed>` with the emoji and count), tip (who reacted; the
//! chip's description, shown on hover and focus, anchored to the chip), add (a
//! button opening the picker), picker (a native `popover="auto"` holding an
//! emoji picker), status (a polite live region). Every part carries
//! `data-scope="reaction-bar"` and its `data-part`.

pub use oxide_machines::reaction_bar::*;

use crate::overlay::{Align, Placement, Side, anchor_style, floating_style};

/// The `data-scope` value on every part.
pub const SCOPE: &str = "reaction-bar";

/// How many names a tip lists before counting the rest.
pub const TIP_NAMES: usize = 6;

/// Element ids for one reaction bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// Root prefix.
    pub root: String,
    /// The add button; also the picker's anchor.
    pub add: String,
    /// The picker popover.
    pub picker: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { root: root.to_string(), add: format!("{root}-add"), picker: format!("{root}-picker") }
}

/// The emoji as an id-safe slug: its code points in hex, "1f44d".
fn slug(emoji: &str) -> String {
    emoji.chars().map(|c| format!("{:x}", u32::from(c))).collect::<Vec<_>>().join("-")
}

/// A chip's id, from its emoji, so it stays put as reactions come and go. Also its CSS anchor name.
#[must_use]
pub fn chip_id(ids: &Ids, emoji: &str) -> String {
    format!("{}-chip-{}", ids.root, slug(emoji))
}

/// A chip's tip id, referenced by its `aria-describedby`.
#[must_use]
pub fn tip_id(ids: &Ids, emoji: &str) -> String {
    format!("{}-tip-{}", ids.root, slug(emoji))
}

/// Inline style naming a chip as its tip's anchor.
#[must_use]
pub fn chip_style(ids: &Ids, emoji: &str) -> String {
    anchor_style(&chip_id(ids, emoji))
}

/// Inline style placing a tip above its chip.
#[must_use]
pub fn tip_style(ids: &Ids, emoji: &str) -> String {
    floating_style(&chip_id(ids, emoji), Placement::new(Side::Top, Align::Center), 6)
}

/// Inline style naming the add button as the picker's anchor.
#[must_use]
pub fn add_style(ids: &Ids) -> String {
    anchor_style(&ids.add)
}

/// Inline style placing the picker below the add button.
#[must_use]
pub fn picker_style(ids: &Ids) -> String {
    floating_style(&ids.add, Placement::new(Side::Bottom, Align::Start), 6)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tips_anchor_to_their_chip() {
        let i = ids("rb");
        assert_eq!(chip_id(&i, "👍"), "rb-chip-1f44d");
        assert_eq!(tip_id(&i, "❤️"), "rb-tip-2764-fe0f");
        assert!(tip_style(&i, "👍").contains("position-anchor: --rb-chip-1f44d;"));
        assert!(chip_style(&i, "👍").contains("anchor-name: --rb-chip-1f44d;"));
        assert!(picker_style(&i).contains("--rb-add"));
    }
}
