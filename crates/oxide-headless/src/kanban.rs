//! Kanban connector. Anatomy: root, column, header, list, card, drop-end,
//! instructions, live. Every part carries `data-scope="kanban"` and its
//! `data-part`. Columns are `role="list"`, cards `role="listitem"`.
//!
//! Pointer drags use native HTML5 drag and drop and mark the landing spot with
//! `data-drop-before` on a card or `data-drop-target` on the column's end zone.
//! Keyboard moves reshuffle the preview layout instead, so the card is seen moving.

pub use oxide_machines::kanban::*;

/// The `data-scope` value on every kanban part.
pub const SCOPE: &str = "kanban";

/// Read by screen readers on every card, through `aria-describedby`.
pub const INSTRUCTIONS: &str =
    "Press Space to pick up the card. Use the arrow keys to move it, Space to drop it, or Escape to cancel.";

/// Attributes for a card (`<li role="listitem">`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardAttrs {
    /// `id`, so the adapter can focus it.
    pub id: String,
    /// `draggable` and `tabindex="0"`; false when the board is disabled.
    pub movable: bool,
    /// `aria-describedby`: the instructions' id.
    pub aria_describedby: String,
    /// Emit `data-grabbed` while this card is being moved.
    pub grabbed: bool,
    /// Emit `data-drop-before` when a pointer drag would land just above this card.
    pub drop_before: bool,
}

/// The card's id for card `card`.
#[must_use]
pub fn card_id(root: &str, card: &str) -> String {
    format!("{root}-card-{card}")
}

/// The column heading's id, which names the list.
#[must_use]
pub fn header_id(root: &str, column: &str) -> String {
    format!("{root}-header-{column}")
}

/// The instructions' id.
#[must_use]
pub fn instructions_id(root: &str) -> String {
    format!("{root}-instructions")
}

/// Where a pointer drag would land: (column, card it goes before, or `None` for the end).
#[must_use]
pub fn pointer_target(state: &KanbanState) -> Option<(&str, Option<&str>)> {
    let g = state.grab.as_ref().filter(|g| !g.keyboard)?;
    let column = state.columns.iter().find(|c| c.id == g.column)?;
    let before = column.cards.iter().filter(|k| k.id != g.card).nth(g.index).map(|k| k.id.as_str());
    Some((column.id.as_str(), before))
}

/// Attributes for card `card`.
#[must_use]
pub fn card_attrs(root: &str, state: &KanbanState, input: &KanbanInput, card: &str) -> CardAttrs {
    let target = pointer_target(state);
    let grabbed = state.grab.as_ref().is_some_and(|g| g.card == card);
    CardAttrs {
        id: card_id(root, card),
        movable: !input.disabled,
        aria_describedby: instructions_id(root),
        grabbed,
        // Landing right above itself changes nothing, so don't mark it.
        drop_before: !grabbed && target.is_some_and(|(_, before)| before == Some(card)),
    }
}

/// Whether a pointer drag would land at the end of `column`.
#[must_use]
pub fn drop_at_end(state: &KanbanState, column: &str) -> bool {
    pointer_target(state).is_some_and(|(c, before)| c == column && before.is_none())
}

/// Map a `KeyboardEvent.key` on card `card` to a machine event. `Some` means: prevent default.
#[must_use]
pub fn card_keydown(key: &str, card: &str, state: &KanbanState) -> Option<KanbanEvent> {
    let carrying = state.grab.as_ref().is_some_and(|g| g.keyboard);
    match (key, carrying) {
        (" ", false) => Some(KanbanEvent::Grab(card.to_string())),
        (" ", true) => Some(KanbanEvent::Drop),
        ("Escape", true) => Some(KanbanEvent::Cancel),
        ("ArrowUp", true) => Some(KanbanEvent::Move(MoveDirection::Up)),
        ("ArrowDown", true) => Some(KanbanEvent::Move(MoveDirection::Down)),
        ("ArrowLeft", true) => Some(KanbanEvent::Move(MoveDirection::Left)),
        ("ArrowRight", true) => Some(KanbanEvent::Move(MoveDirection::Right)),
        _ => None,
    }
}

/// Default English wording for an announcement.
#[must_use]
pub fn announcement_text(a: &Announcement) -> String {
    match a {
        Announcement::Grabbed { card, column, position, count } => {
            format!("Picked up {card}, position {position} of {count} in {column}.")
        }
        Announcement::Moved { card, column, position, count } => {
            format!("{card} moved to position {position} of {count} in {column}.")
        }
        Announcement::Dropped { card, column, position } => {
            format!("Dropped {card} at position {position} in {column}.")
        }
        Announcement::Cancelled { card } => format!("Move cancelled. {card} is back where it started."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> KanbanState {
        KanbanState {
            columns: vec![
                KanbanColumn::new("todo", "To do", vec![KanbanCard::new("a", "A"), KanbanCard::new("b", "B")]),
                KanbanColumn::new("done", "Done", vec![]),
            ],
            ..KanbanState::default()
        }
    }

    #[test]
    fn pointer_markers_follow_the_target() {
        let mut s = state();
        s.grab = Some(Grab { card: "a".into(), from: "todo".into(), column: "todo".into(), index: 1, keyboard: false });
        assert_eq!(pointer_target(&s), Some(("todo", None)));
        assert!(drop_at_end(&s, "todo"));
        s.grab.as_mut().unwrap().index = 0;
        assert!(card_attrs("k", &s, &KanbanInput::default(), "b").drop_before);
        assert!(!card_attrs("k", &s, &KanbanInput::default(), "a").drop_before);
    }

    #[test]
    fn space_picks_up_then_drops() {
        let mut s = state();
        assert_eq!(card_keydown(" ", "a", &s), Some(KanbanEvent::Grab("a".into())));
        assert_eq!(card_keydown("ArrowDown", "a", &s), None);
        s.grab = Some(Grab { card: "a".into(), from: "todo".into(), column: "todo".into(), index: 0, keyboard: true });
        assert_eq!(card_keydown(" ", "a", &s), Some(KanbanEvent::Drop));
        assert_eq!(card_keydown("Escape", "a", &s), Some(KanbanEvent::Cancel));
    }
}
