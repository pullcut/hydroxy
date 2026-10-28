use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::diverging_bar_chart::{DivergingBarChart, Series};

#[component]
pub fn DivergingBarChartDemo() -> Element {
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

    rsx! {
        div { style: "display: grid; gap: var(--space-10);",
            ChartFrame { title: "Staff survey", description: "Share of 240 answers to each statement, spring 2025.",
                DivergingBarChart { rows: questions, levels: answers, label: "Staff survey answers by statement" }
            }
            ChartFrame { title: "Support rating by channel", description: "Share of rated conversations, last 90 days.",
                DivergingBarChart { rows: channels, levels: ratings, label: "Support ratings by channel" }
            }
        }
    }
}
