use dioxus::prelude::*;
use hydroxy_dioxus::kanban::{KanbanBoard, KanbanCard, KanbanColumn, KanbanMove, apply_move};

fn board() -> Vec<KanbanColumn> {
    vec![
        KanbanColumn::new(
            "todo",
            "To do",
            vec![
                KanbanCard::new("interviews", "Tag the Q3 interviews"),
                KanbanCard::new("tickets", "Import September tickets"),
            ],
        ),
        KanbanColumn::new("doing", "In progress", vec![KanbanCard::new("digest", "Draft the weekly digest")]),
        KanbanColumn::new("done", "Done", vec![KanbanCard::new("sources", "Check sources on the board update")]),
    ]
}

#[component]
pub fn KanbanDemo() -> Element {
    // Controlled: the owner applies each move.
    let mut columns = use_signal(board);
    let mut last = use_signal(|| String::from("Nothing moved yet."));

    rsx! {
        div { style: "display: grid; gap: var(--space-4);",
            KanbanBoard {
                columns: Some(columns()),
                on_move: move |mv: KanbanMove| {
                    last.set(format!("{} → {} at {}", mv.from, mv.to, mv.index));
                    apply_move(&mut columns.write(), &mv);
                },
            }
            p { "{last}" }
        }
    }
}
