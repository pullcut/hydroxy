use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::waffle_chart::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let spend = vec![
        Datum::new("Salaries", 5_400.0),
        Datum::new("Cloud", 1_900.0),
        Datum::new("Marketing", 1_350.0),
        Datum::new("Office", 750.0),
        Datum::new("Other", 600.0),
    ];
    let seats = vec![Datum::new("Active", 312.0), Datum::new("Invited", 41.0)];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Spend" description="Each square is 1% of this year's budget.">
                <WaffleChart data=spend label="Spend by category" format=|v: f64| format!("${:.1}m", v / 1000.0) />
            </ChartFrame>
            <ChartFrame title="Seats" description="Of 500 licensed; empty squares are unused.">
                <WaffleChart data=seats label="Seats in use" total=500.0 />
            </ChartFrame>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "waffle-chart",
    name: "Waffle chart",
    group: "Bars",
    summary: "A 10×10 grid of unit squares shared out by category, so each square reads as 1%. Give a total to leave the unused share empty.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("waffle_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/waffle_chart.rs"),
    parts: &[
        &["WaffleChart", "root", "figure with a screen-reader table of every category"],
        &["", "plot", "Focusable grid; arrow keys move cell by cell"],
        &["", "cell", "One unit square in its category colour, or empty"],
        &["", "tooltip", "Category, value and cell count of the active cell"],
        &["", "legend", "Category names; hovering one picks out its cells"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Next or previous cell in the row"],
        &["ArrowDown / ArrowUp", "Cell below or above"],
        &["Home / End", "First or last cell in the row"],
        &["Escape", "Clear the active cell"],
    ],
    props: &[
        &["data", "Vec<Datum>", "Required. Categories in fill order"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["total", "f64", "The whole the grid stands for. Default: the sum"],
        &["columns / rows", "usize", "Grid size. Default 10 × 10"],
        &["format", "Fn(f64) -> String", "Formats values in the tooltip and table"],
    ],
    events: crate::doc::CHART_EVENTS,
};
