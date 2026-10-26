//! Availability picker: a week grid of time slots painted available or not.
//!
//! Drag from one cell to another to paint a rectangle: the first cell decides
//! whether it adds or clears. With the keyboard, Space toggles the focused cell
//! and Shift plus an arrow paints as the cursor moves. While a drag is in
//! progress the owner's value is untouched; the preview comes from [`display`].

use std::collections::BTreeSet;

use oxide_statechart::{Machine, Transition};

use crate::date::Date;

/// One slot: the day and the minute of day it starts at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Slot {
    /// The day.
    pub date: Date,
    /// Minutes after midnight when the slot starts.
    pub minute: u16,
}

/// Props for [`AvailabilityPickerMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailabilityInput {
    /// First day shown.
    pub start: Date,
    /// Number of day columns.
    pub days: u8,
    /// Minute of day the first row starts at.
    pub first_minute: u16,
    /// Minute of day the last row ends at.
    pub last_minute: u16,
    /// Length of one slot in minutes.
    pub slot_minutes: u16,
    /// Controlled value. `Some` at start fixes controlled mode.
    pub value: Option<BTreeSet<Slot>>,
    /// Uncontrolled initial value.
    pub default_value: BTreeSet<Slot>,
    /// Ignore edits.
    pub disabled: bool,
}

impl Default for AvailabilityInput {
    fn default() -> Self {
        Self {
            start: Date::new(2026, 1, 5).unwrap_or_else(|| Date::from_days(0)),
            days: 7,
            first_minute: 9 * 60,
            last_minute: 17 * 60,
            slot_minutes: 30,
            value: None,
            default_value: BTreeSet::new(),
            disabled: false,
        }
    }
}

impl AvailabilityInput {
    /// Number of day columns.
    #[must_use]
    pub fn cols(&self) -> usize {
        usize::from(self.days)
    }

    /// Number of time rows.
    #[must_use]
    pub fn rows(&self) -> usize {
        usize::from(self.last_minute.saturating_sub(self.first_minute) / self.slot_minutes.max(1))
    }

    /// The slot at a column and row, if inside the grid.
    #[must_use]
    pub fn slot(&self, col: usize, row: usize) -> Option<Slot> {
        if col >= self.cols() || row >= self.rows() {
            return None;
        }
        let minute = self.first_minute + u16::try_from(row).ok()? * self.slot_minutes;
        Some(Slot { date: self.start.add_days(i64::try_from(col).ok()?), minute })
    }
}

/// A drag in progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paint {
    /// What the painted cells become: available (`true`) or cleared.
    pub mode: bool,
    /// Where the drag began, as (column, row).
    pub anchor: (usize, usize),
    /// Where the pointer is now.
    pub to: (usize, usize),
}

/// What [`AvailabilityPickerMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AvailabilityState {
    /// The committed slots.
    pub value: BTreeSet<Slot>,
    /// The focused cell, as (column, row).
    pub cursor: (usize, usize),
    /// A drag in progress.
    pub paint: Option<Paint>,
    /// Mode of a Shift-arrow paint in progress.
    pub key_mode: Option<bool>,
    /// Fixed at start: the owner holds `value`.
    pub controlled: bool,
}

/// Inputs to [`AvailabilityPickerMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvailabilityEvent {
    /// Pointer down on a cell.
    PointerDown(usize, usize),
    /// Pointer entered a cell.
    PointerEnter(usize, usize),
    /// Pointer released: commit the drag.
    PointerUp,
    /// The drag was abandoned (pointer left the grid, Escape).
    Cancel,
    /// A cell took focus.
    Focus(usize, usize),
    /// Move the cursor. With `paint`, apply the key paint mode along the way.
    Move {
        /// Columns to move.
        dc: i32,
        /// Rows to move.
        dr: i32,
        /// Shift is held.
        paint: bool,
    },
    /// Space or Enter: toggle the focused cell.
    Toggle,
    /// Shift was released: end a key paint.
    KeyEnd,
}

/// Side effects from [`AvailabilityPickerMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AvailabilityCommand {
    /// Tell the owner the slots changed (or should, when controlled).
    ValueChange(BTreeSet<Slot>),
    /// Move DOM focus to this cell.
    FocusCell(usize, usize),
}

/// Paint cells on a week grid.
#[derive(Debug, Clone, Copy)]
pub struct AvailabilityPickerMachine;

type Step = Transition<AvailabilityState, AvailabilityCommand>;

/// The slots to draw: the value with any drag in progress applied.
#[must_use]
pub fn display(state: &AvailabilityState, input: &AvailabilityInput) -> BTreeSet<Slot> {
    let mut out = state.value.clone();
    if let Some(p) = &state.paint {
        apply_rect(&mut out, input, p.anchor, p.to, p.mode);
    }
    out
}

fn apply_rect(set: &mut BTreeSet<Slot>, input: &AvailabilityInput, a: (usize, usize), b: (usize, usize), mode: bool) {
    for col in a.0.min(b.0)..=a.0.max(b.0) {
        for row in a.1.min(b.1)..=a.1.max(b.1) {
            if let Some(slot) = input.slot(col, row) {
                if mode { set.insert(slot); } else { set.remove(&slot); }
            }
        }
    }
}

fn has(state: &AvailabilityState, input: &AvailabilityInput, (col, row): (usize, usize)) -> bool {
    input.slot(col, row).is_some_and(|s| state.value.contains(&s))
}

fn commit(mut state: AvailabilityState, value: BTreeSet<Slot>, mut commands: Vec<AvailabilityCommand>) -> Step {
    if value != state.value {
        commands.insert(0, AvailabilityCommand::ValueChange(value.clone()));
        if !state.controlled {
            state.value = value;
        }
    }
    Transition { state, commands }
}

fn shifted(cursor: (usize, usize), dc: i32, dr: i32, input: &AvailabilityInput) -> (usize, usize) {
    let clamp = |v: usize, d: i32, len: usize| {
        let max = i64::try_from(len.saturating_sub(1)).unwrap_or(0);
        usize::try_from((i64::try_from(v).unwrap_or(0) + i64::from(d)).clamp(0, max)).unwrap_or(0)
    };
    (clamp(cursor.0, dc, input.cols()), clamp(cursor.1, dr, input.rows()))
}

impl Machine for AvailabilityPickerMachine {
    type Input = AvailabilityInput;
    type State = AvailabilityState;
    type Event = AvailabilityEvent;
    type Command = AvailabilityCommand;

    fn initial(input: &AvailabilityInput) -> AvailabilityState {
        AvailabilityState {
            value: input.value.clone().unwrap_or_else(|| input.default_value.clone()),
            controlled: input.value.is_some(),
            ..AvailabilityState::default()
        }
    }

    fn transition(state: &AvailabilityState, event: AvailabilityEvent, input: &AvailabilityInput) -> Step {
        use AvailabilityEvent as E;
        let mut next = state.clone();
        if let E::Focus(c, r) = event {
            next.cursor = (c, r);
            return Transition::stay(next);
        }
        if input.disabled {
            return Transition::stay(next);
        }
        match event {
            E::PointerDown(c, r) if input.slot(c, r).is_some() => {
                let mode = !has(state, input, (c, r));
                next.cursor = (c, r);
                next.paint = Some(Paint { mode, anchor: (c, r), to: (c, r) });
                Transition::stay(next)
            }
            E::PointerEnter(c, r) => {
                if let Some(p) = &mut next.paint {
                    p.to = (c, r);
                }
                Transition::stay(next)
            }
            E::PointerUp => {
                let Some(p) = next.paint.take() else { return Transition::stay(next) };
                let mut value = state.value.clone();
                apply_rect(&mut value, input, p.anchor, p.to, p.mode);
                commit(next, value, vec![])
            }
            E::Cancel => {
                next.paint = None;
                next.key_mode = None;
                Transition::stay(next)
            }
            E::Toggle => {
                let mut value = state.value.clone();
                apply_rect(&mut value, input, state.cursor, state.cursor, !has(state, input, state.cursor));
                commit(next, value, vec![])
            }
            E::Move { dc, dr, paint } => {
                let to = shifted(state.cursor, dc, dr, input);
                let mut value = state.value.clone();
                if paint {
                    let mode = state.key_mode.unwrap_or_else(|| !has(state, input, state.cursor));
                    next.key_mode = Some(mode);
                    apply_rect(&mut value, input, state.cursor, to, mode);
                } else {
                    next.key_mode = None;
                }
                next.cursor = to;
                commit(next, value, vec![AvailabilityCommand::FocusCell(to.0, to.1)])
            }
            E::KeyEnd => {
                next.key_mode = None;
                Transition::stay(next)
            }
            _ => Transition::stay(next),
        }
    }

    fn sync(state: &AvailabilityState, input: &AvailabilityInput) -> AvailabilityState {
        let mut next = state.clone();
        if let (true, Some(value)) = (state.controlled, &input.value) {
            next.value.clone_from(value);
        }
        next
    }
}

/// Merge neighbouring slots into ranges: (day, start minute, end minute).
#[must_use]
pub fn ranges(value: &BTreeSet<Slot>, slot_minutes: u16) -> Vec<(Date, u16, u16)> {
    let mut out: Vec<(Date, u16, u16)> = Vec::new();
    for s in value {
        match out.last_mut() {
            Some((d, _, end)) if *d == s.date && *end == s.minute => *end = s.minute + slot_minutes,
            _ => out.push((s.date, s.minute, s.minute + slot_minutes)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use AvailabilityEvent as E;

    fn run(events: &[E]) -> (AvailabilityState, Vec<AvailabilityCommand>) {
        let input = AvailabilityInput::default();
        let mut s = AvailabilityPickerMachine::initial(&input);
        let mut all = Vec::new();
        for &e in events {
            let t = AvailabilityPickerMachine::transition(&s, e, &input);
            s = t.state;
            all.extend(t.commands);
        }
        (s, all)
    }

    #[test]
    fn drag_paints_a_rectangle_and_commits_on_release() {
        let input = AvailabilityInput::default();
        let (s, cmds) = run(&[E::PointerDown(1, 2), E::PointerEnter(2, 4)]);
        assert!(s.value.is_empty() && cmds.is_empty());
        assert_eq!(display(&s, &input).len(), 6);
        let (s, cmds) = run(&[E::PointerDown(1, 2), E::PointerEnter(2, 4), E::PointerUp]);
        assert_eq!(s.value.len(), 6);
        assert_eq!(cmds.len(), 1);
    }

    #[test]
    fn first_cell_decides_add_or_clear() {
        let (s, _) = run(&[E::PointerDown(0, 0), E::PointerUp, E::PointerDown(0, 0), E::PointerEnter(0, 2), E::PointerUp]);
        assert!(s.value.is_empty());
    }

    #[test]
    fn keyboard_toggle_and_shift_paint() {
        let (s, _) = run(&[E::Toggle]);
        assert_eq!(s.value.len(), 1);
        let (s, cmds) = run(&[E::Move { dc: 0, dr: 1, paint: true }, E::Move { dc: 0, dr: 1, paint: true }, E::KeyEnd]);
        assert_eq!(s.value.len(), 3);
        assert_eq!(s.cursor, (0, 2));
        assert!(cmds.contains(&AvailabilityCommand::FocusCell(0, 2)));
        let (s, _) = run(&[E::Move { dc: -5, dr: -5, paint: false }, E::Move { dc: 99, dr: 99, paint: false }]);
        assert_eq!(s.cursor, (6, 15));
    }

    #[test]
    fn controlled_requests_without_changing_and_ranges_merge() {
        let input = AvailabilityInput { value: Some(BTreeSet::new()), ..AvailabilityInput::default() };
        let s = AvailabilityPickerMachine::initial(&input);
        let t = AvailabilityPickerMachine::transition(&s, E::Toggle, &input);
        assert!(t.state.value.is_empty());
        let AvailabilityCommand::ValueChange(v) = &t.commands[0] else { panic!("no change") };
        assert_eq!(v.len(), 1);
        let (s, _) = run(&[E::PointerDown(0, 0), E::PointerEnter(0, 2), E::PointerUp]);
        assert_eq!(ranges(&s.value, 30).len(), 1);
        assert_eq!(ranges(&s.value, 30)[0].1, 540);
        assert_eq!(ranges(&s.value, 30)[0].2, 630);
    }
}
