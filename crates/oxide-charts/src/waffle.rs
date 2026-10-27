//! Waffle grids: a fixed grid of unit cells (10×10 by default), each given to a
//! category in proportion to its value. Counts use largest-remainder rounding so
//! they add up exactly; cells fill row by row from the top left.

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "cell counts are small and non-negative")]
/// Cells per category out of `cells`. Against `total` (default: the sum of the values),
/// so a total larger than the sum leaves some cells unassigned.
#[must_use]
pub fn counts(values: &[f64], total: Option<f64>, cells: usize) -> Vec<usize> {
    let clean: Vec<f64> = values.iter().map(|v| if v.is_finite() { v.max(0.0) } else { 0.0 }).collect();
    let sum: f64 = clean.iter().sum();
    let total = total.filter(|t| t.is_finite() && *t > 0.0).unwrap_or(sum).max(sum);
    if total <= 0.0 {
        return vec![0; clean.len()];
    }
    let quotas: Vec<f64> = clean.iter().map(|v| v / total * cells as f64).collect();
    let mut out: Vec<usize> = quotas.iter().map(|q| q.floor() as usize).collect();
    let target = (sum / total * cells as f64).round() as usize;
    let mut order: Vec<usize> = (0..clean.len()).collect();
    order.sort_by(|&a, &b| (quotas[b] - quotas[b].floor()).total_cmp(&(quotas[a] - quotas[a].floor())).then(a.cmp(&b)));
    let short = target.saturating_sub(out.iter().sum());
    for &i in order.iter().take(short) {
        out[i] += 1;
    }
    out
}

/// The category for each of `cells` cells, in reading order. `None` is unassigned.
#[must_use]
pub fn cells(counts: &[usize], cells: usize) -> Vec<Option<usize>> {
    let mut out: Vec<Option<usize>> = counts.iter().enumerate().flat_map(|(i, &n)| std::iter::repeat_n(Some(i), n)).collect();
    out.resize(cells, None);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_add_up_exactly() {
        let c = counts(&[1.0, 1.0, 1.0], None, 100);
        assert_eq!(c.iter().sum::<usize>(), 100);
        assert_eq!(c, [34, 33, 33]);
        assert_eq!(counts(&[62.0, 38.0], None, 100), [62, 38]);
    }

    #[test]
    fn a_larger_total_leaves_cells_empty() {
        let c = counts(&[30.0, 20.0], Some(200.0), 100);
        assert_eq!(c, [15, 10]);
        let grid = cells(&c, 100);
        assert_eq!((grid[0], grid[15], grid[25]), (Some(0), Some(1), None));
        assert_eq!(counts(&[0.0], None, 100), [0]);
    }
}
