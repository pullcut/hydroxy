//! Skeleton: placeholder shapes shown while content loads. There is no state
//! beyond the owner's `loading` flag; this module holds the layout rule adapters
//! share, so placeholder text looks like text and server and client agree.

/// A small, stable hash (splitmix32) so widths are random-looking but deterministic.
fn mix(seed: u32, i: u32) -> u32 {
    let mut z = seed.wrapping_add(i.wrapping_mul(0x9E37_79B9));
    z = (z ^ (z >> 16)).wrapping_mul(0x85EB_CA6B);
    z = (z ^ (z >> 13)).wrapping_mul(0xC2B2_AE35);
    z ^ (z >> 16)
}

/// A value in `lo..=hi` picked by `seed` and `i`.
fn pick(seed: u32, i: u32, lo: u8, hi: u8) -> u8 {
    let span = u32::from(hi - lo) + 1;
    // `mix % span` is below `span` ≤ 256, so it fits in a u8 after adding `lo`.
    lo + u8::try_from(mix(seed, i) % span).unwrap_or(0)
}

/// Widths, in percent of the line box, for `lines` lines of placeholder text.
///
/// Ragged like real prose: full lines run 84–100%, the closing line of the
/// paragraph is shorter (38–68%), and a single line sits at 55–85%. The same
/// `seed` always gives the same widths, so server rendering and hydration match.
#[must_use]
pub fn line_widths(lines: usize, seed: u32) -> Vec<u8> {
    let last = lines.saturating_sub(1);
    (0..lines)
        .map(|i| {
            let n = u32::try_from(i).unwrap_or(u32::MAX);
            match (lines, i == last) {
                (1, _) => pick(seed, n, 55, 85),
                (_, true) => pick(seed, n, 38, 68),
                _ => pick(seed, n, 84, 100),
            }
        })
        .collect()
}

/// Widths, in percent of the cell, for a table placeholder: one ragged value per
/// cell, rows × cols, row by row. Cells run 45–90% so columns don't line up into bars.
#[must_use]
pub fn cell_widths(rows: usize, cols: usize, seed: u32) -> Vec<Vec<u8>> {
    (0..rows)
        .map(|r| (0..cols).map(|c| pick(seed ^ 0xA5A5, u32::try_from(r * cols + c).unwrap_or(0), 45, 90)).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_are_deterministic_and_seed_dependent() {
        assert_eq!(line_widths(5, 7), line_widths(5, 7));
        assert_ne!(line_widths(5, 7), line_widths(5, 8));
    }

    #[test]
    fn last_line_is_short_and_body_lines_are_long() {
        for seed in 0..50 {
            let w = line_widths(4, seed);
            assert_eq!(w.len(), 4);
            assert!(w[..3].iter().all(|&x| (84..=100).contains(&x)));
            assert!((38..=68).contains(&w[3]));
        }
    }

    #[test]
    fn single_line_sits_in_the_middle() {
        for seed in 0..50 {
            let w = line_widths(1, seed);
            assert!((55..=85).contains(&w[0]));
        }
        assert!(line_widths(0, 1).is_empty());
    }

    #[test]
    fn table_cells_have_the_right_shape() {
        let cells = cell_widths(3, 4, 2);
        assert_eq!(cells.len(), 3);
        assert!(cells.iter().all(|r| r.len() == 4 && r.iter().all(|w| (45..=90).contains(w))));
    }
}
