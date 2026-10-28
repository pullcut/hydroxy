//! Data table: sort, row selection and paging. The machine only sees row ids
//! and column keys; rows, columns and cell rendering stay in the adapter.

use std::cmp::Ordering;
use std::ops::Range;

use hydroxy_statechart::{Machine, Transition};

use crate::checkbox::CheckedState;

/// Which way a sorted column runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Smallest first.
    Ascending,
    /// Largest first.
    Descending,
}

/// The sorted column and its direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sort {
    /// The column key.
    pub key: String,
    /// Which way it runs.
    pub direction: SortDirection,
}

impl Sort {
    /// Ascending on `key`.
    #[must_use]
    pub fn asc(key: impl Into<String>) -> Self {
        Self { key: key.into(), direction: SortDirection::Ascending }
    }

    /// Descending on `key`.
    #[must_use]
    pub fn desc(key: impl Into<String>) -> Self {
        Self { key: key.into(), direction: SortDirection::Descending }
    }
}

/// Props for [`DataTableMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataTableInput {
    /// Every row id, in data order. Select-all covers all of them.
    pub row_ids: Vec<String>,
    /// Controlled selection. `Some` at start fixes controlled mode.
    pub selected: Option<Vec<String>>,
    /// Uncontrolled initial selection.
    pub default_selected: Vec<String>,
    /// Initial sort. Sorting is uncontrolled.
    pub default_sort: Option<Sort>,
    /// Rows per page. `None` shows every row.
    pub page_size: Option<usize>,
}

/// What [`DataTableMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataTableState {
    /// The sorted column, if any.
    pub sort: Option<Sort>,
    /// Selected row ids.
    pub selected: Vec<String>,
    /// Zero-based page.
    pub page: usize,
    /// Fixed at start: the owner holds `selected`.
    pub controlled: bool,
}

/// Inputs to [`DataTableMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataTableEvent {
    /// A sortable header was activated: none, ascending, descending, none.
    ToggleSort(String),
    /// A row's checkbox was toggled.
    ToggleRow(String),
    /// The select-all checkbox was toggled.
    ToggleAll,
    /// Previous page.
    PrevPage,
    /// Next page.
    NextPage,
}

/// Side effects from [`DataTableMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataTableCommand {
    /// The sort changed.
    SortChange(Option<Sort>),
    /// The selection changed (or should, when controlled).
    SelectionChange(Vec<String>),
}

/// Sort, selection and paging for a table.
#[derive(Debug, Clone, Copy)]
pub struct DataTableMachine;

impl Machine for DataTableMachine {
    type Input = DataTableInput;
    type State = DataTableState;
    type Event = DataTableEvent;
    type Command = DataTableCommand;

    fn initial(input: &DataTableInput) -> DataTableState {
        DataTableState {
            sort: input.default_sort.clone(),
            selected: input.selected.clone().unwrap_or_else(|| input.default_selected.clone()),
            page: 0,
            controlled: input.selected.is_some(),
        }
    }

    fn transition(
        state: &DataTableState,
        event: DataTableEvent,
        input: &DataTableInput,
    ) -> Transition<DataTableState, DataTableCommand> {
        let mut next = state.clone();
        match event {
            DataTableEvent::ToggleSort(key) => {
                next.sort = match &state.sort {
                    Some(Sort { key: k, direction: SortDirection::Ascending }) if *k == key => Some(Sort::desc(key)),
                    Some(Sort { key: k, direction: SortDirection::Descending }) if *k == key => None,
                    _ => Some(Sort::asc(key)),
                };
                // A new order starts from the top.
                next.page = 0;
                let sort = next.sort.clone();
                Transition::with(next, vec![DataTableCommand::SortChange(sort)])
            }
            DataTableEvent::ToggleRow(id) => {
                let mut selected = state.selected.clone();
                if let Some(at) = selected.iter().position(|s| *s == id) {
                    selected.remove(at);
                } else {
                    selected.push(id);
                }
                select(next, selected)
            }
            DataTableEvent::ToggleAll => {
                let selected = if select_all_state(state, input) == CheckedState::Checked {
                    Vec::new()
                } else {
                    input.row_ids.clone()
                };
                select(next, selected)
            }
            DataTableEvent::PrevPage => {
                next.page = state.page.saturating_sub(1);
                Transition::to(next)
            }
            DataTableEvent::NextPage => {
                next.page = (state.page + 1).min(page_count(input.row_ids.len(), input.page_size) - 1);
                Transition::to(next)
            }
        }
    }

    fn sync(state: &DataTableState, input: &DataTableInput) -> Option<DataTableState> {
        let mut next = state.clone();
        match (state.controlled, &input.selected) {
            (true, Some(selected)) => next.selected.clone_from(selected),
            // Forget rows that are gone, so the next change doesn't report them.
            (false, _) => next.selected.retain(|id| input.row_ids.contains(id)),
            (true, None) => {}
        }
        next.page = state.page.min(page_count(input.row_ids.len(), input.page_size) - 1);
        hydroxy_statechart::changed(state, next)
    }
}

fn select(mut next: DataTableState, selected: Vec<String>) -> Transition<DataTableState, DataTableCommand> {
    if !next.controlled {
        next.selected.clone_from(&selected);
    }
    Transition::with(next, vec![DataTableCommand::SelectionChange(selected)])
}

/// The select-all checkbox: checked when every row is selected, mixed when some are.
// O(rows × selected) scan; switch `selected` to a HashSet if tables reach thousands of rows.
#[must_use]
pub fn select_all_state(state: &DataTableState, input: &DataTableInput) -> CheckedState {
    let picked = input.row_ids.iter().filter(|id| state.selected.contains(id)).count();
    match picked {
        0 => CheckedState::Unchecked,
        n if n == input.row_ids.len() => CheckedState::Checked,
        _ => CheckedState::Indeterminate,
    }
}

/// Number of pages, never less than one.
#[must_use]
pub fn page_count(len: usize, page_size: Option<usize>) -> usize {
    match page_size {
        Some(size) if size > 0 => len.div_ceil(size).max(1),
        _ => 1,
    }
}

/// The slice of the sorted rows on `page`.
#[must_use]
pub fn page_range(page: usize, page_size: Option<usize>, len: usize) -> Range<usize> {
    match page_size {
        Some(size) if size > 0 => {
            let start = (page * size).min(len);
            start..(start + size).min(len)
        }
        _ => 0..len,
    }
}

/// Row indices in display order. Stable, so equal rows keep their data order.
pub fn sorted_indices<T>(rows: &[T], direction: SortDirection, compare: impl Fn(&T, &T) -> Ordering) -> Vec<usize> {
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by(|&a, &b| {
        let ord = compare(&rows[a], &rows[b]);
        if direction == SortDirection::Descending { ord.reverse() } else { ord }
    });
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("r{i}")).collect()
    }

    fn step(
        state: &DataTableState,
        event: DataTableEvent,
        input: &DataTableInput,
    ) -> hydroxy_statechart::Step<DataTableState, DataTableCommand> {
        DataTableMachine::transition(state, event, input).resolve(state)
    }

    #[test]
    fn sort_cycles_asc_desc_none_and_resets_on_new_key() {
        let input = DataTableInput::default();
        let s = DataTableMachine::initial(&input);
        let s = step(&s, DataTableEvent::ToggleSort("name".into()), &input).state;
        assert_eq!(s.sort, Some(Sort::asc("name")));
        let s = step(&s, DataTableEvent::ToggleSort("name".into()), &input).state;
        assert_eq!(s.sort, Some(Sort::desc("name")));
        let t = step(&s, DataTableEvent::ToggleSort("name".into()), &input);
        assert_eq!(t.state.sort, None);
        assert_eq!(t.commands, vec![DataTableCommand::SortChange(None)]);
        let s = step(&s, DataTableEvent::ToggleSort("age".into()), &input).state;
        assert_eq!(s.sort, Some(Sort::asc("age")));
    }

    #[test]
    fn select_all_goes_mixed_then_all_then_none() {
        let input = DataTableInput { row_ids: ids(3), ..Default::default() };
        let s = DataTableMachine::initial(&input);
        assert_eq!(select_all_state(&s, &input), CheckedState::Unchecked);
        let s = step(&s, DataTableEvent::ToggleRow("r1".into()), &input).state;
        assert_eq!(select_all_state(&s, &input), CheckedState::Indeterminate);
        let s = step(&s, DataTableEvent::ToggleAll, &input).state;
        assert_eq!(s.selected, ids(3));
        assert_eq!(select_all_state(&s, &input), CheckedState::Checked);
        let t = step(&s, DataTableEvent::ToggleAll, &input);
        assert!(t.state.selected.is_empty());
        assert_eq!(t.commands, vec![DataTableCommand::SelectionChange(Vec::new())]);
    }

    #[test]
    fn controlled_selection_requests_and_waits_for_sync() {
        let mut input = DataTableInput { row_ids: ids(2), selected: Some(Vec::new()), ..Default::default() };
        let s = DataTableMachine::initial(&input);
        let t = step(&s, DataTableEvent::ToggleRow("r0".into()), &input);
        assert!(t.state.selected.is_empty());
        assert_eq!(t.commands, vec![DataTableCommand::SelectionChange(vec!["r0".into()])]);
        input.selected = Some(vec!["r0".into()]);
        assert_eq!(hydroxy_statechart::sync_or_keep::<DataTableMachine>(&t.state, &input).selected, vec!["r0".to_string()]);
    }

    #[test]
    fn uncontrolled_sync_drops_removed_rows() {
        let mut input = DataTableInput { row_ids: ids(3), default_selected: vec!["r2".into()], ..Default::default() };
        let s = DataTableMachine::initial(&input);
        input.row_ids.pop();
        assert!(hydroxy_statechart::sync_or_keep::<DataTableMachine>(&s, &input).selected.is_empty());
    }

    #[test]
    fn paging_is_clamped() {
        let mut input = DataTableInput { row_ids: ids(25), page_size: Some(10), ..Default::default() };
        let mut s = DataTableMachine::initial(&input);
        for _ in 0..5 {
            s = step(&s, DataTableEvent::NextPage, &input).state;
        }
        assert_eq!(s.page, 2);
        assert_eq!(page_range(s.page, input.page_size, 25), 20..25);
        input.row_ids.truncate(5);
        assert_eq!(hydroxy_statechart::sync_or_keep::<DataTableMachine>(&s, &input).page, 0);
        assert_eq!(step(&s, DataTableEvent::PrevPage, &input).state.page, 1);
        assert_eq!(page_count(0, Some(10)), 1);
        assert_eq!(page_range(0, None, 7), 0..7);
    }

    #[test]
    fn sorted_indices_is_stable_and_reversible() {
        let rows = [3, 1, 2, 1];
        assert_eq!(sorted_indices(&rows, SortDirection::Ascending, Ord::cmp), vec![1, 3, 2, 0]);
        assert_eq!(sorted_indices(&rows, SortDirection::Descending, Ord::cmp), vec![0, 2, 1, 3]);
    }
}
