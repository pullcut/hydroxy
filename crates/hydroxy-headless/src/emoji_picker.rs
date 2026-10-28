//! Emoji picker connector. Anatomy: root, search (a native search `<input>`
//! controlling the grid), tabs (`role="tablist"` of category tabs), grid
//! (`role="grid"`, labelled by the tab or "Search results"), row (`role="row"`),
//! cell (`<button role="gridcell">`, roving tabindex), empty, footer, preview
//! (the active emoji and its name), tones (`role="radiogroup"` of skin tones),
//! tone, status (a polite live region for result counts). Every part carries
//! `data-scope="emoji-picker"` and its `data-part`.

pub use hydroxy_machines::emoji_picker::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "emoji-picker";

/// Element ids for one picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// Root prefix.
    pub root: String,
    /// The search input.
    pub search: String,
    /// The grid.
    pub grid: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { root: root.to_string(), search: format!("{root}-search"), grid: format!("{root}-grid") }
}

/// A cell's id.
#[must_use]
pub fn cell_id(ids: &Ids, index: usize) -> String {
    format!("{}-cell-{index}", ids.root)
}

/// A tab's id.
#[must_use]
pub fn tab_id(ids: &Ids, section: Section) -> String {
    format!("{}-tab-{}", ids.root, section.as_str())
}

/// A skin-tone option's id.
#[must_use]
pub fn tone_id(ids: &Ids, tone: SkinTone) -> String {
    format!("{}-tone-{}", ids.root, tone.as_str())
}

/// The grid's accessible name.
#[must_use]
pub fn grid_label(state: &EmojiPickerState) -> String {
    if state.query.trim().is_empty() { state.section.label().to_string() } else { "Search results".to_string() }
}

/// Map a key on a grid cell to an event. `Some` means: prevent default.
#[must_use]
pub fn grid_keydown(key: &str, ctrl: bool) -> Option<EmojiPickerEvent> {
    let step = match key {
        "ArrowRight" => GridMove::Cell(1),
        "ArrowLeft" => GridMove::Cell(-1),
        "ArrowDown" => GridMove::Row(1),
        "ArrowUp" => GridMove::Row(-1),
        "PageDown" => GridMove::Row(4),
        "PageUp" => GridMove::Row(-4),
        "Home" if ctrl => GridMove::First,
        "End" if ctrl => GridMove::Last,
        "Home" => GridMove::RowStart,
        "End" => GridMove::RowEnd,
        _ => return None,
    };
    Some(EmojiPickerEvent::Grid(step))
}

/// Map a key in the search field to an event. `Some` means: prevent default.
#[must_use]
pub fn search_keydown(key: &str) -> Option<EmojiPickerEvent> {
    match key {
        "ArrowDown" => Some(EmojiPickerEvent::SearchKey(SearchKey::Down)),
        "Enter" => Some(EmojiPickerEvent::SearchKey(SearchKey::Enter)),
        _ => None,
    }
}

/// Map a key on a tab to an event (arrows move and select, automatic activation).
#[must_use]
pub fn tab_keydown(key: &str) -> Option<EmojiPickerEvent> {
    match key {
        "ArrowRight" => Some(EmojiPickerEvent::StepSection(1)),
        "ArrowLeft" => Some(EmojiPickerEvent::StepSection(-1)),
        _ => None,
    }
}

/// The tone an arrow key moves to from `tone`, wrapping. `None` for other keys.
#[must_use]
pub fn tone_keydown(key: &str, tone: SkinTone) -> Option<SkinTone> {
    let d: isize = match key {
        "ArrowRight" | "ArrowDown" => 1,
        "ArrowLeft" | "ArrowUp" => -1,
        _ => return None,
    };
    let at = SkinTone::ALL.iter().position(|t| *t == tone).unwrap_or(0).cast_signed();
    Some(SkinTone::ALL[(at + d).rem_euclid(6).cast_unsigned()])
}

/// The emoji shown on the skin-tone options.
///
/// # Panics
///
/// Never in practice: the raised hand is in the bundled emoji data.
#[must_use]
pub fn tone_swatch(tone: SkinTone) -> String {
    let hand = find("✋").expect("raised hand is in the data");
    with_tone(hand, tone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_keys() {
        assert_eq!(grid_keydown("End", true), Some(EmojiPickerEvent::Grid(GridMove::Last)));
        assert_eq!(grid_keydown("End", false), Some(EmojiPickerEvent::Grid(GridMove::RowEnd)));
        assert_eq!(grid_keydown("a", false), None);
        assert_eq!(search_keydown("ArrowDown"), Some(EmojiPickerEvent::SearchKey(SearchKey::Down)));
    }

    #[test]
    fn tone_arrows_wrap() {
        assert_eq!(tone_keydown("ArrowLeft", SkinTone::Default), Some(SkinTone::Dark));
        assert_eq!(tone_keydown("ArrowRight", SkinTone::Dark), Some(SkinTone::Default));
        assert_eq!(tone_swatch(SkinTone::Dark), "✋🏿");
    }
}
