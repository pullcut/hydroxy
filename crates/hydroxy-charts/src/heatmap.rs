//! Heatmap layout: a grid of values mapped to a 0..=1 intensity. Adapters turn the
//! intensity into a colour mixed from tokens, so no colour lives here.

/// One cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    /// Row index, top to bottom.
    pub row: usize,
    /// Column index, left to right.
    pub col: usize,
    /// The value, or `None` for a missing or non-finite entry.
    pub value: Option<f64>,
    /// Position on the colour scale, 0..=1. Zero for missing cells.
    pub level: f64,
}

/// A grid with its value range.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatLayout {
    /// Row-major cells, `rows * cols` of them.
    pub cells: Vec<Cell>,
    /// Row count.
    pub rows: usize,
    /// Column count: the longest row.
    pub cols: usize,
    /// Smallest finite value (0 when there is none).
    pub min: f64,
    /// Largest finite value (0 when there is none).
    pub max: f64,
}

/// Lay out `values` (one `Vec` per row). Short rows are padded with missing cells.
#[must_use]
pub fn layout(values: &[Vec<f64>]) -> HeatLayout {
    let rows = values.len();
    let cols = values.iter().map(Vec::len).max().unwrap_or(0);
    let finite = |v: f64| Some(v).filter(|v| v.is_finite());
    let all = values.iter().flatten().copied().filter_map(finite);
    let (min, max) = all.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| (lo.min(v), hi.max(v)));
    let (min, max) = if min <= max { (min, max) } else { (0.0, 0.0) };
    let span = max - min;
    let cells = (0..rows)
        .flat_map(|row| (0..cols).map(move |col| (row, col)))
        .map(|(row, col)| {
            let value = values[row].get(col).copied().and_then(finite);
            // A flat grid sits at the top of the scale rather than the bottom.
            let level = value.map_or(0.0, |v| if span > 0.0 { (v - min) / span } else { 1.0 });
            Cell { row, col, value, level }
        })
        .collect();
    HeatLayout { cells, rows, cols, min, max }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_span_the_range() {
        let l = layout(&[vec![0.0, 5.0], vec![10.0, f64::NAN]]);
        assert_eq!((l.rows, l.cols), (2, 2));
        assert_eq!((l.min, l.max), (0.0, 10.0));
        assert!((l.cells[1].level - 0.5).abs() < 1e-9);
        assert_eq!(l.cells[3].value, None);
    }

    #[test]
    fn ragged_rows_pad_with_missing_cells() {
        let l = layout(&[vec![1.0, 2.0, 3.0], vec![4.0]]);
        assert_eq!(l.cells.len(), 6);
        assert_eq!(l.cells[4].value, None);
    }

    #[test]
    fn flat_and_empty_grids_do_not_divide_by_zero() {
        assert!((layout(&[vec![3.0, 3.0]]).cells[0].level - 1.0).abs() < 1e-9);
        assert_eq!(layout(&[]).cells.len(), 0);
    }
}
