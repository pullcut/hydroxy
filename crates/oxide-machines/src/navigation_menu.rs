//! Navigation menu: a site nav whose items are links or triggers for flyout
//! panels. One panel is open at a time. Pointer intent is delayed: a panel
//! opens after the pointer rests on its trigger, and closes a moment after it
//! leaves the nav and the panel. While a panel is open, moving to another
//! trigger switches at once.
//!
//! The machine owns the delays but not the clock: it asks the adapter to
//! [`NavigationMenuCommand::Wait`] and acts on the matching
//! [`NavigationMenuEvent::Elapsed`]. A newer wait, or any event that settles the
//! state, makes an older one stale, so adapters never cancel a timer.

use oxide_statechart::{Machine, Transition};

pub use crate::list::Direction;
use crate::list::next_index;

/// One top-level item, identified by its `value`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationMenuEntry {
    /// Unique key.
    pub value: String,
    /// A trigger with a flyout panel (`true`) or a plain link (`false`).
    pub has_panel: bool,
}

impl NavigationMenuEntry {
    /// A trigger with a panel.
    #[must_use]
    pub fn panel(value: impl Into<String>) -> Self {
        Self { value: value.into(), has_panel: true }
    }

    /// A plain link.
    #[must_use]
    pub fn link(value: impl Into<String>) -> Self {
        Self { value: value.into(), has_panel: false }
    }
}

/// Props for [`NavigationMenuMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationMenuInput {
    /// The top-level items, in display order.
    pub items: Vec<NavigationMenuEntry>,
    /// Controlled open panel (`Some(None)` is "none open"). `Some` at start fixes controlled mode.
    pub open: Option<Option<String>>,
    /// Uncontrolled initial open panel.
    pub default_open: Option<String>,
    /// Milliseconds the pointer must rest on a trigger before its panel opens.
    pub open_delay_ms: u32,
    /// Milliseconds after the pointer leaves before the panel closes.
    pub close_delay_ms: u32,
}

impl Default for NavigationMenuInput {
    fn default() -> Self {
        Self { items: Vec::new(), open: None, default_open: None, open_delay_ms: 150, close_delay_ms: 250 }
    }
}

/// A delayed open or close the machine is waiting on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    /// Matches the [`NavigationMenuCommand::Wait`] that started it.
    pub id: u32,
    /// The panel to show when it elapses, or `None` to close.
    pub open: Option<String>,
}

/// What [`NavigationMenuMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NavigationMenuState {
    /// The open panel, if any.
    pub open: Option<String>,
    /// Fixed at start: the owner holds `open` and the machine only requests changes.
    pub controlled: bool,
    /// The panel was opened by hovering, so the next click on its trigger keeps it open.
    pub hovered: bool,
    /// The delayed change in flight, if any.
    pub pending: Option<Pending>,
    /// Last wait id handed out.
    pub seq: u32,
}

/// Inputs to [`NavigationMenuMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationMenuEvent {
    /// The pointer moved onto this item.
    Enter(String),
    /// The pointer left the nav or the panel.
    Leave,
    /// The pointer moved onto the open panel.
    PanelEnter,
    /// Trigger clicked: open its panel, or close it if open.
    Toggle(String),
    /// `ArrowDown` on a trigger: open and move focus into the panel.
    Open(String),
    /// Close at once (focus left the nav).
    Close,
    /// This panel closed itself (Esc or an outside press).
    Dismiss(String),
    /// Arrow key on this item: focus the next or previous item.
    Move {
        /// The item the key was pressed on.
        from: String,
        /// Which way.
        direction: Direction,
    },
    /// Home: focus the first item.
    Home,
    /// End: focus the last item.
    End,
    /// The wait with this id is over.
    Elapsed(u32),
}

/// Side effects from [`NavigationMenuMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationMenuCommand {
    /// Tell the owner the open panel changed (or should, when controlled).
    OpenChange(Option<String>),
    /// Send [`NavigationMenuEvent::Elapsed`] with `id` after `ms` milliseconds.
    Wait {
        /// Echo this back in `Elapsed`.
        id: u32,
        /// How long to wait.
        ms: u32,
    },
    /// Move DOM focus to this item.
    FocusItem(String),
    /// Move DOM focus to the first link in this item's panel.
    FocusPanel(String),
}

/// Open panel and hover intent for one navigation menu.
#[derive(Debug, Clone, Copy)]
pub struct NavigationMenuMachine;

type Step = Transition<NavigationMenuState, NavigationMenuCommand>;

fn has_panel(input: &NavigationMenuInput, value: &str) -> bool {
    input.items.iter().any(|i| i.value == value && i.has_panel)
}

/// Drop any pending wait and move to `open`, requesting the change if it is one.
fn settle(state: &NavigationMenuState, open: Option<String>) -> Step {
    let mut next = state.clone();
    next.pending = None;
    next.hovered = false;
    if open == state.open {
        return Transition::stay(next);
    }
    if !state.controlled {
        next.open.clone_from(&open);
    }
    Transition { state: next, commands: vec![NavigationMenuCommand::OpenChange(open)] }
}

/// [`settle`] on a pointer's say-so: remember the panel was opened by hovering.
fn settle_by_hover(state: &NavigationMenuState, open: Option<String>) -> Step {
    let mut step = settle(state, open);
    step.state.hovered = step.state.pending.is_none()
        && step.commands.iter().any(|c| matches!(c, NavigationMenuCommand::OpenChange(Some(_))));
    step
}

/// Start a delayed change, replacing any other.
fn wait(state: &NavigationMenuState, open: Option<String>, ms: u32) -> Step {
    if ms == 0 {
        return settle(state, open);
    }
    let id = state.seq.wrapping_add(1);
    let next = NavigationMenuState { seq: id, pending: Some(Pending { id, open }), ..state.clone() };
    Transition { state: next, commands: vec![NavigationMenuCommand::Wait { id, ms }] }
}

/// Focus `target`. While a panel is open the panel follows: it switches to the
/// target's panel, or closes if the target is a plain link.
fn focus(state: &NavigationMenuState, input: &NavigationMenuInput, target: &str) -> Step {
    let mut step = if state.open.is_some() {
        settle(state, has_panel(input, target).then(|| target.to_string()))
    } else {
        Transition::stay(state.clone())
    };
    step.commands.push(NavigationMenuCommand::FocusItem(target.to_string()));
    step
}

impl Machine for NavigationMenuMachine {
    type Input = NavigationMenuInput;
    type State = NavigationMenuState;
    type Event = NavigationMenuEvent;
    type Command = NavigationMenuCommand;

    fn initial(input: &NavigationMenuInput) -> NavigationMenuState {
        NavigationMenuState {
            open: input.open.clone().unwrap_or_else(|| input.default_open.clone()),
            controlled: input.open.is_some(),
            ..NavigationMenuState::default()
        }
    }

    fn transition(state: &NavigationMenuState, event: NavigationMenuEvent, input: &NavigationMenuInput) -> Step {
        use NavigationMenuEvent as E;
        let values: Vec<&str> = input.items.iter().map(|i| i.value.as_str()).collect();
        match event {
            E::Enter(v) if has_panel(input, &v) => {
                if state.open.is_some() {
                    settle_by_hover(state, Some(v))
                } else {
                    wait(state, Some(v), input.open_delay_ms)
                }
            }
            // Onto a plain link: same as leaving, so an open panel eventually closes.
            E::Enter(_) | E::Leave if state.open.is_some() => wait(state, None, input.close_delay_ms),
            E::Enter(_) | E::Leave | E::Close => settle(state, None),
            E::PanelEnter => settle(state, state.open.clone()),
            E::Toggle(v) if state.hovered && state.open.as_deref() == Some(v.as_str()) => {
                let mut next = state.clone();
                next.hovered = false;
                Transition::stay(next)
            }
            E::Toggle(v) if has_panel(input, &v) => {
                let open = if state.open.as_deref() == Some(v.as_str()) { None } else { Some(v) };
                settle(state, open)
            }
            E::Open(v) if has_panel(input, &v) => {
                let mut step = settle(state, Some(v.clone()));
                step.commands.push(NavigationMenuCommand::FocusPanel(v));
                step
            }
            E::Dismiss(v) if state.open.as_deref() == Some(v.as_str()) => settle(state, None),
            E::Move { from, direction } if !values.is_empty() => {
                let current = values.iter().position(|v| *v == from);
                focus(state, input, values[next_index(current, values.len(), direction, true)])
            }
            E::Home if !values.is_empty() => focus(state, input, values[0]),
            E::End if !values.is_empty() => focus(state, input, values[values.len() - 1]),
            E::Elapsed(id) => match &state.pending {
                Some(p) if p.id == id => settle_by_hover(state, p.open.clone()),
                _ => Transition::stay(state.clone()),
            },
            _ => Transition::stay(state.clone()),
        }
    }

    fn sync(state: &NavigationMenuState, input: &NavigationMenuInput) -> NavigationMenuState {
        let mut next = state.clone();
        if state.controlled {
            next.open = input.open.clone().flatten();
        }
        next.hovered &= next.open.is_some();
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use NavigationMenuEvent as E;

    fn input() -> NavigationMenuInput {
        NavigationMenuInput {
            items: vec![
                NavigationMenuEntry::panel("products"),
                NavigationMenuEntry::panel("company"),
                NavigationMenuEntry::link("pricing"),
            ],
            ..Default::default()
        }
    }

    fn run(state: &NavigationMenuState, event: E, input: &NavigationMenuInput) -> Step {
        NavigationMenuMachine::transition(state, event, input)
    }

    fn open_state(input: &NavigationMenuInput, panel: &str) -> NavigationMenuState {
        run(&NavigationMenuMachine::initial(input), E::Toggle(panel.into()), input).state
    }

    #[test]
    fn hover_waits_then_opens() {
        let input = input();
        let step = run(&NavigationMenuMachine::initial(&input), E::Enter("products".into()), &input);
        assert!(step.state.open.is_none());
        assert_eq!(step.commands, vec![NavigationMenuCommand::Wait { id: 1, ms: 150 }]);
        let done = run(&step.state, E::Elapsed(1), &input);
        assert_eq!(done.state.open.as_deref(), Some("products"));
        assert_eq!(done.commands, vec![NavigationMenuCommand::OpenChange(Some("products".into()))]);
    }

    #[test]
    fn a_click_on_a_hover_opened_trigger_keeps_it_open_once() {
        let input = input();
        let waiting = run(&NavigationMenuMachine::initial(&input), E::Enter("products".into()), &input).state;
        let hovered = run(&waiting, E::Elapsed(1), &input).state;
        let first = run(&hovered, E::Toggle("products".into()), &input);
        assert_eq!(first.state.open.as_deref(), Some("products"));
        assert!(first.commands.is_empty());
        assert_eq!(run(&first.state, E::Toggle("products".into()), &input).state.open, None);
    }

    #[test]
    fn leaving_before_the_delay_cancels_the_open() {
        let input = input();
        let waiting = run(&NavigationMenuMachine::initial(&input), E::Enter("products".into()), &input).state;
        let left = run(&waiting, E::Leave, &input);
        assert!(left.state.pending.is_none());
        assert!(run(&left.state, E::Elapsed(1), &input).commands.is_empty());
    }

    #[test]
    fn leaving_closes_after_the_close_delay() {
        let input = input();
        let step = run(&open_state(&input, "products"), E::Leave, &input);
        assert_eq!(step.commands, vec![NavigationMenuCommand::Wait { id: 1, ms: 250 }]);
        assert_eq!(run(&step.state, E::Elapsed(1), &input).state.open, None);
    }

    #[test]
    fn entering_the_panel_cancels_the_close() {
        let input = input();
        let leaving = run(&open_state(&input, "products"), E::Leave, &input).state;
        let back = run(&leaving, E::PanelEnter, &input);
        assert!(back.state.pending.is_none());
        assert!(run(&back.state, E::Elapsed(1), &input).state.open.is_some());
    }

    #[test]
    fn switching_triggers_while_open_is_immediate() {
        let input = input();
        let step = run(&open_state(&input, "products"), E::Enter("company".into()), &input);
        assert_eq!(step.state.open.as_deref(), Some("company"));
    }

    #[test]
    fn hovering_a_link_while_open_closes_after_the_delay() {
        let input = input();
        let step = run(&open_state(&input, "products"), E::Enter("pricing".into()), &input);
        assert_eq!(step.commands, vec![NavigationMenuCommand::Wait { id: 1, ms: 250 }]);
    }

    #[test]
    fn toggle_and_arrow_down_open_and_close() {
        let input = input();
        let open = open_state(&input, "products");
        assert_eq!(run(&open, E::Toggle("products".into()), &input).state.open, None);
        let step = run(&NavigationMenuMachine::initial(&input), E::Open("company".into()), &input);
        assert_eq!(step.commands.last(), Some(&NavigationMenuCommand::FocusPanel("company".into())));
        assert!(run(&NavigationMenuMachine::initial(&input), E::Toggle("pricing".into()), &input).commands.is_empty());
    }

    #[test]
    fn arrows_move_focus_and_carry_an_open_panel() {
        let input = input();
        let closed = run(
            &NavigationMenuMachine::initial(&input),
            E::Move { from: "company".into(), direction: Direction::Next },
            &input,
        );
        assert_eq!(closed.commands, vec![NavigationMenuCommand::FocusItem("pricing".into())]);
        let open = run(
            &open_state(&input, "products"),
            E::Move { from: "products".into(), direction: Direction::Next },
            &input,
        );
        assert_eq!(open.state.open.as_deref(), Some("company"));
        let to_link = run(&open.state, E::Move { from: "company".into(), direction: Direction::Next }, &input);
        assert_eq!(to_link.state.open, None);
    }

    #[test]
    fn dismiss_only_closes_the_open_panel() {
        let input = input();
        let open = open_state(&input, "products");
        assert!(run(&open, E::Dismiss("company".into()), &input).commands.is_empty());
        assert_eq!(run(&open, E::Dismiss("products".into()), &input).state.open, None);
    }

    #[test]
    fn controlled_requests_and_waits_for_sync() {
        let mut input = NavigationMenuInput { open: Some(None), ..input() };
        let step = run(&NavigationMenuMachine::initial(&input), E::Toggle("products".into()), &input);
        assert_eq!(step.state.open, None);
        assert_eq!(step.commands, vec![NavigationMenuCommand::OpenChange(Some("products".into()))]);
        input.open = Some(Some("products".into()));
        assert_eq!(NavigationMenuMachine::sync(&step.state, &input).open.as_deref(), Some("products"));
    }
}
