use leptos::prelude::*;
use oxide_leptos::bullet_chart::*;
use oxide_leptos::chart_frame::ChartFrame;

pub fn demo() -> AnyView {
    let rows = vec![
        BulletRow::new("Revenue ($k)", 271.0).target(250.0).ranges([150.0, 225.0, 300.0]),
        BulletRow::new("Profit (%)", 22.5).target(26.0).ranges([20.0, 25.0, 30.0]),
        BulletRow::new("New customers", 1_640.0).target(2_000.0).ranges([1_400.0, 2_000.0, 2_500.0]),
        BulletRow::new("Satisfaction", 4.4).target(4.5).ranges([3.5, 4.25, 5.0]),
    ];

    view! {
        <ChartFrame title="Q3 scorecard" description="Each measure on its own scale, against target.">
            <BulletChart rows=rows label="Q3 scorecard" range_labels=vec!["Poor".to_string(), "Fair".into(), "Good".into()] />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "bullet-chart",
    name: "Bullet chart",
    group: "Bars",
    summary: "A compact replacement for a gauge: each measure is a bar against a target tick, over grey bands for poor, fair and good. One row per measure, each on its own zero-based scale.",
    demo,
    leptos: include_str!("bullet_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/bullet_chart.rs"),
    parts: &[
        &["BulletChart", "root", "figure with a screen-reader table of every row"],
        &["", "plot", "Focusable list of rows; arrow keys step through them"],
        &["", "row", "Name and value beside the track"],
        &["", "band", "Qualitative range, darkest for the lowest"],
        &["", "measure", "The value bar, in the first series colour"],
        &["", "target", "The goal tick"],
        &["", "tooltip", "Value, target and the band the value falls in"],
        &["", "legend", "Keys for the bar, target and named bands"],
    ],
    keys: &[
        &["ArrowDown / ArrowRight", "Next row"],
        &["ArrowUp / ArrowLeft", "Previous row"],
        &["Home / End", "First or last row"],
        &["Escape", "Clear the active row"],
    ],
    props: &[
        &["rows", "Vec<BulletRow>", "Required. BulletRow::new(label, value).target(t).ranges([..])"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["range_labels", "Vec<String>", "Band names from the lowest up. Default \"Range n\""],
        &["format", "Fn(f64) -> String", "Formats ticks, tooltip and table"],
    ],
};
