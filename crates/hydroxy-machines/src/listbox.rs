//! The shared core of filtered lists with a highlighted option (combobox,
//! command palette): query matching and highlight movement over the visible,
//! enabled options. DOM focus stays in the text input; the highlight is virtual.

pub use crate::list::Direction;
use crate::list::next_index;

/// `query` without surrounding space, lowercased: the form [`matches`] takes.
/// Build it once per query, not once per option.
#[must_use]
pub fn needle(query: &str) -> String {
    query.trim().to_lowercase()
}

/// Whether `text` contains `needle` (from [`needle`]), ignoring case.
/// An empty needle matches everything.
#[must_use]
pub fn matches(text: &str, needle: &str) -> bool {
    needle.is_empty() || contains_lower(text, needle)
}

/// Whether `text`, lowercased, contains `needle` (already lowercase). Allocation-free
/// for ASCII `text`.
#[must_use]
pub fn contains_lower(text: &str, needle: &str) -> bool {
    if !text.is_ascii() {
        return text.to_lowercase().contains(needle);
    }
    let n = needle.as_bytes();
    n.is_empty() || text.as_bytes().windows(n.len()).any(|w| w.iter().zip(n).all(|(a, b)| a.to_ascii_lowercase() == *b))
}

/// The highlight after an arrow key, among `enabled` (visible, enabled values in
/// display order). Wraps at the ends. Enters at the first or last when nothing is
/// highlighted. `None` when nothing is enabled.
#[must_use]
pub fn step(enabled: &[&str], current: Option<&str>, direction: Direction) -> Option<String> {
    if enabled.is_empty() {
        return None;
    }
    let at = current.and_then(|c| enabled.iter().position(|v| *v == c));
    Some(enabled[next_index(at, enabled.len(), direction, true)].to_string())
}

/// Keep `current` if it is still among `enabled`; otherwise fall back to the first.
#[must_use]
pub fn keep(enabled: &[&str], current: Option<&str>) -> Option<String> {
    current.filter(|c| enabled.contains(c)).or_else(|| enabled.first().copied()).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_is_case_insensitive_and_trims() {
        assert!(matches("Open the project", &needle("  PROJ ")));
        assert!(matches("anything", &needle("")));
        assert!(!matches("Billing", &needle("invoice")));
        assert!(matches("Ünïcode Name", &needle("ÜNÏ")));
        assert!(!contains_lower("ab", "abc"));
    }

    #[test]
    fn step_wraps_and_enters_at_ends() {
        let items = ["a", "b", "c"];
        assert_eq!(step(&items, None, Direction::Next).as_deref(), Some("a"));
        assert_eq!(step(&items, None, Direction::Prev).as_deref(), Some("c"));
        assert_eq!(step(&items, Some("c"), Direction::Next).as_deref(), Some("a"));
        assert_eq!(step(&[], Some("a"), Direction::Next), None);
    }

    #[test]
    fn keep_falls_back_to_first_visible() {
        assert_eq!(keep(&["b", "c"], Some("c")).as_deref(), Some("c"));
        assert_eq!(keep(&["b", "c"], Some("a")).as_deref(), Some("b"));
        assert_eq!(keep(&[], Some("a")), None);
    }
}
