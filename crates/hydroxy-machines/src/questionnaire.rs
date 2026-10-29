//! Questionnaire: a one-question-at-a-time flow with branching and a review step.
//!
//! A question can carry a [`Condition`] on an earlier answer and is skipped
//! while it does not hold. Next checks that a required question is answered;
//! after the last visible question comes a review, and Submit delivers the
//! answers to visible questions only.

use std::collections::BTreeMap;

use hydroxy_statechart::{Controllable, Machine, Transition};

/// Answers by question id. Single choice, text and scale hold one value; multi choice holds many.
pub type Answers = BTreeMap<String, Vec<String>>;

/// One option of a choice question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// Stored in the answer.
    pub value: String,
    /// Shown to the user.
    pub label: String,
}

impl Choice {
    /// A choice.
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self { value: value.into(), label: label.into() }
    }
}

/// What kind of answer a question takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// Pick one.
    Single(Vec<Choice>),
    /// Pick any number.
    Multi(Vec<Choice>),
    /// Free text.
    Text {
        /// A textarea rather than a one-line input.
        multiline: bool,
    },
    /// A whole number on a range.
    Scale {
        /// Lowest value.
        min: i32,
        /// Highest value.
        max: i32,
        /// Caption for the low end.
        min_label: String,
        /// Caption for the high end.
        max_label: String,
    },
}

/// How a [`Condition`] reads an answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Test {
    /// The answer is exactly this value.
    Equals(String),
    /// There is an answer and it is not this value.
    NotEquals(String),
    /// The answer includes this value (multi choice).
    Includes(String),
    /// The answer is a number at least this big.
    AtLeast(i32),
}

/// Show a question only when an earlier answer passes a test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Condition {
    /// Id of the question to look at.
    pub question: String,
    /// What the answer must satisfy.
    pub test: Test,
}

impl Condition {
    fn holds(&self, answers: &Answers) -> bool {
        let values = answers.get(&self.question).map_or(&[][..], Vec::as_slice);
        match &self.test {
            Test::Equals(v) => values.first() == Some(v),
            Test::NotEquals(v) => values.first().is_some_and(|x| x != v),
            Test::Includes(v) => values.contains(v),
            Test::AtLeast(n) => values.first().and_then(|x| x.parse::<i32>().ok()).is_some_and(|x| x >= *n),
        }
    }
}

/// One question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// Unique id; the key in [`Answers`].
    pub id: String,
    /// The question text.
    pub prompt: String,
    /// Help text under the prompt.
    pub hint: Option<String>,
    /// The answer type.
    pub kind: Kind,
    /// Must be answered to continue.
    pub required: bool,
    /// Skipped unless this holds.
    pub show_if: Option<Condition>,
}

impl Question {
    fn new(id: &str, prompt: &str, kind: Kind) -> Self {
        Self { id: id.into(), prompt: prompt.into(), hint: None, kind, required: false, show_if: None }
    }

    /// Pick one of `choices`, given as (value, label).
    #[must_use]
    pub fn single(id: &str, prompt: &str, choices: &[(&str, &str)]) -> Self {
        Self::new(id, prompt, Kind::Single(choices.iter().map(|&(v, l)| Choice::new(v, l)).collect()))
    }

    /// Pick any of `choices`, given as (value, label).
    #[must_use]
    pub fn multi(id: &str, prompt: &str, choices: &[(&str, &str)]) -> Self {
        Self::new(id, prompt, Kind::Multi(choices.iter().map(|&(v, l)| Choice::new(v, l)).collect()))
    }

    /// Free text; `multiline` for a textarea.
    #[must_use]
    pub fn text(id: &str, prompt: &str, multiline: bool) -> Self {
        Self::new(id, prompt, Kind::Text { multiline })
    }

    /// A whole number from `min` to `max` with captions for the ends.
    #[must_use]
    pub fn scale(id: &str, prompt: &str, min: i32, max: i32, min_label: &str, max_label: &str) -> Self {
        Self::new(id, prompt, Kind::Scale { min, max, min_label: min_label.into(), max_label: max_label.into() })
    }

    /// Must be answered.
    #[must_use]
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Help text.
    #[must_use]
    pub fn hint(mut self, hint: &str) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Only ask when the answer to `question` passes `test`.
    #[must_use]
    pub fn show_if(mut self, question: &str, test: Test) -> Self {
        self.show_if = Some(Condition { question: question.into(), test });
        self
    }

    /// Whether `answers` satisfy this question.
    #[must_use]
    pub fn is_answered(&self, answers: &Answers) -> bool {
        answers.get(&self.id).is_some_and(|v| v.iter().any(|x| !x.trim().is_empty()))
    }
}

/// Props for [`QuestionnaireMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QuestionnaireInput {
    /// The questions, in order.
    pub questions: Vec<Question>,
    /// Controlled answers. `Some` at start fixes controlled mode.
    pub answers: Option<Answers>,
    /// Uncontrolled initial answers.
    pub default_answers: Answers,
}

impl QuestionnaireInput {
    /// Indexes of the questions that are visible given `answers`, in order.
    #[must_use]
    pub fn path(&self, answers: &Answers) -> Vec<usize> {
        let mut shown: Vec<&str> = Vec::new();
        let mut out = Vec::new();
        for (i, q) in self.questions.iter().enumerate() {
            let visible = q.show_if.as_ref().is_none_or(|c| shown.contains(&c.question.as_str()) && c.holds(answers));
            if visible {
                shown.push(&q.id);
                out.push(i);
            }
        }
        out
    }
}

/// Where the flow is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Step {
    /// On the question at this index.
    #[default]
    Question,
    /// Looking over the answers.
    Review,
    /// Submitted.
    Done,
}

/// What [`QuestionnaireMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QuestionnaireState {
    /// Which stage the flow is in.
    pub step: Step,
    /// Index of the question on screen while `step` is [`Step::Question`].
    pub current: usize,
    /// The answers. Controlled when the owner holds them.
    pub answers: Controllable<Answers>,
    /// Next was pressed on an unanswered required question.
    pub attempted: bool,
}

/// Inputs to [`QuestionnaireMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionnaireEvent {
    /// Replace a question's answer. Empty values clear it.
    Set {
        /// Question id.
        id: String,
        /// New values.
        values: Vec<String>,
    },
    /// Add or remove one value of a multi choice question.
    Toggle {
        /// Question id.
        id: String,
        /// The value.
        value: String,
    },
    /// Continue to the next visible question, or the review.
    Next,
    /// Go to the previous visible question.
    Back,
    /// Jump to a question by index (from the review).
    GoTo(usize),
    /// Submit from the review.
    Submit,
    /// Clear everything and start over.
    Restart,
}

/// One submitted answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    /// Question id.
    pub id: String,
    /// The values.
    pub values: Vec<String>,
}

/// Side effects from [`QuestionnaireMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionnaireCommand {
    /// Tell the owner the answers changed (or should, when controlled).
    AnswersChange(Answers),
    /// The user submitted: answers to visible questions, in order.
    Submit(Vec<Answer>),
}

/// Questions, answers, branching and review.
#[derive(Debug, Clone, Copy)]
pub struct QuestionnaireMachine;

type Out = Transition<QuestionnaireState, QuestionnaireCommand>;

fn with_answers(mut state: QuestionnaireState, answers: Answers) -> Out {
    let commands = vec![QuestionnaireCommand::AnswersChange(answers.clone())];
    state.answers.set(answers);
    Transition::with(state, commands)
}

impl Machine for QuestionnaireMachine {
    type Input = QuestionnaireInput;
    type State = QuestionnaireState;
    type Event = QuestionnaireEvent;
    type Command = QuestionnaireCommand;

    fn initial(input: &QuestionnaireInput) -> QuestionnaireState {
        let answers = input.answers.clone().unwrap_or_else(|| input.default_answers.clone());
        let first = input.path(&answers).first().copied();
        QuestionnaireState {
            step: if first.is_some() { Step::Question } else { Step::Review },
            current: first.unwrap_or(0),
            answers: Controllable::new(answers, input.answers.is_some()),
            attempted: false,
        }
    }

    #[allow(clippy::too_many_lines, reason = "one arm per event")]
    fn transition(state: &QuestionnaireState, event: QuestionnaireEvent, input: &QuestionnaireInput) -> Out {
        let mut next = state.clone();
        let path = input.path(&state.answers.value);
        let pos = path.iter().position(|&i| i == state.current);
        match event {
            QuestionnaireEvent::Set { id, values } => {
                let mut answers = state.answers.value.clone();
                if values.is_empty() {
                    answers.remove(&id);
                } else {
                    answers.insert(id, values);
                }
                if answers == state.answers.value {
                    return Transition::update(state, next, Vec::new());
                }
                next.attempted = false;
                with_answers(next, answers)
            }
            QuestionnaireEvent::Toggle { id, value } => {
                let mut values = state.answers.value.get(&id).cloned().unwrap_or_default();
                if let Some(at) = values.iter().position(|v| *v == value) {
                    values.remove(at);
                } else {
                    values.push(value);
                }
                let mut answers = state.answers.value.clone();
                if values.is_empty() {
                    answers.remove(&id);
                } else {
                    answers.insert(id, values);
                }
                next.attempted = false;
                with_answers(next, answers)
            }
            QuestionnaireEvent::Next if state.step == Step::Question => {
                let ok = input
                    .questions
                    .get(state.current)
                    .is_none_or(|q| !q.required || q.is_answered(&state.answers.value));
                if !ok {
                    next.attempted = true;
                    return Transition::update(state, next, Vec::new());
                }
                next.attempted = false;
                match pos.and_then(|p| path.get(p + 1)) {
                    Some(&i) => next.current = i,
                    None => next.step = Step::Review,
                }
                Transition::update(state, next, Vec::new())
            }
            QuestionnaireEvent::Back => {
                next.attempted = false;
                match state.step {
                    Step::Question => {
                        if let Some(&i) = pos.and_then(|p| p.checked_sub(1)).and_then(|p| path.get(p)) {
                            next.current = i;
                        }
                    }
                    Step::Review => {
                        if let Some(&i) = path.last() {
                            next.current = i;
                            next.step = Step::Question;
                        }
                    }
                    Step::Done => {}
                }
                Transition::update(state, next, Vec::new())
            }
            QuestionnaireEvent::GoTo(i) if state.step != Step::Done && path.contains(&i) => {
                next.step = Step::Question;
                next.current = i;
                next.attempted = false;
                Transition::update(state, next, Vec::new())
            }
            QuestionnaireEvent::Submit if state.step == Step::Review => {
                let invalid = path
                    .iter()
                    .copied()
                    .find(|&i| input.questions[i].required && !input.questions[i].is_answered(&state.answers.value));
                if let Some(i) = invalid {
                    next.step = Step::Question;
                    next.current = i;
                    next.attempted = true;
                    return Transition::update(state, next, Vec::new());
                }
                next.step = Step::Done;
                let answers = path
                    .iter()
                    .filter_map(|&i| {
                        let id = &input.questions[i].id;
                        state.answers.value.get(id).map(|values| Answer { id: id.clone(), values: values.clone() })
                    })
                    .collect();
                Transition::update(state, next, vec![QuestionnaireCommand::Submit(answers)])
            }
            QuestionnaireEvent::Restart => {
                let mut fresh = Self::initial(&QuestionnaireInput {
                    answers: None,
                    default_answers: Answers::new(),
                    ..input.clone()
                });
                fresh.answers = state.answers.clone();
                fresh.answers.set(Answers::new());
                let had_answers = !state.answers.value.is_empty();
                let mut out = Transition::to(fresh);
                if had_answers {
                    out.commands.push(QuestionnaireCommand::AnswersChange(Answers::new()));
                }
                out
            }
            QuestionnaireEvent::Next | QuestionnaireEvent::GoTo(..) | QuestionnaireEvent::Submit => {
                Transition::update(state, next, Vec::new())
            }
        }
    }

    fn sync(state: &QuestionnaireState, input: &QuestionnaireInput) -> Option<QuestionnaireState> {
        let mut next = state.clone();
        next.answers.sync(input.answers.clone());
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use QuestionnaireEvent as E;
    use hydroxy_statechart::step;

    fn input() -> QuestionnaireInput {
        QuestionnaireInput {
            questions: vec![
                Question::single("role", "Role?", &[("dev", "Developer"), ("other", "Other")]).required(),
                Question::text("which", "Which?", false).show_if("role", Test::Equals("other".into())).required(),
                Question::scale("rating", "Rating?", 1, 5, "Low", "High"),
            ],
            ..QuestionnaireInput::default()
        }
    }

    fn run(events: Vec<E>) -> (QuestionnaireState, Vec<QuestionnaireCommand>) {
        let input = input();
        let mut s = QuestionnaireMachine::initial(&input);
        let mut all = Vec::new();
        for e in events {
            let t = step::<QuestionnaireMachine>(&s, e, &input);
            s = t.state;
            all.extend(t.commands);
        }
        (s, all)
    }

    fn set(id: &str, v: &str) -> E {
        E::Set { id: id.into(), values: vec![v.into()] }
    }

    #[test]
    fn required_blocks_next_and_branch_is_skipped() {
        let (s, _) = run(vec![E::Next]);
        assert!(s.attempted && s.current == 0);
        let (s, _) = run(vec![set("role", "dev"), E::Next]);
        assert_eq!(s.current, 2);
        let (s, _) = run(vec![set("role", "other"), E::Next]);
        assert_eq!(s.current, 1);
    }

    #[test]
    fn back_follows_the_path_and_review_comes_last() {
        let (s, _) = run(vec![set("role", "dev"), E::Next, E::Back]);
        assert_eq!(s.current, 0);
        let (s, _) = run(vec![set("role", "dev"), E::Next, E::Next]);
        assert_eq!(s.step, Step::Review);
        let (s, _) = run(vec![set("role", "dev"), E::Next, E::Next, E::Back]);
        assert_eq!((s.step, s.current), (Step::Question, 2));
    }

    #[test]
    fn submit_drops_hidden_answers() {
        let (s, cmds) = run(vec![
            set("role", "other"),
            set("which", "pilot"),
            set("role", "dev"),
            E::GoTo(2),
            set("rating", "4"),
            E::Next,
            E::Submit,
        ]);
        assert_eq!(s.step, Step::Done);
        let Some(QuestionnaireCommand::Submit(a)) = cmds.last() else { panic!("no submit") };
        assert_eq!(a.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["role", "rating"]);
    }

    #[test]
    fn toggle_and_restart() {
        let input = QuestionnaireInput {
            questions: vec![Question::multi("m", "Many?", &[("a", "A"), ("b", "B")])],
            ..QuestionnaireInput::default()
        };
        let mut s = QuestionnaireMachine::initial(&input);
        for v in ["a", "b", "a"] {
            s = step::<QuestionnaireMachine>(&s, E::Toggle { id: "m".into(), value: v.into() }, &input).state;
        }
        assert_eq!(s.answers.value["m"], ["b"]);
        let s = step::<QuestionnaireMachine>(&s, E::Restart, &input).state;
        assert!(s.answers.value.is_empty());
    }

    #[test]
    fn controlled_answers_wait_for_the_owner() {
        let i = QuestionnaireInput { answers: Some(Answers::new()), ..input() };
        let s = QuestionnaireMachine::initial(&i);
        let t = step::<QuestionnaireMachine>(&s, set("role", "dev"), &i);
        assert!(t.state.answers.value.is_empty());
        assert_eq!(t.commands.len(), 1);
    }
}
