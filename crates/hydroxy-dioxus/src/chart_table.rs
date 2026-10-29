//! `ChartTable`: a sortable data table whose cells carry small visuals: bars on a scale
//! shared down each column, sparklines and up/down deltas. Built on [`DataTable`];
//! scales come from `hydroxy_charts::chart_table`.

use dioxus::prelude::*;
use hydroxy_charts::chart_table::{self as ct, BarScale, ColumnKind};

pub use hydroxy_charts::chart_table::{Cell, TableColumn, TableRow};

use crate::chart_frame::use_export;
use crate::data_table::{Column, DataTable};
use crate::sparkline::Sparkline;

/// A row with its original position, which staggers its entrance.
type Indexed = (usize, TableRow);

fn cell_view(col: usize, column: &TableColumn, (i, row): &Indexed, scale: Option<BarScale>) -> Element {
    let cell = row.cells.get(col);
    match (column.kind, cell) {
        (ColumnKind::Bar, Some(Cell::Number(v))) => {
            let (start, width) = scale.map_or((0.0, 0.0), |s| s.span(*v));
            let bar = format!("--a: {:.3}%; --w: {:.3}%;", start * 100.0, width * 100.0);
            rsx! {
                span { "data-part": "bar-cell", style: "--i: {i};",
                    span { "data-part": "value", {column.format(*v)} }
                    span { "data-part": "track", "aria-hidden": "true",
                        span {
                            "data-part": "bar",
                            "data-negative": (*v < 0.0).then_some(""),
                            style: bar,
                        }
                    }
                }
            }
        }
        (ColumnKind::Delta, Some(Cell::Number(v))) => {
            let dir = ct::direction(*v);
            let glyph = ct::glyph(dir);
            rsx! {
                span { "data-part": "delta", "data-direction": dir, style: "--i: {i};",
                    span { "data-part": "glyph", "aria-hidden": "true", "{glyph}" }
                    {column.format(*v)}
                }
            }
        }
        (ColumnKind::Sparkline, Some(Cell::Series(values))) => {
            let unit = column.clone();
            rsx! {
                span { "data-part": "sparkline-cell", style: "--i: {i};",
                    Sparkline {
                        values: values.clone(),
                        label: "{row.label}, {column.header}",
                        width: 96,
                        height: 24,
                        format: move |v: f64| unit.format(v),
                    }
                }
            }
        }
        _ => rsx! { {ct::text(column, cell)} },
    }
}

/// A table of rows with inline charts. Bar columns share one scale down the column, so
/// lengths compare; sparklines draw each row's trend; deltas show their direction.
/// Every column sorts from its header.
#[component]
pub fn ChartTable(
    /// The columns after the row label, in order.
    columns: Vec<TableColumn>,
    /// The rows; each has a label and one cell per column.
    rows: ReadSignal<Vec<TableRow>>,
    /// What the table shows, such as "Channels this quarter". Used as the caption.
    label: String,
    /// Header of the row-label column.
    #[props(default)]
    row_header: String,
) -> Element {
    let export_cols = columns.clone();
    let export_head = row_header.clone();
    use_export(move |()| ct::export(&export_head, &export_cols, &rows.read()));
    let data = rows();
    let scales: Vec<Option<BarScale>> =
        columns.iter().enumerate().map(|(i, c)| (c.kind == ColumnKind::Bar).then(|| BarScale::of(&data, i))).collect();
    let first =
        Column::new("label", row_header, |(_, r): &Indexed| rsx! { span { "data-part": "row-label", "{r.label}" } })
            .sort_by(|(_, a): &Indexed, (_, b): &Indexed| ct::compare_labels(a, b));
    let table_columns: Vec<Column<Indexed>> = std::iter::once(first)
        .chain(columns.into_iter().enumerate().map(|(i, column)| {
            let numeric = column.kind.is_numeric();
            let scale = scales.get(i).copied().flatten();
            let c = Column::new(format!("c{i}"), column.header.clone(), move |row: &Indexed| {
                cell_view(i, &column, row, scale)
            })
            .sort_by(move |(_, a): &Indexed, (_, b): &Indexed| ct::compare_rows(a, b, i));
            if numeric { c.numeric() } else { c }
        }))
        .collect();
    let indexed: Vec<Indexed> = data.into_iter().enumerate().collect();

    rsx! {
        div { "data-scope": "chart-table", "data-part": "root",
            DataTable { caption: label, columns: table_columns, rows: indexed, row_id: |(i, _): &Indexed| i.to_string() }
        }
    }
}
