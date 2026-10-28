//! Assistant composer connector. Parts, all under `data-scope="assistant-composer"`:
//! root, quote, attachments, input, popover, option, queue, actions, add-attachment,
//! send, steer, cancel. The input is a native `<textarea>`; the popover is a `listbox`
//! whose `option`s are chosen with the arrow keys, Enter or Tab.

pub use hydroxy_machines::assistant_composer::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "assistant-composer";

/// What a key press in the input should do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    /// Send an event to the machine.
    Event(AssistantComposerEvent),
    /// Let the browser handle it.
    Pass,
}

/// A key press in the input.
#[derive(Debug, Clone, Copy)]
pub struct KeyPress<'a> {
    /// `KeyboardEvent.key`.
    pub key: &'a str,
    /// Shift is held.
    pub shift: bool,
    /// Ctrl or Cmd is held.
    pub mod_key: bool,
    /// An IME is composing.
    pub composing: bool,
}

/// What the input needs to know about the popover and the run.
#[derive(Debug, Clone, Copy)]
pub struct Popover<'a> {
    /// Rows currently listed.
    pub rows: usize,
    /// Label of the highlighted row.
    pub accept: Option<&'a str>,
    /// A reply is running.
    pub running: bool,
}

/// Map a key press in the input.
///
/// - Enter sends, Shift+Enter adds a line, Ctrl/Cmd+Enter steers.
/// - Escape closes the popover, or cancels a running reply.
/// - Up and Down move through the popover, or through earlier messages when the field
///   is empty (Up) or already recalling.
/// - Enter or Tab accepts the highlighted popover row (`accept` is its label).
#[must_use]
pub fn input_key(press: &KeyPress, state: &AssistantComposerState, popover: &Popover) -> KeyAction {
    use AssistantComposerEvent as E;
    let KeyPress { key, shift, mod_key, composing } = *press;
    let Popover { rows, accept, running } = *popover;
    if composing {
        return KeyAction::Pass;
    }
    let popover = state.trigger.is_some() && rows > 0;
    match key {
        "ArrowDown" if popover => KeyAction::Event(E::MoveActive { delta: 1, count: rows }),
        "ArrowUp" if popover => KeyAction::Event(E::MoveActive { delta: -1, count: rows }),
        "Enter" | "Tab" if popover && !shift => {
            accept.map_or(KeyAction::Pass, |label| KeyAction::Event(E::AcceptTrigger(label.to_string())))
        }
        "Escape" if state.trigger.is_some() => KeyAction::Event(E::DismissTrigger),
        "Escape" if running => KeyAction::Event(E::Cancel),
        "Enter" if mod_key => KeyAction::Event(E::Steer),
        "Enter" if !shift => KeyAction::Event(E::Send),
        "ArrowUp" if state.text.is_empty() || state.recall.is_some() => KeyAction::Event(E::HistoryPrev),
        "ArrowDown" if state.recall.is_some() => KeyAction::Event(E::HistoryNext),
        _ => KeyAction::Pass,
    }
}

/// `id` of the popover option at `index`, for `aria-activedescendant`.
#[must_use]
pub fn option_id(root_id: &str, index: usize) -> String {
    format!("{root_id}-option-{index}")
}

/// Accessible label for the send button, which becomes Queue while a reply runs.
#[must_use]
pub fn send_label(running: bool) -> &'static str {
    if running { "Queue message" } else { "Send message" }
}

/// A file the person picked or dropped, as reported by the composer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentFile {
    /// File name.
    pub name: String,
    /// Size in bytes.
    pub size: u64,
    /// MIME type, empty when unknown.
    pub mime: String,
}

/// Where an attachment is in its upload.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum AttachmentStatus {
    /// Uploading, with progress from 0 to 100.
    Uploading(u8),
    /// Ready to send.
    #[default]
    Ready,
    /// The upload failed, with the reason.
    Failed(String),
}

impl AttachmentStatus {
    /// The `data-status` value.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            AttachmentStatus::Uploading(_) => "uploading",
            AttachmentStatus::Ready => "ready",
            AttachmentStatus::Failed(_) => "failed",
        }
    }
}

/// An attachment chip's data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentItem {
    /// Stable key.
    pub id: String,
    /// File name.
    pub name: String,
    /// Size in bytes.
    pub size: Option<u64>,
    /// MIME type, empty when unknown.
    pub mime: String,
    /// Upload state.
    pub status: AttachmentStatus,
}

impl AttachmentItem {
    /// A ready attachment.
    pub fn new(id: impl Into<String>, name: impl Into<String>, size: u64) -> Self {
        Self { id: id.into(), name: name.into(), size: Some(size), mime: String::new(), status: AttachmentStatus::Ready }
    }

    /// Set the upload state.
    #[must_use]
    pub fn status(mut self, status: AttachmentStatus) -> Self {
        self.status = status;
        self
    }
}

/// The line under an attachment's name: type and size, or the upload state.
#[must_use]
pub fn attachment_meta(item: &AttachmentItem) -> String {
    match &item.status {
        AttachmentStatus::Uploading(p) => format!("Uploading {}%", (*p).min(100)),
        AttachmentStatus::Failed(reason) if reason.is_empty() => "Upload failed".to_string(),
        AttachmentStatus::Failed(reason) => format!("Upload failed: {reason}"),
        AttachmentStatus::Ready => {
            crate::attachment::meta_text(crate::attachment::kind_of(&item.name, &item.mime), item.size)
        }
    }
}

/// Convert a char index in `text` to the UTF-16 offset a textarea uses.
#[must_use]
pub fn char_to_utf16(text: &str, chars: usize) -> usize {
    text.chars().take(chars).map(char::len_utf16).sum()
}

/// Convert a textarea's UTF-16 offset in `text` to a char index.
#[must_use]
pub fn utf16_to_char(text: &str, units: usize) -> usize {
    let mut seen = 0;
    for (i, c) in text.chars().enumerate() {
        if seen >= units {
            return i;
        }
        seen += c.len_utf16();
    }
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use AssistantComposerEvent as E;

    fn st(text: &str) -> AssistantComposerState {
        AssistantComposerState { text: text.into(), ..AssistantComposerState::default() }
    }

    fn key(k: &str, shift: bool, md: bool, s: &AssistantComposerState, rows: usize, running: bool) -> KeyAction {
        let press = KeyPress { key: k, shift, mod_key: md, composing: false };
        input_key(&press, s, &Popover { rows, accept: Some("alice"), running })
    }

    #[test]
    fn enter_sends_shift_adds_a_line_mod_steers() {
        let s = st("hi");
        assert_eq!(key("Enter", false, false, &s, 0, false), KeyAction::Event(E::Send));
        assert_eq!(key("Enter", true, false, &s, 0, false), KeyAction::Pass);
        assert_eq!(key("Enter", false, true, &s, 0, true), KeyAction::Event(E::Steer));
    }

    #[test]
    fn composing_never_acts() {
        let press = KeyPress { key: "Enter", shift: false, mod_key: false, composing: true };
        assert_eq!(input_key(&press, &st("x"), &Popover { rows: 0, accept: None, running: false }), KeyAction::Pass);
    }

    #[test]
    fn popover_takes_the_arrows_and_enter() {
        let s = AssistantComposerState {
            trigger: Some(Trigger { char: '@', start: 0, query: String::new() }),
            ..st("@")
        };
        assert_eq!(key("ArrowDown", false, false, &s, 2, false), KeyAction::Event(E::MoveActive { delta: 1, count: 2 }));
        assert_eq!(key("Enter", false, false, &s, 2, false), KeyAction::Event(E::AcceptTrigger("alice".into())));
        assert_eq!(key("Escape", false, false, &s, 2, true), KeyAction::Event(E::DismissTrigger));
    }

    #[test]
    fn escape_cancels_a_running_reply() {
        assert_eq!(key("Escape", false, false, &st(""), 0, true), KeyAction::Event(E::Cancel));
        assert_eq!(key("Escape", false, false, &st(""), 0, false), KeyAction::Pass);
    }

    #[test]
    fn up_recalls_only_from_an_empty_field() {
        assert_eq!(key("ArrowUp", false, false, &st(""), 0, false), KeyAction::Event(E::HistoryPrev));
        assert_eq!(key("ArrowUp", false, false, &st("text"), 0, false), KeyAction::Pass);
    }

    #[test]
    fn send_button_becomes_queue() {
        assert_eq!(send_label(true), "Queue message");
        assert_eq!(option_id("c", 2), "c-option-2");
    }

    #[test]
    fn caret_converts_between_chars_and_utf16() {
        assert_eq!(char_to_utf16("a😀b", 2), 3);
        assert_eq!(utf16_to_char("a😀b", 3), 2);
        assert_eq!(utf16_to_char("abc", 99), 3);
    }

    #[test]
    fn attachment_meta_follows_status() {
        let a = AttachmentItem::new("1", "q3.pdf", 2_516_582);
        assert_eq!(attachment_meta(&a), "PDF, 2.4 MB");
        assert_eq!(attachment_meta(&a.clone().status(AttachmentStatus::Uploading(40))), "Uploading 40%");
        assert_eq!(attachment_meta(&a.status(AttachmentStatus::Failed("too large".into()))), "Upload failed: too large");
    }
}
