use dioxus::prelude::*;
use hydroxy_dioxus::grid::{Grid, GridColumn, GridEdit, GridRow};

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

#[component]
pub fn GridDemo() -> Element {
    let mut last = use_signal(|| String::from("Focus a cell, then press Enter or F2 to edit Owner or Items."));

    rsx! {
        div { style: "display: grid; gap: var(--space-3);",
            Grid {
                label: "Research sources",
                columns: vec![
                    GridColumn::new("source", "Source"),
                    GridColumn::new("owner", "Owner").editable(),
                    GridColumn::new("status", "Status"),
                    GridColumn::new("items", "Items").editable(),
                ],
                default_rows: rows(),
                page_size: 4,
                on_edit: move |e: GridEdit| last.set(format!("{} · {}: {} → {}", e.row, e.column, e.previous, e.value)),
            }
            p { style: "margin: 0; font: var(--text-label); color: var(--text-3);", "{last}" }
        }
    }
}
