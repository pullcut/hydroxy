//! Combobox owns the typed query, the open list, the highlighted option and the
//! selected value. Focus never leaves the input; the highlight is announced with
//! `aria-activedescendant`.

use hydroxy_statechart::{Controllable, Machine, Transition};

pub use crate::list::Direction;
use crate::listbox;

/// One option, identified by its `value`.
#[derive(Debug, Clone, PartialEq)]
pub struct ComboboxOption {
    /// Unique key. Reported by `on_value_change`.
    pub value: String,
    /// Shown in the list and, once chosen, in the input. Matched against the query.
    pub label: String,
    /// Shown but not selectable, and skipped by the arrow keys.
    pub disabled: bool,
}

impl ComboboxOption {
    /// An enabled option.
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self { value: value.into(), label: label.into(), disabled: false }
    }

    /// Mark this option disabled.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Props for [`ComboboxMachine`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComboboxInput {
    /// The options, in display order.
    pub options: Vec<ComboboxOption>,
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<String>,
    /// Uncontrolled initial value.
    pub default_value: Option<String>,
}

impl ComboboxInput {
    fn label_of(&self, value: Option<&str>) -> String {
        value.and_then(|v| self.options.iter().find(|o| o.value == v)).map(|o| o.label.clone()).unwrap_or_default()
    }
}

/// What [`ComboboxMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComboboxState {
    /// The selected option.
    pub value: Controllable<Option<String>>,
    /// The input's text.
    pub query: String,
    /// Whether the list is showing.
    pub open: bool,
    /// The highlighted option.
    pub highlighted: Option<String>,
}

/// Inputs to [`ComboboxMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum ComboboxEvent {
    /// The user typed: filter and open.
    Input(String),
    /// Open the list (click on the input or its chevron).
    Open,
    /// Arrow key: open, or move the highlight.
    Move(Direction),
    /// Pointer moved over this option.
    Highlight(String),
    /// Choose this option.
    Select(String),
    /// Enter: choose the highlighted option.
    Enter,
    /// Esc: close, or clear when already closed.
    Escape,
    /// Focus left: close and show the selected option's label again.
    Blur,
}

/// Side effects from [`ComboboxMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq)]
pub enum ComboboxCommand {
    /// Tell the owner the value changed (or should, when controlled). `None` is cleared.
    ValueChange(Option<String>),
}

/// Query, list and selection for a filterable select.
#[derive(Debug, Clone, Copy)]
pub struct ComboboxMachine;

type Step = Transition<ComboboxState, ComboboxCommand>;

/// The options shown for `query`: label contains the query, in display order.
#[must_use]
pub fn visible<'a>(input: &'a ComboboxInput, query: &str) -> Vec<&'a ComboboxOption> {
    let needle = listbox::needle(query);
    input.options.iter().filter(|o| listbox::matches(&o.label, &needle)).collect()
}

fn enabled<'a>(input: &'a ComboboxInput, query: &str) -> Vec<&'a str> {
    visible(input, query).into_iter().filter(|o| !o.disabled).map(|o| o.value.as_str()).collect()
}

/// Open on the selected option when it is visible, else the first.
fn open(mut state: ComboboxState, input: &ComboboxInput) -> Step {
    let items = enabled(input, &state.query);
    state.highlighted = listbox::keep(&items, state.value.value.as_deref().or(state.highlighted.as_deref()));
    state.open = true;
    Transition::to(state)
}

fn set_value(mut state: ComboboxState, input: &ComboboxInput, value: Option<String>) -> Step {
    state.value.set(value.clone());
    state.query = input.label_of(state.value.value.as_deref());
    state.open = false;
    Transition::with(state, vec![ComboboxCommand::ValueChange(value)])
}

impl Machine for ComboboxMachine {
    type Input = ComboboxInput;
    type State = ComboboxState;
    type Event = ComboboxEvent;
    type Command = ComboboxCommand;

    fn initial(input: &ComboboxInput) -> ComboboxState {
        let value = input.value.clone().or_else(|| input.default_value.clone());
        ComboboxState {
            query: input.label_of(value.as_deref()),
            value: Controllable::new(value, input.value.is_some()),
            open: false,
            highlighted: None,
        }
    }

    fn transition(state: &ComboboxState, event: ComboboxEvent, input: &ComboboxInput) -> Step {
        let mut next = state.clone();
        match event {
            ComboboxEvent::Input(query) => {
                next.highlighted = listbox::keep(&enabled(input, &query), None);
                next.query = query;
                next.open = true;
                Transition::update(state, next, Vec::new())
            }
            ComboboxEvent::Open if !state.open => open(next, input),
            ComboboxEvent::Move(direction) => {
                if !state.open {
                    return open(next, input);
                }
                let items = enabled(input, &state.query);
                next.highlighted = listbox::step(&items, state.highlighted.as_deref(), direction);
                Transition::update(state, next, Vec::new())
            }
            ComboboxEvent::Highlight(value) => {
                let needle = listbox::needle(&state.query);
                if input.options.iter().any(|o| o.value == value && !o.disabled && listbox::matches(&o.label, &needle))
                {
                    next.highlighted = Some(value);
                }
                Transition::update(state, next, Vec::new())
            }
            ComboboxEvent::Select(value) => {
                if !enabled(input, &state.query).contains(&value.as_str()) {
                    return Transition::update(state, next, Vec::new());
                }
                set_value(next, input, Some(value))
            }
            ComboboxEvent::Enter => match state.highlighted.clone().filter(|_| state.open) {
                Some(value) => set_value(next, input, Some(value)),
                None => Transition::update(state, next, Vec::new()),
            },
            ComboboxEvent::Escape if state.open => {
                next.open = false;
                next.query = input.label_of(state.value.value.as_deref());
                Transition::update(state, next, Vec::new())
            }
            ComboboxEvent::Escape if state.value.value.is_some() || !state.query.is_empty() => {
                set_value(next, input, None)
            }
            ComboboxEvent::Open | ComboboxEvent::Escape => Transition::update(state, next, Vec::new()),
            ComboboxEvent::Blur => {
                next.open = false;
                next.query = input.label_of(state.value.value.as_deref());
                Transition::update(state, next, Vec::new())
            }
        }
    }

    fn sync(state: &ComboboxState, input: &ComboboxInput) -> Option<ComboboxState> {
        let mut next = state.clone();
        if state.value.controlled && state.value.value != input.value {
            next.value.sync(Some(input.value.clone()));
            if !state.open {
                next.query = input.label_of(next.value.value.as_deref());
            }
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step as advance;

    fn input() -> ComboboxInput {
        ComboboxInput {
            options: vec![
                ComboboxOption::new("gb", "United Kingdom"),
                ComboboxOption::new("ie", "Ireland").disabled(),
                ComboboxOption::new("is", "Iceland"),
                ComboboxOption::new("it", "Italy"),
            ],
            value: None,
            default_value: None,
        }
    }

    #[test]
    fn typing_filters_opens_and_highlights_first_enabled() {
        let input = input();
        let step =
            advance::<ComboboxMachine>(&ComboboxMachine::initial(&input), ComboboxEvent::Input("i".into()), &input);
        assert!(step.state.open);
        // "United Kingdom" contains an "i" too; Ireland is disabled and skipped.
        assert_eq!(step.state.highlighted.as_deref(), Some("gb"));
        assert_eq!(visible(&input, "ice").len(), 1);
    }

    #[test]
    fn arrows_skip_disabled_and_enter_selects() {
        let input = input();
        let s = advance::<ComboboxMachine>(&ComboboxMachine::initial(&input), ComboboxEvent::Input("I".into()), &input)
            .state;
        let s = advance::<ComboboxMachine>(&s, ComboboxEvent::Move(Direction::Next), &input).state;
        assert_eq!(s.highlighted.as_deref(), Some("is"));
        let step = advance::<ComboboxMachine>(&s, ComboboxEvent::Enter, &input);
        assert_eq!(step.state.value.value.as_deref(), Some("is"));
        assert_eq!(step.state.query, "Iceland");
        assert!(!step.state.open);
        assert_eq!(step.commands, vec![ComboboxCommand::ValueChange(Some("is".into()))]);
    }

    #[test]
    fn escape_closes_then_clears() {
        let mut input = input();
        input.default_value = Some("it".into());
        let s =
            advance::<ComboboxMachine>(&ComboboxMachine::initial(&input), ComboboxEvent::Input("Ic".into()), &input)
                .state;
        let s = advance::<ComboboxMachine>(&s, ComboboxEvent::Escape, &input).state;
        assert!(!s.open);
        assert_eq!(s.query, "Italy");
        let step = advance::<ComboboxMachine>(&s, ComboboxEvent::Escape, &input);
        assert_eq!(step.state.value.value, None);
        assert_eq!(step.state.query, "");
        assert_eq!(step.commands, vec![ComboboxCommand::ValueChange(None)]);
    }

    #[test]
    fn controlled_requests_and_waits_for_sync() {
        let mut input = input();
        input.value = Some("gb".into());
        let s = ComboboxMachine::initial(&input);
        let step = advance::<ComboboxMachine>(&s, ComboboxEvent::Select("it".into()), &input);
        assert_eq!(step.state.value.value.as_deref(), Some("gb"));
        assert_eq!(step.state.query, "United Kingdom");
        input.value = Some("it".into());
        assert_eq!(hydroxy_statechart::sync_or_keep::<ComboboxMachine>(&step.state, &input).query, "Italy");
    }

    #[test]
    fn blur_restores_the_selected_label() {
        let mut input = input();
        input.default_value = Some("gb".into());
        let s =
            advance::<ComboboxMachine>(&ComboboxMachine::initial(&input), ComboboxEvent::Input("zz".into()), &input)
                .state;
        let s = advance::<ComboboxMachine>(&s, ComboboxEvent::Blur, &input).state;
        assert_eq!(s.query, "United Kingdom");
        assert!(!s.open);
    }
}
