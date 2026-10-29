//! Notification center: the list behind a bell. The machine owns the items and
//! their read state; whether the popover is open is the browser's business.
//! Controlled or uncontrolled: the owner can hold the list and take change
//! requests.

use hydroxy_statechart::{Machine, Transition};

/// One notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    /// Unique key.
    pub id: String,
    /// One line.
    pub title: String,
    /// Optional second line.
    pub description: Option<String>,
    /// When it happened, as the owner wants it shown ("2m ago").
    pub time: String,
    /// Not yet read.
    pub unread: bool,
}

impl Notification {
    /// An unread notification with no description.
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>, time: impl Into<String>) -> Self {
        Self { id: id.into(), title: title.into(), description: None, time: time.into(), unread: true }
    }

    /// Add a second line.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Mark it read.
    #[must_use]
    pub fn read(mut self) -> Self {
        self.unread = false;
        self
    }
}

/// How many of `items` are unread.
#[must_use]
pub fn unread_count(items: &[Notification]) -> usize {
    items.iter().filter(|n| n.unread).count()
}

/// Props for [`NotificationCenterMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotificationCenterInput {
    /// Controlled items. `Some` at start fixes controlled mode.
    pub items: Option<Vec<Notification>>,
    /// Uncontrolled initial items.
    pub default_items: Vec<Notification>,
}

/// What [`NotificationCenterMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotificationCenterState {
    /// The notifications, newest first as the owner ordered them.
    pub items: Vec<Notification>,
    /// Fixed at start: the owner holds `items` and the machine only requests changes.
    pub controlled: bool,
}

/// Inputs to [`NotificationCenterMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationCenterEvent {
    /// Mark one notification read.
    MarkRead(String),
    /// Mark every notification read.
    MarkAllRead,
    /// Remove one notification.
    Dismiss(String),
    /// Remove every notification.
    Clear,
}

/// Side effects from [`NotificationCenterMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationCenterCommand {
    /// Tell the owner the items changed (or should, when controlled).
    ItemsChange(Vec<Notification>),
}

/// The list and read state for one notification center.
#[derive(Debug, Clone, Copy)]
pub struct NotificationCenterMachine;

type Step = Transition<NotificationCenterState, NotificationCenterCommand>;

/// Commit (when uncontrolled) and request the change, unless nothing changed.
fn request(state: &NotificationCenterState, items: Vec<Notification>) -> Step {
    if items == state.items {
        return Transition::stay();
    }
    let mut next = state.clone();
    if !state.controlled {
        next.items.clone_from(&items);
    }
    Transition::update(state, next, vec![NotificationCenterCommand::ItemsChange(items)])
}

impl Machine for NotificationCenterMachine {
    type Input = NotificationCenterInput;
    type State = NotificationCenterState;
    type Event = NotificationCenterEvent;
    type Command = NotificationCenterCommand;

    fn initial(input: &NotificationCenterInput) -> NotificationCenterState {
        NotificationCenterState {
            items: input.items.clone().unwrap_or_else(|| input.default_items.clone()),
            controlled: input.items.is_some(),
        }
    }

    fn transition(
        state: &NotificationCenterState,
        event: NotificationCenterEvent,
        _input: &NotificationCenterInput,
    ) -> Step {
        let mut items = state.items.clone();
        match event {
            NotificationCenterEvent::MarkRead(id) => {
                items.iter_mut().filter(|n| n.id == id).for_each(|n| n.unread = false);
            }
            NotificationCenterEvent::MarkAllRead => items.iter_mut().for_each(|n| n.unread = false),
            NotificationCenterEvent::Dismiss(id) => items.retain(|n| n.id != id),
            NotificationCenterEvent::Clear => items.clear(),
        }
        request(state, items)
    }

    fn sync(state: &NotificationCenterState, input: &NotificationCenterInput) -> Option<NotificationCenterState> {
        let mut next = state.clone();
        if let (true, Some(items)) = (state.controlled, &input.items) {
            next.items.clone_from(items);
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    fn input() -> NotificationCenterInput {
        NotificationCenterInput {
            default_items: vec![
                Notification::new("a", "Build passed", "1m"),
                Notification::new("b", "New comment", "5m").read(),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn counts_unread() {
        let state = NotificationCenterMachine::initial(&input());
        assert_eq!(unread_count(&state.items), 1);
    }

    #[test]
    fn mark_read_and_mark_all_read() {
        let input = input();
        let state = NotificationCenterMachine::initial(&input);
        let step = hydroxy_statechart::step::<NotificationCenterMachine>(
            &state,
            NotificationCenterEvent::MarkRead("a".into()),
            &input,
        );
        assert_eq!(unread_count(&step.state.items), 0);
        assert_eq!(step.commands.len(), 1);
        // Already read: nothing to report.
        assert!(
            hydroxy_statechart::step::<NotificationCenterMachine>(
                &step.state,
                NotificationCenterEvent::MarkAllRead,
                &input
            )
            .commands
            .is_empty()
        );
        assert_eq!(
            unread_count(
                &hydroxy_statechart::step::<NotificationCenterMachine>(
                    &state,
                    NotificationCenterEvent::MarkAllRead,
                    &input
                )
                .state
                .items
            ),
            0
        );
    }

    #[test]
    fn dismiss_and_clear_remove_items() {
        let input = input();
        let state = NotificationCenterMachine::initial(&input);
        let one = step::<NotificationCenterMachine>(&state, NotificationCenterEvent::Dismiss("a".into()), &input);
        assert_eq!(one.state.items.len(), 1);
        assert!(
            step::<NotificationCenterMachine>(&state, NotificationCenterEvent::Dismiss("zzz".into()), &input)
                .commands
                .is_empty()
        );
        assert!(
            step::<NotificationCenterMachine>(&state, NotificationCenterEvent::Clear, &input).state.items.is_empty()
        );
    }

    #[test]
    fn controlled_requests_and_waits_for_sync() {
        let mut input = NotificationCenterInput { items: Some(input().default_items), ..Default::default() };
        let state = NotificationCenterMachine::initial(&input);
        let step = hydroxy_statechart::step::<NotificationCenterMachine>(
            &state,
            NotificationCenterEvent::Dismiss("a".into()),
            &input,
        );
        assert_eq!(step.state.items.len(), 2);
        let NotificationCenterCommand::ItemsChange(requested) = &step.commands[0];
        assert_eq!(requested.len(), 1);
        input.items = Some(requested.clone());
        assert_eq!(hydroxy_statechart::sync_or_keep::<NotificationCenterMachine>(&step.state, &input).items.len(), 1);
    }
}
