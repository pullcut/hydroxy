//! Keyboard focus over chart marks. Every interactive chart uses these, so arrow keys,
//! Home/End and Escape behave the same everywhere.

pub use crate::line::crosshair_key as list_key;

/// Arrow-key movement over a grid of marks (heatmap cells, matrix cells, grouped bars).
/// Returns the new cell (or `None` to clear) when `key` is handled.
/// Left/Right move along a row, Up/Down between rows; Home/End jump within the row.
#[must_use]
pub fn grid_key(key: &str, current: Option<(usize, usize)>, rows: usize, cols: usize) -> Option<Option<(usize, usize)>> {
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
