use dioxus::prelude::*;
use oxide_dioxus::chat_bubble::{ChatBubble, ChatRole};

#[component]
pub fn ChatBubbleDemo() -> Element {
    rsx! {
        div { style: "display: grid; gap: var(--space-7); max-width: 560px;",
            ChatBubble { role: ChatRole::User, time: "09:41", "What did people say about onboarding?" }
            ChatBubble {
                sources: vec!["interviews-q3.pdf".to_string(), "survey-results.xlsx".to_string()],
                time: "09:41",
                "Nine of twelve interviewees raised onboarding. Most wanted a shorter first setup; three asked for sample data."
            }
        }
    }
}
