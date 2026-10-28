//! Dashboard layout connector. Anatomy: root, board, item, handle, resize,
//! instructions, live. Every part carries `data-scope="dashboard-layout"` and
//! its `data-part`. The board is a CSS grid; each item names its cells with
//! `grid-column` and `grid-row`, so moving a tile is a style change.
//!
//! The move handle is a button: Enter or Space picks the tile up, arrows move
//! it, Shift with arrows resizes it, Enter or Space drops it, Escape cancels.
//! The resize grip is for pointers only and hidden from assistive tech.

pub use hydroxy_machines::dashboard_layout::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "dashboard-layout";

/// Read by screen readers on every move handle, through `aria-describedby`.
pub const INSTRUCTIONS: &str = "Press Enter or Space to pick up the widget. Use the arrow keys to move it and Shift with the arrow keys to resize it. Press Enter or Space to drop it, or Escape to cancel.";

/// Attributes for a tile (`<article>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemAttrs {
    /// Inline style placing the tile on the grid.
    pub style: String,
    /// Emit `data-active` while this tile is being moved or resized.
    pub active: bool,
}

/// Attributes for a tile's move handle (`<button>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleAttrs {
    /// `id`, so the adapter can focus it.
    pub id: String,
    /// `aria-pressed`: picked up with the keyboard.
    pub aria_pressed: bool,
    /// `aria-describedby`: the instructions' id.
    pub aria_describedby: String,
}

/// The handle's id.
#[must_use]
pub fn handle_id(root: &str, item: &str) -> String {
    format!("{root}-handle-{item}")
}

/// The instructions' id.
#[must_use]
pub fn instructions_id(root: &str) -> String {
    format!("{root}-instructions")
}

/// Inline style for the board: its column count, row height and gap.
#[must_use]
pub fn board_style(columns: u32, row_height: f64, gap: f64) -> String {
    format!("--ox-dashboard-columns: {columns}; --ox-dashboard-row: {row_height}px; --ox-dashboard-gap: {gap}px;")
}

/// Inline style placing `item` on the grid.
#[must_use]
pub fn item_style(item: &DashboardItem) -> String {
    format!("grid-column: {} / span {}; grid-row: {} / span {};", item.x + 1, item.w, item.y + 1, item.h)
}

/// Attributes for tile `item` in the layout being shown.
#[must_use]
pub fn item_attrs(state: &DashboardState, item: &DashboardItem) -> ItemAttrs {
    ItemAttrs { style: item_style(item), active: state.active() == Some(item.id.as_str()) }
}

/// Attributes for tile `item`'s handle.
#[must_use]
pub fn handle_attrs(root: &str, state: &DashboardState, item: &str) -> HandleAttrs {
    HandleAttrs {
        id: handle_id(root, item),
        aria_pressed: state.gesture.as_ref().is_some_and(|g| g.keyboard && g.origin.id == item),
        aria_describedby: instructions_id(root),
    }
}

/// The event for a key on tile `item`'s handle, if the key does anything. `Some` means: prevent default.
#[must_use]
pub fn handle_keydown(key: &str, shift: bool, item: &str, state: &DashboardState) -> Option<DashboardEvent> {
    let carrying = state.gesture.as_ref().is_some_and(|g| g.keyboard);
    let nudge = |dx, dy| Some(DashboardEvent::Nudge { dx, dy, resize: shift });
    match (key, carrying) {
        ("Enter" | " ", false) => Some(DashboardEvent::Grab(item.to_string())),
        ("Enter" | " ", true) => Some(DashboardEvent::Drop),
        ("Escape", true) => Some(DashboardEvent::Cancel),
        ("ArrowLeft", true) => nudge(-1, 0),
        ("ArrowRight", true) => nudge(1, 0),
        ("ArrowUp", true) => nudge(0, -1),
        ("ArrowDown", true) => nudge(0, 1),
        _ => None,
    }
}

/// How many whole cells a pointer travelled, from its travel in pixels.
/// `board_width` is the board's width; cells are `columns` wide with `gap` between.
#[must_use]
pub fn cell_delta(travel_px: (f64, f64), board_width: f64, columns: u32, row_height: f64, gap: f64) -> (i32, i32) {
    if board_width <= 0.0 {
        return (0, 0);
    }
    let col_pitch = (board_width + gap) / f64::from(columns.max(1));
    let row_pitch = row_height + gap;
    #[allow(clippy::cast_possible_truncation, reason = "pixel travel is far below i32::MAX")]
    let cells = |px: f64, pitch: f64| if pitch > 0.0 { (px / pitch).round() as i32 } else { 0 };
    (cells(travel_px.0, col_pitch), cells(travel_px.1, row_pitch))
}

/// Default English wording for an announcement. `label` is the tile's visible name.
#[must_use]
pub fn announcement_text(a: &Announcement, label: &str) -> String {
    let at = |s: &Spot| format!("column {}, row {}, {} by {}", s.column, s.row, s.w, s.h);
    match a {
        Announcement::Grabbed(s) => format!("Picked up {label}, {}.", at(s)),
        Announcement::Moved(s) => format!("{label} now at {}.", at(s)),
        Announcement::Dropped(s) => format!("Dropped {label} at {}.", at(s)),
        Announcement::Cancelled(_) => format!("Move cancelled. {label} is back where it started."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_name_their_cells() {
        let item = DashboardItem::new("a", 2, 0, 4, 3);
        assert_eq!(item_style(&item), "grid-column: 3 / span 4; grid-row: 1 / span 3;");
    }

    #[test]
    fn keys_pick_up_move_and_drop() {
        let mut s = DashboardState::default();
        assert_eq!(handle_keydown(" ", false, "a", &s), Some(DashboardEvent::Grab("a".into())));
        assert_eq!(handle_keydown("ArrowDown", false, "a", &s), None);
        s.gesture = Some(Gesture { origin: DashboardItem::new("a", 0, 0, 1, 1), shift: (0, 0), grow: (0, 0), resizing: false, keyboard: true });
        assert_eq!(handle_keydown("ArrowRight", true, "a", &s), Some(DashboardEvent::Nudge { dx: 1, dy: 0, resize: true }));
        assert_eq!(handle_keydown("Enter", false, "a", &s), Some(DashboardEvent::Drop));
        assert!(handle_attrs("d", &s, "a").aria_pressed);
    }

    #[test]
    fn pixels_become_whole_cells() {
        // 12 columns across 1200px with a 0 gap: 100px per column; rows 72px + 8px gap.
        assert_eq!(cell_delta((230.0, 85.0), 1200.0, 12, 72.0, 0.0), (2, 1));
        assert_eq!(cell_delta((-140.0, -40.0), 1200.0, 12, 72.0, 8.0), (-1, -1));
    }
}
