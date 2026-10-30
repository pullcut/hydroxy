use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::diverging_bar_chart::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let questions: Vec<String> = [
        "I have the tools I need",
        "My workload is manageable",
        "I get useful feedback",
        "I would recommend working here",
        "Meetings are a good use of time",
    ]
    .map(String::from)
    .to_vec();
    // One series per answer, most negative first. The middle answer is neutral and straddles the centre.
    let answers = vec![
        Series::new("Strongly disagree", [6.0, 22.0, 14.0, 9.0, 41.0]),
        Series::new("Disagree", [18.0, 47.0, 33.0, 21.0, 68.0]),
        Series::new("Neutral", [29.0, 51.0, 48.0, 37.0, 54.0]),
        Series::new("Agree", [112.0, 84.0, 92.0, 101.0, 52.0]),
        Series::new("Strongly agree", [75.0, 36.0, 53.0, 72.0, 25.0]),
    ];
    let channels: Vec<String> = ["Live chat", "Email", "Phone", "Help centre"].map(String::from).to_vec();
    // An even number of answers has no neutral: the split falls between the middle two.
    let ratings = vec![
        Series::new("Very poor", [31.0, 58.0, 22.0, 64.0]),
        Series::new("Poor", [74.0, 121.0, 49.0, 140.0]),
        Series::new("Good", [402.0, 288.0, 175.0, 233.0]),
        Series::new("Very good", [361.0, 141.0, 168.0, 98.0]),
    ];

    view! {
        <div style="display: grid; gap: var(--space-10);">
            <ChartFrame title="Staff survey" description="Share of 240 answers to each statement, spring 2025.">
                <DivergingBarChart rows=questions levels=answers label="Staff survey answers by statement" />
            </ChartFrame>
            <ChartFrame title="Support rating by channel" description="Share of rated conversations, last 90 days.">
                <DivergingBarChart rows=channels levels=ratings label="Support ratings by channel" />
            </ChartFrame>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "diverging-bar-chart",
    name: "Diverging bar chart",
    group: "Bars",
    summary: "Survey (Likert) answers as stacked bars split around a centre line: negative answers run left, positive ones right, and a neutral middle answer straddles the centre. Each row is shown as shares of its own total, with the totals either side at the bar ends.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("diverging_bar_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/diverging_bar_chart.rs"),
    parts: &[
        &["DivergingBarChart", "root", "figure with a screen-reader table of every share"],
        &["", "bar", "One segment per answer; `data-edge` marks a row's rounded outer ends"],
        &["", "centre", "The zero line the rows grow out from"],
        &["", "total", "Share of negative answers at the left end, positive at the right"],
        &["", "x-label", "Row names, down the left"],
        &["", "tooltip", "Every answer's share for the row, the hovered one in bold"],
        &["", "legend", "Answer levels in order, red to grey to blue"],
    ],
    keys: &[
        &["ArrowUp / ArrowDown", "Previous or next row"],
        &["ArrowLeft / ArrowRight", "Previous or next answer in the row"],
        &["Home / End", "First or last answer in the row"],
        &["Escape", "Hide the tooltip"],
    ],
    props: &[
        &["rows", "Vec<String>", "Required. One label per question"],
        &[
            "levels",
            "Vec<Series>",
            "Required. One series per answer, most negative first; an odd count makes the middle one neutral",
        ],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["height", "u16", "Plot height in px. Default 40 per row"],
    ],
    events: crate::doc::CHART_EVENTS,
};
