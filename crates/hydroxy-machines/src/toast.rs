//! Toast: a queue of brief notifications. The machine owns the list and ids;
//! timing lives in CSS. Each toast's lifetime is a CSS animation, and the
//! adapter sends [`ToastEvent::Dismiss`] when it ends, so no runtime timer is
//! needed. Dismissing marks a toast as leaving; [`ToastEvent::Remove`] drops it
//! once its exit animation has played.

use hydroxy_statechart::{Machine, Transition};

/// Default lifetime of a toast, in milliseconds.
pub const DEFAULT_DURATION_MS: u32 = 5000;

/// How a toast reads.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ToastTone {
    /// Plain information.
    #[default]
    Neutral,
    /// Something finished.
    Success,
    /// Something failed. Announced assertively.
    Danger,
}

impl ToastTone {
    /// The `data-tone` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ToastTone::Neutral => "neutral",
            ToastTone::Success => "success",
            ToastTone::Danger => "danger",
        }
    }
}

/// What to show. Build with [`ToastOptions::new`] and the chained setters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastOptions {
    /// One line, sentence case.
    pub title: String,
    /// Optional second line.
    pub description: Option<String>,
    /// See [`ToastTone`].
    pub tone: ToastTone,
    /// Lifetime in milliseconds. `None` stays until dismissed.
    pub duration_ms: Option<u32>,
    /// Label of an optional action button. The adapter reports clicks with the toast's id.
    pub action: Option<String>,
}

impl ToastOptions {
    /// A neutral toast that lasts [`DEFAULT_DURATION_MS`].
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            tone: ToastTone::Neutral,
            duration_ms: Some(DEFAULT_DURATION_MS),
            action: None,
        }
    }

    /// Add a second line.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the tone.
    #[must_use]
    pub fn tone(mut self, tone: ToastTone) -> Self {
        self.tone = tone;
        self
    }

    /// Set the lifetime; `None` stays until dismissed.
    #[must_use]
    pub fn duration_ms(mut self, duration_ms: Option<u32>) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Add an action button with this label.
    #[must_use]
    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.action = Some(label.into());
        self
    }
}

/// One queued toast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    /// Unique within its toaster.
    pub id: u64,
    /// What it shows.
    pub options: ToastOptions,
    /// Dismissed and playing its exit animation.
    pub leaving: bool,
}

/// Props for [`ToastMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastInput {
    /// How many toasts show at once. The rest wait their turn.
    pub max_visible: usize,
}

impl Default for ToastInput {
    fn default() -> Self {
        Self { max_visible: 3 }
    }
}

/// What [`ToastMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToastState {
    /// Every queued toast, oldest first.
    pub toasts: Vec<Toast>,
    /// The id the next pushed toast gets.
    pub next_id: u64,
}

impl ToastState {
    /// The toasts on screen: the oldest `max_visible`.
    #[must_use]
    pub fn visible(&self, input: &ToastInput) -> &[Toast] {
        &self.toasts[..self.toasts.len().min(input.max_visible)]
    }
}

/// Inputs to [`ToastMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastEvent {
    /// Queue a toast. It gets `state.next_id`.
    Push(ToastOptions),
    /// Start a toast's exit (lifetime ended, close button, or action).
    Dismiss(u64),
    /// Drop a leaving toast once its exit animation ends.
    Remove(u64),
}

/// The toast queue. It has no side effects for the adapter to run.
#[derive(Debug, Clone, Copy)]
pub struct ToastMachine;

impl Machine for ToastMachine {
    type Input = ToastInput;
    type State = ToastState;
    type Event = ToastEvent;
    type Command = ();

    fn initial(_input: &ToastInput) -> ToastState {
        ToastState::default()
    }

    fn transition(state: &ToastState, event: ToastEvent, _input: &ToastInput) -> Transition<ToastState, ()> {
        let mut next = state.clone();
        match event {
            ToastEvent::Push(options) => {
                next.toasts.push(Toast { id: next.next_id, options, leaving: false });
                next.next_id += 1;
            }
            ToastEvent::Dismiss(id) => match next.toasts.iter_mut().find(|t| t.id == id && !t.leaving) {
                Some(toast) => toast.leaving = true,
                None => return Transition::update(state, next, Vec::new()),
            },
            ToastEvent::Remove(id) => next.toasts.retain(|t| !(t.id == id && t.leaving)),
        }
        Transition::update(state, next, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    fn run(state: &ToastState, event: ToastEvent) -> ToastState {
        step::<ToastMachine>(state, event, &ToastInput::default()).state
    }

    #[test]
    fn push_assigns_increasing_ids() {
        let s = run(&ToastState::default(), ToastEvent::Push(ToastOptions::new("Saved")));
        let s = run(&s, ToastEvent::Push(ToastOptions::new("Shared")));
        assert_eq!(s.toasts.iter().map(|t| t.id).collect::<Vec<_>>(), vec![0, 1]);
        assert_eq!(s.next_id, 2);
    }

    #[test]
    fn dismiss_marks_leaving_then_remove_drops() {
        let s = run(&ToastState::default(), ToastEvent::Push(ToastOptions::new("Saved")));
        let s = run(&s, ToastEvent::Dismiss(0));
        assert!(s.toasts[0].leaving);
        let s = run(&s, ToastEvent::Remove(0));
        assert!(s.toasts.is_empty());
    }

    #[test]
    fn remove_ignores_toasts_that_are_not_leaving() {
        let s = run(&ToastState::default(), ToastEvent::Push(ToastOptions::new("Saved")));
        assert_eq!(run(&s, ToastEvent::Remove(0)).toasts.len(), 1);
    }

    #[test]
    fn only_max_visible_show_and_the_rest_wait() {
        let input = ToastInput { max_visible: 2 };
        let mut s = ToastState::default();
        for title in ["a", "b", "c"] {
            s = run(&s, ToastEvent::Push(ToastOptions::new(title)));
        }
        assert_eq!(s.visible(&input).len(), 2);
        s = run(&run(&s, ToastEvent::Dismiss(0)), ToastEvent::Remove(0));
        assert_eq!(s.visible(&input).iter().map(|t| t.id).collect::<Vec<_>>(), vec![1, 2]);
    }

    #[test]
    fn options_builder_sets_fields() {
        let o = ToastOptions::new("Couldn't read tickets-sept.csv")
            .tone(ToastTone::Danger)
            .duration_ms(None)
            .action("Retry");
        assert_eq!(o.tone, ToastTone::Danger);
        assert_eq!(o.duration_ms, None);
        assert_eq!(o.action.as_deref(), Some("Retry"));
    }
}
