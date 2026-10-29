//! Keyboard movement over the rows that are options.

use crate::sections::is_header;

/// The first row that is not a header.
#[must_use]
pub fn first_option(headers: &[usize], count: usize) -> Option<usize> {
    (0..count).find(|&i| !is_header(headers, i))
}

/// The last row that is not a header.
#[must_use]
pub fn last_option(headers: &[usize], count: usize) -> Option<usize> {
    (0..count).rev().find(|&i| !is_header(headers, i))
}

/// Which way to move or search: towards the first item or the last.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Towards the first item.
    Prev,
    /// Towards the last item.
    Next,
}

/// The nearest option at or after `from` ([`Direction::Next`]) or at or before it.
#[must_use]
pub fn option_from(headers: &[usize], count: usize, from: usize, direction: Direction) -> Option<usize> {
    match direction {
        Direction::Next => (from..count).find(|&i| !is_header(headers, i)),
        Direction::Prev => (0..=from.min(count.checked_sub(1)?)).rev().find(|&i| !is_header(headers, i)),
    }
}

/// Where a key moves the active option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nav {
    /// The next option (`ArrowDown`).
    Next,
    /// The previous option (`ArrowUp`).
    Prev,
    /// The first option (Home).
    First,
    /// The last option (End).
    Last,
    /// About a viewport further (`PageDown`): this many rows.
    PageDown(usize),
    /// About a viewport back (`PageUp`): this many rows.
    PageUp(usize),
}

/// The option `nav` moves to from `from` (`None`: nothing active yet). `None` when the
/// list has no options. Stops at the ends.
#[must_use]
pub fn navigate(headers: &[usize], count: usize, from: Option<usize>, nav: Nav) -> Option<usize> {
    let first = first_option(headers, count)?;
    let Some(from) = from.filter(|&f| f < count) else {
        return match nav {
            Nav::Prev | Nav::Last | Nav::PageUp(_) => last_option(headers, count),
            _ => Some(first),
        };
    };
    let target = match nav {
        Nav::First => Some(first),
        Nav::Last => last_option(headers, count),
        Nav::Next => option_from(headers, count, from + 1, Direction::Next),
        Nav::Prev => from.checked_sub(1).and_then(|f| option_from(headers, count, f, Direction::Prev)),
        Nav::PageDown(page) => {
            let to = from.saturating_add(page.max(1)).min(count - 1);
            option_from(headers, count, to, Direction::Next)
                .or_else(|| option_from(headers, count, to, Direction::Prev))
        }
        Nav::PageUp(page) => {
            let to = from.saturating_sub(page.max(1));
            option_from(headers, count, to, Direction::Prev)
                .or_else(|| option_from(headers, count, to, Direction::Next))
        }
    };
    Some(target.unwrap_or(from))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_skips_headers_and_stops_at_ends() {
        let hs = [0, 3, 4, 8];
        let n = 10;
        assert_eq!(navigate(&hs, n, None, Nav::Next), Some(1));
        assert_eq!(navigate(&hs, n, None, Nav::Prev), Some(9));
        assert_eq!(navigate(&hs, n, Some(2), Nav::Next), Some(5));
        assert_eq!(navigate(&hs, n, Some(5), Nav::Prev), Some(2));
        assert_eq!(navigate(&hs, n, Some(1), Nav::Prev), Some(1));
        assert_eq!(navigate(&hs, n, Some(9), Nav::Next), Some(9));
        assert_eq!(navigate(&hs, n, Some(5), Nav::First), Some(1));
        assert_eq!(navigate(&hs, n, Some(1), Nav::Last), Some(9));
        assert_eq!(navigate(&hs, n, Some(1), Nav::PageDown(7)), Some(9));
        assert_eq!(navigate(&hs, n, Some(1), Nav::PageDown(2)), Some(5));
        assert_eq!(navigate(&hs, n, Some(9), Nav::PageUp(5)), Some(2));
        assert_eq!(navigate(&hs, n, Some(9), Nav::PageUp(100)), Some(1));
        assert_eq!(navigate(&[0, 1], 2, None, Nav::Next), None);
        assert_eq!(navigate(&[], 0, None, Nav::Next), None);
    }
}
