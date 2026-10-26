use dioxus::prelude::*;
use oxide_dioxus::filter_builder::{
    Choice, FilterBuilder, FilterField, FilterFieldKind, FilterJoin, FilterOp, FilterRule, FilterValue,
};

fn fields() -> Vec<FilterField> {
    vec![
        FilterField::new("title", "Title", FilterFieldKind::Text),
        FilterField::new("score", "Score", FilterFieldKind::Number),
        FilterField::new(
            "status",
            "Status",
            FilterFieldKind::Choice(vec![Choice::new("open", "Open"), Choice::new("review", "In review"), Choice::new("done", "Done")]),
        ),
    ]
}

const RECORDS: [(&str, &str, &str); 5] = [
    ("Weekly digest", "12", "open"),
    ("Board update", "30", "review"),
    ("Tag interviews", "7", "done"),
    ("Import tickets", "18", "open"),
    ("Source check", "25", "review"),
];

#[component]
pub fn FilterBuilderDemo() -> Element {
    // Uncontrolled, and tell the page about every edit.
    let mut filter = use_signal(FilterValue::default);
    let f = filter.read().clone();

    rsx! {
        div { style: "display: grid; gap: var(--space-4);",
            FilterBuilder {
                fields: fields(),
                default_filter: FilterValue {
                    join: FilterJoin::And,
                    rules: vec![FilterRule { id: 0, field: "score".into(), op: FilterOp::Gte, value: "10".into() }],
                },
                on_change: move |next| filter.set(next),
            }
            ul { style: "margin: 0; padding-left: var(--space-5); font: var(--text-sm); color: var(--text-2);",
                for (title , score , status) in RECORDS {
                    if f.matches(|field| match field {
                        "title" => Some(title.to_string()),
                        "score" => Some(score.to_string()),
                        "status" => Some(status.to_string()),
                        _ => None,
                    }) {
                        li { key: "{title}", "{title} · score {score} · {status}" }
                    }
                }
            }
        }
    }
}
