//! Diverging stacked bars for survey (Likert) answers: each row's answers become shares
//! of the row, stacked outwards from a centre line. Levels below the middle run left,
//! levels above it run right, and with an odd number of levels the middle (neutral) one
//! straddles the centre. Shares sit on a symmetric percentage axis.

use crate::line::Series;
use crate::plot::ValueAxis;
use crate::scale::ScaleKind;

/// The neutral level of `levels` answer levels: the middle one when there is an odd number.
#[must_use]
pub fn neutral(levels: usize) -> Option<usize> {
    (levels % 2 == 1).then_some(levels / 2)
}

/// The colour of answer level `level` of `levels`: the negative side in `--chart-2`, the
/// positive side in `--chart-3`, both deepening away from the centre, and neutral in
/// `--chart-muted`. Lighter shades mix towards the card surface, so they hold in dark themes.
#[allow(clippy::cast_precision_loss, reason = "level counts are tiny")]
#[must_use]
pub fn level_colour(level: usize, levels: usize) -> String {
    if neutral(levels) == Some(level) {
        return "var(--chart-muted)".into();
    }
    let half = levels / 2;
    let (hue, depth) = if level < half { (2, half - level) } else { (3, level + half + 1 - levels) };
    let strength = if half <= 1 { 100.0 } else { 45.0 + 55.0 * (depth - 1) as f64 / (half - 1) as f64 };
    if strength >= 100.0 {
        format!("var(--chart-{hue})")
    } else {
        format!("color-mix(in oklab, var(--chart-{hue}) {strength:.0}%, var(--surface-card))")
    }
}

/// One answer level's segment in a row.
#[derive(Debug, Clone, PartialEq)]
pub struct DivergingSegment {
    /// The answer level, an index into the levels.
    pub level: usize,
    /// The share of the row's answers, 0..=1.
    pub share: f64,
    /// Left end along the value axis, 0..=1.
    pub a: f64,
    /// Right end along the value axis, 0..=1.
    pub b: f64,
    /// Where the centre line falls across the segment, as a fraction of its width (can
    /// fall outside 0..=1). Segments grow from here, so a row unrolls from the centre.
    pub origin: f64,
    /// `"start"`, `"end"` or `"both"` for a row's outermost segments, else empty: only the
    /// outer ends are rounded.
    pub edge: &'static str,
}

/// One row of a diverging chart.
#[derive(Debug, Clone, PartialEq)]
pub struct DivergingRow {
    /// Segments with a share above zero, left to right.
    pub segments: Vec<DivergingSegment>,
    /// Total share of the levels below neutral.
    pub negative: f64,
    /// Total share of the levels above neutral.
    pub positive: f64,
    /// Left end of the row's bar along the value axis, 0..=1.
    pub start: f64,
    /// Right end of the row's bar along the value axis, 0..=1.
    pub end: f64,
}

/// Everything a diverging bar chart needs to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct DivergingLayout {
    /// One per row.
    pub rows: Vec<DivergingRow>,
    /// A symmetric axis of shares (-1..=1), zero in the middle.
    pub axis: ValueAxis,
}

/// Lay out `rows` rows of answers. `levels` holds one series per answer level, ordered
/// from most negative to most positive, each with one count (or share) per row.
#[must_use]
pub fn layout(levels: &[Series], rows: usize) -> DivergingLayout {
    let count = levels.len();
    let half = count / 2;
    let mid = neutral(count);
    let shares: Vec<Vec<f64>> = (0..rows)
        .map(|r| {
            let raw: Vec<f64> = levels.iter().map(|s| s.values.get(r).copied().filter(|v| v.is_finite()).unwrap_or(0.0).max(0.0)).collect();
            let total: f64 = raw.iter().sum();
            raw.iter().map(|v| if total > 0.0 { v / total } else { 0.0 }).collect()
        })
        .collect();
    let centre_half = |s: &[f64]| mid.map_or(0.0, |m| s[m] / 2.0);
    let negative = |s: &[f64]| s[..half].iter().sum::<f64>();
    let positive = |s: &[f64]| s[count - half..].iter().sum::<f64>();
    let reach = shares.iter().map(|s| negative(s).max(positive(s)) + centre_half(s)).fold(0.0, f64::max);
    // A little room past the longest side keeps the end totals inside the plot.
    let reach = if reach > 0.0 { (reach + 0.08).min(1.0) } else { 0.5 };
    let first = ValueAxis::over(ScaleKind::Linear, -reach, reach);
    let edge = first.scale.domain.0.abs().max(first.scale.domain.1.abs());
    let axis = ValueAxis::over(ScaleKind::Linear, -edge, edge);
    let rows = shares
        .iter()
        .map(|s| {
            let h = centre_half(s);
            // (level, low, high) in shares, built outwards from the centre.
            let mut spans: Vec<(usize, f64, f64)> = Vec::with_capacity(count);
            let mut x = -h;
            for k in (0..half).rev() {
                spans.push((k, x - s[k], x));
                x -= s[k];
            }
            spans.reverse();
            if let Some(m) = mid {
                spans.push((m, -h, h));
            }
            let mut x = h;
            for (k, share) in s.iter().enumerate().skip(count - half) {
                spans.push((k, x, x + share));
                x += share;
            }
            spans.retain(|&(k, _, _)| s[k] > 0.0);
            let centre = axis.at(0.0);
            let last = spans.len().saturating_sub(1);
            let segments = spans
                .iter()
                .enumerate()
                .map(|(i, &(level, lo, hi))| {
                    let (a, b) = (axis.at(lo), axis.at(hi));
                    let edge = match (i == 0, i == last) {
                        (true, true) => "both",
                        (true, false) => "start",
                        (false, true) => "end",
                        _ => "",
                    };
                    DivergingSegment { level, share: s[level], a, b, origin: if b > a { (centre - a) / (b - a) } else { 0.5 }, edge }
                })
                .collect();
            let (neg, pos) = (negative(s), positive(s));
            DivergingRow { segments, negative: neg, positive: pos, start: axis.at(-neg - h), end: axis.at(pos + h) }
        })
        .collect();
    DivergingLayout { rows, axis }
}

/// The segment of `row` under value position `p`, or the nearest one.
#[must_use]
pub fn segment_at(row: &DivergingRow, p: f64) -> Option<usize> {
    row.segments
        .iter()
        .enumerate()
        .map(|(i, s)| (i, if p < s.a { s.a - p } else if p > s.b { p - s.b } else { 0.0 }))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn likert() -> Vec<Series> {
        vec![
            Series::new("Strongly disagree", [10.0, 0.0]),
            Series::new("Disagree", [20.0, 0.0]),
            Series::new("Neutral", [20.0, 50.0]),
            Series::new("Agree", [30.0, 50.0]),
            Series::new("Strongly agree", [20.0, 0.0]),
        ]
    }

    #[test]
    fn neutral_straddles_the_centre() {
        let l = layout(&likert(), 2);
        let row = &l.rows[0];
        let n = row.segments.iter().find(|s| s.level == 2).expect("neutral");
        assert!((n.a + n.b - 1.0).abs() < 1e-9, "centred on the middle of the axis");
        assert!((n.origin - 0.5).abs() < 1e-9);
        assert!((row.negative - 0.3).abs() < 1e-9 && (row.positive - 0.5).abs() < 1e-9);
        let levels: Vec<usize> = row.segments.iter().map(|s| s.level).collect();
        assert_eq!(levels, [0, 1, 2, 3, 4], "left to right");
        assert_eq!((row.segments[0].edge, row.segments[4].edge, row.segments[2].edge), ("start", "end", ""));
        // Left segments grow from their right edge or beyond, right ones from their left.
        assert!(row.segments[0].origin > 1.0 && row.segments[4].origin < 0.0);
    }

    #[test]
    fn axis_is_symmetric_and_empty_levels_drop() {
        let l = layout(&likert(), 2);
        let (lo, hi) = l.axis.scale.domain;
        assert!((lo + hi).abs() < 1e-12 && hi >= 0.6);
        assert_eq!(l.rows[1].segments.len(), 2);
        assert_eq!(l.rows[1].segments[0].edge, "start");
        assert_eq!(segment_at(&l.rows[0], 0.0), Some(0));
        assert_eq!(segment_at(&l.rows[0], 0.5), Some(2));
        let empty = layout(&likert(), 0);
        assert!(empty.rows.is_empty() && empty.axis.scale.domain.1 > 0.0);
    }

    #[test]
    fn colours_diverge_around_grey() {
        assert_eq!(level_colour(2, 5), "var(--chart-muted)");
        assert_eq!(level_colour(0, 5), "var(--chart-2)");
        assert_eq!(level_colour(4, 5), "var(--chart-3)");
        assert!(level_colour(1, 5).contains("--chart-2) 45%"));
        assert!(level_colour(3, 5).contains("--chart-3) 45%"));
        assert_eq!(level_colour(0, 4), "var(--chart-2)");
        assert!(level_colour(2, 4).contains("--chart-3) 45%"));
        assert_eq!(neutral(4), None);
    }
}
