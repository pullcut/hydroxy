use dioxus::prelude::*;
use hydroxy_dioxus::rating::Rating;

#[component]
pub fn RatingDemo() -> Element {
    let mut score = use_signal(|| 3.5);

    rsx! {
        div { style: "display: grid; gap: var(--space-6); justify-items: start;",
            div { style: "display: flex; align-items: center; gap: var(--space-4);",
                Rating {
                    label: "Answer quality",
                    value: Some(score()),
                    allow_half: true,
                    clearable: true,
                    on_value_change: move |v| score.set(v),
                }
                span { style: "font: 500 12px/1 var(--font-mono); color: var(--text-2);", "{score} / 5" }
            }
            Rating { label: "Difficulty", count: 3, default_value: 2.0, size: 16, name: "difficulty" }
            Rating { default_value: 4.5, allow_half: true, read_only: true }
        }
    }
}
