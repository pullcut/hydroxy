use dioxus::prelude::*;
use oxide_dioxus::insight_note::InsightNote;

#[component]
pub fn InsightNoteDemo() -> Element {
    rsx! {
        div { style: "display: grid; gap: var(--space-5); max-width: 520px;",
            InsightNote { sources: vec!["interviews-q3.pdf".to_string(), "tickets-sept.csv".to_string()],
                p {
                    "Nine of twelve interviewees raised onboarding. The September export has 410 fewer tickets than the August summary, so two figures need checking."
                }
            }
            InsightNote { label: "Weekly digest",
                p { "Three projects moved to ready. One is waiting on a review from Amara Kaur." }
            }
        }
    }
}
