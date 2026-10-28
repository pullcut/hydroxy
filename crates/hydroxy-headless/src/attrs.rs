//! Small attribute and key helpers shared by the connectors and adapters.

use hydroxy_virtual::Nav;

use crate::Orientation;

/// The `data-state` value for something that opens: `"open"` or `"closed"`.
#[must_use]
pub const fn data_state(open: bool) -> &'static str {
    if open { "open" } else { "closed" }
}

/// The `tabindex` for a roving-focus item: `0` on the one tab stop, `-1` on the rest.
#[must_use]
pub const fn roving(is_stop: bool) -> i32 {
    if is_stop { 0 } else { -1 }
}

/// The character a `KeyboardEvent.key` types, or `None` for named keys such as `Enter`.
/// A key is printable when it is exactly one character.
#[must_use]
pub fn printable_char(key: &str) -> Option<char> {
    let mut chars = key.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Some(c),
        _ => None,
    }
}

/// How a key moves the active option in a list. `PageUp` and `PageDown` are left to the
/// caller, because the page size is the caller's to know.
///
/// `Home` and `End` always map to first and last. The arrows depend on `orientation`:
/// `Some(Horizontal)` takes Left and Right, `Some(Vertical)` takes Up and Down, and
/// `None` takes both pairs.
#[must_use]
pub fn nav_key(key: &str, orientation: Option<Orientation>) -> Option<Nav> {
    let horizontal = orientation != Some(Orientation::Vertical);
    let vertical = orientation != Some(Orientation::Horizontal);
    match key {
        "ArrowDown" if vertical => Some(Nav::Next),
        "ArrowUp" if vertical => Some(Nav::Prev),
        "ArrowRight" if horizontal => Some(Nav::Next),
        "ArrowLeft" if horizontal => Some(Nav::Prev),
        "Home" => Some(Nav::First),
        "End" => Some(Nav::Last),
        _ => None,
    }
}

/// How a key changes a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// One step up (Right or Up arrow).
    Inc,
    /// One step down (Left or Down arrow).
    Dec,
    /// A large step up (`PageUp`).
    PageInc,
    /// A large step down (`PageDown`).
    PageDec,
    /// Jump to the minimum (Home).
    Min,
    /// Jump to the maximum (End).
    Max,
}

/// The value step a key asks for, treating all four arrows as direction-agnostic.
#[must_use]
pub fn step_key(key: &str) -> Option<Step> {
    match key {
        "ArrowRight" | "ArrowUp" => Some(Step::Inc),
        "ArrowLeft" | "ArrowDown" => Some(Step::Dec),
        "PageUp" => Some(Step::PageInc),
        "PageDown" => Some(Step::PageDec),
        "Home" => Some(Step::Min),
        "End" => Some(Step::Max),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_helpers() {
        assert_eq!((data_state(true), data_state(false)), ("open", "closed"));
        assert_eq!((roving(true), roving(false)), (0, -1));
        assert_eq!(printable_char("a"), Some('a'));
        assert_eq!(printable_char("é"), Some('é'));
        assert_eq!(printable_char("Enter"), None);
        assert_eq!(printable_char(""), None);
    }

    #[test]
    fn nav_and_step_keys() {
        assert_eq!(nav_key("ArrowDown", Some(Orientation::Vertical)), Some(Nav::Next));
        assert_eq!(nav_key("ArrowDown", Some(Orientation::Horizontal)), None);
        assert_eq!(nav_key("ArrowLeft", Some(Orientation::Horizontal)), Some(Nav::Prev));
        assert_eq!(nav_key("ArrowLeft", None), Some(Nav::Prev));
        assert_eq!(nav_key("End", Some(Orientation::Vertical)), Some(Nav::Last));
        assert_eq!(step_key("PageDown"), Some(Step::PageDec));
        assert_eq!(step_key("x"), None);
    }
}
