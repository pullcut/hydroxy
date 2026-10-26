use dioxus::prelude::*;
use oxide_dioxus::priority_mark::PriorityMark;

#[component]
pub fn PriorityMarkDemo() -> Element {
    let tasks = [
        (0, "Fix the September export count"),
        (1, "Review two flagged summaries"),
        (2, "Tag the Q3 interviews"),
        (3, "Tidy the archive"),
    ];
    rsx! {
        ul { style: "list-style: none; margin: 0; padding: 0; display: grid; gap: var(--space-4);",
            for (level, task) in tasks {
                li { style: "display: flex; align-items: center; gap: 10px; font: var(--text-sm);",
                    PriorityMark { level }
                    "{task}"
                }
            }
        }
    }
}
