use leptos::prelude::*;
use hydroxy_leptos::grid::{Grid, GridColumn, GridEdit, GridRow};

fn rows() -> Vec<GridRow> {
    [
        ("Interview notes", "Maya", "In review", "14"),
        ("Support tickets", "Jon", "Tagging", "210"),
        ("Survey export", "Priya", "Done", "86"),
        ("Churn calls", "Sam", "In review", "9"),
        ("Usage logs", "Maya", "Queued", "1200"),
        ("Roadmap votes", "Jon", "Done", "47"),
        ("Pricing feedback", "Priya", "Tagging", "31"),
        ("Onboarding videos", "Sam", "Queued", "18"),
    ]
    .iter()
    .enumerate()
    .map(|(i, (a, b, c, d))| GridRow::new(format!("row-{i}"), &[a, b, c, d]))
    .collect()
}

pub fn demo() -> AnyView {
    let (last, set_last) = signal(String::from("Focus a cell, then press Enter or F2 to edit Owner or Items."));

    view! {
        <div style="display: grid; gap: var(--space-3);">
            <Grid
                label="Research sources"
                columns=vec![
                    GridColumn::new("source", "Source"),
                    GridColumn::new("owner", "Owner").editable(),
                    GridColumn::new("status", "Status"),
                    GridColumn::new("items", "Items").editable(),
                ]
                default_rows=rows()
                page_size=4
                on_edit=move |e: GridEdit| set_last.set(format!("{} · {}: {} → {}", e.row, e.column, e.previous, e.value))
            />
            <p style="margin: 0; font: var(--text-label); color: var(--text-3);">{last}</p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "grid",
    name: "Grid",
    group: "Data",
    summary: "An editable data grid. One cell is in the tab order, arrow keys move in two dimensions, and Enter or F2 edits a cell. Use Data table for sortable, selectable rows.",
    demo,
    leptos: include_str!("grid.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/grid.rs"),
    parts: &[
        &["Grid", "root", "role=grid with aria-rowcount and aria-colcount. Scrolls when --ox-grid-max-height is set"],
        &["", "header, header-row, header-cell", "Sticky header: role=rowgroup, row and columnheader"],
        &["", "body, row", "role=rowgroup and role=row. Rows are aria-rowindex 2 and up"],
        &["", "cell", "role=gridcell. tabindex=0 on one cell, -1 on the rest. aria-readonly on cells that cannot be edited. data-editing while edited"],
        &["", "editor", "Text input inside the cell being edited"],
    ],
    keys: &[
        &["Arrow keys", "Move to the neighbouring cell"],
        &["Home / End", "First or last cell of the row"],
        &["Ctrl + Home / End", "First or last cell of the grid"],
        &["PageUp / PageDown", "Move up or down one page of rows"],
        &["Enter / F2", "Edit the focused cell, if its column is editable"],
        &["Enter (editing)", "Save and return to the cell"],
        &["Escape (editing)", "Discard and return to the cell"],
        &["Tab (editing)", "Save and move on"],
    ],
    props: &[
        &["columns", "Vec<GridColumn>", "Required. GridColumn::new(key, header), then .editable()"],
        &["label", "String", "Required. The grid's accessible name"],
        &["default_rows", "Vec<GridRow>", "Initial rows when uncontrolled. GridRow::new(id, &[cell text])"],
        &["rows", "Option<Vec<GridRow>>", "Controlled rows. Pair with on_edit"],
        &["page_size", "usize", "Rows a PageUp or PageDown skips. Default 10"],
        &["on_edit", "Callback<GridEdit>", "Called with row id, column key, new and previous text"],
    ],
};
