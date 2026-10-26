//! `<DataTable>`: a native `<table>` with sortable columns, optional row
//! selection and optional client-side paging.

use std::cmp::Ordering;
use std::sync::Arc;

use leptos::prelude::*;
use oxide_headless::data_table::{
    self as h, DataTableCommand, DataTableEvent, DataTableInput, DataTableMachine, Sort,
};
use oxide_statechart::Machine;

use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::checkbox::{Checkbox, CheckedState};

pub use oxide_headless::data_table::{Sort as DataTableSort, SortDirection};

type CellFn<T> = Arc<dyn Fn(&T) -> AnyView + Send + Sync>;
type CompareFn<T> = Arc<dyn Fn(&T, &T) -> Ordering + Send + Sync>;

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
            cell: Arc::clone(&self.cell),
            compare: self.compare.clone(),
            numeric: self.numeric,
        }
    }
}

impl<T: 'static> Column<T> {
    /// A column keyed `key`, headed `header`, rendering each row with `cell`.
    pub fn new<V: IntoView + 'static>(
        key: impl Into<String>,
        header: impl Into<String>,
        cell: impl Fn(&T) -> V + Send + Sync + 'static,
    ) -> Self {
        Self {
            key: key.into(),
            header: header.into(),
            cell: Arc::new(move |row| cell(row).into_any()),
            compare: None,
            numeric: false,
        }
    }

    /// Make the column sortable with a comparator.
    #[must_use]
    pub fn sort_by(mut self, compare: impl Fn(&T, &T) -> Ordering + Send + Sync + 'static) -> Self {
        self.compare = Some(Arc::new(compare));
        self
    }

    /// Make the column sortable by a key, such as `|r| r.age`.
    #[must_use]
    pub fn sort_by_key<K: Ord>(self, key: impl Fn(&T) -> K + Send + Sync + 'static) -> Self {
        self.sort_by(move |a, b| key(a).cmp(&key(b)))
    }

    /// Right-align with tabular figures, for numbers.
    #[must_use]
    pub fn numeric(mut self) -> Self {
        self.numeric = true;
        self
    }
}

/// A table of `rows`, one `<td>` per column.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn DataTable<T>(
    /// Accessible name, rendered as a visually hidden `<caption>`.
    #[prop(into)]
    caption: String,
    /// Columns in display order.
    columns: Vec<Column<T>>,
    /// The data.
    #[prop(into)]
    rows: Signal<Vec<T>>,
    /// A stable, unique id per row. Used for selection.
    row_id: fn(&T) -> String,
    /// Show a checkbox column with select-all.
    #[prop(optional)]
    selectable: bool,
    /// Controlled selected ids. Pair with `on_selection_change`.
    #[prop(optional, into)]
    selected: MaybeProp<Vec<String>>,
    /// Initial selected ids when uncontrolled.
    #[prop(optional)]
    default_selected: Vec<String>,
    /// Called with the new selected ids.
    #[prop(optional, into)]
    on_selection_change: Option<Callback<Vec<String>>>,
    /// Initial sort.
    #[prop(optional)]
    default_sort: Option<Sort>,
    /// Called when a header changes the sort.
    #[prop(optional, into)]
    on_sort_change: Option<Callback<Option<Sort>>>,
    /// Rows per page. Adds a footer with previous and next. Default: every row.
    #[prop(optional)]
    page_size: Option<usize>,
    /// Shown in place of rows when there are none. Default "No results".
    #[prop(optional, into)]
    empty: Option<ViewFn>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    let input = Memo::new(move |_| DataTableInput {
        row_ids: rows.with(|r| r.iter().map(row_id).collect()),
        selected: selected.get(),
        default_selected: default_selected.clone(),
        default_sort: default_sort.clone(),
        page_size,
    });
    let state = RwSignal::new(input.with_untracked(DataTableMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| DataTableMachine::sync(s, i)));
        state.set(synced);
    });

    let send = move |event: DataTableEvent| {
        // Events can still arrive after unmount.
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| DataTableMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        state.set(step.state);
        for command in step.commands {
            match command {
                DataTableCommand::SortChange(sort) => {
                    if let Some(cb) = on_sort_change {
                        cb.run(sort);
                    }
                }
                DataTableCommand::SelectionChange(ids) => {
                    if let Some(cb) = on_selection_change {
                        cb.run(ids);
                    }
                }
            }
        }
    };

    let columns = StoredValue::new(columns);
    // Separate memos so a selection change doesn't rebuild the rows.
    let sort = Memo::new(move |_| state.with(|s| s.sort.clone()));
    let page = Memo::new(move |_| state.with(|s| s.page));
    let len = Memo::new(move |_| rows.with(Vec::len));
    let span = columns.with_value(Vec::len) + usize::from(selectable);

    let head = move || {
        columns.with_value(|cols| {
            cols.iter()
                .map(|col| {
                    let key = StoredValue::new(col.key.clone());
                    let sortable = col.compare.is_some();
                    let header = col.header.clone();
                    let cell = if sortable {
                        view! {
                            <button
                                type="button"
                                data-scope=h::SCOPE
                                data-part="sort-button"
                                data-sort=move || state.with(|s| h::data_sort(s, &key.read_value()))
                                on:click=move |_| send(DataTableEvent::ToggleSort(key.get_value()))
                            >
                                {header}
                                <SortGlyph />
                            </button>
                        }
                        .into_any()
                    } else {
                        header.into_any()
                    };
                    view! {
                        <th
                            scope="col"
                            data-scope=h::SCOPE
                            data-part="header-cell"
                            data-numeric=col.numeric.then_some("")
                            aria-sort=move || sortable.then(|| state.with(|s| h::aria_sort(s, &key.read_value())))
                        >
                            {cell}
                        </th>
                    }
                })
                .collect_view()
        })
    };

    let render_row = move |row: &T| {
        let id = StoredValue::new(row_id(row));
        let picked = Memo::new(move |_| state.with(|s| h::is_selected(s, &id.read_value())));
        let check = selectable.then(|| {
            view! {
                <td data-scope=h::SCOPE data-part="cell" data-select="">
                    <Checkbox
                        checked=Signal::derive(move || Some(CheckedState::from(picked.get())))
                        on_checked_change=move |_| send(DataTableEvent::ToggleRow(id.get_value()))
                    >
                        <span data-scope=h::SCOPE data-part="sr">"Select row"</span>
                    </Checkbox>
                </td>
            }
        });
        let cells = columns.with_value(|cols| {
            cols.iter()
                .map(|col| {
                    view! {
                        <td data-scope=h::SCOPE data-part="cell" data-numeric=col.numeric.then_some("")>
                            {(col.cell)(row)}
                        </td>
                    }
                })
                .collect_view()
        });
        view! {
            <tr
                data-scope=h::SCOPE
                data-part="row"
                data-selected=move || picked.get().then_some("")
            >
                {check}
                {cells}
            </tr>
        }
    };

    // Rows are keyed by id, plus a version that bumps whenever `rows` changes: sorting,
    // paging and selection move or keep existing rows, and new data re-renders them,
    // since rows can't be compared for edits.
    let version = Memo::new(move |prev: Option<&u64>| {
        rows.track();
        prev.map_or(0, |v| v + 1)
    });
    let visible = move || {
        let sort = sort.get();
        let range = h::page_range(page.get(), page_size, len.get());
        let v = version.get();
        rows.with(|data| {
            let compare = sort.as_ref().and_then(|s| {
                columns.with_value(|cols| cols.iter().find(|c| c.key == s.key).and_then(|c| c.compare.clone()))
            });
            let order: Vec<usize> = match (sort, compare) {
                (Some(s), Some(cmp)) => h::sorted_indices(data, s.direction, |a, b| cmp(a, b)),
                _ => (0..data.len()).collect(),
            };
            order[range].iter().map(|&i| (row_id(&data[i]), v, data[i].clone())).collect::<Vec<_>>()
        })
    };
    let body = view! {
        <Show
            when=move || rows.with(|data| !data.is_empty())
            fallback=move || {
                let empty = empty.as_ref().map_or_else(|| "No results".into_any(), ViewFn::run);
                view! {
                    <tr data-scope=h::SCOPE data-part="row">
                        <td data-scope=h::SCOPE data-part="empty" colspan=span>{empty}</td>
                    </tr>
                }
            }
        >
            <For each=visible key=|(id, v, _)| (id.clone(), *v) children=move |(_, _, row)| render_row(&row) />
        </Show>
    };

    let footer = page_size.map(|size| {
        let pages = move || h::page_count(len.get(), Some(size));
        view! {
            <div data-scope=h::SCOPE data-part="footer">
                <span data-scope=h::SCOPE data-part="range" aria-live="polite">
                    {move || h::range_label(page.get(), Some(size), len.get())}
                </span>
                <div data-scope=h::SCOPE data-part="pager">
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Sm
                        disabled=Signal::derive(move || page.get() == 0)
                        on_click=move |_| send(DataTableEvent::PrevPage)
                    >
                        "Previous"
                    </Button>
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Sm
                        disabled=Signal::derive(move || page.get() + 1 >= pages())
                        on_click=move |_| send(DataTableEvent::NextPage)
                    >
                        "Next"
                    </Button>
                </div>
            </div>
        }
    });

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <div data-scope=h::SCOPE data-part="scroller">
                <table data-scope=h::SCOPE data-part="table">
                    <caption data-scope=h::SCOPE data-part="caption">{caption}</caption>
                    <thead data-scope=h::SCOPE data-part="header">
                        <tr>
                            {selectable.then(|| view! {
                                <th scope="col" data-scope=h::SCOPE data-part="header-cell" data-select="">
                                    <Checkbox
                                        checked=Signal::derive(move || Some(input.with(|i| state.with(|s| h::select_all_state(s, i)))))
                                        disabled=Signal::derive(move || len.get() == 0)
                                        on_checked_change=move |_| send(DataTableEvent::ToggleAll)
                                    >
                                        <span data-scope=h::SCOPE data-part="sr">"Select all rows"</span>
                                    </Checkbox>
                                </th>
                            })}
                            {head}
                        </tr>
                    </thead>
                    <tbody data-scope=h::SCOPE data-part="body">{body}</tbody>
                </table>
            </div>
            {footer}
        </div>
    }
}

/// Up/down arrows; CSS shows the half that matches `data-sort`.
#[component]
fn SortGlyph() -> impl IntoView {
    view! {
        <svg data-scope=h::SCOPE data-part="sort-glyph" viewBox="0 0 16 16" aria-hidden="true">
            <path data-glyph="asc" d="M5 7l3-3 3 3" />
            <path data-glyph="desc" d="M5 9l3 3 3-3" />
        </svg>
    }
}
