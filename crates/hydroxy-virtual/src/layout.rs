//! Row sizes and offsets: the Fenwick-tree [`Layout`] and its scroll alignment.

use std::ops::Range;

/// Rows rendered beyond each edge of the viewport, so fast scrolling doesn't flash blank.
pub const DEFAULT_OVERSCAN: usize = 6;

/// Size changes smaller than this (in pixels) are ignored, so sub-pixel measurement
/// jitter doesn't re-lay the list.
const EPSILON: f64 = 1e-3;

/// A usable row size: at least one pixel, so offsets strictly increase.
fn normalize(size: f64) -> f64 {
    if size.is_finite() { size.max(1.0) } else { 1.0 }
}

/// A finite, non-negative pixel value.
fn clean(value: f64) -> f64 {
    if value.is_finite() { value.max(0.0) } else { 0.0 }
}

/// Where a row lands in the viewport after [`Layout::scroll_to`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Align {
    /// Top edge at the top of the viewport.
    Start,
    /// Centred in the viewport.
    Center,
    /// Bottom edge at the bottom of the viewport.
    End,
    /// Don't move if the row is fully visible; otherwise the nearest edge.
    #[default]
    Auto,
}

/// Row sizes and offsets along one axis.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    count: usize,
    /// The size of rows that haven't been given one, and of every row while `sizes` is empty.
    estimate: f64,
    /// One size per row, or empty while every row is `estimate`.
    sizes: Vec<f64>,
    /// Fenwick tree over `sizes`, 1-based (`tree[0]` unused). Empty with `sizes`.
    tree: Vec<f64>,
}

impl Layout {
    /// `count` rows of `size` pixels each.
    #[must_use]
    pub fn fixed(count: usize, size: f64) -> Self {
        Self { count, estimate: normalize(size), sizes: Vec::new(), tree: Vec::new() }
    }

    /// `count` rows sized by `size(index)`. Rows added later by [`Layout::set_len`] get
    /// `estimate`.
    #[must_use]
    pub fn from_fn(count: usize, estimate: f64, mut size: impl FnMut(usize) -> f64) -> Self {
        let estimate = normalize(estimate);
        let sizes: Vec<f64> = (0..count).map(|i| normalize(size(i))).collect();
        let mut layout = Self { count, estimate, sizes, tree: Vec::new() };
        if layout.sizes.iter().all(|&s| (s - estimate).abs() < EPSILON) {
            layout.sizes = Vec::new();
        } else {
            layout.build();
        }
        layout
    }

    /// Rebuild the tree from `sizes` in `O(n)`.
    fn build(&mut self) {
        let n = self.count;
        self.tree = vec![0.0; n + 1];
        for i in 1..=n {
            self.tree[i] += self.sizes[i - 1];
            let parent = i + (i & i.wrapping_neg());
            if parent <= n {
                let carried = self.tree[i];
                self.tree[parent] += carried;
            }
        }
    }

    fn uniform(&self) -> bool {
        self.sizes.is_empty()
    }

    /// Sum of the sizes of rows `0..k`.
    fn prefix(&self, k: usize) -> f64 {
        let mut i = k;
        let mut sum = 0.0;
        while i > 0 {
            sum += self.tree[i];
            i &= i - 1;
        }
        sum
    }

    /// The largest `k` in `0..=len` whose row `k` starts at or before `position`, i.e.
    /// the row containing it (or `len` past the end).
    fn rank(&self, position: f64) -> usize {
        if self.count == 0 || position.is_nan() || position < 0.0 {
            return 0;
        }
        if self.uniform() {
            return ((position / self.estimate).floor() as usize).min(self.count);
        }
        // Binary lifting down the Fenwick tree: one descent, no prefix recomputation.
        let mut k = 0;
        let mut rest = position;
        let mut step = self.count.checked_ilog2().map_or(0, |b| 1_usize << b);
        while step > 0 {
            let next = k + step;
            if next <= self.count && self.tree[next] <= rest {
                k = next;
                rest -= self.tree[next];
            }
            step >>= 1;
        }
        k
    }

    /// Number of rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether there are no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// The size given to rows that haven't been sized.
    #[must_use]
    pub fn estimate(&self) -> f64 {
        self.estimate
    }

    /// Size of row `index`; 0 past the end.
    #[must_use]
    pub fn size(&self, index: usize) -> f64 {
        if index >= self.count {
            0.0
        } else if self.uniform() {
            self.estimate
        } else {
            self.sizes[index]
        }
    }

    /// Where row `index` starts. `offset(len())` is the total size.
    #[must_use]
    pub fn offset(&self, index: usize) -> f64 {
        let i = index.min(self.count);
        if self.uniform() { i as f64 * self.estimate } else { self.prefix(i) }
    }

    /// Size of every row together: the height of the scrolled content.
    #[must_use]
    pub fn total(&self) -> f64 {
        self.offset(self.count)
    }

    /// The row covering `position`, clamped to the first and last rows. 0 when empty.
    #[must_use]
    pub fn index_at(&self, position: f64) -> usize {
        self.rank(clean(position)).min(self.count.saturating_sub(1))
    }

    /// Rows at least partly inside a viewport of `viewport` pixels scrolled to `scroll`.
    /// At least one row while there are any.
    #[must_use]
    pub fn visible(&self, scroll: f64, viewport: f64) -> Range<usize> {
        if self.count == 0 {
            return 0..0;
        }
        let scroll = clean(scroll);
        let first = self.index_at(scroll);
        let bottom = scroll + clean(viewport);
        let k = self.rank(bottom);
        // A row starting exactly at the bottom edge isn't visible.
        let end = if k >= self.count {
            self.count
        } else if self.offset(k) < bottom {
            k + 1
        } else {
            k
        };
        first..end.max(first + 1).min(self.count)
    }

    /// [`Layout::visible`], widened by `overscan` rows each side.
    #[must_use]
    pub fn range(&self, scroll: f64, viewport: f64, overscan: usize) -> Range<usize> {
        let v = self.visible(scroll, viewport);
        v.start.saturating_sub(overscan)..v.end.saturating_add(overscan).min(self.count)
    }

    /// The scroll position that brings row `index` into view. `inset` is how much of the
    /// viewport's top is covered (a sticky header), so the row isn't hidden under it.
    /// Clamped to the scrollable range.
    #[must_use]
    pub fn scroll_to(&self, index: usize, align: Align, scroll: f64, viewport: f64, inset: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        let index = index.min(self.count - 1);
        let (scroll, viewport, inset) = (clean(scroll), clean(viewport), clean(inset));
        let top = self.offset(index);
        let size = self.size(index);
        let start = top - inset;
        let end = top + size - viewport;
        let target = match align {
            Align::Start => start,
            Align::Center => start - (viewport - inset - size) / 2.0,
            Align::Auto if start >= scroll && top + size <= scroll + viewport => scroll,
            // Taller than the viewport, or above it: show its top.
            Align::Auto if start < scroll || size > viewport - inset => start,
            Align::End | Align::Auto => end,
        };
        target.clamp(0.0, (self.total() - viewport).max(0.0))
    }

    /// Give row `index` a size. Returns whether anything moved.
    pub fn set_size(&mut self, index: usize, size: f64) -> bool {
        if index >= self.count {
            return false;
        }
        let size = normalize(size);
        let old = self.size(index);
        if (size - old).abs() < EPSILON {
            return false;
        }
        if self.uniform() {
            self.sizes = vec![self.estimate; self.count];
            self.build();
        }
        self.sizes[index] = size;
        let delta = size - old;
        let mut i = index + 1;
        while i <= self.count {
            self.tree[i] += delta;
            i += i & i.wrapping_neg();
        }
        true
    }

    /// Change the number of rows, keeping the sizes of rows that remain. New rows get
    /// the estimate.
    pub fn set_len(&mut self, count: usize) {
        if count == self.count {
            return;
        }
        self.count = count;
        if !self.uniform() {
            // O(n) rebuild per change; append in O(log n) if tails of millions churn.
            self.sizes.resize(count, self.estimate);
            self.build();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks;

    fn varied() -> Layout {
        // Sizes 10, 20, 30, 10, 20, 30, ...
        Layout::from_fn(9, 10.0, |i| [10.0, 20.0, 30.0][i % 3])
    }

    #[test]
    fn fixed_layout_is_arithmetic() {
        let l = Layout::fixed(100_000, 32.0);
        assert_eq!(l.len(), 100_000);
        assert!((l.offset(10) - 320.0).abs() < 1e-9);
        assert!((l.total() - 3_200_000.0).abs() < 1e-9);
        assert_eq!(l.index_at(319.9), 9);
        assert_eq!(l.index_at(320.0), 10);
        assert_eq!(l.index_at(1e12), 99_999);
        assert_eq!(l.visible(320.0, 64.0), 10..12);
        assert_eq!(l.visible(330.0, 64.0), 10..13);
        assert_eq!(l.range(320.0, 64.0, 3), 7..15);
        assert_eq!(l.range(0.0, 64.0, 3), 0..5);
    }

    #[test]
    fn varied_offsets_and_lookup() {
        let l = varied();
        let offsets: Vec<f64> = (0..=9).map(|i| l.offset(i)).collect();
        assert_eq!(offsets, [0.0, 10.0, 30.0, 60.0, 70.0, 90.0, 120.0, 130.0, 150.0, 180.0]);
        assert_eq!(l.index_at(0.0), 0);
        assert_eq!(l.index_at(9.99), 0);
        assert_eq!(l.index_at(10.0), 1);
        assert_eq!(l.index_at(59.0), 2);
        assert_eq!(l.index_at(60.0), 3);
        assert_eq!(l.index_at(179.0), 8);
        assert_eq!(l.index_at(500.0), 8);
        assert_eq!(l.index_at(-5.0), 0);
        assert_eq!(l.index_at(f64::NAN), 0);
        // 25..85 touches rows 1 (10..30) through 4 (70..90).
        assert_eq!(l.visible(25.0, 60.0), 1..5);
        // A row starting exactly at the bottom edge is out.
        assert_eq!(l.visible(0.0, 30.0), 0..2);
    }

    #[test]
    fn lookup_matches_a_linear_scan() {
        let l = Layout::from_fn(1000, 20.0, |i| 5.0 + (i * 37 % 41) as f64);
        let mut offsets = vec![0.0];
        for i in 0..1000 {
            offsets.push(offsets[i] + l.size(i));
        }
        for p in (0..(l.total() as usize)).step_by(7) {
            let p = p as f64 + 0.5;
            let expected = offsets.iter().rposition(|&o| o <= p).unwrap().min(999);
            assert_eq!(l.index_at(p), expected, "at {p}");
            assert!((l.offset(expected) - offsets[expected]).abs() < 1e-6);
        }
    }

    #[test]
    fn measuring_updates_offsets_after_the_row() {
        let mut l = Layout::fixed(5, 10.0);
        assert!(!l.set_size(2, 10.0), "same size changes nothing");
        assert!(l.set_size(2, 50.0));
        assert!((l.offset(2) - 20.0).abs() < 1e-9);
        assert!((l.offset(3) - 70.0).abs() < 1e-9);
        assert!((l.total() - 90.0).abs() < 1e-9);
        assert_eq!(l.index_at(69.0), 2);
        assert!(l.set_size(0, 0.0), "sizes floor at one pixel");
        assert!((l.size(0) - 1.0).abs() < 1e-9);
        assert!(!l.set_size(9, 40.0), "out of range is ignored");
    }

    #[test]
    fn set_len_keeps_measured_rows() {
        let mut l = Layout::fixed(3, 10.0);
        l.set_size(1, 30.0);
        l.set_len(5);
        assert!((l.size(1) - 30.0).abs() < 1e-9);
        assert!((l.size(4) - 10.0).abs() < 1e-9);
        assert!((l.total() - 70.0).abs() < 1e-9);
        l.set_len(1);
        assert!((l.total() - 10.0).abs() < 1e-9);
        let mut u = Layout::fixed(3, 10.0);
        u.set_len(10);
        assert!((u.total() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn uniform_sizes_from_fn_stay_compact() {
        let l = Layout::from_fn(10, 12.0, |_| 12.0);
        assert!(l.uniform());
        assert_eq!(l, Layout::fixed(10, 12.0));
    }

    #[test]
    fn empty_layout() {
        let l = Layout::fixed(0, 20.0);
        assert!(l.is_empty());
        assert_eq!(l.visible(100.0, 100.0), 0..0);
        assert_eq!(l.range(0.0, 100.0, 5), 0..0);
        assert!(l.scroll_to(3, Align::Start, 0.0, 100.0, 0.0).abs() < 1e-9);
        assert_eq!(blocks(&l, &[], 0..0, Some(2)).len(), 0);
    }

    #[test]
    fn scroll_to_aligns() {
        let l = Layout::fixed(100, 10.0);
        let at = |i, a, s| l.scroll_to(i, a, s, 50.0, 0.0);
        assert!((at(20, Align::Start, 0.0) - 200.0).abs() < 1e-9);
        assert!((at(20, Align::End, 0.0) - 160.0).abs() < 1e-9);
        assert!((at(20, Align::Center, 0.0) - 180.0).abs() < 1e-9);
        // Auto: visible stays, above goes to start, below goes to end.
        assert!((at(20, Align::Auto, 180.0) - 180.0).abs() < 1e-9);
        assert!((at(20, Align::Auto, 300.0) - 200.0).abs() < 1e-9);
        assert!((at(20, Align::Auto, 0.0) - 160.0).abs() < 1e-9);
        // Clamped to the scrollable range.
        assert!(at(0, Align::End, 0.0).abs() < 1e-9);
        assert!((at(99, Align::Start, 0.0) - 950.0).abs() < 1e-9);
        // A sticky header covering 10px keeps the row below it.
        assert!((l.scroll_to(20, Align::Start, 0.0, 50.0, 10.0) - 190.0).abs() < 1e-9);
        assert!((l.scroll_to(20, Align::Auto, 195.0, 50.0, 10.0) - 190.0).abs() < 1e-9);
    }
}
