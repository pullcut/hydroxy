//! Marimekko (variable-width 100% stacks): each column's width is its share of the
//! grand total and its segments split the column by share. Positions are fractions
//! of the plot, x from the left and y from the bottom.

/// One series' piece of a column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    /// Lower edge, 0..=1.
    pub y0: f64,
    /// Upper edge, 0..=1.
    pub y1: f64,
    /// Share of the column, 0..=1.
    pub share: f64,
}

/// One category's column.
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    /// Left edge, 0..=1.
    pub x0: f64,
    /// Right edge, 0..=1.
    pub x1: f64,
    /// Sum of the column's values.
    pub total: f64,
    /// Share of the grand total, 0..=1.
    pub share: f64,
    /// One per series, stacked from the bottom in series order.
    pub segments: Vec<Segment>,
}

/// Columns for `categories` categories from `series[s][c]` values. Missing, negative and
/// non-finite values count as zero.
#[must_use]
pub fn layout(series: &[Vec<f64>], categories: usize) -> Vec<Column> {
    let v = |s: &Vec<f64>, c: usize| s.get(c).copied().filter(|v| v.is_finite()).unwrap_or(0.0).max(0.0);
    let totals: Vec<f64> = (0..categories).map(|c| series.iter().map(|s| v(s, c)).sum()).collect();
    let grand: f64 = totals.iter().sum();
    let mut x = 0.0;
    totals
        .iter()
        .enumerate()
        .map(|(c, &total)| {
            let share = if grand > 0.0 { total / grand } else { 0.0 };
            let mut y = 0.0;
            let segments = series
                .iter()
                .map(|s| {
                    let share = if total > 0.0 { v(s, c) / total } else { 0.0 };
                    let seg = Segment { y0: y, y1: y + share, share };
                    y += share;
                    seg
                })
                .collect();
            let col = Column { x0: x, x1: x + share, total, share, segments };
            x += share;
            col
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_follow_totals_and_segments_fill_each_column() {
        let cols = layout(&[vec![30.0, 10.0], vec![10.0, 50.0]], 2);
        assert_eq!((cols[0].x0, cols[0].x1, cols[1].x1), (0.0, 0.4, 1.0));
        assert_eq!(cols[0].segments[0], Segment { y0: 0.0, y1: 0.75, share: 0.75 });
        assert!((cols[1].segments[1].y1 - 1.0).abs() < 1e-9);
        assert_eq!(cols[1].total, 60.0);
    }

    #[test]
    fn missing_and_empty_columns_are_zero_width() {
        let cols = layout(&[vec![5.0], vec![]], 2);
        assert_eq!(cols[1].share, 0.0);
        assert_eq!(cols[0].segments[1].share, 0.0);
    }
}
