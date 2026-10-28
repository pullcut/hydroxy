//! `Questionnaire`: one question at a time, with branching and a review step.

use dioxus::prelude::*;
use hydroxy_headless::questionnaire::{
    self as h, Answer, Answers, Kind, Question, QuestionnaireCommand, QuestionnaireEvent, QuestionnaireInput,
    QuestionnaireMachine, Step,
};
use hydroxy_statechart::Machine;

use crate::button::{Button, ButtonType, ButtonVariant};

pub use h::{Answer as QuestionnaireAnswer, Choice, Condition, Question as QuestionnaireQuestion, Test};

/// A form that asks its questions one by one. Questions can depend on earlier
/// answers, a progress bar follows the visible path, and a review step lets
/// people jump back to edit before submitting.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn Questionnaire(
    /// Accessible name for the form.
    label: String,
    /// The questions, in order. See [`Question`] for the builders.
    questions: Vec<Question>,
    /// Controlled answers. Pair with `on_answers_change`.
    #[props(default)]
    answers: ReadSignal<Option<Answers>>,
    /// Initial answers when uncontrolled.
    #[props(default)]
    default_answers: Answers,
    /// Text of the submit button on the review. Default "Submit".
    #[props(default = "Submit")]
    submit_label: &'static str,
    /// Message shown after submitting.
    #[props(default = "Thanks. Your answers are in.")]
    done_message: &'static str,
    /// Called with the new answers whenever they change or a change is requested.
    on_answers_change: Option<EventHandler<Answers>>,
    /// Called with the answers to visible questions when the user submits.
    on_submit: Option<EventHandler<Vec<Answer>>>,
) -> Element {
    let ids = use_hook(|| h::ids(&hydroxy_headless::next_id(h::SCOPE)));
    let list = questions.clone();
    let input = use_memo(move || QuestionnaireInput {
        questions: list.clone(),
        answers: answers(),
        default_answers: default_answers.clone(),
    });
    let mut state = use_signal(|| QuestionnaireMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = QuestionnaireMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    // Focus moves to the new step once the user has navigated, not on first render.
    let mut navigated = use_signal(|| false);

    let mut send = move |event: QuestionnaireEvent| {
        if !matches!(event, QuestionnaireEvent::Set { .. } | QuestionnaireEvent::Toggle { .. }) {
            navigated.set(true);
        }
        let step = QuestionnaireMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                QuestionnaireCommand::AnswersChange(a) => {
                    if let Some(handler) = on_answers_change {
                        handler.call(a);
                    }
                }
                QuestionnaireCommand::Submit(a) => {
                    if let Some(handler) = on_submit {
                        handler.call(a);
                    }
                }
            }
        }
    };
    let focus_step = move |e: MountedEvent| {
        if navigated() {
            let el = e.data();
            spawn(async move {
                let _ = el.set_focus(true).await;
            });
        }
    };

    let s = state.read().clone();
    let (position, total) = h::progress(&s, &input.read());
    let progress_text = h::progress_text(&s, &input.read());
    let forward = if s.step == Step::Review { submit_label } else { h::next_label(&s, &input.read()) };
    let done = s.step == Step::Done;
    let state_name = match s.step {
        Step::Question => "question",
        Step::Review => "review",
        Step::Done => "done",
    };
    let values_of = |id: &str| s.answers.get(id).cloned().unwrap_or_default();
    let review: Vec<(usize, String, String)> = if s.step == Step::Review {
        input
            .read()
            .path(&s.answers)
            .into_iter()
            .map(|i| (i, questions[i].prompt.clone(), h::answer_text(&questions[i], &s.answers)))
            .collect()
    } else {
        Vec::new()
    };

    rsx! {
        form {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "aria-label": "{label}",
            novalidate: true,
            "data-state": state_name,
            onsubmit: move |e| {
                e.prevent_default();
                let reviewing = state.peek().step == Step::Review;
                send(if reviewing { QuestionnaireEvent::Submit } else { QuestionnaireEvent::Next });
            },
            div { "data-scope": h::SCOPE, "data-part": "progress", hidden: done,
                progress {
                    "data-scope": h::SCOPE,
                    "data-part": "bar",
                    max: "{total.max(1)}",
                    value: "{position}",
                    "aria-label": "Progress",
                }
                span { "data-scope": h::SCOPE, "data-part": "step", "aria-live": "polite", "{progress_text}" }
            }
            match s.step {
                Step::Question => rsx! {
                    if let Some(q) = questions.get(s.current) {
                        {
                            let q = q.clone();
                            let id = q.id.clone();
                            let name = format!("{}-{}", ids.step, q.id);
                            let current = values_of(&id);
                            let missing = q.required && s.attempted && !q.is_answered(&s.answers);
                            rsx! {
                                fieldset {
                                    key: "{s.current}",
                                    id: "{ids.step}",
                                    tabindex: "-1",
                                    "data-scope": h::SCOPE,
                                    "data-part": "question",
                                    onmounted: focus_step,
                                    legend { "data-scope": h::SCOPE, "data-part": "prompt",
                                        "{q.prompt}"
                                        if q.required {
                                            span { "aria-hidden": "true", " *" }
                                        }
                                    }
                                    if let Some(hint) = q.hint.clone() {
                                        p { "data-scope": h::SCOPE, "data-part": "hint", "{hint}" }
                                    }
                                    div {
                                        "data-scope": h::SCOPE,
                                        "data-part": "controls",
                                        "data-kind": match q.kind {
                                            Kind::Single(_) => "single",
                                            Kind::Multi(_) => "multi",
                                            Kind::Text { .. } => "text",
                                            Kind::Scale { .. } => "scale",
                                        },
                                        match q.kind.clone() {
                                            Kind::Single(choices) => rsx! {
                                                for c in choices {
                                                    label { key: "{c.value}", "data-scope": h::SCOPE, "data-part": "choice",
                                                        input {
                                                            r#type: "radio",
                                                            name: "{name}",
                                                            value: "{c.value}",
                                                            checked: current.contains(&c.value),
                                                            onchange: {
                                                                let (id, value) = (id.clone(), c.value.clone());
                                                                move |_| send(QuestionnaireEvent::Set { id: id.clone(), values: vec![value.clone()] })
                                                            },
                                                        }
                                                        span { "{c.label}" }
                                                    }
                                                }
                                            },
                                            Kind::Multi(choices) => rsx! {
                                                for c in choices {
                                                    label { key: "{c.value}", "data-scope": h::SCOPE, "data-part": "choice",
                                                        input {
                                                            r#type: "checkbox",
                                                            name: "{name}",
                                                            value: "{c.value}",
                                                            checked: current.contains(&c.value),
                                                            onchange: {
                                                                let (id, value) = (id.clone(), c.value.clone());
                                                                move |_| send(QuestionnaireEvent::Toggle { id: id.clone(), value: value.clone() })
                                                            },
                                                        }
                                                        span { "{c.label}" }
                                                    }
                                                }
                                            },
                                            Kind::Text { multiline } => {
                                                let text = current.first().cloned().unwrap_or_default();
                                                let mut set = {
                                                    let id = id.clone();
                                                    move |v: String| {
                                                        let values = if v.trim().is_empty() { vec![] } else { vec![v] };
                                                        send(QuestionnaireEvent::Set { id: id.clone(), values });
                                                    }
                                                };
                                                let mut set2 = set.clone();
                                                rsx! {
                                                    if multiline {
                                                        textarea {
                                                            "data-scope": h::SCOPE,
                                                            "data-part": "input",
                                                            rows: "4",
                                                            "aria-describedby": "{ids.error}",
                                                            value: "{text}",
                                                            oninput: move |e| set(e.value()),
                                                        }
                                                    } else {
                                                        input {
                                                            r#type: "text",
                                                            "data-scope": h::SCOPE,
                                                            "data-part": "input",
                                                            "aria-describedby": "{ids.error}",
                                                            value: "{text}",
                                                            oninput: move |e| set2(e.value()),
                                                        }
                                                    }
                                                }
                                            }
                                            Kind::Scale { min, max, min_label, max_label } => rsx! {
                                                div { "data-scope": h::SCOPE, "data-part": "scale",
                                                    for n in min..=max {
                                                        label { key: "{n}", "data-scope": h::SCOPE, "data-part": "scale-option",
                                                            input {
                                                                r#type: "radio",
                                                                name: "{name}",
                                                                value: "{n}",
                                                                checked: current.first() == Some(&n.to_string()),
                                                                onchange: {
                                                                    let id = id.clone();
                                                                    move |_| send(QuestionnaireEvent::Set { id: id.clone(), values: vec![n.to_string()] })
                                                                },
                                                            }
                                                            span { "{n}" }
                                                        }
                                                    }
                                                }
                                                div { "data-scope": h::SCOPE, "data-part": "scale-labels", "aria-hidden": "true",
                                                    span { "{min_label}" }
                                                    span { "{max_label}" }
                                                }
                                            },
                                        }
                                    }
                                    p { "data-scope": h::SCOPE, "data-part": "error", id: "{ids.error}", role: "alert",
                                        if missing {
                                            {h::required_text()}
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Step::Review => rsx! {
                    section {
                        id: "{ids.step}",
                        tabindex: "-1",
                        "data-scope": h::SCOPE,
                        "data-part": "review",
                        "aria-labelledby": "{ids.step}-title",
                        onmounted: focus_step,
                        h2 { id: "{ids.step}-title", "data-scope": h::SCOPE, "data-part": "prompt", "Review your answers" }
                        dl { "data-scope": h::SCOPE, "data-part": "summary",
                            for (index , prompt , text) in review {
                                div { key: "{index}", "data-scope": h::SCOPE, "data-part": "answer",
                                    dt { "{prompt}" }
                                    dd { "{text}" }
                                    button {
                                        r#type: "button",
                                        "data-scope": h::SCOPE,
                                        "data-part": "edit",
                                        onclick: move |_| send(QuestionnaireEvent::GoTo(index)),
                                        "Edit"
                                    }
                                }
                            }
                        }
                    }
                },
                Step::Done => rsx! {
                    div {
                        id: "{ids.step}",
                        tabindex: "-1",
                        "data-scope": h::SCOPE,
                        "data-part": "done",
                        role: "status",
                        onmounted: focus_step,
                        p { "{done_message}" }
                        Button { variant: ButtonVariant::Secondary, on_click: move |_| send(QuestionnaireEvent::Restart), "Start over" }
                    }
                },
            }
            div { "data-scope": h::SCOPE, "data-part": "nav", hidden: done,
                Button {
                    variant: ButtonVariant::Ghost,
                    disabled: s.step == Step::Question && position <= 1,
                    on_click: move |_| send(QuestionnaireEvent::Back),
                    "Back"
                }
                Button { button_type: ButtonType::Submit, variant: ButtonVariant::Primary, "{forward}" }
            }
        }
    }
}
