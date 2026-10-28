use dioxus::prelude::*;
use hydroxy_dioxus::data_table::{Column, DataTable, DataTableSort};

#[derive(Clone, PartialEq)]
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
pub fn DataTableDemo() -> Element {
    let rows: Vec<Interview> = (1..)
        .zip(INTERVIEWS)
        .map(|(id, (name, company, signals))| Interview { id, name, company, signals })
        .collect();
    let mut selected = use_signal(Vec::<String>::new);

    let columns = vec![
        Column::new("name", "Participant", |r: &Interview| rsx! { {r.name} }).sort_by_key(|r| r.name),
        Column::new("company", "Company", |r: &Interview| rsx! { {r.company} }).sort_by_key(|r| r.company),
        Column::new("signals", "Signals", |r: &Interview| rsx! { "{r.signals}" }).sort_by_key(|r| r.signals).numeric(),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-8);",
            div { style: "display: grid; gap: var(--space-3);",
                DataTable {
                    caption: "Interviews",
                    columns: columns.clone(),
                    rows,
                    row_id: |r: &Interview| r.id.to_string(),
                    selectable: true,
                    selected: Some(selected()),
                    on_selection_change: move |ids| selected.set(ids),
                    default_sort: DataTableSort::desc("signals"),
                    page_size: 5,
                }
                p { style: "margin: 0; font: var(--text-sm); color: var(--text-3);", "{selected.read().len()} selected" }
            }
            DataTable {
                caption: "Archived interviews",
                columns,
                rows: Vec::<Interview>::new(),
                row_id: |r: &Interview| r.id.to_string(),
                empty: rsx! { "Nothing archived yet" },
            }
        }
    }
}
