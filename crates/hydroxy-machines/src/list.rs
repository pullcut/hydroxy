//! Roving-focus helpers shared by list-like machines (tabs, radio group, accordion).

/// Which way an arrow key moves focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Towards the first item.
    Prev,
    /// Towards the last item.
    Next,
}

/// Index of the next enabled item after an arrow key.
///
/// - `current`: position among enabled items, or `None` if nothing is focused
///   or selected yet.
/// - `len`: number of enabled items, always > 0.
/// - `wrap`: whether to loop past the ends.
pub(crate) fn next_index(current: Option<usize>, len: usize, direction: Direction, wrap: bool) -> usize {
    let last = len - 1;
    match (current, direction) {
        (None, Direction::Next) => 0,
        (None, Direction::Prev) => last,
        (Some(i), Direction::Next) if i == last => {
            if wrap {
                0
            } else {
                last
            }
        }
        (Some(i), Direction::Next) => i + 1,
        (Some(0), Direction::Prev) => {
            if wrap {
                last
            } else {
                0
            }
        }
        (Some(i), Direction::Prev) => i - 1,
    }
}
