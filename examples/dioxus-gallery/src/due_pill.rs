use dioxus::prelude::*;
use hydroxy_dioxus::due_pill::DuePill;

#[component]
pub fn DuePillDemo() -> Element {
    let tasks = [
        ("Send the Q3 summary", Some("2026-10-24")),
        ("Review flagged answers", Some("2026-10-27")),
        ("Plan the November interviews", Some("2026-11-12")),
        ("Archive old drafts", None),
    ];
    rsx! {
        ul { style: "list-style: none; margin: 0; padding: 0; display: grid; gap: var(--space-4); max-width: 360px;",
            for (task, date) in tasks {
                li { style: "display: flex; align-items: center; justify-content: space-between; gap: 10px; font: var(--text-sm);",
                    "{task}"
                    DuePill { date: date.map(String::from), today: "2026-10-26" }
                }
            }
        }
    }
}
