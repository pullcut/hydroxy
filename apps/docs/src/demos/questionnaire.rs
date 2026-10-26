use leptos::prelude::*;
use oxide_leptos::questionnaire::{Questionnaire, QuestionnaireAnswer, QuestionnaireQuestion as Question, Test};

pub fn demo() -> AnyView {
    let (sent, set_sent) = signal(None::<String>);

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

    // Built outside view! because the macro can't parse a closure type with generics.
    let on_submit = Callback::new(move |answers: Vec<QuestionnaireAnswer>| {
        let text = answers.iter().map(|a| format!("{}: {}", a.id, a.values.join(" / "))).collect::<Vec<_>>().join("; ");
        set_sent.set(Some(text));
    });

    view! {
        <div style="display: grid; gap: var(--space-6); max-width: 560px;">
            <Questionnaire
                label="Product survey"
                questions=questions
                submit_label="Send answers"
                on_submit=on_submit
            />
            {move || {
                sent.get()
                    .map(|text| {
                        view! { <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">{format!("Submitted: {text}")}</p> }
                    })
            }}
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "questionnaire",
    name: "Questionnaire",
    group: "Forms",
    summary: "A survey that asks one question at a time. Single choice, multiple choice, text and scale questions, branching on earlier answers, a progress bar and a review step before submitting.",
    demo,
    leptos: include_str!("questionnaire.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/questionnaire.rs"),
    parts: &[
        &["Questionnaire", "root", "A <form>. data-state is question, review or done"],
        &["(inside)", "progress / bar / step", "A native <progress> plus \"Question 2 of 4\", counting only visible questions"],
        &["(inside)", "question", "A <fieldset> with the prompt as its legend. Takes focus when the step changes"],
        &["(inside)", "prompt / hint", "The question and its help text"],
        &["(inside)", "controls", "data-kind is single, multi, text or scale"],
        &["(inside)", "choice / scale-option / input", "Native radios, checkboxes, number radios, input and textarea"],
        &["(inside)", "error", "role=alert, shown when Next is pressed on an empty required question"],
        &["(inside)", "review / summary / answer / edit", "Every answer with an Edit button that jumps back to its question"],
        &["(inside)", "nav", "Back and Next (a submit button, so Enter works)"],
        &["(inside)", "done", "role=status message with a Start over button"],
    ],
    keys: &[
        &["Enter", "In a radio, checkbox or one-line input: Next, or Submit on the review"],
        &["Arrow keys", "Move between the radios of a question"],
        &["Space", "Toggle a checkbox or select the focused radio"],
    ],
    props: &[
        &["label", "String", "Required. Accessible name for the form"],
        &["questions", "Vec<Question>", "Required. Build with Question::single, multi, text and scale, then .required(), .hint() and .show_if()"],
        &["answers", "Option<Answers>", "Controlled answers by question id. Pair with on_answers_change"],
        &["default_answers", "Answers", "Initial answers when uncontrolled"],
        &["submit_label", "&str", "Review button text. Default \"Submit\""],
        &["done_message", "&str", "Shown after submitting"],
        &["on_answers_change", "Callback<Answers>", "Called with the new answers"],
        &["on_submit", "Callback<Vec<Answer>>", "Answers to visible questions only, in order"],
    ],
};
