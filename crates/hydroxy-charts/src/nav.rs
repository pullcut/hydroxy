//! Keyboard focus over chart marks. Every interactive chart uses these, so arrow keys,
//! Home/End and Escape behave the same everywhere.

pub use crate::line::crosshair_key as list_key;

/// Arrow keys for a chart whose marks form one row: Down acts as Right and Up as Left, so
/// vertical keys step through the marks too. Other keys pass through for [`list_key`].
#[must_use]
pub fn linear_key(key: &str) -> &str {
    match key {
        "ArrowDown" => "ArrowRight",
        "ArrowUp" => "ArrowLeft",
        k => k,
    }
}

/// Hint: step through the marks with the left and right arrow keys.
pub const HINT_STEP_LR: &str = "Use the left and right arrow keys to step through.";
/// Hint: step through values with the arrow keys.
pub const HINT_STEP_VALUES: &str = "Use the arrow keys to step through values.";
/// Hint: read values with the left and right arrow keys.
pub const HINT_READ_VALUES_LR: &str = "Use the left and right arrow keys to read values.";
/// Hint: move between cells with the arrow keys.
pub const HINT_CELLS: &str = "Use the arrow keys to move between cells.";
/// Hint: step through nodes with the arrow keys.
pub const HINT_NODES: &str = "Use the arrow keys to step through the nodes.";
/// Hint: step through points with the arrow keys.
pub const HINT_POINTS: &str = "Use the arrow keys to step through points.";
/// Hint: step through groups with the arrow keys.
pub const HINT_GROUPS: &str = "Use the arrow keys to step through groups.";
/// Hint: read each region with the arrow keys.
pub const HINT_REGIONS: &str = "Use the arrow keys to read each region.";

/// An accessible name with its keyboard hint: "Label. Hint".
#[must_use]
pub fn hinted(label: &str, hint: &str) -> String {
    format!("{label}. {hint}")
}

/// Arrow-key movement over a grid of marks (heatmap cells, matrix cells, grouped bars).
/// Returns the new cell (or `None` to clear) when `key` is handled.
/// Left/Right move along a row, Up/Down between rows; Home/End jump within the row.
#[must_use]
pub fn grid_key(
    key: &str,
    current: Option<(usize, usize)>,
    rows: usize,
    cols: usize,
) -> Option<Option<(usize, usize)>> {
    if rows == 0 || cols == 0 {
        return None;
    }
    let (r, c) = current.unwrap_or((0, 0));
    let fresh = current.is_none();
    let next = match key {
        "ArrowRight" if fresh => (0, 0),
        "ArrowRight" => (r, (c + 1).min(cols - 1)),
        "ArrowLeft" => (r, c.saturating_sub(usize::from(!fresh))),
        "ArrowDown" => ((r + usize::from(!fresh)).min(rows - 1), c),
        "ArrowUp" => (r.saturating_sub(1), c),
        "Home" => (r, 0),
        "End" => (r, cols - 1),
        "Escape" if !fresh => return Some(None),
        _ => return None,
    };
    Some(Some(next))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_keys_become_horizontal_and_hints_join_labels() {
        assert_eq!((linear_key("ArrowDown"), linear_key("ArrowUp")), ("ArrowRight", "ArrowLeft"));
        assert_eq!((linear_key("ArrowLeft"), linear_key("Home")), ("ArrowLeft", "Home"));
        assert_eq!(hinted("Sales", HINT_CELLS), "Sales. Use the arrow keys to move between cells.");
    }

    #[test]
    fn grid_keys_move_and_clamp() {
        assert_eq!(grid_key("ArrowRight", None, 3, 4), Some(Some((0, 0))));
        assert_eq!(grid_key("ArrowRight", Some((1, 3)), 3, 4), Some(Some((1, 3))));
        assert_eq!(grid_key("ArrowDown", Some((2, 1)), 3, 4), Some(Some((2, 1))));
        assert_eq!(grid_key("ArrowUp", Some((2, 1)), 3, 4), Some(Some((1, 1))));
        assert_eq!(grid_key("End", Some((1, 0)), 3, 4), Some(Some((1, 3))));
        assert_eq!(grid_key("Escape", Some((1, 0)), 3, 4), Some(None));
        assert_eq!(grid_key("Escape", None, 3, 4), None);
        assert_eq!(grid_key("a", Some((0, 0)), 3, 4), None);
    }
}
