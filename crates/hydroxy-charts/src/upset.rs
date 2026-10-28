//! `UpSet` layout: exclusive set intersections as a bar per column on top, a dot matrix
//! of sets below (filled dots joined by a line mark the sets in that intersection) and
//! set-size bars on the left. Coordinates are px; the height follows the row count.

#![allow(
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    reason = "item counts are small; the geometry reads best in maths notation"
)]

use crate::plot::{Tip, TipRow};
use crate::scale;
use std::fmt::Write as _;

/// Members found in exactly these sets and no others.
#[derive(Debug, Clone, PartialEq)]
pub struct Intersection {
    /// Names of the sets, in any order. Repeats are ignored.
    pub sets: Vec<String>,
    /// How many members. Zero, negative and non-finite counts are ignored.
    pub count: f64,
}

impl Intersection {
    /// An intersection of `sets` holding `count` members.
    #[must_use]
    pub fn new<S: Into<String>>(sets: impl IntoIterator<Item = S>, count: f64) -> Self {
        Self { sets: sets.into_iter().map(Into::into).collect(), count }
    }
}

/// Set names in order of first appearance, and each usable intersection as a sorted,
/// de-duplicated list of indices into them with its count.
#[must_use]
pub fn index(data: &[Intersection]) -> (Vec<String>, Vec<(Vec<usize>, f64)>) {
    let mut names: Vec<String> = Vec::new();
    let mut out: Vec<(Vec<usize>, f64)> = Vec::new();
    for d in data {
        if !(d.count.is_finite() && d.count > 0.0) || d.sets.is_empty() {
            continue;
        }
        let mut ids: Vec<usize> = d
            .sets
            .iter()
            .map(|s| {
                names.iter().position(|n| n == s).unwrap_or_else(|| {
                    names.push(s.clone());
                    names.len() - 1
                })
            })
            .collect();
        ids.sort_unstable();
        ids.dedup();
        // The same combination listed twice adds up.
        if let Some(seen) = out.iter_mut().find(|(s, _)| *s == ids) {
            seen.1 += d.count;
        } else {
            out.push((ids, d.count));
        }
    }
    (names, out)
}

/// One set: a row of the matrix with its size bar.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// Set name.
    pub name: String,
    /// Members in the set, across every intersection that includes it.
    pub size: f64,
    /// Centre line of the row.
    pub y: f64,
    /// Left edge of the size bar; the bar ends at [`Layout::set_right`].
    pub bar_x: f64,
}

/// One intersection: a column with its bar and its dots.
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    /// Rows (indices into [`Layout::rows`]) of the sets in this intersection, top to bottom.
    pub sets: Vec<usize>,
    /// Members in exactly these sets.
    pub count: f64,
    /// Centre line of the column.
    pub x: f64,
    /// Top of the bar; it ends at [`Layout::baseline`].
    pub bar_y: f64,
}

/// The laid-out chart.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    /// Sets, largest first.
    pub rows: Vec<Row>,
    /// Intersections, largest first, at most the requested number.
    pub columns: Vec<Column>,
    /// Intersections left out by the column limit.
    pub hidden: usize,
    /// Intersection-size ticks as (value, y).
    pub ticks: Vec<(f64, f64)>,
    /// Bottom of the intersection bars.
    pub baseline: f64,
    /// Width of one intersection bar.
    pub bar_width: f64,
    /// Width of one matrix column.
    pub column_width: f64,
    /// Height of one matrix row.
    pub row_height: f64,
    /// Radius of a matrix dot.
    pub dot: f64,
    /// Right end of the set-size bars.
    pub set_right: f64,
    /// Left edge of the set name column.
    pub label_x: f64,
    /// Left edge of the matrix.
    pub matrix_x: f64,
    /// Full width.
    pub width: f64,
    /// Full height.
    pub height: f64,
}

/// "Email only", or the set names joined with ∩.
#[must_use]
pub fn combo_name(names: &[&str]) -> String {
    if let [one] = names { format!("{one} only") } else { names.join(" ∩ ") }
}

/// A share as a whole percentage, with "<1%" for small non-zero shares.
#[must_use]
pub fn percent(part: f64, whole: f64) -> String {
    let p = if whole > 0.0 { part / whole * 100.0 } else { 0.0 };
    if p > 0.0 && p < 1.0 { "<1%".to_string() } else { format!("{p:.0}%") }
}

impl Layout {
    /// The name of column `c`'s intersection.
    #[must_use]
    pub fn column_name(&self, c: usize) -> String {
        let names: Vec<&str> = self
            .columns
            .get(c)
            .map(|col| col.sets.iter().map(|&r| self.rows[r].name.as_str()).collect())
            .unwrap_or_default();
        combo_name(&names)
    }

    /// Tooltip for mark `mark` (columns, then rows). `total` is every member, for shares.
    #[must_use]
    pub fn tip(&self, mark: usize, total: f64, fmt: &dyn Fn(f64) -> String) -> Option<Tip> {
        let (x, y) = self.anchor(mark)?;
        let (x, y) = (x / self.width.max(1.0), 1.0 - y / self.height.max(1.0));
        if let Some(c) = self.columns.get(mark) {
            let rows = vec![
                TipRow::new(None, "Members", fmt(c.count)),
                TipRow::new(None, "Share of all", percent(c.count, total)),
                TipRow::new(None, "Sets", c.sets.len().to_string()),
            ];
            return Some(Tip { x, y, title: self.column_name(mark), rows });
        }
        let r = mark - self.columns.len();
        let row = self.rows.get(r)?;
        let shown = self.columns.iter().filter(|c| c.sets.contains(&r)).count();
        let rows = vec![
            TipRow::new(None, "Set size", fmt(row.size)),
            TipRow::new(None, "Share of all", percent(row.size, total)),
            TipRow::new(None, "Intersections", shown.to_string()),
        ];
        Some(Tip { x, y, title: row.name.clone(), rows })
    }

    /// Which columns and rows mark `mark` (columns, then rows) lights: a column lights
    /// itself and its sets, a set lights itself and every column that includes it.
    #[must_use]
    pub fn lit(&self, mark: usize) -> (Vec<bool>, Vec<bool>) {
        let mut cols = vec![false; self.columns.len()];
        let mut rows = vec![false; self.rows.len()];
        if let Some(c) = self.columns.get(mark) {
            cols[mark] = true;
            for &r in &c.sets {
                rows[r] = true;
            }
        } else if let Some(r) = mark.checked_sub(self.columns.len()).filter(|&r| r < rows.len()) {
            rows[r] = true;
            for (k, c) in self.columns.iter().enumerate() {
                cols[k] = c.sets.contains(&r);
            }
        }
        (cols, rows)
    }

    /// Where to pin a tooltip for mark `mark`, in px. Marks count the columns first, then
    /// the rows: a column anchors at the top of its bar, a row at the end of its size bar.
    #[must_use]
    pub fn anchor(&self, mark: usize) -> Option<(f64, f64)> {
        if let Some(c) = self.columns.get(mark) {
            return Some((c.x, c.bar_y));
        }
        let r = self.rows.get(mark - self.columns.len())?;
        Some((r.bar_x, r.y))
    }
}

/// SVG `d` for a `w`×`h` bar at (`x`, `y`) with only its data end rounded by `r`: the
/// top for an upright bar, the left for one that grows leftwards.
#[must_use]
pub fn bar_path(x: f64, y: f64, w: f64, h: f64, r: f64, leftwards: bool) -> String {
    let (w, h) = (w.max(0.0), h.max(0.0));
    let mut d = String::new();
    if leftwards {
        let r = r.min(h / 2.0).min(w);
        let _ = write!(
            d,
            "M{:.2} {y:.2}H{:.2}Q{x:.2} {y:.2} {x:.2} {:.2}V{:.2}Q{x:.2} {:.2} {:.2} {:.2}H{:.2}Z",
            x + w,
            x + r,
            y + r,
            y + h - r,
            y + h,
            x + r,
            y + h,
            x + w
        );
    } else {
        let r = r.min(w / 2.0).min(h);
        let _ = write!(
            d,
            "M{x:.2} {:.2}V{:.2}Q{x:.2} {y:.2} {:.2} {y:.2}H{:.2}Q{:.2} {y:.2} {:.2} {:.2}V{:.2}Z",
            y + h,
            y + r,
            x + r,
            x + w - r,
            x + w,
            x + w,
            y + r,
            y + h
        );
    }
    d
}

/// Room above the bars for their count labels.
const HEAD: f64 = 18.0;
/// Height of the intersection bar area.
const BARS: f64 = 150.0;
/// Gap between the bars and the matrix.
const GAP: f64 = 10.0;

/// Lay out `data` in a chart `width` px wide, showing at most `max_columns` intersections
/// (the largest). Rows are 26px tall.
#[must_use]
pub fn layout(data: &[Intersection], width: f64, max_columns: usize) -> Layout {
    let (names, mut combos) = index(data);
    if names.is_empty() {
        return Layout { width, ..Layout::default() };
    }
    let mut size = vec![0.0; names.len()];
    for (sets, count) in &combos {
        for &s in sets {
            size[s] += count;
        }
    }
    let mut order: Vec<usize> = (0..names.len()).collect();
    order.sort_by(|&a, &b| size[b].total_cmp(&size[a]).then(a.cmp(&b)));
    let mut row_of = vec![0; names.len()];
    for (r, &s) in order.iter().enumerate() {
        row_of[s] = r;
    }
    combos.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.len().cmp(&b.0.len())));
    let hidden = combos.len().saturating_sub(max_columns.max(1));
    combos.truncate(max_columns.max(1));

    let row_height = 26.0;
    let label_w = crate::plot::axis_width(names.iter().map(String::as_str), 64.0).min(width * 0.22);
    let set_w = (width * 0.2).clamp(60.0, 180.0);
    // Room on the left for the set-size value at the end of the longest bar.
    let value_room = 40.0;
    let set_right = value_room + set_w;
    let label_x = set_right + 10.0;
    let matrix_x = label_x + label_w;
    let column_width = ((width - matrix_x) / combos.len() as f64).max(1.0);
    let bar_width = (column_width * 0.62).min(28.0);
    let baseline = HEAD + BARS;
    let rows_top = baseline + GAP;

    let max_count = combos.iter().map(|c| c.1).fold(0.0, f64::max);
    let ticks = scale::ticks(0.0, max_count, 4);
    let top = ticks.last().copied().unwrap_or(1.0).max(f64::MIN_POSITIVE);
    let y_of = |v: f64| baseline - v / top * BARS;
    let max_size = size.iter().copied().fold(0.0, f64::max).max(f64::MIN_POSITIVE);

    let rows = order
        .iter()
        .enumerate()
        .map(|(r, &s)| Row {
            name: names[s].clone(),
            size: size[s],
            y: (r as f64 + 0.5).mul_add(row_height, rows_top),
            bar_x: set_right - size[s] / max_size * set_w,
        })
        .collect::<Vec<_>>();
    let columns = combos
        .iter()
        .enumerate()
        .map(|(c, (sets, count))| {
            let mut sets: Vec<usize> = sets.iter().map(|&s| row_of[s]).collect();
            sets.sort_unstable();
            Column { sets, count: *count, x: (c as f64 + 0.5).mul_add(column_width, matrix_x), bar_y: y_of(*count) }
        })
        .collect();
    Layout {
        height: (rows.len() as f64).mul_add(row_height, rows_top) + 4.0,
        rows,
        columns,
        hidden,
        ticks: ticks.into_iter().map(|v| (v, y_of(v))).collect(),
        baseline,
        bar_width,
        column_width,
        row_height,
        dot: (row_height * 0.24).min(column_width * 0.3),
        set_right,
        label_x,
        matrix_x,
        width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Intersection> {
        vec![
            Intersection::new(["Email"], 120.0),
            Intersection::new(["Push"], 60.0),
            Intersection::new(["Email", "Push"], 80.0),
            Intersection::new(["SMS", "Email"], 15.0),
            Intersection::new(["SMS"], 30.0),
            Intersection::new(["Push", "Email"], 5.0),
            Intersection::new(["Email"], 0.0),
        ]
    }

    #[test]
    fn index_merges_repeats_and_drops_empty_counts() {
        let (names, combos) = index(&sample());
        assert_eq!(names, ["Email", "Push", "SMS"]);
        assert_eq!(combos.len(), 5);
        assert!(combos.contains(&(vec![0, 1], 85.0)));
    }

    #[test]
    fn rows_are_sets_by_size_and_columns_by_count() {
        let l = layout(&sample(), 600.0, 10);
        let names: Vec<&str> = l.rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["Email", "Push", "SMS"]);
        assert_eq!(l.rows[0].size, 220.0);
        let counts: Vec<f64> = l.columns.iter().map(|c| c.count).collect();
        assert_eq!(counts, [120.0, 85.0, 60.0, 30.0, 15.0]);
        // Email + Push: rows 0 and 1, filled dots joined top to bottom.
        assert_eq!(l.columns[1].sets, vec![0, 1]);
        // SMS + Email sorts into row order.
        assert_eq!(l.columns[4].sets, vec![0, 2]);
    }

    #[test]
    fn bars_scale_to_the_nice_top_tick_and_stay_in_bounds() {
        let l = layout(&sample(), 600.0, 10);
        let top = l.ticks.last().expect("ticks");
        assert!(top.0 >= 120.0 && (top.1 - HEAD).abs() < 1e-9);
        assert!(l.columns.iter().all(|c| c.bar_y >= HEAD - 1e-9 && c.bar_y < l.baseline));
        assert!(l.columns.iter().all(|c| c.x > l.matrix_x && c.x < l.width));
        assert!(
            (l.rows[0].bar_x - (l.set_right - (l.set_right - 40.0))).abs() < 1e-9,
            "the largest set fills its bar area"
        );
        assert!(l.height > l.rows.last().expect("row").y);
    }

    #[test]
    fn the_column_limit_keeps_the_largest() {
        let l = layout(&sample(), 600.0, 2);
        assert_eq!(l.columns.len(), 2);
        assert_eq!(l.hidden, 3);
        assert_eq!(l.anchor(0), Some((l.columns[0].x, l.columns[0].bar_y)));
        assert_eq!(l.anchor(2), Some((l.rows[0].bar_x, l.rows[0].y)));
        assert_eq!(l.anchor(9), None);
        let fmt = |v: f64| format!("{v}");
        let tip = l.tip(1, 310.0, &fmt).expect("tip");
        assert_eq!(tip.title, "Email ∩ Push");
        assert_eq!(tip.rows[1].value, "27%");
        assert_eq!(l.tip(2, 310.0, &fmt).expect("row").title, "Email");
        assert_eq!(percent(1.0, 400.0), "<1%");
        assert_eq!(l.lit(1), (vec![false, true], vec![true, true, false]));
        assert_eq!(l.lit(3), (vec![false, true], vec![false, true, false]));
    }

    #[test]
    fn bars_round_only_their_data_end() {
        assert_eq!(
            bar_path(0.0, 10.0, 8.0, 20.0, 2.0, false),
            "M0.00 30.00V12.00Q0.00 10.00 2.00 10.00H6.00Q8.00 10.00 8.00 12.00V30.00Z"
        );
        assert_eq!(
            bar_path(10.0, 0.0, 30.0, 8.0, 2.0, true),
            "M40.00 0.00H12.00Q10.00 0.00 10.00 2.00V6.00Q10.00 8.00 12.00 8.00H40.00Z"
        );
    }

    #[test]
    fn empty_input_is_an_empty_layout() {
        assert_eq!(layout(&[], 400.0, 8).columns.len(), 0);
    }
}
