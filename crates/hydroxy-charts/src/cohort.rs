//! Cohort retention: one row per cohort (people who started in the same period), one
//! column per period since they started. Later cohorts have fewer periods behind them,
//! so the grid is a triangle.

/// One cohort: its name, how many started, and how many were still active each period.
#[derive(Debug, Clone, PartialEq)]
pub struct Cohort {
    /// The starting period, such as "Jan 2026".
    pub name: String,
    /// How many started.
    pub size: f64,
    /// How many were still active in period 0, 1, 2… Only the periods that have happened.
    pub retained: Vec<f64>,
}

impl Cohort {
    /// A cohort of `size` with `retained` counts per period.
    #[must_use]
    pub fn new(name: impl Into<String>, size: f64, retained: impl IntoIterator<Item = f64>) -> Self {
        Self { name: name.into(), size, retained: retained.into_iter().collect() }
    }
}

/// One cell of the triangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CohortCell {
    /// Cohort index, top to bottom.
    pub row: usize,
    /// Periods since the cohort started.
    pub col: usize,
    /// How many were still active.
    pub count: f64,
    /// `count` as a share of the cohort, 0..=1.
    pub rate: f64,
    /// Position on the colour ramp, 0..=1.
    pub level: f64,
}

/// The triangle, row by row.
#[derive(Debug, Clone, PartialEq)]
pub struct CohortLayout {
    /// Every cell that has happened, row-major. Missing or non-finite counts are skipped.
    pub cells: Vec<CohortCell>,
    /// Cohort count.
    pub rows: usize,
    /// Period count: the longest cohort.
    pub cols: usize,
}

impl CohortLayout {
    /// The cell at (`row`, `col`), if that period has happened.
    #[must_use]
    pub fn at(&self, row: usize, col: usize) -> Option<&CohortCell> {
        self.cells.iter().find(|c| c.row == row && c.col == col)
    }

    /// How many periods cohort `row` has.
    #[must_use]
    pub fn periods(&self, row: usize) -> usize {
        self.cells.iter().filter(|c| c.row == row).map(|c| c.col + 1).max().unwrap_or(0)
    }
}

/// Lay out `cohorts`. The colour ramp runs from zero to the highest rate after period 0,
/// since period 0 is usually everyone and would wash every later period out.
#[must_use]
pub fn layout(cohorts: &[Cohort]) -> CohortLayout {
    let rate = |c: &Cohort, v: f64| if c.size > 0.0 { (v / c.size).clamp(0.0, 1.0) } else { 0.0 };
    let finite = |v: &&f64| v.is_finite();
    let later = cohorts
        .iter()
        .flat_map(|c| c.retained.iter().skip(1).filter(finite).map(move |&v| rate(c, v)))
        .fold(0.0, f64::max);
    let top = if later > 0.0 { later } else { 1.0 };
    let cells = cohorts
        .iter()
        .enumerate()
        .flat_map(|(row, c)| {
            c.retained.iter().enumerate().filter(|(_, v)| v.is_finite()).map(move |(col, &count)| {
                let rate = rate(c, count);
                CohortCell { row, col, count, rate, level: (rate / top).min(1.0) }
            })
        })
        .collect();
    CohortLayout { cells, rows: cohorts.len(), cols: cohorts.iter().map(|c| c.retained.len()).max().unwrap_or(0) }
}

/// A rate as a percentage: whole numbers from 10%, one decimal place below.
#[must_use]
pub fn percent(rate: f64) -> String {
    let p = rate * 100.0;
    if p >= 9.95 || p == 0.0 { format!("{p:.0}%") } else { format!("{p:.1}%") }
}

/// Arrow-key movement over the triangle: like [`crate::nav::grid_key`], but a move into a
/// period a cohort hasn't reached lands on its last cell.
#[must_use]
pub fn key(layout: &CohortLayout, key: &str, current: Option<(usize, usize)>) -> Option<Option<(usize, usize)>> {
    let next = crate::nav::grid_key(key, current, layout.rows, layout.cols)?;
    Some(next.map(|(r, c)| (r, c.min(layout.periods(r).saturating_sub(1)))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cohorts() -> Vec<Cohort> {
        vec![
            Cohort::new("Jan", 200.0, [200.0, 80.0, 50.0]),
            Cohort::new("Feb", 100.0, [100.0, 30.0]),
            Cohort::new("Mar", 50.0, [50.0]),
        ]
    }

    #[test]
    fn rates_and_levels() {
        let l = layout(&cohorts());
        assert_eq!((l.rows, l.cols, l.cells.len()), (3, 3, 6));
        let c = l.at(0, 1).copied().unwrap_or(CohortCell { row: 0, col: 0, count: 0.0, rate: 0.0, level: 0.0 });
        assert!((c.rate - 0.4).abs() < 1e-12);
        // The highest later rate (40%) tops the ramp; period 0 clamps to it.
        assert!((c.level - 1.0).abs() < 1e-12);
        assert!(l.at(0, 0).is_some_and(|c| (c.level - 1.0).abs() < 1e-12));
        assert!(l.at(1, 1).is_some_and(|c| (c.level - 0.75).abs() < 1e-12));
        assert!(l.at(2, 1).is_none());
        assert_eq!((l.periods(0), l.periods(2)), (3, 1));
    }

    #[test]
    fn keys_stay_inside_the_triangle() {
        let l = layout(&cohorts());
        assert_eq!(key(&l, "ArrowDown", Some((1, 1))), Some(Some((2, 0))));
        assert_eq!(key(&l, "End", Some((1, 0))), Some(Some((1, 1))));
        assert_eq!(key(&l, "ArrowRight", None), Some(Some((0, 0))));
    }

    #[test]
    fn percentages_and_empty_cohorts() {
        assert_eq!(
            (percent(1.0), percent(0.254), percent(0.031), percent(0.0)),
            ("100%".into(), "25%".into(), "3.1%".into(), "0%".into())
        );
        let l = layout(&[Cohort::new("Empty", 0.0, [0.0, f64::NAN])]);
        assert_eq!(l.cells.len(), 1);
        assert_eq!(l.cells[0].rate, 0.0);
    }
}
