use leptos::prelude::*;
use hydroxy_leptos::kanban::*;

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

pub fn demo() -> AnyView {
    // Controlled: the owner applies each move.
    let (columns, set_columns) = signal(board());
    let (last, set_last) = signal(String::from("Nothing moved yet."));

    view! {
        <div style="display: grid; gap: var(--space-4);">
            <KanbanBoard
                columns=columns
                on_move=move |mv: KanbanMove| {
                    set_last.set(format!("{} → {} at {}", mv.from, mv.to, mv.index));
                    set_columns.update(|cols| apply_move(cols, &mv));
                }
            />
            <p>{last}</p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "kanban-board",
    name: "Kanban board",
    group: "Data",
    summary: "Columns of cards. Drag a card to reorder it or move it to another column, or pick it up with Space and move it with the arrow keys.",
    demo,
    leptos: include_str!("kanban.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/kanban.rs"),
    parts: &[
        &["KanbanBoard", "root", "Grid of columns. data-dragging while a card is moving"],
        &["KanbanBoard", "column", "<section> per column"],
        &["KanbanBoard", "header, count", "<h3> that names the list, with the card count"],
        &["KanbanBoard", "list", "<ul role=list> drop zone. data-drop-end when a drag would land at the end"],
        &["KanbanBoard", "card", "<li role=listitem draggable tabindex=0>. data-grabbed while moving, data-drop-before when a drag would land above it"],
        &["KanbanBoard", "instructions", "Visually hidden keyboard help, linked from every card by aria-describedby"],
        &["KanbanBoard", "live", "role=status region that announces pick-up, moves, drop and cancel"],
    ],
    keys: &[
        &["Space", "Picks up the focused card, or drops the card you're carrying"],
        &["ArrowUp / ArrowDown", "Moves the card up or down its column"],
        &["ArrowLeft / ArrowRight", "Moves the card to the previous or next column"],
        &["Escape", "Puts the card back where it started"],
        &["Tab", "Moves between cards"],
    ],
    props: &[
        &["columns", "Option<Vec<KanbanColumn>>", "Controlled layout. Pair with on_move and apply_move"],
        &["default_columns", "Vec<KanbanColumn>", "Initial layout when uncontrolled"],
        &["on_move", "Callback<KanbanMove>", "Called when a card lands somewhere new: card, from, to, index"],
        &["card", "Callback<KanbanCard, AnyView>", "Renders a card's content. Default: its title"],
        &["disabled", "bool", "Block every move"],
    ],
};
