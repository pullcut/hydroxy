use dioxus::prelude::*;
use hydroxy_dioxus::chart_frame::ChartFrame;
use hydroxy_dioxus::word_cloud_chart::{Word, WordCloudChart};

#[component]
pub fn WordCloudChartDemo() -> Element {
    let themes = [
        ("export", 64.0),
        ("search", 58.0),
        ("slow dashboards", 47.0),
        ("SSO", 41.0),
        ("pricing", 36.0),
        ("dark mode", 31.0),
        ("mobile app", 29.0),
        ("API limits", 26.0),
        ("onboarding", 22.0),
        ("notifications", 19.0),
        ("permissions", 17.0),
        ("integrations", 15.0),
        ("billing", 13.0),
        ("audit log", 11.0),
        ("templates", 10.0),
        ("offline", 8.0),
        ("keyboard shortcuts", 7.0),
        ("comments", 6.0),
        ("time zones", 5.0),
        ("CSV import", 4.0),
    ];
    let words: Vec<Word> = themes.iter().map(|(t, n)| Word::new(*t, *n)).collect();

    rsx! {
        ChartFrame { title: "Feedback themes", description: "Mentions in support tickets this month.",
            WordCloudChart { words, label: "Themes in feedback", weight_label: "Mentions" }
        }
    }
}
