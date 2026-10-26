//! Transcript viewer connector. Anatomy: root, toolbar, search, resume-follow,
//! viewport, list, segment, speaker, time, text. Every part carries
//! `data-scope="transcript-viewer"` and its `data-part`.

pub use oxide_machines::transcript_viewer::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "transcript-viewer";

/// Default accessible name for the scrolling region.
pub const LABEL: &str = "Transcript";

/// The search field's accessible name.
pub const SEARCH_LABEL: &str = "Search transcript";

/// The resume button's text and name.
pub const RESUME_LABEL: &str = "Follow playback";

/// Default viewport height in pixels.
pub const MAX_HEIGHT: u32 = 320;

/// Keys that scroll the viewport by hand and so suspend following.
#[must_use]
pub fn is_scroll_key(key: &str) -> bool {
    matches!(key, "ArrowUp" | "ArrowDown" | "PageUp" | "PageDown" | "Home" | "End" | " ")
}

/// Enter or Space on a segment seeks to it.
#[must_use]
pub fn is_activate_key(key: &str) -> bool {
    matches!(key, "Enter" | " ")
}

/// The id of a segment's element.
#[must_use]
pub fn segment_dom_id(root: &str, segment_id: &str) -> String {
    format!("{root}-segment-{segment_id}")
}

/// The status line under the search field.
#[must_use]
pub fn count_text(shown: usize, total: usize) -> String {
    if shown == total {
        format!("{total} segments")
    } else {
        format!("{shown} of {total} segments")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_and_counts() {
        assert!(is_scroll_key("PageDown"));
        assert!(!is_scroll_key("Enter"));
        assert!(is_activate_key(" "));
        assert_eq!(count_text(2, 10), "2 of 10 segments");
        assert_eq!(count_text(3, 3), "3 segments");
        assert_eq!(segment_dom_id("t1", "a"), "t1-segment-a");
    }
}
