use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::chart_table::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let columns = vec![
        TableColumn::bar("Visitors"),
        TableColumn::number("Conversion").unit("%").decimals(1),
        TableColumn::bar("Revenue").unit("k").decimals(1),
        TableColumn::delta("vs last quarter").unit("%").decimals(1),
        TableColumn::sparkline("Weekly revenue"),
    ];
    // Revenue in £ thousands; the sparkline is the last twelve weeks.
    let row = |name: &str, visitors: f64, conversion: f64, revenue: f64, change: f64, weeks: [f64; 12]| {
        TableRow::new(
            name,
            vec![
                Cell::Number(visitors),
                Cell::Number(conversion),
                Cell::Number(revenue),
                Cell::Number(change),
                Cell::Series(weeks.to_vec()),
            ],
        )
    };
    let rows = vec![
        row(
            "Organic search",
            48_210.0,
            3.4,
            182.6,
            8.2,
            [12.1, 12.8, 13.0, 13.9, 14.2, 13.8, 15.1, 15.6, 16.0, 16.4, 17.2, 17.9],
        ),
        row(
            "Paid search",
            31_540.0,
            4.1,
            141.3,
            -3.6,
            [12.9, 12.4, 12.6, 11.8, 12.0, 11.5, 11.9, 11.2, 11.0, 10.8, 11.1, 10.6],
        ),
        row("Email", 12_880.0, 6.8, 97.4, 14.9, [6.2, 6.8, 7.0, 7.5, 7.4, 8.1, 8.6, 8.4, 9.0, 9.3, 9.8, 10.4]),
        row("Social", 22_310.0, 1.6, 38.9, -11.2, [4.2, 4.0, 3.8, 3.9, 3.5, 3.4, 3.1, 3.3, 3.0, 2.9, 2.8, 2.7]),
        row("Referral", 6_420.0, 5.2, 36.1, 2.4, [2.8, 3.0, 2.9, 3.1, 3.0, 3.2, 3.1, 3.0, 3.3, 3.2, 3.4, 3.3]),
        row("Direct", 19_760.0, 3.9, 84.2, 0.0, [7.0, 7.1, 6.9, 7.0, 7.2, 7.0, 6.9, 7.1, 7.0, 7.0, 7.1, 7.0]),
    ];

    view! {
        <ChartFrame title="Acquisition channels" description="This quarter. Bars share a scale down each column; click a header to sort." export=true>
            <ChartTable columns rows label="Acquisition channels this quarter" row_header="Channel" />
        </ChartFrame>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "chart-table",
    name: "Chart table",
    group: "Grids and tables",
    summary: "A sortable table whose cells carry small charts: bars on one scale per column so lengths compare down it, sparklines for each row's trend, and deltas with an up or down arrow. Built on DataTable, so sorting, headers and keyboard work the same.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("chart_table.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/chart_table.rs"),
    parts: &[
        &["ChartTable", "root", "Wraps a DataTable; every DataTable part applies"],
        &["", "row-label", "The row's name in the first column"],
        &["", "bar-cell", "A value and its bar; --i staggers the bars growing in"],
        &["", "track", "The bar's full width, on the column's shared scale"],
        &["", "bar", "Grows from zero; data-negative bars grow left"],
        &["", "delta", "Signed change; data-direction is up, down or flat"],
        &["", "sparkline-cell", "A Sparkline that draws in after the row above"],
    ],
    keys: &[
        &["Tab", "Moves to each sort button, then each sparkline"],
        &["Enter / Space", "Sorts by the focused column; again to reverse"],
        &["ArrowLeft / ArrowRight", "On a sparkline, steps through its points"],
    ],
    props: &[
        &[
            "columns",
            "Vec<TableColumn>",
            "Required. TableColumn::text, number, bar, delta or sparkline, with .unit(\"%\") and .decimals(n)",
        ],
        &[
            "rows",
            "Vec<TableRow>",
            "Required. TableRow::new(label, cells): Cell::Number, Cell::Text or Cell::Series per column",
        ],
        &["label", "String", "Required. The table's caption"],
        &["row_header", "String", "Header of the row-label column. Default empty"],
    ],
    events: &[],
};
