use leptos::prelude::*;
use hydroxy_leptos::chart_frame::ChartFrame;
use hydroxy_leptos::word_cloud_chart::*;

pub fn demo() -> AnyView {
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

    view! {
        <ChartFrame title="Feedback themes" description="Mentions in support tickets this month.">
            <WordCloudChart words=words label="Themes in feedback" weight_label="Mentions" />
        </ChartFrame>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "word-cloud-chart",
    name: "Word cloud",
    group: "Parts of a whole",
    summary: "Weighted words, largest first, placed along a spiral from the centre without overlapping. Placement is deterministic, so the same words always land in the same places.",
    demo,
    leptos: include_str!("word_cloud_chart.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/word_cloud_chart.rs"),
    parts: &[
        &["WordCloudChart", "root", "figure with a screen-reader table of every word"],
        &["", "plot", "Focusable cloud; arrow keys step from heaviest to lightest"],
        &["", "word", "Sized by weight; the heaviest third in the first series colour"],
        &["", "tooltip", "Word and weight"],
    ],
    keys: &[
        &["ArrowRight / ArrowDown", "Next word, heaviest first"],
        &["ArrowLeft / ArrowUp", "Previous word"],
        &["Home / End", "Heaviest or lightest word"],
        &["Escape", "Clear the active word"],
    ],
    props: &[
        &["words", "Vec<Word>", "Required. Word::new(text, weight)"],
        &["label", "String", "Required. What the chart shows; the accessible name"],
        &["width / height", "u16", "Layout box in px; scales down to fit. Default 520 × 300"],
        &["weight_label", "String", "Name of the weight. Default \"Weight\""],
        &["format", "Fn(f64) -> String", "Formats weights in the tooltip and table"],
    ],
};
