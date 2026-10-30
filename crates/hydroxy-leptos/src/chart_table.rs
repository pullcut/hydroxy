//! `<ChartTable>`: a sortable data table whose cells carry small visuals: bars on a scale
//! shared down each column, sparklines and up/down deltas. Built on [`DataTable`];
//! scales come from `hydroxy_charts::chart_table`.

use hydroxy_charts::chart_table::{self as ct, BarScale, ColumnKind};
use leptos::prelude::*;

pub use hydroxy_charts::chart_table::{Cell, TableColumn, TableRow};

use crate::chart_frame::provide_export;
use crate::data_table::{Column, DataTable};
use crate::sparkline::Sparkline;

/// A row with its original position, which staggers its entrance.
type Indexed = (usize, TableRow);

fn cell_view(col: usize, column: &TableColumn, (i, row): &Indexed, scale: Option<BarScale>, label: &str) -> AnyView {
    let style = format!("--i: {i};");
    let cell = row.cells.get(col);
    match (column.kind, cell) {
        (ColumnKind::Bar, Some(Cell::Number(v))) => {
            let (start, width) = scale.map_or((0.0, 0.0), |s| s.span(*v));
            let bar = format!("--a: {:.3}%; --w: {:.3}%;", start * 100.0, width * 100.0);
            view! {
                <span data-part="bar-cell" style=style>
                    <span data-part="value">{column.format(*v)}</span>
                    <span data-part="track" aria-hidden="true">
                        <span data-part="bar" data-negative=(*v < 0.0).then_some("") style=bar></span>
                    </span>
                </span>
            }
            .into_any()
        }
        (ColumnKind::Delta, Some(Cell::Number(v))) => {
            let dir = ct::direction(*v);
            let glyph = ct::glyph(dir);
            view! {
                <span data-part="delta" data-direction=dir style=style>
                    <span data-part="glyph" aria-hidden="true">{glyph}</span>
                    {column.format(*v)}
                </span>
            }
            .into_any()
        }
        (ColumnKind::Sparkline, Some(Cell::Series(values))) => {
            let unit = column.clone();
            let name = format!("{label}, {}", column.header);
            view! {
                <span data-part="sparkline-cell" style=style>
                    <Sparkline values=values.clone() label=name width=96 height=24 format=Callback::new(move |v| unit.format(v)) />
                </span>
            }
            .into_any()
        }
        _ => ct::text(column, cell).into_any(),
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
    #[prop(into)]
    rows: Signal<Vec<TableRow>>,
    /// What the table shows, such as "Channels this quarter". Used as the caption.
    #[prop(into)]
    label: String,
    /// Header of the row-label column.
    #[prop(default = String::new(), into)]
    row_header: String,
) -> impl IntoView {
    let scales = Memo::new({
        let kinds: Vec<ColumnKind> = columns.iter().map(|c| c.kind).collect();
        move |_| {
            rows.with(|r| {
                kinds
                    .iter()
                    .enumerate()
                    .map(|(i, k)| (*k == ColumnKind::Bar).then(|| BarScale::of(r, i)))
                    .collect::<Vec<_>>()
            })
        }
    });
    let export_cols = columns.clone();
    let export_head = row_header.clone();
    provide_export(move || rows.with_untracked(|r| ct::export(&export_head, &export_cols, r)));
    let first = Column::new(
        "label",
        row_header,
        |(_, r): &Indexed| view! { <span data-part="row-label">{r.label.clone()}</span> },
    )
    .sort_by(|(_, a): &Indexed, (_, b): &Indexed| ct::compare_labels(a, b));
    let table_columns: Vec<Column<Indexed>> = std::iter::once(first)
        .chain(columns.into_iter().enumerate().map(|(i, column)| {
            let numeric = column.kind.is_numeric();
            let header = column.header.clone();
            let c = Column::new(format!("c{i}"), header, move |row: &Indexed| {
                let scale = scales.with_untracked(|s| s.get(i).copied().flatten());
                cell_view(i, &column, row, scale, &row.1.label)
            })
            .sort_by(move |(_, a): &Indexed, (_, b): &Indexed| ct::compare_rows(a, b, i));
            if numeric { c.numeric() } else { c }
        }))
        .collect();
    let indexed = Signal::derive(move || rows.get().into_iter().enumerate().collect::<Vec<_>>());

    view! {
        <div data-scope="chart-table" data-part="root" style=move || format!("--ox-n: {};", rows.with(Vec::len))>
            <DataTable caption=label columns=table_columns rows=indexed row_id=|(i, _): &Indexed| i.to_string() />
        </div>
    }
}
