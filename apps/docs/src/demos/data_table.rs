use hydroxy_leptos::data_table::{Column, DataTable, DataTableSort};
use leptos::prelude::*;

#[derive(Clone)]
struct Interview {
    id: u32,
    name: &'static str,
    company: &'static str,
    signals: u32,
}

const INTERVIEWS: [(&str, &str, u32); 12] = [
    ("Maya Chen", "Northwind", 14),
    ("Jonas Weber", "Fabrikam", 6),
    ("Aisha Okafor", "Contoso", 21),
    ("Luis Ortega", "Tailspin", 3),
    ("Hana Sato", "Litware", 9),
    ("Omar Haddad", "Adatum", 17),
    ("Priya Nair", "Northwind", 11),
    ("Elena Rossi", "Proseware", 5),
    ("Tom Becker", "Contoso", 8),
    ("Sofia Lind", "Fabrikam", 12),
    ("Kwame Mensah", "Adatum", 2),
    ("Ines Duarte", "Litware", 19),
];

#[component]
pub fn Demo() -> impl IntoView {
    let rows: Vec<Interview> =
        (1..).zip(INTERVIEWS).map(|(id, (name, company, signals))| Interview { id, name, company, signals }).collect();
    let no_rows: Vec<Interview> = Vec::new();
    let (selected, set_selected) = signal(Vec::<String>::new());

    let columns = vec![
        Column::new("name", "Participant", |r: &Interview| r.name).sort_by_key(|r| r.name),
        Column::new("company", "Company", |r: &Interview| r.company).sort_by_key(|r| r.company),
        Column::new("signals", "Signals", |r: &Interview| r.signals).sort_by_key(|r| r.signals).numeric(),
    ];

    view! {
        <div style="display: grid; gap: var(--space-8);">
            <div style="display: grid; gap: var(--space-3);">
                <DataTable
                    caption="Interviews"
                    columns=columns.clone()
                    rows=rows
                    row_id=|r: &Interview| r.id.to_string()
                    selectable=true
                    selected=selected
                    on_selection_change=move |ids| set_selected.set(ids)
                    default_sort=DataTableSort::desc("signals")
                    page_size=5
                />
                <p style="margin: 0; font: var(--text-sm); color: var(--text-3);">
                    {move || format!("{} selected", selected.get().len())}
                </p>
            </div>

            // Empty state.
            <DataTable
                caption="Archived interviews"
                columns=columns
                rows=no_rows
                row_id=|r: &Interview| r.id.to_string()
                empty=|| "Nothing archived yet"
            />
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "data-table",
    name: "Data table",
    group: "Data",
    summary: "Rows and columns in a native table. Sortable headers, checkbox selection with select-all, optional paging and a sticky header.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("data_table.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/data_table.rs"),
    parts: &[
        &["DataTable", "root", "Bordered wrapper"],
        &["", "scroller", "Scroll container. Set --ox-data-table-max-height to scroll under the sticky header"],
        &["", "table / caption", "Native <table>; the caption is visually hidden and names it"],
        &["", "header-cell", "<th scope=col>. aria-sort on sortable columns; data-numeric right-aligns"],
        &["", "sort-button", "Native button in a sortable header. data-sort=asc|desc|none"],
        &["", "row", "<tr>. data-selected when checked"],
        &["", "cell", "<td>. data-select on the checkbox column"],
        &["", "empty", "The single cell shown when there are no rows"],
        &["", "footer / range / pager", "Paging footer, only with page_size"],
    ],
    keys: &[
        &["Enter / Space", "On a header: sort ascending, descending, then off"],
        &["Space", "On a checkbox: select or deselect the row, or all rows"],
        &["Tab", "Moves through sort buttons, checkboxes and paging"],
    ],
    props: &[
        &["caption", "String", "Required. Accessible name"],
        &[
            "columns",
            "Vec<Column<T>>",
            "Column::new(key, header, cell), then .sort_by_key(..), .sort_by(..), .numeric()",
        ],
        &["rows", "Signal<Vec<T>>", "The data"],
        &["row_id", "fn(&T) -> String", "Stable, unique id per row"],
        &["selectable", "bool", "Checkbox column with select-all"],
        &["selected", "Vec<String>", "Controlled selected ids. Pair with on_selection_change"],
        &["default_selected", "Vec<String>", "Initial selection when uncontrolled"],
        &["on_selection_change", "Callback<Vec<String>>", "Called with the new selection"],
        &["default_sort", "Sort", "Initial sort, e.g. Sort::desc(\"signals\")"],
        &["on_sort_change", "Callback<Option<Sort>>", "Called when a header changes the sort"],
        &["page_size", "usize", "Rows per page; adds previous and next"],
        &["empty", "ViewFn", "Shown when there are no rows. Default \"No results\""],
    ],
    events: &[],
};
