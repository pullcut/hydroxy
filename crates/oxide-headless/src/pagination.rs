//! Pagination connector. Anatomy: root (`<nav>`), list, item, prev, next, page,
//! ellipsis. Every part carries `data-scope="pagination"` and its `data-part`.
//!
//! No machine: the page is one number, so this is pure functions. [`items`]
//! lays out the page buttons with ellipses and keeps their count fixed as the
//! page moves, so the buttons never shift under the pointer.

/// The `data-scope` value on every part.
pub const SCOPE: &str = "pagination";

/// The `<nav>`'s default `aria-label`.
pub const DEFAULT_LABEL: &str = "Pagination";

/// One slot in the page list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageItem {
    /// A page button, 1-based.
    Page(usize),
    /// A gap standing in for the pages between its neighbours.
    Ellipsis,
}

/// How many pages `total` items fill at `page_size` per page. A size of 0 counts as 1.
#[must_use]
pub fn page_count(total: usize, page_size: usize) -> usize {
    total.div_ceil(page_size.max(1))
}

/// `page` clamped to `1..=count`. Returns 1 when there are no pages.
#[must_use]
pub fn clamp_page(page: usize, count: usize) -> usize {
    page.clamp(1, count.max(1))
}

/// The page list for `page` of `count`: `boundaries` pages at each end (at
/// least 1), `siblings` either side of the current page, and an ellipsis for
/// each gap. An ellipsis only ever replaces two or more pages; a single hidden
/// page is shown instead.
#[must_use]
#[expect(clippy::cast_possible_wrap, clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "page numbers are far below i64::MAX")]
pub fn items(page: usize, count: usize, siblings: usize, boundaries: usize) -> Vec<PageItem> {
    if count == 0 {
        return Vec::new();
    }
    let (c, s, b) = (count as i64, siblings as i64, boundaries.max(1) as i64);
    let p = clamp_page(page, count) as i64;
    let pages = |from: i64, to: i64| (from..=to).map(|n| PageItem::Page(n as usize));

    // The sibling window, slid inwards near the ends so the list length stays fixed.
    let window_start = (p - s).min(c - b - 2 * s - 1).max(b + 2);
    let window_end = (p + s).max(b + 2 * s + 2).min(c - b - 1);

    let mut out: Vec<PageItem> = pages(1, b.min(c)).collect();
    if window_start > b + 2 {
        out.push(PageItem::Ellipsis);
    } else if b + 1 < c - b {
        out.push(PageItem::Page((b + 1) as usize));
    }
    out.extend(pages(window_start, window_end));
    if window_end < c - b - 1 {
        out.push(PageItem::Ellipsis);
    } else if c - b > b {
        out.push(PageItem::Page((c - b) as usize));
    }
    out.extend(pages((c - b + 1).max(b + 1), c));
    out
}

/// `aria-label` for a page button.
#[must_use]
pub fn page_label(page: usize) -> String {
    format!("Page {page}")
}

/// Whether there is a page before `page`.
#[must_use]
pub fn has_prev(page: usize) -> bool {
    page > 1
}

/// Whether there is a page after `page` of `count`.
#[must_use]
pub fn has_next(page: usize, count: usize) -> bool {
    page < count
}

#[cfg(test)]
mod tests {
    use super::PageItem::{Ellipsis as E, Page as P};
    use super::*;

    #[test]
    fn short_lists_show_every_page() {
        assert!(items(1, 0, 1, 1).is_empty());
        assert_eq!(items(1, 1, 1, 1), vec![P(1)]);
        assert_eq!(items(2, 3, 1, 1), vec![P(1), P(2), P(3)]);
        assert_eq!(items(4, 7, 1, 1), (1..=7).map(P).collect::<Vec<_>>());
    }

    #[test]
    fn ellipses_appear_on_the_far_side() {
        assert_eq!(items(1, 10, 1, 1), vec![P(1), P(2), P(3), P(4), P(5), E, P(10)]);
        assert_eq!(items(5, 10, 1, 1), vec![P(1), E, P(4), P(5), P(6), E, P(10)]);
        assert_eq!(items(10, 10, 1, 1), vec![P(1), E, P(6), P(7), P(8), P(9), P(10)]);
    }

    #[test]
    fn length_stays_fixed_as_the_page_moves() {
        for page in 1..=20 {
            assert_eq!(items(page, 20, 1, 1).len(), 7, "page {page}");
            assert_eq!(items(page, 20, 2, 2).len(), 11, "page {page}");
        }
    }

    #[test]
    fn wider_boundaries_and_siblings() {
        assert_eq!(items(10, 20, 2, 2), vec![P(1), P(2), E, P(8), P(9), P(10), P(11), P(12), E, P(19), P(20)]);
    }

    #[test]
    fn out_of_range_pages_clamp() {
        assert_eq!(items(0, 10, 1, 1), items(1, 10, 1, 1));
        assert_eq!(items(99, 10, 1, 1), items(10, 10, 1, 1));
        assert_eq!(clamp_page(0, 0), 1);
    }

    #[test]
    fn counts_and_edges() {
        assert_eq!(page_count(95, 10), 10);
        assert_eq!(page_count(0, 10), 0);
        assert_eq!(page_count(3, 0), 3);
        assert!(!has_prev(1) && has_prev(2));
        assert!(has_next(9, 10) && !has_next(10, 10));
        assert_eq!(page_label(3), "Page 3");
    }
}
