use dioxus::prelude::*;
use oxide_dioxus::questionnaire::{Questionnaire, QuestionnaireAnswer, QuestionnaireQuestion as Question, Test};

#[component]
pub fn QuestionnaireDemo() -> Element {
    let mut sent = use_signal(|| None::<String>);

    // "Which role?" only appears for "Other", and the closing question only for happy users.
    let questions = vec![
        Question::single(
            "role",
            "What best describes your role?",
            &[("dev", "Developer"), ("design", "Designer"), ("pm", "Product manager"), ("other", "Something else")],
        )
        .required(),
        Question::text("role-other", "Tell us what you do", false).show_if("role", Test::Equals("other".into())).required(),
        Question::multi(
            "frameworks",
            "Which frameworks do you use oxide with?",
            &[("leptos", "Leptos"), ("dioxus", "Dioxus"), ("none", "Neither yet")],
        )
        .hint("Pick any that apply."),
        Question::scale("satisfaction", "How happy are you with oxide so far?", 1, 5, "Not happy", "Very happy").required(),
        Question::text("love", "What do you like most?", true).show_if("satisfaction", Test::AtLeast(4)),
        Question::text("improve", "What should we fix first?", true).show_if("satisfaction", Test::NotEquals("5".into())),
    ];

    rsx! {
        div { style: "display: grid; gap: var(--space-6); max-width: 560px;",
            Questionnaire {
                label: "Product survey",
                questions,
                submit_label: "Send answers",
                on_submit: move |answers: Vec<QuestionnaireAnswer>| {
                    let text = answers.iter().map(|a| format!("{}: {}", a.id, a.values.join(" / "))).collect::<Vec<_>>().join("; ");
                    sent.set(Some(text));
                },
            }
            if let Some(text) = sent() {
                p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);", "Submitted: {text}" }
            }
        }
    }
}
