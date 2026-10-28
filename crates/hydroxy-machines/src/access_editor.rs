//! Access editor: who can see something and what each person may do.
//!
//! A list of people with one role each, plus an email draft to add more. The
//! owner's role is fixed and the owner cannot be removed. Emails are checked
//! for shape and for duplicates before they are added.

use hydroxy_statechart::{Controllable, Machine, Transition};

/// Someone with access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    /// Their email, which identifies them.
    pub email: String,
    /// Display name, when known.
    pub name: Option<String>,
    /// Their role's value.
    pub role: String,
    /// The owner: role fixed, cannot be removed.
    pub owner: bool,
}

impl Person {
    /// A person with a role.
    #[must_use]
    pub fn new(email: impl Into<String>, name: impl Into<String>, role: impl Into<String>) -> Self {
        Self { email: email.into(), name: Some(name.into()), role: role.into(), owner: false }
    }

    /// Mark as the owner.
    #[must_use]
    pub fn owner(mut self) -> Self {
        self.owner = true;
        self
    }

    /// The name, or the email when there is none.
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.name.as_deref().filter(|n| !n.is_empty()).unwrap_or(&self.email)
    }
}

/// Props for [`AccessEditorMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccessEditorInput {
    /// Controlled people. `Some` at start fixes controlled mode.
    pub people: Option<Vec<Person>>,
    /// Uncontrolled initial people.
    pub default_people: Vec<Person>,
    /// Role values people can hold. Role changes outside this list are ignored.
    pub roles: Vec<String>,
    /// Role given to people added by email.
    pub new_role: String,
    /// Ignore edits.
    pub disabled: bool,
}

/// What [`AccessEditorMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccessEditorState {
    /// Everyone with access.
    pub people: Controllable<Vec<Person>>,
    /// The email being typed.
    pub draft: String,
    /// Why the last add failed. Cleared by typing.
    pub error: Option<AddError>,
}

/// Why an email was not added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddError {
    /// Empty or not shaped like an email.
    Invalid,
    /// Already has access.
    Duplicate,
}

/// Inputs to [`AccessEditorMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessEditorEvent {
    /// The email draft changed.
    SetDraft(String),
    /// Enter or the Add button: add the draft.
    Add,
    /// A person's role was picked.
    SetRole {
        /// Whose role.
        email: String,
        /// The new role's value.
        role: String,
    },
    /// A person's remove button was pressed.
    Remove(String),
}

/// Side effects from [`AccessEditorMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessEditorCommand {
    /// Tell the owner the people changed (or should, when controlled).
    PeopleChange(Vec<Person>),
    /// Someone was added. For the live region.
    Added(String),
    /// Someone was removed. For the live region.
    Removed(String),
    /// An add was refused. For the live region.
    Rejected(AddError),
}

/// People, roles and the add-by-email draft.
#[derive(Debug, Clone, Copy)]
pub struct AccessEditorMachine;

type Step = Transition<AccessEditorState, AccessEditorCommand>;

/// Whether `text` looks like an email: one `@`, a name before it, a dotted domain after, no spaces.
#[must_use]
pub fn is_valid_email(text: &str) -> bool {
    let Some((local, domain)) = text.split_once('@') else { return false };
    !local.is_empty()
        && !text.chars().any(char::is_whitespace)
        && !domain.contains('@')
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
}

fn change(mut state: AccessEditorState, people: Vec<Person>, extra: AccessEditorCommand) -> Step {
    let commands = vec![AccessEditorCommand::PeopleChange(people.clone()), extra];
    state.people.set(people);
    Transition::with(state, commands)
}

impl Machine for AccessEditorMachine {
    type Input = AccessEditorInput;
    type State = AccessEditorState;
    type Event = AccessEditorEvent;
    type Command = AccessEditorCommand;

    fn initial(input: &AccessEditorInput) -> AccessEditorState {
        AccessEditorState {
            people: Controllable::new(
                input.people.clone().unwrap_or_else(|| input.default_people.clone()),
                input.people.is_some(),
            ),
            ..AccessEditorState::default()
        }
    }

    fn transition(state: &AccessEditorState, event: AccessEditorEvent, input: &AccessEditorInput) -> Step {
        let mut next = state.clone();
        if let AccessEditorEvent::SetDraft(text) = event {
            next.draft = text;
            next.error = None;
            return Transition::to(next);
        }
        if input.disabled {
            return Transition::to(next);
        }
        match event {
            AccessEditorEvent::Add => {
                let email = state.draft.trim().to_string();
                let error = if !is_valid_email(&email) {
                    Some(AddError::Invalid)
                } else if state.people.value.iter().any(|p| p.email.eq_ignore_ascii_case(&email)) {
                    Some(AddError::Duplicate)
                } else {
                    None
                };
                if let Some(error) = error {
                    next.error = Some(error);
                    return Transition::with(next, vec![AccessEditorCommand::Rejected(error)]);
                }
                next.draft.clear();
                next.error = None;
                let mut people = state.people.value.clone();
                people.push(Person { email: email.clone(), name: None, role: input.new_role.clone(), owner: false });
                change(next, people, AccessEditorCommand::Added(email))
            }
            AccessEditorEvent::SetRole { email, role } => {
                if !(input.roles.is_empty() || input.roles.contains(&role)) {
                    return Transition::to(next);
                }
                let mut people = state.people.value.clone();
                let Some(p) = people.iter_mut().find(|p| p.email == email && !p.owner && p.role != role) else {
                    return Transition::to(next);
                };
                p.role = role;
                // Role changes are reported through the people list; the live region stays quiet.
                let commands = vec![AccessEditorCommand::PeopleChange(people.clone())];
                next.people.set(people);
                Transition::with(next, commands)
            }
            AccessEditorEvent::Remove(email) => {
                let Some(p) = state.people.value.iter().find(|p| p.email == email && !p.owner) else {
                    return Transition::to(next);
                };
                let name = p.display_name().to_string();
                let people = state.people.value.iter().filter(|p| p.email != email).cloned().collect();
                change(next, people, AccessEditorCommand::Removed(name))
            }
            AccessEditorEvent::SetDraft(_) => Transition::to(next),
        }
    }

    fn sync(state: &AccessEditorState, input: &AccessEditorInput) -> Option<AccessEditorState> {
        let mut next = state.clone();
        next.people.sync(input.people.clone());
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step as advance;

    fn input() -> AccessEditorInput {
        AccessEditorInput {
            default_people: vec![
                Person::new("ana@example.com", "Ana", "owner").owner(),
                Person::new("jo@example.com", "Jo", "viewer"),
            ],
            roles: vec!["owner".into(), "editor".into(), "viewer".into()],
            new_role: "viewer".into(),
            ..AccessEditorInput::default()
        }
    }

    fn send(
        state: &AccessEditorState,
        e: AccessEditorEvent,
    ) -> hydroxy_statechart::Step<AccessEditorState, AccessEditorCommand> {
        advance::<AccessEditorMachine>(state, e, &input())
    }

    #[test]
    fn email_shape() {
        assert!(is_valid_email("a@b.co"));
        for bad in ["", "a", "a@b", "@b.co", "a@@b.co", "a b@c.co", "a@.co", "a@b.", "a@b..co"] {
            assert!(!is_valid_email(bad), "{bad}");
        }
    }

    #[test]
    fn add_validates_and_dedupes() {
        let s = AccessEditorMachine::initial(&input());
        let s = send(&s, AccessEditorEvent::SetDraft("nope".into())).state;
        let t = send(&s, AccessEditorEvent::Add);
        assert_eq!(t.state.error, Some(AddError::Invalid));
        let s = send(&t.state, AccessEditorEvent::SetDraft("JO@example.com".into())).state;
        assert_eq!(send(&s, AccessEditorEvent::Add).state.error, Some(AddError::Duplicate));
        let s = send(&s, AccessEditorEvent::SetDraft(" kim@example.com ".into())).state;
        let t = send(&s, AccessEditorEvent::Add);
        assert_eq!(t.state.people.value.len(), 3);
        assert_eq!(t.state.people.value[2].role, "viewer");
        assert!(t.state.draft.is_empty());
    }

    #[test]
    fn owner_is_fixed_and_others_can_change() {
        let s = AccessEditorMachine::initial(&input());
        assert_eq!(send(&s, AccessEditorEvent::Remove("ana@example.com".into())).state.people.value.len(), 2);
        let t = send(&s, AccessEditorEvent::SetRole { email: "ana@example.com".into(), role: "viewer".into() });
        assert_eq!(t.state.people.value[0].role, "owner");
        let t = send(&s, AccessEditorEvent::SetRole { email: "jo@example.com".into(), role: "editor".into() });
        assert_eq!(t.state.people.value[1].role, "editor");
        let t = send(&s, AccessEditorEvent::SetRole { email: "jo@example.com".into(), role: "admin".into() });
        assert_eq!(t.state.people.value[1].role, "viewer");
        assert_eq!(send(&s, AccessEditorEvent::Remove("jo@example.com".into())).state.people.value.len(), 1);
    }

    #[test]
    fn controlled_only_requests() {
        let i = AccessEditorInput { people: Some(input().default_people), ..input() };
        let s = AccessEditorMachine::initial(&i);
        let t = advance::<AccessEditorMachine>(&s, AccessEditorEvent::Remove("jo@example.com".into()), &i);
        assert_eq!(t.state.people.value.len(), 2);
        assert!(matches!(&t.commands[0], AccessEditorCommand::PeopleChange(p) if p.len() == 1));
    }
}
