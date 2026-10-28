//! Access editor connector. Anatomy: root, list, person, who, name, email,
//! role, remove, form, field, add, error, live. Every part carries
//! `data-scope="access-editor"` and its `data-part`.

pub use hydroxy_machines::access_editor::*;

use crate::jewel::Jewel;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "access-editor";

/// Element ids for one access editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The email `<input>`, which the label points at.
    pub control: String,
    /// The error message, referenced by `aria-describedby`.
    pub error: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { control: format!("{root}-control"), error: format!("{root}-error") }
}

/// Up to two initials from a name or email.
#[must_use]
pub fn initials(person: &Person) -> String {
    let source = person.display_name().split('@').next().unwrap_or_default();
    let words: Vec<&str> = source
        .split(|c: char| c.is_whitespace() || matches!(c, '.' | '_' | '-' | '@'))
        .filter(|w| !w.is_empty())
        .collect();
    let letters: String = match words.as_slice() {
        [] => "?".to_string(),
        [one] => one.chars().take(2).collect(),
        [first, .., last] => first.chars().take(1).chain(last.chars().take(1)).collect(),
    };
    letters.to_uppercase()
}

/// A stable avatar tint for an email.
#[must_use]
pub fn jewel_for(email: &str) -> Jewel {
    const ALL: [Jewel; 6] =
        [Jewel::Amethyst, Jewel::Sapphire, Jewel::Emerald, Jewel::Citrine, Jewel::Rhodolite, Jewel::Garnet];
    let sum = email.bytes().fold(0usize, |acc, b| acc.wrapping_mul(31).wrapping_add(usize::from(b)));
    ALL[sum % ALL.len()]
}

/// The visible error under the email field.
#[must_use]
pub const fn error_text(error: AddError) -> &'static str {
    match error {
        AddError::Invalid => "Enter a valid email address.",
        AddError::Duplicate => "That person already has access.",
    }
}

/// The remove button's accessible name.
#[must_use]
pub fn remove_label(person: &Person) -> String {
    format!("Remove {}", person.display_name())
}

/// The role select's accessible name.
#[must_use]
pub fn role_label(person: &Person) -> String {
    format!("Role for {}", person.display_name())
}

/// What the live region says after a command. `None` for commands it stays quiet about.
#[must_use]
pub fn announcement(command: &AccessEditorCommand) -> Option<String> {
    match command {
        AccessEditorCommand::PeopleChange(_) => None,
        AccessEditorCommand::Added(email) => Some(format!("{email} added")),
        AccessEditorCommand::Removed(name) => Some(format!("{name} removed")),
        AccessEditorCommand::Rejected(error) => Some(error_text(*error).to_string()),
    }
}

/// The live-region text for a batch of commands.
#[must_use]
pub fn last_announcement(commands: &[AccessEditorCommand]) -> Option<String> {
    commands.iter().rev().find_map(announcement)
}

/// Map a `KeyboardEvent.key` on the email input to a machine event.
#[must_use]
pub fn keydown(key: &str) -> Option<AccessEditorEvent> {
    (key == "Enter").then_some(AccessEditorEvent::Add)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_from_names_and_emails() {
        assert_eq!(initials(&Person::new("a@x.co", "Ana Kowal", "viewer")), "AK");
        assert_eq!(initials(&Person::new("a@x.co", "Jo", "viewer")), "JO");
        let bare = Person { email: "sam.lee@x.co".into(), name: None, role: "viewer".into(), owner: false };
        assert_eq!(initials(&bare), "SL");
    }

    #[test]
    fn labels_and_announcements() {
        let p = Person::new("a@x.co", "Ana", "viewer");
        assert_eq!(remove_label(&p), "Remove Ana");
        assert_eq!(
            last_announcement(&[AccessEditorCommand::Rejected(AddError::Duplicate)]).as_deref(),
            Some("That person already has access.")
        );
        assert_eq!(keydown("Enter"), Some(AccessEditorEvent::Add));
    }
}
