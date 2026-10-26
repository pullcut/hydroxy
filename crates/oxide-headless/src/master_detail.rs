//! Master/detail connector. Anatomy: root, list, list-heading, row, detail,
//! back. Every part carries `data-scope="master-detail"` and its `data-part`;
//! the root's `data-mobile-pane` says which pane a narrow screen shows.

pub use oxide_machines::master_detail::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "master-detail";

/// The Back button's text and name.
pub const BACK_LABEL: &str = "Back";

/// The element id of a row, used to restore focus.
#[must_use]
pub fn row_dom_id(root: &str, row_id: &str) -> String {
    format!("{root}-row-{row_id}")
}

/// The element id of the list heading, the fallback focus target.
#[must_use]
pub fn heading_dom_id(root: &str) -> String {
    format!("{root}-heading")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids() {
        assert_eq!(row_dom_id("m", "42"), "m-row-42");
        assert_eq!(heading_dom_id("m"), "m-heading");
    }
}
