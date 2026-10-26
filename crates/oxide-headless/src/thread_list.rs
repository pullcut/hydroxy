//! Thread list connector. Anatomy: root, search, list, item (a row holding the open button
//! and the rename/delete actions), title, preview, time, unread marker, rename field.
//! Every part carries `data-scope="thread-list"` and its `data-part`.
//!
//! Each row's open button is a native `<button>` with `aria-current` on the selected one.
//! In the rename field Enter commits and Escape cancels.

pub use oxide_machines::thread_list::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "thread-list";

/// One conversation row.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThreadSummary {
    /// Stable key.
    pub id: String,
    /// Title.
    pub title: String,
    /// First line of the last message.
    pub preview: String,
    /// Display time, such as "2h" or "26 Oct".
    pub time: String,
    /// Has messages the reader hasn't seen.
    pub unread: bool,
}

impl ThreadSummary {
    /// A read conversation.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        preview: impl Into<String>,
        time: impl Into<String>,
    ) -> Self {
        Self { id: id.into(), title: title.into(), preview: preview.into(), time: time.into(), unread: false }
    }

    /// Mark as unread.
    #[must_use]
    pub fn unread(mut self) -> Self {
        self.unread = true;
        self
    }
}

/// Rows whose title or preview contains `query`, ignoring case. A blank query keeps all.
#[must_use]
pub fn filter<'a>(items: &'a [ThreadSummary], query: &str) -> Vec<&'a ThreadSummary> {
    let q = query.trim().to_lowercase();
    items
        .iter()
        .filter(|t| q.is_empty() || t.title.to_lowercase().contains(&q) || t.preview.to_lowercase().contains(&q))
        .collect()
}

/// Key press in the rename field: commit, cancel, or nothing.
#[must_use]
pub fn rename_key(key: &str, composing: bool) -> Option<ThreadListEvent> {
    match key {
        "Enter" if !composing => Some(ThreadListEvent::CommitRename),
        "Escape" => Some(ThreadListEvent::CancelRename),
        _ => None,
    }
}

/// Accessible name for a row's open button.
#[must_use]
pub fn item_label(item: &ThreadSummary) -> String {
    if item.unread { format!("{}, unread", item.title) } else { item.title.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<ThreadSummary> {
        vec![
            ThreadSummary::new("1", "Churn drivers", "Pricing came up most", "2h"),
            ThreadSummary::new("2", "Board update", "Checked the sources", "1d").unread(),
        ]
    }

    #[test]
    fn filter_matches_title_and_preview() {
        let items = items();
        assert_eq!(filter(&items, "").len(), 2);
        assert_eq!(filter(&items, " CHURN ").len(), 1);
        assert_eq!(filter(&items, "sources")[0].id, "2");
        assert!(filter(&items, "zzz").is_empty());
    }

    #[test]
    fn rename_keys() {
        assert_eq!(rename_key("Enter", false), Some(ThreadListEvent::CommitRename));
        assert_eq!(rename_key("Enter", true), None);
        assert_eq!(rename_key("Escape", false), Some(ThreadListEvent::CancelRename));
        assert_eq!(rename_key("a", false), None);
    }

    #[test]
    fn unread_is_in_the_name() {
        assert_eq!(item_label(&items()[1]), "Board update, unread");
    }
}
