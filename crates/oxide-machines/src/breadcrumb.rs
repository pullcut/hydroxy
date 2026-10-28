//! Breadcrumb: a trail of links to the current page. The one rule is how the
//! trail shrinks: when it doesn't fit, middle items fold into a "…" menu. The
//! first item and the last two always stay.

use std::ops::Range;

/// Which items to fold into the "…" item so the trail fits in `available` pixels.
///
/// - `widths[i]`: item `i`'s full width, separator included.
/// - `ellipsis`: the width of the "…" item with its separator.
///
/// Folds from the second item onward (nearest the root first), one more at a
/// time, until the rest fits. Never folds the first item or the last two, so it
/// may still overflow when even that is too wide. An empty range means nothing folds.
#[must_use]
pub fn collapse(widths: &[f64], available: f64, ellipsis: f64) -> Range<usize> {
    let len = widths.len();
    let total: f64 = widths.iter().sum();
    if len <= 3 || total <= available {
        return 0..0;
    }
    let last_foldable = len - 2;
    let mut folded = 0.0;
    for end in 2..=last_foldable {
        folded += widths[end - 1];
        if total - folded + ellipsis <= available {
            return 1..end;
        }
    }
    1..last_foldable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_fits_folds_nothing() {
        assert_eq!(collapse(&[50.0, 50.0, 50.0, 50.0], 200.0, 20.0), 0..0);
    }

    #[test]
    fn short_trails_never_fold() {
        assert_eq!(collapse(&[100.0, 100.0, 100.0], 10.0, 20.0), 0..0);
    }

    #[test]
    fn folds_from_the_root_end_until_it_fits() {
        // Total 300. Folding item 1 gives 300 - 60 + 20 = 260.
        let widths = [40.0, 60.0, 60.0, 60.0, 40.0, 40.0];
        assert_eq!(collapse(&widths, 260.0, 20.0), 1..2);
        // Folding items 1 and 2 gives 200.
        assert_eq!(collapse(&widths, 200.0, 20.0), 1..3);
    }

    #[test]
    fn keeps_first_and_last_two_even_when_too_wide() {
        let widths = [40.0, 60.0, 60.0, 60.0, 40.0, 40.0];
        assert_eq!(collapse(&widths, 10.0, 20.0), 1..4);
    }
}
