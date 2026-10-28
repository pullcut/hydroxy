//! Tags input connector. Anatomy: root, label, field, tag, remove, control,
//! counter, live. Every part carries `data-scope="tags-input"` and its `data-part`.

pub use hydroxy_machines::tags_input::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "tags-input";

/// Element ids for one tags input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The draft `<input>`, which the label points at.
    pub control: String,
    /// The "3 of 5" counter, referenced by `aria-describedby`.
    pub counter: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { control: format!("{root}-control"), counter: format!("{root}-counter") }
}

/// The counter text, such as "3 of 5". `None` without a `max`.
#[must_use]
pub fn counter(count: usize, max: Option<usize>) -> Option<String> {
    max.map(|max| format!("{count} of {max}"))
}

/// The remove button's accessible name.
#[must_use]
pub fn remove_label(tag: &str) -> String {
    format!("Remove {tag}")
}

/// What the live region says after a command. `None` for commands it stays quiet about.
#[must_use]
pub fn announcement(command: &TagsInputCommand) -> Option<String> {
    match command {
        TagsInputCommand::ValueChange(_) => None,
        TagsInputCommand::Added(tag) => Some(format!("Added {tag}")),
        TagsInputCommand::Removed(tag) => Some(format!("Removed {tag}")),
        TagsInputCommand::Rejected(tag, RejectReason::Duplicate) => Some(format!("{tag} is already added")),
        TagsInputCommand::Rejected(_, RejectReason::Limit) => Some("Tag limit reached".to_string()),
        TagsInputCommand::Highlighted(tag) => Some(format!("{tag} selected. Press Backspace to remove it")),
    }
}

/// Map a `KeyboardEvent.key` on the draft input to a machine event.
/// `Some` means: prevent the default.
#[must_use]
pub fn keydown(key: &str, state: &TagsInputState) -> Option<TagsInputEvent> {
    let empty = state.draft.is_empty();
    match key {
        "Enter" => Some(TagsInputEvent::Commit),
        "Backspace" if empty => Some(TagsInputEvent::Backspace),
        "Delete" if state.highlighted.is_some() => Some(TagsInputEvent::Delete),
        "ArrowLeft" if empty => Some(TagsInputEvent::MovePrev),
        "ArrowRight" if empty && state.highlighted.is_some() => Some(TagsInputEvent::MoveNext),
        _ => None,
    }
}

/// The live-region text for a batch of commands: the last thing worth saying.
#[must_use]
pub fn last_announcement(commands: &[TagsInputCommand]) -> Option<String> {
    commands.iter().rev().find_map(announcement)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_depend_on_the_draft_and_highlight() {
        let empty = TagsInputState::default();
        assert_eq!(keydown("Backspace", &empty), Some(TagsInputEvent::Backspace));
        assert_eq!(keydown("ArrowRight", &empty), None);
        let typing = TagsInputState { draft: "x".into(), ..TagsInputState::default() };
        assert_eq!(keydown("Backspace", &typing), None);
        assert_eq!(keydown("Enter", &typing), Some(TagsInputEvent::Commit));
    }

    #[test]
    fn counter_and_announcements_read_plainly() {
        assert_eq!(counter(3, Some(5)).as_deref(), Some("3 of 5"));
        assert_eq!(counter(3, None), None);
        let cmds = [TagsInputCommand::ValueChange(vec![]), TagsInputCommand::Removed("billing".into())];
        assert_eq!(last_announcement(&cmds).as_deref(), Some("Removed billing"));
    }
}
