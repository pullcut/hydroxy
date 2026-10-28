//! `DataTable`: a native `<table>` with sortable columns, optional row
//! selection and optional client-side paging.

#![allow(
    unpredictable_function_pointer_comparisons,
    reason = "props compare `row_id` only to skip re-renders; a false mismatch just re-renders"
)]

use std::cmp::Ordering;
use std::rc::Rc;

use dioxus::prelude::*;
use hydroxy_headless::data_table::{
    self as h, DataTableCommand, DataTableEvent, DataTableInput, DataTableMachine, DataTableState, Sort,
};
use hydroxy_statechart::Machine;

use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::checkbox::{Checkbox, CheckedState};
use crate::drive::drive;

pub use hydroxy_headless::data_table::{Sort as DataTableSort, SortDirection};

type CellFn<T> = Rc<dyn Fn(&T) -> Element>;
type CompareFn<T> = Rc<dyn Fn(&T, &T) -> Ordering>;

/// One column: a key, header text, a cell renderer and an optional sort order.
pub struct Column<T> {
    key: String,
    header: String,
    cell: CellFn<T>,
    compare: Option<CompareFn<T>>,
    numeric: bool,
}

impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            header: self.header.clone(),
            cell: Rc::clone(&self.cell),
            compare: self.compare.clone(),
            numeric: self.numeric,
        }
    }
}

impl<T> PartialEq for Column<T> {
    fn eq(&self, other: &Self) -> bool {
        let same_compare = match (&self.compare, &other.compare) {
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        self.key == other.key
            && self.header == other.header
            && self.numeric == other.numeric
            && Rc::ptr_eq(&self.cell, &other.cell)
            && same_compare
    }
}

impl<T: 'static> Column<T> {
    /// A column keyed `key`, headed `header`, rendering each row with `cell`.
    pub fn new(key: impl Into<String>, header: impl Into<String>, cell: impl Fn(&T) -> Element + 'static) -> Self {
        Self { key: key.into(), header: header.into(), cell: Rc::new(cell), compare: None, numeric: false }
    }

    /// Make the column sortable with a comparator.
    #[must_use]
    pub fn sort_by(mut self, compare: impl Fn(&T, &T) -> Ordering + 'static) -> Self {
        self.compare = Some(Rc::new(compare));
        self
    }

    /// Make the column sortable by a key, such as `|r| r.age`.
    #[must_use]
    pub fn sort_by_key<K: Ord>(self, key: impl Fn(&T) -> K + 'static) -> Self {
        self.sort_by(move |a, b| key(a).cmp(&key(b)))
    }

    /// Right-align with tabular figures, for numbers.
    #[must_use]
    pub fn numeric(mut self) -> Self {
        self.numeric = true;
        self
    }
}

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<DataTableState>,
    input: Memo<DataTableInput>,
    on_selection_change: Option<EventHandler<Vec<String>>>,
    on_sort_change: Option<EventHandler<Option<Sort>>>,
}

impl Ctx {
    fn send(self, event: DataTableEvent) {
        let commands = drive::<DataTableMachine>(self.state, self.input, event);
        for command in commands {
            match command {
                DataTableCommand::SortChange(sort) => {
                    if let Some(handler) = self.on_sort_change {
                        handler.call(sort);
                    }
                }
                DataTableCommand::SelectionChange(ids) => {
                    if let Some(handler) = self.on_selection_change {
                        handler.call(ids);
                    }
                }
            }
        }
    }
}

/// A table of `rows`, one `<td>` per column.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn DataTable<T: Clone + PartialEq + 'static>(
    /// Accessible name, rendered as a visually hidden `<caption>`.
    caption: String,
    /// Columns in display order.
    columns: Vec<Column<T>>,
    /// The data.
    rows: ReadSignal<Vec<T>>,
    /// A stable, unique id per row. Used for selection.
    row_id: fn(&T) -> String,
    /// Show a checkbox column with select-all.
    #[props(default)]
    selectable: bool,
    /// Controlled selected ids. Pair with `on_selection_change`.
    #[props(default)]
    selected: ReadSignal<Option<Vec<String>>>,
    /// Initial selected ids when uncontrolled.
    #[props(default)]
    default_selected: Vec<String>,
    /// Called with the new selected ids.
    on_selection_change: Option<EventHandler<Vec<String>>>,
    /// Initial sort.
    default_sort: Option<Sort>,
    /// Called when a header changes the sort.
    on_sort_change: Option<EventHandler<Option<Sort>>>,
    /// Rows per page. Adds a footer with previous and next. Default: every row.
    page_size: Option<usize>,
    /// Shown in place of rows when there are none. Default "No results".
    empty: Option<Element>,
) -> Element {
    let input = use_memo(move || DataTableInput {
        row_ids: rows.read().iter().map(row_id).collect(),
        selected: selected(),
        default_selected: default_selected.clone(),
        default_sort: default_sort.clone(),
        page_size,
    });
    let mut state = use_signal(|| DataTableMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = DataTableMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ctx = Ctx { state, input, on_selection_change, on_sort_change };

    let st = state.read().clone();
    let all = h::select_all_state(&st, &input.read());
    let data = rows.read();
    let span = columns.len() + usize::from(selectable);
    let compare = st.sort.as_ref().and_then(|s| columns.iter().find(|c| c.key == s.key)?.compare.clone());
    let order: Vec<usize> = match (&st.sort, compare) {
        (Some(s), Some(cmp)) => h::sorted_indices(&data, s.direction, |a, b| cmp(a, b)),
        _ => (0..data.len()).collect(),
    };
    let visible = &order[h::page_range(st.page, page_size, data.len())];
    let pages = h::page_count(data.len(), page_size);

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            div { "data-scope": h::SCOPE, "data-part": "scroller",
                table { "data-scope": h::SCOPE, "data-part": "table",
                    caption { "data-scope": h::SCOPE, "data-part": "caption", {caption} }
                    thead { "data-scope": h::SCOPE, "data-part": "header",
                        tr {
                            if selectable {
                                th { scope: "col", "data-scope": h::SCOPE, "data-part": "header-cell", "data-select": "",
                                    Checkbox {
                                        checked: Some(all),
                                        disabled: data.is_empty(),
                                        on_checked_change: move |_| ctx.send(DataTableEvent::ToggleAll),
                                        span { "data-scope": h::SCOPE, "data-part": "sr", "Select all rows" }
                                    }
                                }
                            }
                            for col in columns.iter() {
                                th {
                                    key: "{col.key}",
                                    scope: "col",
                                    "data-scope": h::SCOPE,
                                    "data-part": "header-cell",
                                    "data-numeric": col.numeric.then_some(""),
                                    "aria-sort": col.compare.is_some().then(|| h::aria_sort(&st, &col.key)),
                                    if col.compare.is_some() {
                                        button {
                                            r#type: "button",
                                            "data-scope": h::SCOPE,
                                            "data-part": "sort-button",
                                            "data-sort": h::data_sort(&st, &col.key),
                                            onclick: {
                                                let key = col.key.clone();
                                                move |_| ctx.send(DataTableEvent::ToggleSort(key.clone()))
                                            },
                                            {col.header.clone()}
                                            svg { "data-scope": h::SCOPE, "data-part": "sort-glyph", view_box: "0 0 16 16", "aria-hidden": "true",
                                                path { "data-glyph": "asc", d: "M5 7l3-3 3 3" }
                                                path { "data-glyph": "desc", d: "M5 9l3 3 3-3" }
                                            }
                                        }
                                    } else {
                                        {col.header.clone()}
                                    }
                                }
                            }
                        }
                    }
                    tbody { "data-scope": h::SCOPE, "data-part": "body",
                        if data.is_empty() {
                            tr { "data-scope": h::SCOPE, "data-part": "row",
                                td { "data-scope": h::SCOPE, "data-part": "empty", colspan: span,
                                    if let Some(empty) = empty { {empty} } else { "No results" }
                                }
                            }
                        }
                        for &i in visible {
                            {
                                let row = &data[i];
                                let id = row_id(row);
                                let picked = h::is_selected(&st, &id);
                                rsx! {
                                    tr {
                                        key: "{id}",
                                        "data-scope": h::SCOPE,
                                        "data-part": "row",
                                        "data-selected": picked.then_some(""),
                                        if selectable {
                                            td { "data-scope": h::SCOPE, "data-part": "cell", "data-select": "",
                                                Checkbox {
                                                    checked: Some(CheckedState::from(picked)),
                                                    on_checked_change: {
                                                        let id = id.clone();
                                                        move |_| ctx.send(DataTableEvent::ToggleRow(id.clone()))
                                                    },
                                                    span { "data-scope": h::SCOPE, "data-part": "sr", "Select row" }
                                                }
                                            }
                                        }
                                        for col in columns.iter() {
                                            td {
                                                key: "{col.key}",
                                                "data-scope": h::SCOPE,
                                                "data-part": "cell",
                                                "data-numeric": col.numeric.then_some(""),
                                                {(col.cell)(row)}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(size) = page_size {
                div { "data-scope": h::SCOPE, "data-part": "footer",
                    span { "data-scope": h::SCOPE, "data-part": "range", "aria-live": "polite",
                        {h::range_label(st.page, Some(size), data.len())}
                    }
                    div { "data-scope": h::SCOPE, "data-part": "pager",
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            disabled: st.page == 0,
                            on_click: move |_| ctx.send(DataTableEvent::PrevPage),
                            "Previous"
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            disabled: st.page + 1 >= pages,
                            on_click: move |_| ctx.send(DataTableEvent::NextPage),
                            "Next"
                        }
                    }
                }
            }
        }
    }
}
