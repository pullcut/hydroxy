//! Table of contents connector. Anatomy: root, list, item, link. Every part
//! carries `data-scope="table-of-contents"` and its `data-part`. Links are real
//! `#hash` anchors, so the outline works before any script runs.

pub use hydroxy_machines::table_of_contents::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "table-of-contents";

/// Default accessible name for the outline landmark.
pub const LABEL: &str = "On this page";

/// The `aria-current` value on the active link.
#[must_use]
pub const fn aria_current(active: bool) -> Option<&'static str> {
    if active { Some("true") } else { None }
}

/// The inline style that sets the heading clearance property on the root.
#[must_use]
pub fn root_style(offset: f64) -> String {
    let px = if offset.is_finite() && offset >= 0.0 { offset } else { DEFAULT_HEADING_OFFSET };
    format!("--ox-toc-heading-offset: {px}px")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_and_style() {
        assert_eq!(aria_current(true), Some("true"));
        assert_eq!(aria_current(false), None);
        assert_eq!(root_style(-3.0), "--ox-toc-heading-offset: 72px");
        assert_eq!(root_style(96.0), "--ox-toc-heading-offset: 96px");
    }
}
