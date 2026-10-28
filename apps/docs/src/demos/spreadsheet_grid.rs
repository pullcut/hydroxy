use std::collections::BTreeMap;

use hydroxy_leptos::spreadsheet_grid::{Cell, CellChange, SpreadsheetGrid, cell_name, cells_from_a1};
use leptos::prelude::*;

const ROWS: usize = 10_000;
const CATEGORIES: [&str; 6] = ["Travel", "Software", "Meals", "Hardware", "Training", "Office"];
const VENDORS: [&str; 8] =
    ["Northwind Rail", "Figma", "Dishoom", "Framework", "O'Reilly", "Muji", "Linear", "Eurostar"];

/// A year of expense claims: 9,999 rows, VAT as a formula on each, and totals that read every row.
fn ledger() -> BTreeMap<Cell, String> {
    let mut cells = cells_from_a1(&[
        ("A1", "Date"),
        ("B1", "Category"),
        ("C1", "Vendor"),
        ("D1", "Amount"),
        ("E1", "VAT"),
        ("G1", "Summary"),
        ("G2", "Total with VAT"),
        ("H2", "=SUM(D2:E10000)"),
        ("G3", "Average claim"),
        ("H3", "=AVERAGE(D2:D10000)"),
        ("G4", "Largest"),
        ("H4", "=MAX(D2:D10000)"),
        ("G5", "Smallest"),
        ("H5", "=MIN(D2:D10000)"),
        ("G6", "VAT owed"),
        ("H6", "=SUM(E2:E10000)"),
    ]);
    for row in 1..ROWS {
        let cents = (row * 7919) % 90_000 + 1_250;
        cells.insert((row, 0), format!("2026-{:02}-{:02}", (row / 28) % 12 + 1, row % 28 + 1));
        cells.insert((row, 1), CATEGORIES[(row * 7 + row / 3) % 6].to_string());
        cells.insert((row, 2), VENDORS[(row * 5 + row / 7) % 8].to_string());
        cells.insert((row, 3), format!("{}.{:02}", cents / 100, cents % 100));
        cells.insert((row, 4), format!("=D{}*0.2", row + 1));
    }
    cells
}

pub fn demo() -> AnyView {
    let (last, set_last) = signal(String::from("Type into a cell, or select a range and paste from a spreadsheet."));
    let on_change = move |changes: Vec<CellChange>| {
        let text = match changes.as_slice() {
            [one] => format!(
                "{}: {} → {}",
                cell_name((one.row, one.col)),
                if one.previous.is_empty() { "empty" } else { one.previous.as_str() },
                if one.value.is_empty() { "empty" } else { one.value.as_str() },
            ),
            many => format!("{} cells changed", many.len()),
        };
        set_last.set(text);
    };

    view! {
        <div style="display: grid; gap: var(--space-3);">
            <SpreadsheetGrid
                label="Expense claims"
                rows=ROWS
                cols=8
                default_cells=ledger()
                col_width=120.0
                height=420.0
                on_change=on_change
            />
            <p style="margin: 0; font: var(--text-label); color: var(--text-3);">{last}</p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "spreadsheet-grid",
    name: "Spreadsheet grid",
    group: "Data",
    summary: "An editable sheet with A1 columns and row numbers. Select ranges with the pointer or Shift, type numbers, text or formulas (=SUM(D2:D9), =A1*1.2), copy and paste as tab-separated text, and resize columns. Only the rows in view are rendered, so 10,000 rows scroll smoothly.",
    demo,
    leptos: include_str!("spreadsheet_grid.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/spreadsheet_grid.rs"),
    parts: &[
        &["SpreadsheetGrid", "root", "Formula bar over the viewport, plus the live status"],
        &[
            "",
            "formula-bar, name-box, formula",
            "The selection's name (B3 or A1:C4) and the active cell's raw text, formula included",
        ],
        &[
            "",
            "viewport",
            "role=grid, the one tab stop. aria-activedescendant names the active cell; aria-multiselectable. Height from the height prop",
        ],
        &["", "header-row, corner, column-header", "Sticky A, B, C header. data-selected on columns in the selection"],
        &["", "resize-handle", "role=separator on each column's right edge. Drag, or arrow keys when focused"],
        &[
            "",
            "body, row, row-header",
            "Rows are absolutely placed at 28px each; only those in view are rendered. Row numbers are sticky",
        ],
        &[
            "",
            "cell",
            "role=gridcell with aria-selected. data-kind: number, text, error or empty. data-active, data-editing",
        ],
        &["", "selection", "One outline around the selected range. data-range when it spans more than one cell"],
        &["", "editor", "Text input over the cell being edited"],
        &["", "status", "Polite live region: range size, formula results, pastes and clears"],
    ],
    keys: &[
        &["Arrow keys", "Move the active cell"],
        &["Shift + arrow keys", "Extend the selection"],
        &["Ctrl + arrow keys", "Jump to the sheet's edge"],
        &["Home / End", "First or last column of the row; with Ctrl, A1 or the last cell"],
        &["PageUp / PageDown", "Move one screen"],
        &["Enter / F2", "Edit the active cell"],
        &["Any character", "Start editing, replacing the cell"],
        &["Enter / Shift + Enter (editing)", "Save and move down or up"],
        &["Tab / Shift + Tab (editing)", "Save and move right or left"],
        &["Escape", "Editing: discard. Otherwise: shrink the selection to the active cell"],
        &["Delete / Backspace", "Clear the selected cells"],
        &["Ctrl + A", "Select every cell"],
        &["Ctrl + C / X / V", "Copy, cut or paste the selection as tab-separated text"],
        &["Left / Right (resize handle)", "Narrow or widen the column by 8px, 32px with Shift"],
    ],
    props: &[
        &["label", "String", "Required. The grid's accessible name"],
        &["rows / cols", "usize", "Sheet size. Defaults 1000 and 10"],
        &[
            "default_cells",
            "BTreeMap<Cell, String>",
            "Initial text by (row, column). cells_from_a1(&[(\"B2\", \"=A1*2\")]) builds one from A1 names",
        ],
        &["col_width", "f64", "Initial column width in pixels. Default 112"],
        &["height", "f64", "Viewport height in pixels. Default 360"],
        &[
            "on_change",
            "Callback<Vec<CellChange>>",
            "Each edit, paste, cut or clear, with row, col, previous and new text",
        ],
    ],
};
