//! Roving-focus helpers shared by list-like machines (tabs, radio group, accordion).

/// Which way an arrow key moves focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Towards the first item.
    Prev,
    /// Towards the last item.
    Next,
}

/// Move `i` by `delta` within `0..len`. With `wrap` it loops past either end, otherwise
/// it stops at the end. An empty list gives 0.
#[must_use]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "list lengths fit in isize, and the result is within 0..len"
)]
pub fn step_index(i: usize, delta: isize, len: usize, wrap: bool) -> usize {
    if len == 0 {
        return 0;
    }
    let (i, len) = (i as isize, len as isize);
    let moved = if wrap { (i + delta).rem_euclid(len) } else { (i + delta).clamp(0, len - 1) };
    moved as usize
}

/// Index of the next enabled item after an arrow key.
///
/// - `current`: position among enabled items, or `None` if nothing is focused
///   or selected yet.
/// - `len`: number of enabled items; 0 gives 0.
/// - `wrap`: whether to loop past the ends.
pub(crate) fn next_index(current: Option<usize>, len: usize, direction: Direction, wrap: bool) -> usize {
    let delta = match direction {
        Direction::Next => 1,
        Direction::Prev => -1,
    };
    match current {
        Some(i) => step_index(i, delta, len, wrap),
        None => step_index(0, delta.min(0), len, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_index_wraps_or_clamps() {
        assert_eq!(step_index(2, 1, 3, true), 0);
        assert_eq!(step_index(2, 1, 3, false), 2);
        assert_eq!(step_index(0, -1, 3, true), 2);
        assert_eq!(step_index(0, -1, 3, false), 0);
        assert_eq!(step_index(1, 5, 3, true), 0);
        assert_eq!(step_index(1, 5, 3, false), 2);
        assert_eq!(step_index(0, 1, 0, true), 0);
    }

    #[test]
    fn next_index_starts_at_either_end() {
        assert_eq!(next_index(None, 4, Direction::Next, false), 0);
        assert_eq!(next_index(None, 4, Direction::Prev, false), 3);
        assert_eq!(next_index(Some(3), 4, Direction::Next, true), 0);
        assert_eq!(next_index(None, 0, Direction::Prev, true), 0);
    }
}
