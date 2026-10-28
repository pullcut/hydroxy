//! A text field whose value is checked by the owner's async validator.
//! Typing restarts a debounce (a `Wait` command). When the wait for the latest
//! keystroke elapses, the machine asks for a `Validate`. Results carry the
//! request id, and any result that is not for the latest request is ignored,
//! so a slow answer for old text never overwrites a newer one.

use hydroxy_statechart::{Machine, Transition};

/// Where validation stands.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Validity {
    /// Nothing checked yet, or the text changed since the last check.
    #[default]
    Idle,
    /// The validator is running.
    Validating,
    /// The validator accepted the text.
    Valid,
    /// The validator rejected the text, with its message.
    Invalid(String),
}

/// Props for [`AsyncValidatedMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncValidatedInput {
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<String>,
    /// Uncontrolled initial value.
    pub default_value: String,
    /// Quiet time after the last keystroke before validating, in milliseconds.
    pub debounce_ms: u32,
}

impl Default for AsyncValidatedInput {
    fn default() -> Self {
        Self { value: None, default_value: String::new(), debounce_ms: 400 }
    }
}

/// What [`AsyncValidatedMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AsyncValidatedState {
    /// The text in the field.
    pub value: String,
    /// The validation outcome for the latest text.
    pub validity: Validity,
    /// Id of the latest keystroke. Waits and results for older ids are stale.
    pub request: u64,
    /// Fixed at start: the owner holds the value.
    pub controlled: bool,
}

/// Inputs to [`AsyncValidatedMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncValidatedEvent {
    /// The text changed.
    Input(String),
    /// The wait with this id ran out.
    Elapsed(u64),
    /// The validator answered the request with this id.
    Resolved {
        /// The request being answered.
        request: u64,
        /// `Ok` to accept, `Err` with the message to reject.
        result: Result<(), String>,
    },
}

/// Side effects from [`AsyncValidatedMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncValidatedCommand {
    /// The text changed (or should, when controlled).
    Change(String),
    /// Send `Elapsed(request)` after `ms` milliseconds.
    Wait {
        /// The request the wait belongs to.
        request: u64,
        /// Delay in milliseconds.
        ms: u32,
    },
    /// Run the validator on `value`, then send `Resolved` with this `request`.
    Validate {
        /// The request to answer.
        request: u64,
        /// The text to check.
        value: String,
    },
}

/// Debounced async validation state.
#[derive(Debug, Clone, Copy)]
pub struct AsyncValidatedMachine;

type Step = Transition<AsyncValidatedState, AsyncValidatedCommand>;

impl Machine for AsyncValidatedMachine {
    type Input = AsyncValidatedInput;
    type State = AsyncValidatedState;
    type Event = AsyncValidatedEvent;
    type Command = AsyncValidatedCommand;

    fn initial(input: &AsyncValidatedInput) -> AsyncValidatedState {
        AsyncValidatedState {
            value: input.value.clone().unwrap_or_else(|| input.default_value.clone()),
            controlled: input.value.is_some(),
            ..AsyncValidatedState::default()
        }
    }

    fn transition(state: &AsyncValidatedState, event: AsyncValidatedEvent, input: &AsyncValidatedInput) -> Step {
        match event {
            AsyncValidatedEvent::Input(text) => {
                let mut next = state.clone();
                next.request += 1;
                next.validity = Validity::Idle;
                next.value.clone_from(&text);
                let mut commands = vec![AsyncValidatedCommand::Change(text.clone())];
                // Empty text has nothing to validate.
                if !text.trim().is_empty() {
                    commands.push(AsyncValidatedCommand::Wait { request: next.request, ms: input.debounce_ms });
                }
                Transition::with(next, commands)
            }
            AsyncValidatedEvent::Elapsed(id) if id == state.request && state.validity == Validity::Idle => {
                let mut next = state.clone();
                next.validity = Validity::Validating;
                let value = state.value.clone();
                Transition::with(next, vec![AsyncValidatedCommand::Validate { request: id, value }])
            }
            AsyncValidatedEvent::Resolved { request, result }
                if request == state.request && state.validity == Validity::Validating =>
            {
                let mut next = state.clone();
                next.validity = match result {
                    Ok(()) => Validity::Valid,
                    Err(message) => Validity::Invalid(message),
                };
                Transition::to(next)
            }
            _ => Transition::stay(),
        }
    }

    fn sync(state: &AsyncValidatedState, input: &AsyncValidatedInput) -> Option<AsyncValidatedState> {
        let mut next = state.clone();
        if state.controlled {
            next.value = input.value.clone().unwrap_or_default();
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(state: &AsyncValidatedState, event: AsyncValidatedEvent) -> hydroxy_statechart::Step<AsyncValidatedState, AsyncValidatedCommand> {
        AsyncValidatedMachine::transition(state, event, &AsyncValidatedInput::default()).resolve(state)
    }

    fn typed(state: &AsyncValidatedState, text: &str) -> hydroxy_statechart::Step<AsyncValidatedState, AsyncValidatedCommand> {
        run(state, AsyncValidatedEvent::Input(text.into()))
    }

    fn start() -> AsyncValidatedState {
        AsyncValidatedMachine::initial(&AsyncValidatedInput::default())
    }

    #[test]
    fn typing_waits_then_validates() {
        let step = typed(&start(), "ab");
        assert_eq!(
            step.commands,
            [AsyncValidatedCommand::Change("ab".into()), AsyncValidatedCommand::Wait { request: 1, ms: 400 }]
        );
        let step = run(&step.state, AsyncValidatedEvent::Elapsed(1));
        assert_eq!(step.state.validity, Validity::Validating);
        assert_eq!(step.commands, [AsyncValidatedCommand::Validate { request: 1, value: "ab".into() }]);
        let step = run(&step.state, AsyncValidatedEvent::Resolved { request: 1, result: Ok(()) });
        assert_eq!(step.state.validity, Validity::Valid);
    }

    #[test]
    fn a_newer_keystroke_makes_the_old_wait_stale() {
        let s = typed(&typed(&start(), "a").state, "ab").state;
        let step = run(&s, AsyncValidatedEvent::Elapsed(1));
        assert_eq!(step.state.validity, Validity::Idle);
        assert!(step.commands.is_empty());
    }

    #[test]
    fn stale_results_are_ignored() {
        let s = typed(&start(), "a").state;
        let s = run(&s, AsyncValidatedEvent::Elapsed(1)).state;
        // The user types again while request 1 is still out.
        let s = typed(&s, "ab").state;
        assert_eq!(s.validity, Validity::Idle);
        let s = run(&s, AsyncValidatedEvent::Resolved { request: 1, result: Err("taken".into()) }).state;
        assert_eq!(s.validity, Validity::Idle);
        let s = run(&s, AsyncValidatedEvent::Elapsed(2)).state;
        let s = run(&s, AsyncValidatedEvent::Resolved { request: 2, result: Err("taken".into()) }).state;
        assert_eq!(s.validity, Validity::Invalid("taken".into()));
    }

    #[test]
    fn empty_text_is_not_validated() {
        let step = typed(&start(), "  ");
        assert_eq!(step.commands, [AsyncValidatedCommand::Change("  ".into())]);
    }

    #[test]
    fn controlled_value_follows_the_owner() {
        let input = AsyncValidatedInput { value: Some("x".into()), ..AsyncValidatedInput::default() };
        let s = AsyncValidatedMachine::initial(&input);
        assert!(s.controlled);
        let s = typed(&s, "xy").state;
        let input = AsyncValidatedInput { value: Some("x".into()), ..AsyncValidatedInput::default() };
        assert_eq!(hydroxy_statechart::sync_or_keep::<AsyncValidatedMachine>(&s, &input).value, "x");
    }
}
