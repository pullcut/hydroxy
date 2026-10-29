//! Listbox selection over a virtualised list. The windowing maths (row offsets, visible
//! ranges, sticky headers, keyboard movement) lives in [`hydroxy_virtual`] and is re-exported
//! here; this module adds the active/selected state machine on top.

use hydroxy_statechart::{Machine, Transition, changed};
pub use hydroxy_virtual::*;

/// A [`Layout`] for `count` rows of `row` px, with `header` px rows at the `headers`
/// indices. Row sizes are estimates until measured.
#[must_use]
pub fn layout_for(count: usize, headers: &[usize], row: f64, header: f64) -> Layout {
    if headers.is_empty() {
        Layout::fixed(count, row)
    } else {
        Layout::from_fn(count, row, |i| if is_header(headers, i) { header } else { row })
    }
}

/// Apply measured `(index, size)` pairs to `layout`. Returns whether any size changed and
/// how far the row at `scroll` moved as a result: scroll by that much to keep the view still.
pub fn anchor_shift(layout: &mut Layout, scroll: f64, sizes: impl IntoIterator<Item = (usize, f64)>) -> (bool, f64) {
    let anchor = layout.index_at(scroll);
    let before = layout.offset(anchor);
    let mut moved = false;
    for (index, size) in sizes {
        moved |= layout.set_size(index, size);
    }
    (moved, layout.offset(anchor) - before)
}

/// Props for the list's selection machine.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VirtualListInput {
    /// Number of rows, headers included.
    pub count: usize,
    /// Header rows, ascending.
    pub headers: Vec<usize>,
    /// The option selected at first.
    pub default_selected: Option<usize>,
}

impl VirtualListInput {
    fn is_option(&self, index: usize) -> bool {
        index < self.count && !is_header(&self.headers, index)
    }
}

/// What the list remembers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VirtualListState {
    /// The option the keyboard is on (`aria-activedescendant`).
    pub active: Option<usize>,
    /// The chosen option (`aria-selected`).
    pub selected: Option<usize>,
}

/// Things that happen to the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtualListEvent {
    /// The listbox took focus. Carries the first visible row: with nothing selected,
    /// the first option from there becomes active, so focusing doesn't jump the list.
    Focus(usize),
    /// A pointer went down on a row: make it active without scrolling, before the
    /// listbox takes focus.
    Point(usize),
    /// A movement key.
    Move(Nav),
    /// Enter or Space: select the active option.
    SelectActive,
    /// A row was clicked or tapped.
    Click(usize),
}

/// Side effects for the adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtualListCommand {
    /// Bring this row into view ([`Align::Auto`], below any sticky header).
    ScrollTo(usize),
    /// The selection changed to this option.
    SelectedChange(usize),
}

/// Keyboard and selection for a virtual listbox. Windowing lives in [`Layout`], outside
/// the state, so events never copy per-row data.
#[derive(Debug, Clone, Copy)]
pub struct VirtualListMachine;

type Step = Transition<VirtualListState, VirtualListCommand>;

impl Machine for VirtualListMachine {
    type Input = VirtualListInput;
    type State = VirtualListState;
    type Event = VirtualListEvent;
    type Command = VirtualListCommand;

    fn initial(input: &VirtualListInput) -> VirtualListState {
        let selected = input.default_selected.filter(|&i| input.is_option(i));
        VirtualListState { active: None, selected }
    }

    fn transition(state: &VirtualListState, event: VirtualListEvent, input: &VirtualListInput) -> Step {
        match event {
            VirtualListEvent::Focus(first_visible) => {
                if state.active.is_some_and(|a| input.is_option(a)) {
                    return Transition::stay();
                }
                let target = state
                    .selected
                    .filter(|&s| input.is_option(s))
                    .or_else(|| option_from(&input.headers, input.count, first_visible, Direction::Next))
                    .or_else(|| first_option(&input.headers, input.count));
                match target {
                    Some(t) => Transition::with(
                        VirtualListState { active: Some(t), ..state.clone() },
                        vec![VirtualListCommand::ScrollTo(t)],
                    ),
                    None => Transition::stay(),
                }
            }
            VirtualListEvent::Move(nav) => {
                let from = state.active.filter(|&a| input.is_option(a));
                match navigate(&input.headers, input.count, from, nav) {
                    Some(t) if Some(t) != state.active => Transition::with(
                        VirtualListState { active: Some(t), ..state.clone() },
                        vec![VirtualListCommand::ScrollTo(t)],
                    ),
                    // Same option: keep it in view in case the list was scrolled away.
                    Some(t) => Transition::emit(vec![VirtualListCommand::ScrollTo(t)]),
                    None => Transition::stay(),
                }
            }
            VirtualListEvent::SelectActive => match state.active.filter(|&a| input.is_option(a)) {
                Some(a) if state.selected != Some(a) => Transition::with(
                    VirtualListState { selected: Some(a), ..state.clone() },
                    vec![VirtualListCommand::SelectedChange(a)],
                ),
                _ => Transition::stay(),
            },
            VirtualListEvent::Click(i) if input.is_option(i) => {
                let next = VirtualListState { active: Some(i), selected: Some(i) };
                let commands =
                    if state.selected == Some(i) { Vec::new() } else { vec![VirtualListCommand::SelectedChange(i)] };
                match changed(state, next) {
                    Some(next) => Transition::update(state, next, commands),
                    None => Transition::stay(),
                }
            }
            VirtualListEvent::Point(i) if input.is_option(i) && state.active != Some(i) => {
                Transition::to(VirtualListState { active: Some(i), ..state.clone() })
            }
            VirtualListEvent::Click(_) | VirtualListEvent::Point(_) => Transition::stay(),
        }
    }

    /// Drop an active or selected row that is no longer an option (rows removed, or it
    /// became a header).
    fn sync(state: &VirtualListState, input: &VirtualListInput) -> Option<VirtualListState> {
        let keep = |i: Option<usize>| i.filter(|&i| input.is_option(i));
        changed(state, VirtualListState { active: keep(state.active), selected: keep(state.selected) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_for_places_headers() {
        let l = layout_for(4, &[0], 10.0, 30.0);
        assert!((l.offset(1) - 30.0).abs() < f64::EPSILON);
        assert!((layout_for(4, &[], 10.0, 30.0).offset(2) - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn anchor_shift_reports_growth_above_the_view() {
        let mut l = layout_for(10, &[], 10.0, 10.0);
        let (moved, shift) = anchor_shift(&mut l, 50.0, [(1, 25.0)]);
        assert!(moved);
        assert!((shift - 15.0).abs() < f64::EPSILON);
        let (moved, shift) = anchor_shift(&mut l, 50.0, [(1, 25.0)]);
        assert!(!moved);
        assert!(shift.abs() < f64::EPSILON);
    }

    fn input() -> VirtualListInput {
        VirtualListInput { count: 10, headers: vec![0, 5], default_selected: None }
    }

    #[test]
    fn focus_activates_selected_or_first_and_scrolls() {
        let i = input();
        let s = VirtualListMachine::initial(&i);
        let step = hydroxy_statechart::step::<VirtualListMachine>(&s, VirtualListEvent::Focus(0), &i);
        assert_eq!(step.state.active, Some(1));
        assert_eq!(step.commands, vec![VirtualListCommand::ScrollTo(1)]);
        // Nothing selected: start at the first option on screen, past a header.
        let mid = hydroxy_statechart::step::<VirtualListMachine>(&s, VirtualListEvent::Focus(5), &i);
        assert_eq!(mid.state.active, Some(6));
        let i = VirtualListInput { default_selected: Some(7), ..input() };
        let s = VirtualListMachine::initial(&i);
        assert_eq!(s.selected, Some(7));
        let step = hydroxy_statechart::step::<VirtualListMachine>(&s, VirtualListEvent::Focus(0), &i);
        assert_eq!(step.state.active, Some(7));
        let again = VirtualListMachine::transition(&step.state, VirtualListEvent::Focus(0), &i);
        assert_eq!(again.state, None);
    }

    #[test]
    fn a_header_is_never_selected_by_default() {
        let i = VirtualListInput { default_selected: Some(5), ..input() };
        assert_eq!(VirtualListMachine::initial(&i).selected, None);
    }

    #[test]
    fn moving_and_selecting() {
        let i = input();
        let s = VirtualListState { active: Some(4), selected: None };
        let step = hydroxy_statechart::step::<VirtualListMachine>(&s, VirtualListEvent::Move(Nav::Next), &i);
        assert_eq!(step.state.active, Some(6));
        assert_eq!(step.commands, vec![VirtualListCommand::ScrollTo(6)]);
        let sel = hydroxy_statechart::step::<VirtualListMachine>(&step.state, VirtualListEvent::SelectActive, &i);
        assert_eq!(sel.state.selected, Some(6));
        assert_eq!(sel.commands, vec![VirtualListCommand::SelectedChange(6)]);
        let again = VirtualListMachine::transition(&sel.state, VirtualListEvent::SelectActive, &i);
        assert_eq!(again.state, None);
        assert!(again.commands.is_empty());
        let end = VirtualListState { active: Some(9), selected: None };
        let stay = VirtualListMachine::transition(&end, VirtualListEvent::Move(Nav::Next), &i);
        assert_eq!(stay.state, None);
        assert_eq!(stay.commands, vec![VirtualListCommand::ScrollTo(9)]);
    }

    #[test]
    fn clicking_selects_options_only() {
        let i = input();
        let s = VirtualListState::default();
        let step = hydroxy_statechart::step::<VirtualListMachine>(&s, VirtualListEvent::Click(3), &i);
        assert_eq!(step.state, VirtualListState { active: Some(3), selected: Some(3) });
        assert_eq!(step.commands, vec![VirtualListCommand::SelectedChange(3)]);
        assert_eq!(VirtualListMachine::transition(&s, VirtualListEvent::Click(5), &i).state, None);
        assert_eq!(VirtualListMachine::transition(&step.state, VirtualListEvent::Click(3), &i).state, None);
    }

    #[test]
    fn pointing_activates_without_scrolling() {
        let i = input();
        let s = VirtualListState::default();
        let step = VirtualListMachine::transition(&s, VirtualListEvent::Point(2), &i);
        assert_eq!(step.state, Some(VirtualListState { active: Some(2), selected: None }));
        assert!(step.commands.is_empty());
        assert_eq!(VirtualListMachine::transition(&s, VirtualListEvent::Point(0), &i).state, None);
    }

    #[test]
    fn sync_drops_rows_that_went_away() {
        let s = VirtualListState { active: Some(8), selected: Some(2) };
        let shorter = VirtualListInput { count: 5, ..input() };
        assert_eq!(
            hydroxy_statechart::sync_or_keep::<VirtualListMachine>(&s, &shorter),
            VirtualListState { active: None, selected: Some(2) }
        );
        assert_eq!(VirtualListMachine::sync(&s, &input()), None);
    }
}
