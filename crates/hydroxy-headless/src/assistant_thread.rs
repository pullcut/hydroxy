//! Assistant thread connector. The message, its parts, the action bar, branch picker,
//! chain of thought, error, queue item, suggestion, attachment, selection toolbar, modal
//! and thread list item share this module: they are one family under the thread.
//!
//! Each component has its own `data-scope`, listed as the `SCOPE_*` constants. Parts
//! carry `data-part`. Thread-level pieces (scroller, composer, bubbles) are reused from
//! [`crate::thread`], [`crate::message_scroller`] and [`crate::attachment`].

use std::fmt::Write;

pub use hydroxy_machines::assistant_thread::*;

/// `data-scope` of the thread root.
pub const SCOPE_THREAD: &str = "assistant-thread";
/// `data-scope` of a message.
pub const SCOPE_MESSAGE: &str = "assistant-message";
/// `data-scope` of a message part.
pub const SCOPE_PART: &str = "assistant-message-part";
/// `data-scope` of the action bar.
pub const SCOPE_ACTION_BAR: &str = "assistant-action-bar";
/// `data-scope` of the branch picker.
pub const SCOPE_BRANCH_PICKER: &str = "assistant-branch-picker";
/// `data-scope` of the chain of thought.
pub const SCOPE_CHAIN: &str = "assistant-chain-of-thought";
/// `data-scope` of the error.
pub const SCOPE_ERROR: &str = "assistant-error";
/// `data-scope` of a queued message.
pub const SCOPE_QUEUE_ITEM: &str = "assistant-queue-item";
/// `data-scope` of a suggestion.
pub const SCOPE_SUGGESTION: &str = "assistant-suggestion";
/// `data-scope` of the selection toolbar.
pub const SCOPE_SELECTION_TOOLBAR: &str = "assistant-selection-toolbar";
/// `data-scope` of the modal.
pub const SCOPE_MODAL: &str = "assistant-modal";
/// `data-scope` of the thread list.
pub const SCOPE_THREAD_LIST: &str = "assistant-thread-list";
/// `data-scope` of a thread list item.
pub const SCOPE_THREAD_LIST_ITEM: &str = "assistant-thread-list-item";
/// `data-scope` of an attachment.
pub const SCOPE_ATTACHMENT: &str = "assistant-attachment";

pub use crate::thread::Role;

/// How far a part has got.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PartStatus {
    /// Done.
    #[default]
    Complete,
    /// Still arriving or running.
    Running,
    /// Stopped by an error.
    Failed,
}

impl PartStatus {
    /// The status label shown beside a part: "Done", "Running" or "Failed".
    #[must_use]
    pub const fn status_text(self) -> &'static str {
        match self {
            PartStatus::Complete => "Done",
            PartStatus::Running => "Running",
            PartStatus::Failed => "Failed",
        }
    }

    /// The `data-status` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            PartStatus::Complete => "complete",
            PartStatus::Running => "running",
            PartStatus::Failed => "failed",
        }
    }
}

/// One piece of a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    /// Markdown text.
    Text(String),
    /// An image by URL.
    Image {
        /// Image URL.
        src: String,
        /// Alt text.
        alt: String,
    },
    /// The model's reasoning.
    Reasoning(String),
    /// A tool call.
    Tool {
        /// Tool name.
        name: String,
        /// Arguments as shown.
        args: String,
        /// Result as shown, once there is one.
        result: Option<String>,
        /// Where the call stands.
        status: PartStatus,
    },
    /// A source the answer used.
    Source {
        /// Title.
        title: String,
        /// Link.
        url: String,
    },
}

impl Part {
    /// The `data-type` value.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Part::Text(_) => "text",
            Part::Image { .. } => "image",
            Part::Reasoning(_) => "reasoning",
            Part::Tool { .. } => "tool-call",
            Part::Source { .. } => "source",
        }
    }
}

/// One message in an assistant thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantMessageData {
    /// Stable key.
    pub id: String,
    /// Who wrote it.
    pub role: Role,
    /// Its parts, in order.
    pub parts: Vec<Part>,
    /// The run failed with this message.
    pub error: Option<String>,
    /// Still being written.
    pub streaming: bool,
}

impl AssistantMessageData {
    /// A message with the given parts.
    pub fn new(id: impl Into<String>, role: Role, parts: Vec<Part>) -> Self {
        Self { id: id.into(), role, parts, error: None, streaming: false }
    }

    /// A message of one text part.
    pub fn text(id: impl Into<String>, role: Role, text: impl Into<String>) -> Self {
        Self::new(id, role, vec![Part::Text(text.into())])
    }

    /// Mark as failed.
    #[must_use]
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Mark as still being written.
    #[must_use]
    pub fn streaming(mut self) -> Self {
        self.streaming = true;
        self
    }

    /// The text parts joined by blank lines. This is what Copy writes.
    #[must_use]
    pub fn plain_text(&self) -> String {
        let texts: Vec<&str> =
            self.parts.iter().filter_map(|p| if let Part::Text(t) = p { Some(t.as_str()) } else { None }).collect();
        texts.join("\n\n")
    }
}

/// A run of parts rendered together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartGroup {
    /// One text, image or source part.
    Single(Part),
    /// Consecutive reasoning and tool parts, shown as a chain of thought.
    Chain(Vec<Part>),
}

/// Fold consecutive reasoning and tool parts into one chain of thought each.
#[must_use]
pub fn group_parts(parts: &[Part]) -> Vec<PartGroup> {
    let mut groups: Vec<PartGroup> = Vec::new();
    for part in parts {
        let chained = matches!(part, Part::Reasoning(_) | Part::Tool { .. });
        match (chained, groups.last_mut()) {
            (true, Some(PartGroup::Chain(run))) => run.push(part.clone()),
            (true, _) => groups.push(PartGroup::Chain(vec![part.clone()])),
            (false, _) => groups.push(PartGroup::Single(part.clone())),
        }
    }
    groups
}

/// The chain's trigger text: how many steps, and whether any is still running.
#[must_use]
pub fn chain_summary(parts: &[Part]) -> String {
    let steps = parts.len();
    let running = parts.iter().any(|p| matches!(p, Part::Tool { status: PartStatus::Running, .. }));
    let noun = if steps == 1 { "step" } else { "steps" };
    if running { format!("Working, {steps} {noun} so far") } else { format!("{steps} {noun}") }
}

/// A message as Markdown, for the export action.
#[must_use]
pub fn export_markdown(message: &AssistantMessageData) -> String {
    let mut out: Vec<String> = Vec::new();
    for part in &message.parts {
        match part {
            Part::Text(t) if !t.is_empty() => out.push(t.clone()),
            Part::Image { src, alt } => out.push(format!("![{alt}]({src})")),
            Part::Reasoning(t) => out.push(format!("> {}", t.replace('\n', "\n> "))),
            Part::Tool { name, args, result, .. } => {
                let mut block = format!("**Tool: {name}**\n\n```json\n{args}\n```");
                if let Some(r) = result {
                    let _ = write!(block, "\n\nResult:\n\n```\n{r}\n```");
                }
                out.push(block);
            }
            Part::Source { title, url } => out.push(format!("[{title}]({url})")),
            Part::Text(_) => {}
        }
    }
    out.join("\n\n")
}

/// Label for the branch picker: `2 / 3`.
#[must_use]
pub fn branch_label(index: usize, count: usize) -> String {
    format!("{} / {count}", index + 1)
}

/// The action bar's buttons for a message: the person's own messages offer Edit and
/// Copy; the assistant's offer Copy, Reload, Export, Speak and the two votes.
#[must_use]
pub fn actions_for(role: Role) -> &'static [&'static str] {
    match role {
        Role::User => &["edit", "copy"],
        Role::Assistant => &["copy", "reload", "export", "speak", "positive", "negative"],
    }
}

/// A conversation in the thread list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadListEntry {
    /// Stable key.
    pub id: String,
    /// Title.
    pub title: String,
    /// Archived conversations show in their own section.
    pub archived: bool,
}

impl ThreadListEntry {
    /// An active conversation.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self { id: id.into(), title: title.into(), archived: false }
    }

    /// Mark as archived.
    #[must_use]
    pub fn archived(mut self) -> Self {
        self.archived = true;
        self
    }
}

/// Split entries into (active, archived), keeping order.
#[must_use]
pub fn split_archived(entries: &[ThreadListEntry]) -> (Vec<&ThreadListEntry>, Vec<&ThreadListEntry>) {
    entries.iter().partition(|e| !e.archived)
}

/// A suggestion card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// Bold first line.
    pub title: String,
    /// Dimmer second line.
    pub description: String,
    /// The text sent. Falls back to the title and description when empty.
    pub prompt: String,
}

impl Suggestion {
    /// A suggestion that sends `prompt`.
    pub fn new(title: impl Into<String>, description: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self { title: title.into(), description: description.into(), prompt: prompt.into() }
    }

    /// The text to send when picked.
    #[must_use]
    pub fn text(&self) -> String {
        if self.prompt.is_empty() {
            format!("{} {}", self.title, self.description).trim().to_string()
        } else {
            self.prompt.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(status: PartStatus) -> Part {
        Part::Tool { name: "search".into(), args: "{}".into(), result: None, status }
    }

    #[test]
    fn grouping_chains_consecutive_reasoning_and_tools() {
        let parts = vec![
            Part::Reasoning("think".into()),
            tool(PartStatus::Complete),
            Part::Text("answer".into()),
            tool(PartStatus::Complete),
        ];
        let groups = group_parts(&parts);
        assert_eq!(groups.len(), 3);
        assert!(matches!(&groups[0], PartGroup::Chain(c) if c.len() == 2));
        assert!(matches!(&groups[1], PartGroup::Single(Part::Text(_))));
        assert!(matches!(&groups[2], PartGroup::Chain(c) if c.len() == 1));
    }

    #[test]
    fn chain_summary_counts_and_flags_running() {
        assert_eq!(chain_summary(&[tool(PartStatus::Complete)]), "1 step");
        assert_eq!(chain_summary(&[tool(PartStatus::Complete), tool(PartStatus::Running)]), "Working, 2 steps so far");
    }

    #[test]
    fn plain_text_joins_text_parts_only() {
        let m = AssistantMessageData::new(
            "1",
            Role::Assistant,
            vec![Part::Text("a".into()), Part::Reasoning("x".into()), Part::Text("b".into())],
        );
        assert_eq!(m.plain_text(), "a\n\nb");
    }

    #[test]
    fn export_covers_each_part() {
        let m = AssistantMessageData::new(
            "1",
            Role::Assistant,
            vec![
                Part::Text("hello".into()),
                Part::Reasoning("a\nb".into()),
                Part::Tool {
                    name: "t".into(),
                    args: "{}".into(),
                    result: Some("ok".into()),
                    status: PartStatus::Complete,
                },
                Part::Source { title: "Doc".into(), url: "https://x.dev".into() },
            ],
        );
        let md = export_markdown(&m);
        assert!(md.starts_with("hello\n\n> a\n> b"));
        assert!(md.contains("**Tool: t**"));
        assert!(md.contains("Result:"));
        assert!(md.ends_with("[Doc](https://x.dev)"));
    }

    #[test]
    fn branch_label_is_one_based() {
        assert_eq!(branch_label(1, 3), "2 / 3");
    }

    #[test]
    fn roles_get_their_own_actions() {
        assert_eq!(actions_for(Role::User), ["edit", "copy"]);
        assert!(actions_for(Role::Assistant).contains(&"reload"));
    }

    #[test]
    fn archived_entries_split_out() {
        let e = vec![ThreadListEntry::new("1", "A"), ThreadListEntry::new("2", "B").archived()];
        let (active, archived) = split_archived(&e);
        assert_eq!((active.len(), archived.len()), (1, 1));
    }

    #[test]
    fn suggestion_falls_back_to_its_words() {
        assert_eq!(Suggestion::new("Summarise", "this thread", "").text(), "Summarise this thread");
        assert_eq!(Suggestion::new("A", "B", "Do it").text(), "Do it");
    }
}
