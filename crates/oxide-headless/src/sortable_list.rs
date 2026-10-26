//! Sortable list connector. Anatomy: root, list, item, instructions, live.
//! Every part carries `data-scope="sortable-list"` and its `data-part`. The
//! list is `role="list"` and items `role="listitem"`.
//!
//! Pointer drags use native HTML5 drag and drop and mark the landing spot with
//! `data-drop-before` on an item or `data-drop-end` on the list. Keyboard moves
//! reshuffle the preview, so the item is seen moving. The drag and move logic is
//! the kanban connector's, run over one column.

use crate::kanban::{self, CardAttrs, KanbanInput};
pub use oxide_machines::sortable_list::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "sortable-list";

/// Read by screen readers on every item, through `aria-describedby`.
pub const INSTRUCTIONS: &str =
    "Press Space to pick up the item. Use the up and down arrow keys to move it, Space to drop it, or Escape to cancel.";

/// Attributes for an item (`<li role="listitem">`). Same shape as a kanban card's.
pub type ItemAttrs = CardAttrs;

/// The item's DOM id.
#[must_use]
pub fn item_id(root: &str, item: &str) -> String {
    format!("{root}-item-{item}")
}

/// The instructions' id.
#[must_use]
pub fn instructions_id(root: &str) -> String {
    format!("{root}-instructions")
}

/// Attributes for item `item`.
#[must_use]
pub fn item_attrs(root: &str, state: &SortableState, disabled: bool, item: &str) -> ItemAttrs {
    let input = KanbanInput { disabled, ..KanbanInput::default() };
    CardAttrs {
        id: item_id(root, item),
        aria_describedby: instructions_id(root),
        ..kanban::card_attrs(root, state, &input, item)
    }
}

/// Whether a pointer drag would land at the end of the list.
#[must_use]
pub fn drop_at_end(state: &SortableState) -> bool {
    kanban::drop_at_end(state, COLUMN)
}

/// Map a `KeyboardEvent.key` on item `item` to a machine event. `Some` means: prevent default.
#[must_use]
pub fn item_keydown(key: &str, item: &str, state: &SortableState) -> Option<SortableEvent> {
    if matches!(key, "ArrowLeft" | "ArrowRight") {
        return None;
    }
    kanban::card_keydown(key, item, state)
}

/// Default English wording for an announcement. `position` counts from 1.
#[must_use]
pub fn announcement_text(a: &Announcement) -> String {
    match a {
        Announcement::Grabbed { card, position, count, .. } => {
            format!("Picked up {card}, position {position} of {count}.")
        }
        Announcement::Moved { card, position, count, .. } => format!("{card} moved to position {position} of {count}."),
        Announcement::Dropped { card, position, .. } => format!("Dropped {card} at position {position}."),
        Announcement::Cancelled { card } => format!("Move cancelled. {card} is back where it started."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kanban::{Grab, KanbanColumn};

    fn state() -> SortableState {
        SortableState {
            columns: vec![KanbanColumn::new(COLUMN, "List", vec![SortItem::new("a", "A"), SortItem::new("b", "B")])],
            ..SortableState::default()
        }
    }

    #[test]
    fn items_get_their_own_ids_and_keys() {
        let s = state();
        let a = item_attrs("s", &s, false, "a");
        assert_eq!(a.id, "s-item-a");
        assert_eq!(a.aria_describedby, "s-instructions");
        assert!(a.movable);
        assert!(!item_attrs("s", &s, true, "a").movable);
        assert_eq!(item_keydown(" ", "a", &s), Some(SortableEvent::Grab("a".into())));
    }

    #[test]
    fn horizontal_arrows_do_nothing() {
        let mut s = state();
        s.grab = Some(Grab { card: "a".into(), from: COLUMN.into(), column: COLUMN.into(), index: 0, keyboard: true });
        assert_eq!(item_keydown("ArrowDown", "a", &s), Some(SortableEvent::Move(MoveDirection::Down)));
        assert_eq!(item_keydown("ArrowLeft", "a", &s), None);
    }

    #[test]
    fn announcements_read_naturally() {
        let a = Announcement::Moved { card: "A".into(), column: "x".into(), position: 2, count: 3 };
        assert_eq!(announcement_text(&a), "A moved to position 2 of 3.");
    }
}
