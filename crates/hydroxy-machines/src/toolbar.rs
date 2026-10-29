//! Toolbar: a row of buttons and toggles with one tab stop (roving tabindex).
//! Arrow keys move between the items that are showing; items that don't fit
//! move into a "More" menu at the end, lowest priority first.
//!
//! The adapter measures and reports widths with [`ToolbarEvent::Measured`];
//! the machine decides what overflows with [`overflow`], so ordering stays stable.

use std::collections::BTreeSet;

use hydroxy_statechart::{Controllable, Machine, Transition};

pub use crate::list::Direction;
use crate::list::next_index;

/// The roving-focus value of the "More" button.
pub const MORE: &str = "__more";

/// One toolbar item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolbarEntry {
    /// Unique key, reported by `on_action` / `on_toggle`.
    pub value: String,
    /// A toggle button (`aria-pressed`) rather than a plain action.
    pub toggle: bool,
    /// Skipped by arrow keys and not clickable.
    pub disabled: bool,
    /// Higher stays in the bar longer when space runs out. Ties: later items go first.
    pub priority: i32,
}

impl ToolbarEntry {
    /// An enabled action button with priority 0.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self { value: value.into(), toggle: false, disabled: false, priority: 0 }
    }
}

/// Props for [`ToolbarMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolbarInput {
    /// The items, in display order.
    pub items: Vec<ToolbarEntry>,
    /// Controlled set of pressed toggles. `Some` at start fixes controlled mode.
    pub pressed: Option<BTreeSet<String>>,
    /// Uncontrolled initial pressed toggles.
    pub default_pressed: BTreeSet<String>,
}

/// What [`ToolbarMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolbarState {
    /// The item holding the tab stop. `None` means the first enabled item.
    pub active: Option<String>,
    /// Pressed toggles. Controlled when the owner holds them.
    pub pressed: Controllable<BTreeSet<String>>,
    /// Items moved into the "More" menu.
    pub hidden: BTreeSet<String>,
}

/// Inputs to [`ToolbarMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum ToolbarEvent {
    /// Focus landed on (or the pointer pressed) this item, or [`MORE`].
    Focus(String),
    /// Arrow key.
    Move(Direction),
    /// First reachable item.
    Home,
    /// Last reachable item.
    End,
    /// Activate an item (click, Enter, Space, or chosen from the "More" menu).
    Press(String),
    /// New measurements, in item order: each item's width with its gap, the room
    /// available, and the "More" button's width with its gap.
    Measured {
        /// Width of each item, in `items` order.
        widths: Vec<f64>,
        /// Room for items.
        available: f64,
        /// Width of the "More" button.
        more: f64,
    },
}

/// Side effects from [`ToolbarMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolbarCommand {
    /// A plain item was activated.
    Action(String),
    /// A toggle changed (or should, when controlled).
    Toggle(String, bool),
    /// Move DOM focus to this item (or [`MORE`]).
    Focus(String),
}

/// Roving focus, toggles and overflow for one toolbar.
#[derive(Debug, Clone, Copy)]
pub struct ToolbarMachine;

type Step = Transition<ToolbarState, ToolbarCommand>;

/// Which items overflow when the bar has `available` pixels.
///
/// `widths` and `priorities` are in item order; `more` is the "More" button's
/// width. Everything fits: nothing hides. Otherwise items hide lowest priority
/// first (later items first on ties) until the rest plus "More" fits.
#[must_use]
pub fn overflow(widths: &[f64], priorities: &[i32], available: f64, more: f64) -> Vec<bool> {
    let mut hidden = vec![false; widths.len()];
    let mut used: f64 = widths.iter().sum();
    if used <= available {
        return hidden;
    }
    let budget = available - more;
    let mut order: Vec<usize> = (0..widths.len()).collect();
    order.sort_by_key(|&i| (priorities.get(i).copied().unwrap_or(0), std::cmp::Reverse(i)));
    for i in order {
        if used <= budget {
            break;
        }
        hidden[i] = true;
        used -= widths[i];
    }
    hidden
}

/// The values arrow keys reach: showing, enabled items, then "More" if anything is hidden.
#[must_use]
pub fn reachable(state: &ToolbarState, input: &ToolbarInput) -> Vec<String> {
    let mut out: Vec<String> = input
        .items
        .iter()
        .filter(|i| !i.disabled && !state.hidden.contains(&i.value))
        .map(|i| i.value.clone())
        .collect();
    if !state.hidden.is_empty() {
        out.push(MORE.to_string());
    }
    out
}

/// The value holding the tab stop: the active item if it is still reachable, else the first.
#[must_use]
pub fn tab_stop(state: &ToolbarState, input: &ToolbarInput) -> Option<String> {
    let items = reachable(state, input);
    match &state.active {
        Some(a) if items.contains(a) => Some(a.clone()),
        _ => items.into_iter().next(),
    }
}

fn focus(state: &ToolbarState, value: String) -> Step {
    let mut next = state.clone();
    next.active = Some(value.clone());
    Transition::update(state, next, vec![ToolbarCommand::Focus(value)])
}

impl Machine for ToolbarMachine {
    type Input = ToolbarInput;
    type State = ToolbarState;
    type Event = ToolbarEvent;
    type Command = ToolbarCommand;

    fn initial(input: &ToolbarInput) -> ToolbarState {
        ToolbarState {
            active: None,
            pressed: Controllable::new(
                input.pressed.clone().unwrap_or_else(|| input.default_pressed.clone()),
                input.pressed.is_some(),
            ),
            hidden: BTreeSet::new(),
        }
    }

    fn transition(state: &ToolbarState, event: ToolbarEvent, input: &ToolbarInput) -> Step {
        let items = reachable(state, input);
        let current = tab_stop(state, input).and_then(|v| items.iter().position(|i| *i == v));
        match event {
            ToolbarEvent::Focus(value) if items.contains(&value) => {
                let mut next = state.clone();
                next.active = Some(value);
                Transition::update(state, next, Vec::new())
            }
            ToolbarEvent::Move(direction) if !items.is_empty() => {
                focus(state, items[next_index(current, items.len(), direction, true)].clone())
            }
            ToolbarEvent::Home if !items.is_empty() => focus(state, items[0].clone()),
            ToolbarEvent::End if !items.is_empty() => focus(state, items[items.len() - 1].clone()),
            ToolbarEvent::Press(value) => {
                let Some(entry) = input.items.iter().find(|i| i.value == value && !i.disabled) else {
                    return Transition::stay();
                };
                if !entry.toggle {
                    return Transition::emit(vec![ToolbarCommand::Action(value)]);
                }
                let on = !state.pressed.value.contains(&value);
                let mut pressed = state.pressed.value.clone();
                if on {
                    pressed.insert(value.clone());
                } else {
                    pressed.remove(&value);
                }
                let mut next = state.clone();
                next.pressed.set(pressed);
                Transition::update(state, next, vec![ToolbarCommand::Toggle(value, on)])
            }
            ToolbarEvent::Measured { widths, available, more } => {
                let priorities: Vec<i32> = input.items.iter().map(|i| i.priority).collect();
                let flags = overflow(&widths, &priorities, available, more);
                let mut next = state.clone();
                next.hidden = input.items.iter().zip(flags).filter(|(_, h)| *h).map(|(i, _)| i.value.clone()).collect();
                Transition::update(state, next, Vec::new())
            }
            ToolbarEvent::Focus(..) | ToolbarEvent::Move(..) | ToolbarEvent::Home | ToolbarEvent::End => {
                Transition::stay()
            }
        }
    }

    fn sync(state: &ToolbarState, input: &ToolbarInput) -> Option<ToolbarState> {
        let mut next = state.clone();
        next.pressed.sync(input.pressed.clone());
        next.hidden.retain(|v| input.items.iter().any(|i| i.value == *v));
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    fn input() -> ToolbarInput {
        let mut bold = ToolbarEntry::new("bold");
        bold.toggle = true;
        bold.priority = 2;
        let mut italic = ToolbarEntry::new("italic");
        italic.toggle = true;
        italic.priority = 2;
        let mut link = ToolbarEntry::new("link");
        link.disabled = true;
        ToolbarInput {
            items: vec![bold, italic, link, ToolbarEntry::new("undo"), ToolbarEntry::new("redo")],
            ..Default::default()
        }
    }

    #[test]
    fn overflow_hides_low_priority_from_the_end() {
        let hidden = overflow(&[40.0, 40.0, 40.0, 40.0], &[1, 0, 0, 1], 130.0, 30.0);
        // 160 > 130: budget 100. Priority 0 goes first, later first: index 2, then 1.
        assert_eq!(hidden, vec![false, true, true, false]);
        assert_eq!(overflow(&[40.0, 40.0], &[0, 0], 80.0, 30.0), vec![false, false]);
    }

    #[test]
    fn arrows_skip_disabled_and_wrap() {
        let input = input();
        let s = ToolbarMachine::initial(&input);
        let step = hydroxy_statechart::step::<ToolbarMachine>(&s, ToolbarEvent::Move(Direction::Next), &input);
        assert_eq!(step.commands, vec![ToolbarCommand::Focus("italic".into())]);
        let step = hydroxy_statechart::step::<ToolbarMachine>(&step.state, ToolbarEvent::Move(Direction::Next), &input);
        assert_eq!(step.state.active.as_deref(), Some("undo"));
        let step = hydroxy_statechart::step::<ToolbarMachine>(&s, ToolbarEvent::Move(Direction::Prev), &input);
        assert_eq!(step.state.active.as_deref(), Some("redo"));
    }

    #[test]
    fn hidden_items_leave_the_order_and_more_joins_it() {
        let input = input();
        let s = ToolbarMachine::initial(&input);
        let measured = ToolbarEvent::Measured { widths: vec![30.0; 5], available: 100.0, more: 30.0 };
        let s = step::<ToolbarMachine>(&s, measured, &input).state;
        // Budget 70: redo, undo, link, then italic would go; 150 → 120 → 90 → 60.
        assert!(s.hidden.contains("redo") && s.hidden.contains("undo") && s.hidden.contains("link"));
        assert!(!s.hidden.contains("bold"));
        assert_eq!(reachable(&s, &input), vec!["bold".to_string(), "italic".into(), MORE.into()]);
        assert_eq!(step::<ToolbarMachine>(&s, ToolbarEvent::End, &input).state.active.as_deref(), Some(MORE));
    }

    #[test]
    fn toggles_flip_and_actions_report() {
        let input = input();
        let s = ToolbarMachine::initial(&input);
        let step = hydroxy_statechart::step::<ToolbarMachine>(&s, ToolbarEvent::Press("bold".into()), &input);
        assert!(step.state.pressed.value.contains("bold"));
        assert_eq!(step.commands, vec![ToolbarCommand::Toggle("bold".into(), true)]);
        let step = hydroxy_statechart::step::<ToolbarMachine>(&step.state, ToolbarEvent::Press("bold".into()), &input);
        assert!(!step.state.pressed.value.contains("bold"));
        let step = hydroxy_statechart::step::<ToolbarMachine>(&s, ToolbarEvent::Press("undo".into()), &input);
        assert_eq!(step.commands, vec![ToolbarCommand::Action("undo".into())]);
        assert!(
            hydroxy_statechart::step::<ToolbarMachine>(&s, ToolbarEvent::Press("link".into()), &input)
                .commands
                .is_empty()
        );
    }

    #[test]
    fn controlled_toggles_wait_for_sync() {
        let mut input = input();
        input.pressed = Some(BTreeSet::new());
        let s = ToolbarMachine::initial(&input);
        let step = hydroxy_statechart::step::<ToolbarMachine>(&s, ToolbarEvent::Press("bold".into()), &input);
        assert!(!step.state.pressed.value.contains("bold"));
        input.pressed = Some(BTreeSet::from(["bold".to_string()]));
        assert!(hydroxy_statechart::sync_or_keep::<ToolbarMachine>(&step.state, &input).pressed.value.contains("bold"));
    }

    #[test]
    fn tab_stop_falls_back_when_the_active_item_hides() {
        let input = input();
        let mut s = ToolbarMachine::initial(&input);
        s.active = Some("redo".into());
        s.hidden.insert("redo".into());
        assert_eq!(tab_stop(&s, &input).as_deref(), Some("bold"));
    }
}
