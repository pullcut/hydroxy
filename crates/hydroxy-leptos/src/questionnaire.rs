//! `<Questionnaire>`: one question at a time, with branching and a review step.

use hydroxy_headless::questionnaire::{
    self as h, Answer, Answers, Kind, Question, QuestionnaireCommand, QuestionnaireEvent, QuestionnaireInput,
    QuestionnaireMachine, Step,
};
use hydroxy_statechart::Machine;
use leptos::either::{EitherOf3, EitherOf4};
use leptos::prelude::*;

use crate::button::{Button, ButtonType, ButtonVariant};
use crate::focus::focus_id;

pub use h::{Answer as QuestionnaireAnswer, Choice, Condition, Question as QuestionnaireQuestion, Test};

/// A form that asks its questions one by one. Questions can depend on earlier
/// answers, a progress bar follows the visible path, and a review step lets
/// people jump back to edit before submitting.
#[component]
#[allow(clippy::too_many_lines, reason = "one view per step")]
pub fn Questionnaire(
    /// Accessible name for the form.
    #[prop(into)]
    label: String,
    /// The questions, in order. See [`Question`] for the builders.
    questions: Vec<Question>,
    /// Controlled answers. Pair with `on_answers_change`.
    #[prop(optional, into)]
    answers: MaybeProp<Answers>,
    /// Initial answers when uncontrolled.
    #[prop(optional)]
    default_answers: Answers,
    /// Text of the submit button on the review. Default "Submit".
    #[prop(default = "Submit")]
    submit_label: &'static str,
    /// Message shown after submitting.
    #[prop(default = "Thanks. Your answers are in.")]
    done_message: &'static str,
    /// Called with the new answers whenever they change or a change is requested.
    #[prop(optional, into)]
    on_answers_change: Option<Callback<Answers>>,
    /// Called with the answers to visible questions when the user submits.
    #[prop(optional, into)]
    on_submit: Option<Callback<Vec<Answer>>>,
) -> impl IntoView {
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let questions = StoredValue::new(questions);
    let input = Memo::new(move |_| QuestionnaireInput {
        questions: questions.get_value(),
        answers: answers.get(),
        default_answers: default_answers.clone(),
    });
    let state = RwSignal::new(input.with_untracked(QuestionnaireMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| QuestionnaireMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let send = move |event: QuestionnaireEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| QuestionnaireMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        for command in step.commands {
            match command {
                QuestionnaireCommand::AnswersChange(a) => {
                    if let Some(cb) = on_answers_change {
                        cb.run(a);
                    }
                }
                QuestionnaireCommand::Submit(a) => {
                    if let Some(cb) = on_submit {
                        cb.run(a);
                    }
                }
            }
        }
    };

    // Move focus to the new step whenever it changes (not on first render).
    let place = Memo::new(move |_| state.with(|s| (s.step, s.current)));
    let step_id = ids.step.clone();
    Effect::new(move |previous: Option<()>| {
        place.track();
        if previous.is_some() {
            let target = step_id.clone();
            request_animation_frame(move || focus_id(&target));
        }
    });

    let values_of = move |id: &str| state.with(|s| s.answers.get(id).cloned().unwrap_or_default());
    let progress = move || input.with(|i| state.with(|s| h::progress(s, i)));

    let question_view = {
        let ids = ids.clone();
        move |index: usize| {
            let q = questions.with_value(|qs| qs.get(index).cloned())?;
            let id = q.id.clone();
            let name = format!("{}-{}", ids.step, q.id);
            let body = match q.kind.clone() {
                Kind::Single(choices) | Kind::Multi(choices) => {
                    let multi = matches!(q.kind, Kind::Multi(_));
                    EitherOf4::A(choices
                        .into_iter()
                        .map(|c| {
                            let (qid, value, name) = (id.clone(), c.value.clone(), name.clone());
                            let checked = {
                                let (qid, value) = (qid.clone(), value.clone());
                                move || values_of(&qid).contains(&value)
                            };
                            view! {
                                <label data-scope=h::SCOPE data-part="choice">
                                    <input
                                        type=if multi { "checkbox" } else { "radio" }
                                        name=name
                                        value=value.clone()
                                        prop:checked=checked
                                        on:change=move |_| {
                                            send(if multi {
                                                QuestionnaireEvent::Toggle { id: qid.clone(), value: value.clone() }
                                            } else {
                                                QuestionnaireEvent::Set { id: qid.clone(), values: vec![value.clone()] }
                                            });
                                        }
                                    />
                                    <span>{c.label}</span>
                                </label>
                            }
                        })
                        .collect_view())
                }
                Kind::Text { multiline } => {
                    let qid = id.clone();
                    let value = {
                        let qid = id.clone();
                        move || values_of(&qid).first().cloned().unwrap_or_default()
                    };
                    let set = move |text: String| {
                        send(QuestionnaireEvent::Set {
                            id: qid.clone(),
                            values: if text.trim().is_empty() { vec![] } else { vec![text] },
                        });
                    };
                    if multiline {
                        EitherOf4::B(view! {
                            <textarea
                                data-scope=h::SCOPE
                                data-part="input"
                                rows="4"
                                aria-describedby=ids.error.clone()
                                prop:value=value
                                on:input=move |ev| set(event_target_value(&ev))
                            ></textarea>
                        })
                    } else {
                        EitherOf4::C(view! {
                            <input
                                type="text"
                                data-scope=h::SCOPE
                                data-part="input"
                                aria-describedby=ids.error.clone()
                                prop:value=value
                                on:input=move |ev| set(event_target_value(&ev))
                            />
                        })
                    }
                }
                Kind::Scale { min, max, min_label, max_label } => {
                    let options = (min..=max)
                        .map(|n| {
                            let (qid, name, text) = (id.clone(), name.clone(), n.to_string());
                            let checked = {
                                let (qid, text) = (id.clone(), text.clone());
                                move || values_of(&qid).first() == Some(&text)
                            };
                            view! {
                                <label data-scope=h::SCOPE data-part="scale-option">
                                    <input
                                        type="radio"
                                        name=name
                                        value=text.clone()
                                        prop:checked=checked
                                        on:change=move |_| {
                                            send(QuestionnaireEvent::Set { id: qid.clone(), values: vec![text.clone()] });
                                        }
                                    />
                                    <span>{n}</span>
                                </label>
                            }
                        })
                        .collect_view();
                    EitherOf4::D(view! {
                        <div data-scope=h::SCOPE data-part="scale">{options}</div>
                        <div data-scope=h::SCOPE data-part="scale-labels" aria-hidden="true">
                            <span>{min_label}</span>
                            <span>{max_label}</span>
                        </div>
                    })
                }
            };
            let required = q.required;
            let qid = q.id.clone();
            let show_error = move || {
                required
                    && state.with(|s| s.attempted)
                    && state.with(|s| s.answers.get(&qid).is_none_or(|v| v.iter().all(|x| x.trim().is_empty())))
            };
            Some(view! {
                <fieldset id=ids.step.clone() tabindex="-1" data-scope=h::SCOPE data-part="question">
                    <legend data-scope=h::SCOPE data-part="prompt">
                        {q.prompt.clone()}
                        {required.then(|| view! { <span aria-hidden="true">" *"</span> })}
                    </legend>
                    {q.hint.clone().map(|t| view! { <p data-scope=h::SCOPE data-part="hint">{t}</p> })}
                    <div data-scope=h::SCOPE data-part="controls" data-kind=match q.kind {
                        Kind::Single(_) => "single",
                        Kind::Multi(_) => "multi",
                        Kind::Text { .. } => "text",
                        Kind::Scale { .. } => "scale",
                    }>{body}</div>
                    <p data-scope=h::SCOPE data-part="error" id=ids.error.clone() role="alert">
                        {move || show_error().then_some(h::required_text())}
                    </p>
                </fieldset>
            })
        }
    };

    let review_view = {
        let step_id = ids.step.clone();
        move || {
            let items = questions.with_value(|qs| {
                state.with_untracked(|s| {
                    input
                        .with_untracked(|i| i.path(&s.answers))
                        .into_iter()
                        .map(|i| (i, qs[i].prompt.clone(), h::answer_text(&qs[i], &s.answers)))
                        .collect::<Vec<_>>()
                })
            });
            view! {
                <section id=step_id.clone() tabindex="-1" data-scope=h::SCOPE data-part="review" aria-labelledby=format!("{step_id}-title")>
                    <h2 id=format!("{step_id}-title") data-scope=h::SCOPE data-part="prompt">"Review your answers"</h2>
                    <dl data-scope=h::SCOPE data-part="summary">
                        {items
                            .into_iter()
                            .map(|(index, prompt, text)| {
                                view! {
                                    <div data-scope=h::SCOPE data-part="answer">
                                        <dt>{prompt}</dt>
                                        <dd>{text}</dd>
                                        <button
                                            type="button"
                                            data-scope=h::SCOPE
                                            data-part="edit"
                                            on:click=move |_| send(QuestionnaireEvent::GoTo(index))
                                        >
                                            "Edit"
                                        </button>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </dl>
                </section>
            }
        }
    };

    let done = move || state.with(|s| s.step == Step::Done);
    view! {
        <form
            data-scope=h::SCOPE
            data-part="root"
            aria-label=label
            novalidate
            data-state=move || match state.with(|s| s.step) {
                Step::Question => "question",
                Step::Review => "review",
                Step::Done => "done",
            }
            on:submit=move |ev| {
                ev.prevent_default();
                send(if state.with_untracked(|s| s.step) == Step::Review {
                    QuestionnaireEvent::Submit
                } else {
                    QuestionnaireEvent::Next
                });
            }
        >
            <div data-scope=h::SCOPE data-part="progress" hidden=done>
                <progress
                    data-scope=h::SCOPE
                    data-part="bar"
                    max=move || progress().1.max(1)
                    value=move || progress().0
                    aria-label="Progress"
                ></progress>
                <span data-scope=h::SCOPE data-part="step" aria-live="polite">
                    {move || input.with(|i| state.with(|s| h::progress_text(s, i)))}
                </span>
            </div>
            {move || match place.get() {
                (Step::Question, index) => EitherOf3::A(question_view(index)),
                (Step::Review, _) => EitherOf3::B(review_view()),
                (Step::Done, _) => {
                    let step_id = ids.step.clone();
                    EitherOf3::C(view! {
                        <div id=step_id tabindex="-1" data-scope=h::SCOPE data-part="done" role="status">
                            <p>{done_message}</p>
                            <Button variant=ButtonVariant::Secondary on_click=move |_| send(QuestionnaireEvent::Restart)>
                                "Start over"
                            </Button>
                        </div>
                    })
                }
            }}
            <div data-scope=h::SCOPE data-part="nav" hidden=done>
                <Button
                    variant=ButtonVariant::Ghost
                    disabled=Signal::derive(move || state.with(|s| s.step == Step::Question) && progress().0 <= 1)
                    on_click=move |_| send(QuestionnaireEvent::Back)
                >
                    "Back"
                </Button>
                <Button button_type=ButtonType::Submit variant=ButtonVariant::Primary>
                    {move || match state.with(|s| s.step) {
                        Step::Review => submit_label,
                        _ => input.with(|i| state.with(|s| h::next_label(s, i))),
                    }}
                </Button>
            </div>
        </form>
    }
}
