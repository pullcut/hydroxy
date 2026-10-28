use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::chart_table::{Cell, ChartTable, TableColumn, TableRow};

#[component]
pub fn ChartTableDemo() -> Element {
    let columns = vec![
        TableColumn::bar("Visitors"),
        TableColumn::number("Conversion").unit("%").decimals(1),
        TableColumn::bar("Revenue").unit("k").decimals(1),
        TableColumn::delta("vs last quarter").unit("%").decimals(1),
        TableColumn::sparkline("Weekly revenue"),
    ];
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

    rsx! {
        ChartFrame {
            title: "Acquisition channels",
            description: "This quarter. Bars share a scale down each column; click a header to sort.",
            export: true,
            ChartTable { columns, rows, label: "Acquisition channels this quarter", row_header: "Channel" }
        }
    }
}
