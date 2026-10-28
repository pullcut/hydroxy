use std::collections::BTreeMap;

use dioxus::prelude::*;
use oxide_dioxus::spreadsheet_grid::{Cell, CellChange, SpreadsheetGrid, cell_name, cells_from_a1};

const ROWS: usize = 10_000;
const CATEGORIES: [&str; 6] = ["Travel", "Software", "Meals", "Hardware", "Training", "Office"];
const VENDORS: [&str; 8] = ["Northwind Rail", "Figma", "Dishoom", "Framework", "O'Reilly", "Muji", "Linear", "Eurostar"];

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

#[component]
pub fn SpreadsheetGridDemo() -> Element {
    let mut last = use_signal(|| String::from("Type into a cell, or select a range and paste from a spreadsheet."));
    let cells = use_hook(ledger);

    rsx! {
        div { style: "display: grid; gap: var(--space-3);",
            SpreadsheetGrid {
                label: "Expense claims",
                rows: ROWS,
                cols: 8,
                default_cells: cells,
                col_width: 120.0,
                height: 420.0,
                on_change: move |changes: Vec<CellChange>| {
                    let text = match changes.as_slice() {
                        [one] => format!(
                            "{}: {} → {}",
                            cell_name((one.row, one.col)),
                            if one.previous.is_empty() { "empty" } else { one.previous.as_str() },
                            if one.value.is_empty() { "empty" } else { one.value.as_str() },
                        ),
                        many => format!("{} cells changed", many.len()),
                    };
                    last.set(text);
                },
            }
            p { style: "margin: 0; font: var(--text-label); color: var(--text-3);", "{last}" }
        }
    }
}
