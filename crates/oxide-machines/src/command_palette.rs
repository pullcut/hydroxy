//! Command palette owns open state, the query and the highlighted command.
//! Focus stays in the search input; the highlight is announced with
//! `aria-activedescendant`. Running a command closes the palette.

use oxide_statechart::{Machine, Transition};

pub use crate::list::Direction;
use crate::listbox;

/// One command, as the machine sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEntry {
    /// Unique key. Reported by `on_select`.
    pub id: String,
    /// Everything the query is matched against (label, group, keywords).
    pub text: String,
}

/// Props for [`CommandPaletteMachine`]. `entries` must be in display order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandPaletteInput {
    /// Commands in the order they are shown (grouped).
    pub entries: Vec<CommandEntry>,
    /// Controlled open state. `Some` at start fixes controlled mode.
    pub open: Option<bool>,
    /// Uncontrolled initial open state.
    pub default_open: bool,
}

/// What [`CommandPaletteMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandPaletteState {
    /// Whether the palette is showing.
    pub open: bool,
    /// The search text.
    pub query: String,
    /// The highlighted command.
    pub highlighted: Option<String>,
    /// Fixed at start: the owner holds `open` and the machine only requests changes.
    pub controlled: bool,
}

/// Inputs to [`CommandPaletteMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum CommandPaletteEvent {
    /// Open with an empty query.
    Open,
    /// Close (Esc, backdrop press).
    Close,
    /// The user typed.
    Input(String),
    /// Arrow key: move the highlight, wrapping.
    Move(Direction),
    /// Pointer moved over this command.
    Highlight(String),
    /// Run this command.
    Run(String),
    /// Enter: run the highlighted command.
    Enter,
}

/// Side effects from [`CommandPaletteMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandPaletteCommand {
    /// Tell the owner the open state changed (or should, when controlled).
    OpenChange(bool),
    /// A command was run.
    Select(String),
}

/// Search, highlight and open state for a command palette.
#[derive(Debug, Clone, Copy)]
pub struct CommandPaletteMachine;

type Step = Transition<CommandPaletteState, CommandPaletteCommand>;

/// Ids of the commands shown for `query`, in display order.
#[must_use]
pub fn visible<'a>(input: &'a CommandPaletteInput, query: &str) -> Vec<&'a str> {
    input.entries.iter().filter(|e| listbox::matches(&e.text, query)).map(|e| e.id.as_str()).collect()
}

/// Fresh palette: empty query, first command highlighted.
fn reset(state: &mut CommandPaletteState, input: &CommandPaletteInput) {
    state.query.clear();
    state.highlighted = listbox::keep(&visible(input, ""), None);
}

fn set_open(mut state: CommandPaletteState, input: &CommandPaletteInput, open: bool) -> Step {
    if !state.controlled {
        state.open = open;
        if open {
            reset(&mut state, input);
        }
    }
    Transition { state, commands: vec![CommandPaletteCommand::OpenChange(open)] }
}

fn run(state: CommandPaletteState, input: &CommandPaletteInput, id: String) -> Step {
    let mut step = set_open(state, input, false);
    step.commands.insert(0, CommandPaletteCommand::Select(id));
    step
}

impl Machine for CommandPaletteMachine {
    type Input = CommandPaletteInput;
    type State = CommandPaletteState;
    type Event = CommandPaletteEvent;
    type Command = CommandPaletteCommand;

    fn initial(input: &CommandPaletteInput) -> CommandPaletteState {
        let mut state = CommandPaletteState {
            open: input.open.unwrap_or(input.default_open),
            controlled: input.open.is_some(),
            ..CommandPaletteState::default()
        };
        reset(&mut state, input);
        state
    }

    fn transition(state: &CommandPaletteState, event: CommandPaletteEvent, input: &CommandPaletteInput) -> Step {
        let mut next = state.clone();
        match event {
            CommandPaletteEvent::Open if !state.open => set_open(next, input, true),
            CommandPaletteEvent::Close if state.open => set_open(next, input, false),
            CommandPaletteEvent::Input(query) => {
                // Matches the design: a new query highlights the first result.
                next.highlighted = listbox::keep(&visible(input, &query), None);
                next.query = query;
                Transition::stay(next)
            }
            CommandPaletteEvent::Move(direction) => {
                next.highlighted =
                    listbox::step(&visible(input, &state.query), state.highlighted.as_deref(), direction);
                Transition::stay(next)
            }
            CommandPaletteEvent::Highlight(id) => {
                if visible(input, &state.query).contains(&id.as_str()) {
                    next.highlighted = Some(id);
                }
                Transition::stay(next)
            }
            CommandPaletteEvent::Run(id) if visible(input, &state.query).contains(&id.as_str()) => run(next, input, id),
            CommandPaletteEvent::Enter => match state.highlighted.clone() {
                Some(id) if state.open => run(next, input, id),
                _ => Transition::stay(next),
            },
            CommandPaletteEvent::Open | CommandPaletteEvent::Close | CommandPaletteEvent::Run(_) => {
                Transition::stay(next)
            }
        }
    }

    fn sync(state: &CommandPaletteState, input: &CommandPaletteInput) -> CommandPaletteState {
        let mut next = state.clone();
        if state.controlled
            && let Some(open) = input.open
        {
            if open && !state.open {
                reset(&mut next, input);
            }
            next.open = open;
        }
        // Commands can change while open: keep the highlight on something visible.
        next.highlighted = listbox::keep(&visible(input, &next.query), next.highlighted.as_deref());
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, text: &str) -> CommandEntry {
        CommandEntry { id: id.into(), text: text.into() }
    }

    fn input() -> CommandPaletteInput {
        CommandPaletteInput {
            entries: vec![
                entry("new", "New project Projects"),
                entry("open", "Open project Projects"),
                entry("theme", "Switch theme Settings dark light"),
            ],
            open: None,
            default_open: false,
        }
    }

    fn step(state: &CommandPaletteState, event: CommandPaletteEvent, input: &CommandPaletteInput) -> Step {
        CommandPaletteMachine::transition(state, event, input)
    }

    #[test]
    fn opening_resets_query_and_highlights_first() {
        let input = input();
        let mut s = CommandPaletteMachine::initial(&input);
        s.query = "old".into();
        let s = step(&s, CommandPaletteEvent::Open, &input).state;
        assert!(s.open);
        assert_eq!(s.query, "");
        assert_eq!(s.highlighted.as_deref(), Some("new"));
    }

    #[test]
    fn query_matches_keywords_and_group() {
        let input = input();
        assert_eq!(visible(&input, "dark"), vec!["theme"]);
        assert_eq!(visible(&input, "projects"), vec!["new", "open"]);
        let s = step(&CommandPaletteMachine::initial(&input), CommandPaletteEvent::Input("dark".into()), &input).state;
        assert_eq!(s.highlighted.as_deref(), Some("theme"));
    }

    #[test]
    fn enter_runs_highlighted_and_closes() {
        let input = input();
        let s = step(&CommandPaletteMachine::initial(&input), CommandPaletteEvent::Open, &input).state;
        let s = step(&s, CommandPaletteEvent::Move(Direction::Prev), &input).state;
        assert_eq!(s.highlighted.as_deref(), Some("theme"));
        let out = step(&s, CommandPaletteEvent::Enter, &input);
        assert!(!out.state.open);
        assert_eq!(
            out.commands,
            vec![CommandPaletteCommand::Select("theme".into()), CommandPaletteCommand::OpenChange(false)]
        );
    }

    #[test]
    fn controlled_open_waits_for_sync_and_resets() {
        let mut input = input();
        input.open = Some(false);
        let s = CommandPaletteMachine::initial(&input);
        let out = step(&s, CommandPaletteEvent::Open, &input);
        assert!(!out.state.open);
        assert_eq!(out.commands, vec![CommandPaletteCommand::OpenChange(true)]);
        input.open = Some(true);
        let synced = CommandPaletteMachine::sync(&out.state, &input);
        assert!(synced.open);
        assert_eq!(synced.highlighted.as_deref(), Some("new"));
    }
}
