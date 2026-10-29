//! Chart tables: a data table whose cells carry small visuals. Bar cells share one scale
//! per column, so lengths compare down the column; this module works out those scales,
//! the delta direction and the sort order.

use std::cmp::Ordering;

/// What a column shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ColumnKind {
    /// Plain text.
    #[default]
    Text,
    /// A right-aligned number.
    Number,
    /// A number with a bar, on a scale shared by the whole column.
    Bar,
    /// A signed change with an up or down arrow.
    Delta,
    /// A small trend line from a series of values.
    Sparkline,
}

impl ColumnKind {
    /// The `data-kind` value.
    #[must_use]
    pub const fn attr(self) -> &'static str {
        match self {
            ColumnKind::Text => "text",
            ColumnKind::Number => "number",
            ColumnKind::Bar => "bar",
            ColumnKind::Delta => "delta",
            ColumnKind::Sparkline => "sparkline",
        }
    }

    /// Right-aligned with tabular figures.
    #[must_use]
    pub const fn is_numeric(self) -> bool {
        matches!(self, ColumnKind::Number | ColumnKind::Bar | ColumnKind::Delta)
    }
}

/// One column after the row label.
#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    /// Header text.
    pub header: String,
    /// What the cells show.
    pub kind: ColumnKind,
    /// Text after each number, such as "%" or " ms". Empty by default.
    pub unit: String,
    /// Decimal places for numbers. Default: whole numbers, else one place.
    pub decimals: Option<usize>,
}

impl TableColumn {
    fn of(header: impl Into<String>, kind: ColumnKind) -> Self {
        Self { header: header.into(), kind, unit: String::new(), decimals: None }
    }

    /// A text column.
    #[must_use]
    pub fn text(header: impl Into<String>) -> Self {
        Self::of(header, ColumnKind::Text)
    }

    /// A number column.
    #[must_use]
    pub fn number(header: impl Into<String>) -> Self {
        Self::of(header, ColumnKind::Number)
    }

    /// A number column with inline bars.
    #[must_use]
    pub fn bar(header: impl Into<String>) -> Self {
        Self::of(header, ColumnKind::Bar)
    }

    /// A signed change column.
    #[must_use]
    pub fn delta(header: impl Into<String>) -> Self {
        Self::of(header, ColumnKind::Delta)
    }

    /// A sparkline column.
    #[must_use]
    pub fn sparkline(header: impl Into<String>) -> Self {
        Self::of(header, ColumnKind::Sparkline)
    }

    /// Text after each number, such as "%".
    #[must_use]
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }

    /// A fixed number of decimal places.
    #[must_use]
    pub fn decimals(mut self, places: usize) -> Self {
        self.decimals = Some(places);
        self
    }

    /// A number in this column's style. Deltas carry their sign; non-finite values read "–".
    #[must_use]
    pub fn format(&self, v: f64) -> String {
        if !v.is_finite() {
            return "–".into();
        }
        let digits = self.decimals.map_or_else(|| crate::format_value(v.abs()), |d| format!("{:.d$}", v.abs()));
        let digits = group_thousands(&digits);
        let sign = match (self.kind, v.partial_cmp(&0.0)) {
            (ColumnKind::Delta, Some(Ordering::Greater)) => "+",
            (_, Some(Ordering::Less)) if digits.trim_start_matches(['0', '.']).is_empty() => "",
            (_, Some(Ordering::Less)) => "−",
            _ => "",
        };
        format!("{sign}{digits}{}", self.unit)
    }
}

/// Thousands separators on the whole part of a plain decimal string.
fn group_thousands(s: &str) -> String {
    let (int, frac) = s.split_once('.').map_or((s, None), |(i, f)| (i, Some(f)));
    let mut out = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if let Some(f) = frac {
        out.push('.');
        out.push_str(f);
    }
    out
}

/// One cell's data.
#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    /// Text, for text columns.
    Text(String),
    /// A number, for number, bar and delta columns.
    Number(f64),
    /// Values oldest first, for sparkline columns.
    Series(Vec<f64>),
}

impl Cell {
    /// The number, if this is a finite number cell.
    #[must_use]
    pub fn number(&self) -> Option<f64> {
        match self {
            Cell::Number(v) if v.is_finite() => Some(*v),
            _ => None,
        }
    }

    /// The value a sort compares: the number, or a series' latest finite value.
    #[must_use]
    pub fn key(&self) -> Option<f64> {
        match self {
            Cell::Series(v) => v.iter().rev().copied().find(|v| v.is_finite()),
            _ => self.number(),
        }
    }
}

/// One row: its label (the first column) and a cell per column.
#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    /// The row's name, shown in the first column.
    pub label: String,
    /// One cell per [`TableColumn`], in order.
    pub cells: Vec<Cell>,
}

impl TableRow {
    /// A row.
    #[must_use]
    pub fn new(label: impl Into<String>, cells: Vec<Cell>) -> Self {
        Self { label: label.into(), cells }
    }
}

/// Order two cells for sorting: numbers (and series by their latest value) numerically,
/// text case-insensitively. Missing values sort last.
#[must_use]
pub fn compare(a: &Cell, b: &Cell) -> Ordering {
    if let (Cell::Text(a), Cell::Text(b)) = (a, b) {
        return a.to_lowercase().cmp(&b.to_lowercase());
    }
    match (a.key(), b.key()) {
        (Some(a), Some(b)) => a.total_cmp(&b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// A bar column's scale: the range from zero to the furthest value either side.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarScale {
    /// Lowest value shown (zero or below).
    pub lo: f64,
    /// Highest value shown (zero or above).
    pub hi: f64,
}

impl BarScale {
    /// The scale for column `col` of `rows`.
    #[must_use]
    pub fn of(rows: &[TableRow], col: usize) -> Self {
        let values = rows.iter().filter_map(|r| r.cells.get(col).and_then(Cell::number));
        let (lo, hi) = values.fold((0.0_f64, 0.0_f64), |(lo, hi), v| (lo.min(v), hi.max(v)));
        Self { lo, hi }
    }

    /// Where a bar for `v` starts and how wide it is, both as fractions of the track.
    /// Bars run from zero, so with negatives in the column zero sits part way across.
    #[must_use]
    pub fn span(self, v: f64) -> (f64, f64) {
        let width = self.hi - self.lo;
        if width <= 0.0 || !v.is_finite() {
            return (0.0, 0.0);
        }
        let zero = -self.lo / width;
        let at = (v - self.lo) / width;
        (zero.min(at), (at - zero).abs())
    }
}

/// Which way a delta points: "up", "down" or "flat".
#[must_use]
pub fn direction(v: f64) -> &'static str {
    if v > 0.0 {
        "up"
    } else if v < 0.0 {
        "down"
    } else {
        "flat"
    }
}

/// Order two rows by column `col`: [`compare`] when both have a cell there, otherwise the row
/// with the cell first (a row missing the cell sorts last).
#[must_use]
pub fn compare_rows(a: &TableRow, b: &TableRow, col: usize) -> Ordering {
    match (a.cells.get(col), b.cells.get(col)) {
        (Some(a), Some(b)) => compare(a, b),
        (a, b) => a.is_none().cmp(&b.is_none()),
    }
}

/// Order two rows by label, case-insensitively: the first column's sort.
#[must_use]
pub fn compare_labels(a: &TableRow, b: &TableRow) -> Ordering {
    a.label.to_lowercase().cmp(&b.label.to_lowercase())
}

/// The glyph for a [`direction`]: an up or down triangle, or a dot when flat.
#[must_use]
pub fn glyph(direction: &str) -> &'static str {
    match direction {
        "up" => "▲",
        "down" => "▼",
        _ => "•",
    }
}

/// The table as text rows for CSV export, header first; sparklines list their values.
#[must_use]
pub fn export(row_header: &str, columns: &[TableColumn], rows: &[TableRow]) -> Vec<Vec<String>> {
    let head = std::iter::once(row_header.to_string()).chain(columns.iter().map(|c| c.header.clone())).collect();
    let body = rows.iter().map(|r| {
        std::iter::once(r.label.clone())
            .chain(columns.iter().enumerate().map(|(i, c)| text(c, r.cells.get(i))))
            .collect()
    });
    std::iter::once(head).chain(body).collect()
}

/// One cell as text, as the table and export show it.
#[must_use]
pub fn text(column: &TableColumn, cell: Option<&Cell>) -> String {
    match cell {
        Some(Cell::Text(t)) => t.clone(),
        Some(Cell::Number(v)) => column.format(*v),
        Some(Cell::Series(v)) => v.iter().map(|&x| column.format(x)).collect::<Vec<_>>().join(" "),
        None => "–".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_share_a_column_scale_from_zero() {
        let rows = vec![TableRow::new("a", vec![Cell::Number(20.0)]), TableRow::new("b", vec![Cell::Number(80.0)])];
        let s = BarScale::of(&rows, 0);
        assert_eq!(s, BarScale { lo: 0.0, hi: 80.0 });
        assert_eq!(s.span(20.0), (0.0, 0.25));
        let mixed = BarScale { lo: -50.0, hi: 50.0 };
        assert_eq!(mixed.span(-25.0), (0.25, 0.25));
        assert_eq!(mixed.span(50.0), (0.5, 0.5));
        assert_eq!(BarScale { lo: 0.0, hi: 0.0 }.span(3.0), (0.0, 0.0));
    }

    #[test]
    fn sorting_puts_missing_last_and_reads_series_by_latest() {
        assert_eq!(compare(&Cell::Number(1.0), &Cell::Number(2.0)), Ordering::Less);
        assert_eq!(compare(&Cell::Number(f64::NAN), &Cell::Number(2.0)), Ordering::Greater);
        assert_eq!(compare(&Cell::Series(vec![9.0, 1.0]), &Cell::Series(vec![0.0, 2.0])), Ordering::Less);
        assert_eq!(compare(&Cell::Text("b".into()), &Cell::Text("A".into())), Ordering::Greater);
    }

    #[test]
    fn formats_units_signs_and_thousands() {
        assert_eq!(TableColumn::delta("Δ").unit("%").decimals(1).format(3.26), "+3.3%");
        assert_eq!(TableColumn::delta("Δ").format(-4.0), "−4");
        assert_eq!(TableColumn::number("n").format(1_234_567.0), "1,234,567");
        assert_eq!(TableColumn::number("n").decimals(2).format(-0.001), "0.00");
        assert_eq!(TableColumn::number("n").format(f64::NAN), "–");
        assert_eq!((direction(2.0), direction(-1.0), direction(0.0)), ("up", "down", "flat"));
    }

    #[test]
    fn export_lists_every_cell() {
        let cols = vec![TableColumn::number("Users"), TableColumn::sparkline("Trend")];
        let rows = vec![TableRow::new("Web", vec![Cell::Number(1200.0), Cell::Series(vec![1.0, 2.0])])];
        assert_eq!(
            export("Channel", &cols, &rows),
            vec![vec!["Channel", "Users", "Trend"], vec!["Web", "1,200", "1 2"]]
        );
    }

    #[test]
    fn row_comparators_sort_by_column_and_label() {
        let a = TableRow::new("alpha", vec![Cell::Number(2.0)]);
        let b = TableRow::new("Beta", vec![Cell::Number(1.0)]);
        let short = TableRow::new("c", vec![]);
        assert_eq!(compare_rows(&a, &b, 0), Ordering::Greater);
        assert_eq!(compare_rows(&a, &short, 0), Ordering::Less);
        assert_eq!(compare_rows(&short, &a, 0), Ordering::Greater);
        assert_eq!(compare_rows(&short, &short, 0), Ordering::Equal);
        assert_eq!(compare_labels(&a, &b), Ordering::Less);
    }

    #[test]
    fn glyphs_follow_direction() {
        assert_eq!((glyph("up"), glyph("down"), glyph("flat")), ("▲", "▼", "•"));
        assert_eq!(glyph(direction(3.0)), "▲");
    }
}
