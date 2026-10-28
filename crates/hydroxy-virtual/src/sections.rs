//! Group headers: lookups over the sorted header list and per-group [`Block`]s.

use std::ops::Range;

use crate::layout::Layout;

/// Whether row `index` is a group header.
#[must_use]
pub fn is_header(headers: &[usize], index: usize) -> bool {
    headers.binary_search(&index).is_ok()
}

/// The header of the group row `index` belongs to (itself, for a header). `None` for
/// rows before the first header.
#[must_use]
pub fn header_for(headers: &[usize], index: usize) -> Option<usize> {
    headers[..headers.partition_point(|&h| h <= index)].last().copied()
}

/// The first header after row `index`.
#[must_use]
pub fn next_header(headers: &[usize], index: usize) -> Option<usize> {
    headers.get(headers.partition_point(|&h| h <= index)).copied()
}

/// How many of `count` rows are options (not headers): `aria-setsize`.
#[must_use]
pub fn option_count(headers: &[usize], count: usize) -> usize {
    count - headers.partition_point(|&h| h < count)
}

/// 1-based position of option `index` among all options: `aria-posinset`.
#[must_use]
pub fn pos_in_set(headers: &[usize], index: usize) -> usize {
    index - headers.partition_point(|&h| h < index) + 1
}

/// How much a sticky header covers when row `index` is scrolled to: the size of its
/// group's header, or 0 for headers and ungrouped rows.
#[must_use]
pub fn sticky_inset(layout: &Layout, headers: &[usize], index: usize) -> f64 {
    header_for(headers, index).filter(|&h| h != index).map_or(0.0, |h| layout.size(h))
}

/// One group's slice of the rendered rows: a box from its header to the next header
/// (or the end), holding the header and the group's rendered rows.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    /// The header row, or `None` for rows before the first header.
    pub header: Option<usize>,
    /// Where the box starts: the header's offset, or 0.
    pub top: f64,
    /// The size of the whole group, header included.
    pub height: f64,
    /// Rendered rows in this group, ascending, header excluded. Position each at
    /// `layout.offset(row) - top`.
    pub rows: Vec<usize>,
}

fn block(layout: &Layout, headers: &[usize], header: Option<usize>) -> Block {
    let top = header.map_or(0.0, |h| layout.offset(h));
    let next = match header {
        Some(h) => next_header(headers, h),
        None => headers.first().copied(),
    };
    let bottom = next.map_or_else(|| layout.total(), |n| layout.offset(n));
    Block { header, top, height: bottom - top, rows: Vec::new() }
}

/// The rows to render, grouped: every row in `range`, plus `pinned` (the active option,
/// so `aria-activedescendant` always points at a real element) wherever it is. Each
/// group touched is a [`Block`]; its header is always rendered so it can stick.
#[must_use]
pub fn blocks(layout: &Layout, headers: &[usize], range: Range<usize>, pinned: Option<usize>) -> Vec<Block> {
    let Range { start, end } = range;
    let end = end.min(layout.len());
    let mut rows: Vec<usize> = (start..end).collect();
    if let Some(p) = pinned.filter(|&p| p < layout.len() && !(start..end).contains(&p)) {
        rows.insert(rows.partition_point(|&r| r < p), p);
    }
    let mut out: Vec<Block> = Vec::new();
    for row in rows {
        let header = header_for(headers, row);
        if out.last().is_none_or(|b| b.header != header) {
            out.push(block(layout, headers, header));
        }
        if header != Some(row)
            && let Some(b) = out.last_mut()
        {
            b.rows.push(row);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_helpers() {
        let hs = [0, 4, 9];
        assert!(is_header(&hs, 4));
        assert!(!is_header(&hs, 5));
        assert_eq!(header_for(&hs, 3), Some(0));
        assert_eq!(header_for(&hs, 4), Some(4));
        assert_eq!(header_for(&[2], 1), None);
        assert_eq!(next_header(&hs, 4), Some(9));
        assert_eq!(next_header(&hs, 9), None);
        assert_eq!(option_count(&hs, 12), 9);
        assert_eq!(option_count(&hs, 5), 3);
        assert_eq!(pos_in_set(&hs, 1), 1);
        assert_eq!(pos_in_set(&hs, 5), 4);
        assert_eq!(pos_in_set(&hs, 10), 8);
        let l = Layout::from_fn(12, 20.0, |i| if is_header(&hs, i) { 30.0 } else { 20.0 });
        assert!((sticky_inset(&l, &hs, 5) - 30.0).abs() < 1e-9);
        assert!(sticky_inset(&l, &hs, 4).abs() < 1e-9);
    }

    #[test]
    fn blocks_group_rows_under_their_header() {
        let hs = [2, 6];
        let l = Layout::fixed(10, 10.0);
        let b = blocks(&l, &hs, 4..8, None);
        assert_eq!(b.len(), 2);
        // The header of the first touched group is included even though row 2 is off-screen.
        assert_eq!(b[0], Block { header: Some(2), top: 20.0, height: 40.0, rows: vec![4, 5] });
        assert_eq!(b[1], Block { header: Some(6), top: 60.0, height: 40.0, rows: vec![7] });
        let lead = blocks(&l, &hs, 0..3, None);
        assert_eq!(lead[0], Block { header: None, top: 0.0, height: 20.0, rows: vec![0, 1] });
        assert_eq!(lead[1].rows, Vec::<usize>::new());
    }

    #[test]
    fn blocks_pin_the_active_row() {
        let l = Layout::fixed(100, 10.0);
        let b = blocks(&l, &[50], 0..3, Some(80));
        assert_eq!(b.len(), 2);
        assert_eq!(b[1].header, Some(50));
        assert_eq!(b[1].rows, vec![80]);
        assert_eq!(blocks(&l, &[], 0..3, Some(1))[0].rows, vec![0, 1, 2]);
        assert_eq!(blocks(&l, &[], 5..7, Some(2))[0].rows, vec![2, 5, 6]);
    }
}
