//! Thread connector. A thread composes the message scroller, chat bubbles and a composer,
//! so there is no machine of its own: this module holds the shared data types and the
//! small decisions the adapters share. Parts: root, scroller, message, actions, typing,
//! stop, composer, all under `data-scope="thread"`.

/// `data-scope` for every part.
pub const SCOPE: &str = "thread";

/// Who wrote a message.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Role {
    /// The assistant.
    #[default]
    Assistant,
    /// The person.
    User,
}

impl Role {
    /// The `data-role` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Role::Assistant => "assistant",
            Role::User => "user",
        }
    }
}

/// One message in a thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadMessage {
    /// Stable key.
    pub id: String,
    /// Who wrote it.
    pub role: Role,
    /// Plain text. Line breaks are kept.
    pub text: String,
    /// Display time, such as "09:42".
    pub time: Option<String>,
    /// File names behind an answer.
    pub sources: Vec<String>,
}

impl ThreadMessage {
    /// A message with no time or sources.
    pub fn new(id: impl Into<String>, role: Role, text: impl Into<String>) -> Self {
        Self { id: id.into(), role, text: text.into(), time: None, sources: Vec::new() }
    }

    /// Set the display time.
    #[must_use]
    pub fn time(mut self, time: impl Into<String>) -> Self {
        self.time = Some(time.into());
        self
    }

    /// Set the sources.
    #[must_use]
    pub fn sources(mut self, sources: Vec<String>) -> Self {
        self.sources = sources;
        self
    }
}

/// A per-message action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageAction {
    /// Copy the text. The adapter also writes it to the clipboard.
    Copy,
    /// Ask for the answer again.
    Retry,
}

/// Show the typing indicator: a reply is coming and has no text yet.
#[must_use]
pub fn typing_visible(messages: &[ThreadMessage], streaming: bool) -> bool {
    streaming && messages.last().is_none_or(|m| m.role == Role::User || m.text.is_empty())
}

/// The message a Retry action applies to: the newest assistant message, when it is the last one.
#[must_use]
pub fn retry_target(messages: &[ThreadMessage]) -> Option<&str> {
    messages.last().filter(|m| m.role == Role::Assistant).map(|m| m.id.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msgs() -> Vec<ThreadMessage> {
        vec![ThreadMessage::new("1", Role::User, "hi"), ThreadMessage::new("2", Role::Assistant, "hello")]
    }

    #[test]
    fn typing_shows_until_the_reply_has_text() {
        let mut m = msgs();
        assert!(!typing_visible(&m, false));
        assert!(!typing_visible(&m, true));
        m.push(ThreadMessage::new("3", Role::User, "more"));
        assert!(typing_visible(&m, true));
        m.push(ThreadMessage::new("4", Role::Assistant, ""));
        assert!(typing_visible(&m, true));
        assert!(typing_visible(&[], true));
    }

    #[test]
    fn retry_only_for_a_trailing_assistant_message() {
        let mut m = msgs();
        assert_eq!(retry_target(&m), Some("2"));
        m.push(ThreadMessage::new("3", Role::User, "x"));
        assert_eq!(retry_target(&m), None);
    }
}
