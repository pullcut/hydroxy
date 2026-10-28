//! Table of contents: resolving an outline and picking the heading being read.
//!
//! The component never scrapes the page. The host supplies the items and, if it
//! wants a scroll spy, the heading positions; this module decides what is current.

/// Default sticky-header clearance in pixels.
pub const DEFAULT_HEADING_OFFSET: f64 = 72.0;

/// One heading in the outline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// The heading's element id. Becomes the `#hash` link.
    pub id: String,
    /// The link text.
    pub label: String,
    /// Outline depth from 1. Zero is read as 1.
    pub level: u8,
}

impl Item {
    /// An outline item.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, level: u8) -> Self {
        Self { id: id.into(), label: label.into(), level }
    }
}

/// An item ready to render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedItem {
    /// The heading id.
    pub id: String,
    /// The link text.
    pub label: String,
    /// Outline depth, at least 1.
    pub level: u8,
    /// `#id`.
    pub href: String,
    /// This is the current heading.
    pub active: bool,
}

/// Why an item was dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalid {
    /// The id is blank.
    MissingId,
    /// An earlier item has the same id.
    DuplicateId,
}

/// Valid items in order, plus the dropped ones with their reason.
#[must_use]
pub fn resolve(items: &[Item], active: Option<&str>) -> (Vec<ResolvedItem>, Vec<(Invalid, Item)>) {
    let active = active.map(str::trim).filter(|a| !a.is_empty());
    let mut seen = std::collections::HashSet::new();
    let (mut ok, mut bad) = (Vec::new(), Vec::new());
    for item in items {
        let id = item.id.trim();
        if id.is_empty() {
            bad.push((Invalid::MissingId, item.clone()));
        } else if !seen.insert(id.to_string()) {
            bad.push((Invalid::DuplicateId, item.clone()));
        } else {
            ok.push(ResolvedItem {
                id: id.to_string(),
                label: item.label.trim().to_string(),
                level: item.level.max(1),
                href: format!("#{id}"),
                active: active == Some(id),
            });
        }
    }
    (ok, bad)
}

/// The current heading: the last whose top has passed the reading line. At the
/// very bottom of the page the final heading wins, so a short last section is
/// still reachable. `None` while no heading has been reached.
#[must_use]
pub fn pick_active(tops: &[f64], line: f64, at_bottom: bool) -> Option<usize> {
    if tops.is_empty() {
        return None;
    }
    if at_bottom {
        return Some(tops.len() - 1);
    }
    let mut index = None;
    for (i, top) in tops.iter().enumerate() {
        if top - line <= 1.0 {
            index = Some(i);
        } else {
            break;
        }
    }
    index
}

/// When the focused item is removed, the next remaining item takes focus.
/// `None` means the nav landmark itself.
#[must_use]
pub fn focus_fallback<'a>(previous: &'a [String], next: &[String], focused: Option<&'a str>) -> Option<&'a str> {
    let focused = focused?;
    if next.iter().any(|n| n == focused) {
        return Some(focused);
    }
    let from = previous.iter().position(|p| p == focused).map_or(0, |i| i + 1);
    previous[from.min(previous.len())..].iter().find(|id| next.contains(id)).map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<Item> {
        vec![Item::new("intro", "Intro", 1), Item::new("setup", " Setup ", 2), Item::new("intro", "Again", 1), Item::new(" ", "Blank", 1), Item::new("usage", "Usage", 0)]
    }

    #[test]
    fn resolves_valid_items_and_reports_the_rest() {
        let (ok, bad) = resolve(&items(), Some("setup"));
        assert_eq!(ok.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(), ["intro", "setup", "usage"]);
        assert_eq!(ok[1].label, "Setup");
        assert!(ok[1].active && !ok[0].active);
        assert_eq!((ok[1].href.as_str(), ok[2].level), ("#setup", 1));
        assert_eq!(bad.iter().map(|(r, _)| *r).collect::<Vec<_>>(), [Invalid::DuplicateId, Invalid::MissingId]);
    }

    #[test]
    fn nothing_active_for_a_blank_host_value() {
        assert!(resolve(&items(), Some("  ")).0.iter().all(|i| !i.active));
    }

    #[test]
    fn spy_picks_the_last_heading_past_the_line() {
        assert_eq!(pick_active(&[200.0, 400.0], 72.0, false), None);
        assert_eq!(pick_active(&[-300.0, 50.0, 400.0], 72.0, false), Some(1));
        assert_eq!(pick_active(&[-300.0, 50.0, 400.0], 72.0, true), Some(2));
        assert_eq!(pick_active(&[], 72.0, true), None);
    }

    #[test]
    fn focus_moves_to_the_next_survivor() {
        let prev: Vec<String> = ["a", "b", "c"].map(String::from).to_vec();
        let next: Vec<String> = ["a", "c"].map(String::from).to_vec();
        assert_eq!(focus_fallback(&prev, &next, Some("b")), Some("c"));
        assert_eq!(focus_fallback(&prev, &next, Some("a")), Some("a"));
        assert_eq!(focus_fallback(&prev, &["a".to_string()], Some("c")), None);
        assert_eq!(focus_fallback(&prev, &next, None), None);
    }
}
