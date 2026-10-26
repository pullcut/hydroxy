//! Comment thread connector. Anatomy: root, thread (one per top-level comment), comment,
//! avatar, author, time, body, replies, actions, resolved summary, reply composer.
//! Every part carries `data-scope="comment-thread"` and its `data-part`; threads carry
//! `data-state="open"` or `"resolved"`.
//!
//! Reply boxes use the Composer: Enter sends, Escape (handled by the adapter) closes.

pub use oxide_machines::comment_thread::*;

pub use crate::jewel::Jewel;

/// `data-scope` for every part.
pub const SCOPE: &str = "comment-thread";

/// One comment. Top-level comments start a thread and carry the replies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    /// Stable key. A thread's id is its top-level comment's id.
    pub id: String,
    /// Author's full name.
    pub author: String,
    /// Avatar initials.
    pub initials: String,
    /// Avatar tint.
    pub tint: Jewel,
    /// Unix seconds when written.
    pub created: i64,
    /// Plain text. Line breaks are kept.
    pub body: String,
    /// Replies, oldest first. Ignored on a reply.
    pub replies: Vec<Comment>,
}

impl Comment {
    /// A comment with no replies.
    pub fn new(
        id: impl Into<String>,
        author: impl Into<String>,
        initials: impl Into<String>,
        created: i64,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            author: author.into(),
            initials: initials.into(),
            tint: Jewel::default(),
            created,
            body: body.into(),
            replies: Vec::new(),
        }
    }

    /// Set the avatar tint.
    #[must_use]
    pub fn tint(mut self, tint: Jewel) -> Self {
        self.tint = tint;
        self
    }

    /// Set the replies.
    #[must_use]
    pub fn replies(mut self, replies: Vec<Comment>) -> Self {
        self.replies = replies;
        self
    }
}

/// A short age such as "just now", "5m", "2h", "3d" or "6w", from unix seconds. The owner
/// passes `now`, so server and browser agree and the machine never reads a clock.
#[must_use]
pub fn relative_time(now: i64, then: i64) -> String {
    let secs = now - then;
    match secs {
        i64::MIN..=44 => "just now".to_string(),
        45..=3_599 => format!("{}m", (secs / 60).max(1)),
        3_600..=86_399 => format!("{}h", secs / 3_600),
        86_400..=604_799 => format!("{}d", secs / 86_400),
        _ => format!("{}w", secs / 604_800),
    }
}

/// `data-state` for a thread.
#[must_use]
pub fn thread_state(state: &CommentThreadState, id: &str) -> &'static str {
    if state.is_resolved(id) { "resolved" } else { "open" }
}

/// Summary line for a collapsed, resolved thread.
#[must_use]
pub fn resolved_summary(author: &str, replies: usize) -> String {
    match replies {
        0 => format!("Resolved thread from {author}"),
        1 => format!("Resolved thread from {author}, 1 reply"),
        n => format!("Resolved thread from {author}, {n} replies"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_times() {
        assert_eq!(relative_time(1000, 1000), "just now");
        assert_eq!(relative_time(1000, 1010), "just now");
        assert_eq!(relative_time(1000, 1000 - 300), "5m");
        assert_eq!(relative_time(100_000, 100_000 - 7_300), "2h");
        assert_eq!(relative_time(1_000_000, 1_000_000 - 3 * 86_400), "3d");
        assert_eq!(relative_time(10_000_000, 10_000_000 - 2_000_000), "3w");
    }

    #[test]
    fn summary_pluralises() {
        assert_eq!(resolved_summary("Jo", 0), "Resolved thread from Jo");
        assert_eq!(resolved_summary("Jo", 1), "Resolved thread from Jo, 1 reply");
        assert_eq!(resolved_summary("Jo", 4), "Resolved thread from Jo, 4 replies");
    }
}
