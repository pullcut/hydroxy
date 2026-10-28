//! Horizon chart layout: each series becomes one thin row whose area is sliced into
//! equal value bands and folded down, so a tall line fits a short row and darker bands
//! read as larger values. Negative values fold up from the same baseline in a second hue.
//!
//! Paths sit in a 100×100 box per row (y down), for an SVG with
//! `preserveAspectRatio="none"`. All rows share one band size, so rows compare.

use crate::line::{self, Curve};
use crate::plot::{Key, Tick};
use crate::scale::time_ticks;

/// One row's folded areas.
#[derive(Debug, Clone, PartialEq)]
pub struct HorizonRow {
    /// One area per band above zero, lightest (band 0) first.
    pub positive: Vec<String>,
    /// One area per band below zero, folded up, lightest first.
    pub negative: Vec<String>,
}

/// Everything a horizon chart needs.
#[derive(Debug, Clone, PartialEq)]
pub struct HorizonLayout {
    /// One per series.
    pub rows: Vec<HorizonRow>,
    /// Value each band spans: the largest absolute value divided by the band count.
    pub band: f64,
    /// Whether any value is below zero (so the legend shows the second hue).
    pub has_negative: bool,
}

/// The smallest 1, 2, 2.5 or 5 × 10ⁿ at or above `x`, so band edges read as round numbers.
fn nice_ceil(x: f64) -> f64 {
    let power = 10f64.powf(x.log10().floor());
    [1.0, 2.0, 2.5, 5.0, 10.0].into_iter().map(|f| f * power).find(|&v| v >= x * (1.0 - 1e-9)).unwrap_or(10.0 * power)
}

/// Fold `series` (one value per entry of `xs`, which are 0..=1 positions) into `bands`
/// bands (clamped to 1..=4). Each row is thinned to about `buckets` columns first with
/// [`line::decimate`], so long series stay light. Non-finite values leave gaps.
#[allow(clippy::cast_precision_loss, reason = "band counts are tiny")]
#[must_use]
pub fn layout(series: &[Vec<f64>], xs: &[f64], bands: usize, buckets: usize) -> HorizonLayout {
    let bands = bands.clamp(1, 4);
    let peak = series.iter().flatten().filter(|v| v.is_finite()).fold(0.0_f64, |m, v| m.max(v.abs()));
    let band = if peak > 0.0 { nice_ceil(peak / bands as f64) } else { 1.0 };
    let has_negative = series.iter().flatten().any(|v| v.is_finite() && *v < 0.0);
    let fold = |points: &[Option<(f64, f64)>], sign: f64, b: usize| -> String {
        let lift = b as f64 * band;
        line::runs(points)
            .iter()
            .map(|run| {
                let thin = line::decimate(run, buckets);
                let drawn: Vec<(f64, f64)> = thin.iter().map(|&(x, v)| (x, 100.0 - ((v * sign - lift) / band).clamp(0.0, 1.0) * 100.0)).collect();
                line::area(&drawn, 100.0, Curve::Linear)
            })
            .collect()
    };
    let rows = series
        .iter()
        .map(|values| {
            let points: Vec<Option<(f64, f64)>> =
                xs.iter().enumerate().map(|(i, &x)| values.get(i).copied().filter(|v| v.is_finite()).map(|v| (x * 100.0, v))).collect();
            let reaches = |sign: f64, b: usize| values.iter().any(|v| v.is_finite() && v * sign > b as f64 * band);
            HorizonRow {
                positive: (0..bands).map(|b| if reaches(1.0, b) { fold(&points, 1.0, b) } else { String::new() }).collect(),
                negative: (0..bands).map(|b| if reaches(-1.0, b) { fold(&points, -1.0, b) } else { String::new() }).collect(),
            }
        })
        .collect();
    HorizonLayout { rows, band, has_negative }
}

/// Fill for band `b` of `bands`: the series hue mixed into the card, darker for higher
/// bands. Above zero is `--chart-3`, below zero `--chart-2`.
#[allow(clippy::cast_precision_loss, reason = "band counts are tiny")]
#[must_use]
pub fn shade(negative: bool, b: usize, bands: usize) -> String {
    let hue = if negative { "var(--chart-2)" } else { "var(--chart-3)" };
    let share = 30.0 + 70.0 * (b + 1) as f64 / bands.max(1) as f64;
    format!("color-mix(in oklab, {hue} {share:.0}%, var(--surface-card))")
}

/// Legend entries: the value range each shade covers, above zero and (when there are
/// negative values) below it.
#[allow(clippy::cast_precision_loss, reason = "band counts are tiny")]
pub fn legend(layout: &HorizonLayout, bands: usize, format: impl Fn(f64) -> String) -> Vec<Key> {
    let bands = bands.clamp(1, 4);
    // `+ 0.0` turns the -0 at the bottom of the negative bands into a plain 0.
    let range = |b: usize, sign: f64| format!("{} to {}", format(sign * b as f64 * layout.band + 0.0), format(sign * (b + 1) as f64 * layout.band));
    let up = (0..bands).map(|b| Key::new(range(b, 1.0), shade(false, b, bands)));
    let down = (0..bands).filter(|_| layout.has_negative).map(|b| Key::new(range(b, -1.0), shade(true, b, bands)));
    up.chain(down).collect()
}

/// X positions (0..=1) for ascending UTC `times` in seconds, and calendar ticks along them.
#[must_use]
pub fn time_axis(times: &[f64]) -> (Vec<f64>, Vec<Tick>) {
    let xs = line::x_positions(times.len(), Some(times));
    let (Some(&t0), Some(&t1)) = (times.first(), times.last()) else { return (xs, Vec::new()) };
    if t1 <= t0 {
        return (xs, Vec::new());
    }
    let ticks = time_ticks(t0, t1, 6).into_iter().map(|t| Tick { at: (t.at - t0) / (t1 - t0), text: t.label, baseline: false }).collect();
    (xs, ticks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_fold_into_bands() {
        let l = layout(&[vec![0.0, 4.0, -2.0]], &[0.0, 0.5, 1.0], 2, 100);
        assert!((l.band - 2.0).abs() < 1e-9);
        assert!((layout(&[vec![6.1]], &[0.0], 3, 10).band - 2.5).abs() < 1e-9, "bands round up to a nice size");
        assert!(l.has_negative);
        let row = &l.rows[0];
        // 4 fills band 0 to the top (y 0) and band 1 to the top as well.
        assert!(row.positive[0].contains("L50.000 0.000"));
        assert!(row.positive[1].contains("L50.000 0.000"));
        // -2 fills the first negative band only.
        assert!(row.negative[0].contains("L100.000 0.000"));
        assert_eq!(row.negative[1], "");
    }

    #[test]
    fn gaps_split_areas_and_flat_rows_stay_empty() {
        let l = layout(&[vec![1.0, f64::NAN, 1.0], vec![0.0, 0.0, 0.0]], &[0.0, 0.5, 1.0], 3, 100);
        assert_eq!(l.rows[0].positive[0].matches('M').count(), 2);
        assert!(l.rows[1].positive.iter().all(String::is_empty));
        assert!(!l.has_negative);
    }

    #[test]
    fn legend_and_axis() {
        let l = layout(&[vec![-1.0, 6.0]], &[0.0, 1.0], 3, 100);
        let keys = legend(&l, 3, |v| format!("{v}"));
        assert_eq!(keys.len(), 6);
        assert_eq!(keys[1].name, "2 to 4");
        assert_eq!(keys[3].name, "0 to -2");
        assert!(keys[2].colour.contains("100%"));
        let day = 86_400.0;
        let (xs, ticks) = time_axis(&[0.0, 5.0 * day, 10.0 * day]);
        assert_eq!(xs, vec![0.0, 0.5, 1.0]);
        assert_eq!(ticks[0].text, "1 Jan");
        assert_eq!(time_axis(&[]).1.len(), 0);
    }

    #[test]
    fn long_rows_are_thinned() {
        let xs: Vec<f64> = (0..10_000).map(|i| f64::from(i) / 9_999.0).collect();
        let values: Vec<f64> = (0..10_000).map(|i| f64::from(i % 97)).collect();
        let l = layout(&[values], &xs, 2, 200);
        assert!(l.rows[0].positive[0].matches('L').count() <= 200 * 4 + 2);
    }
}
