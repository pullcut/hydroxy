//! Questionnaire connector. Anatomy: root, progress, bar, step, question,
//! prompt, hint, choice, input, scale, error, nav, back, next, review, answer,
//! edit, submit, done, live. Every part carries `data-scope="questionnaire"`
//! and its `data-part`.

pub use hydroxy_machines::questionnaire::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "questionnaire";

/// Element ids for one questionnaire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The active fieldset or review heading, which takes focus on each step.
    pub step: String,
    /// The validation message, referenced by `aria-describedby`.
    pub error: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { step: format!("{root}-step"), error: format!("{root}-error") }
}

/// (position, total) among visible questions. On the review, position equals total.
#[must_use]
pub fn progress(state: &QuestionnaireState, input: &QuestionnaireInput) -> (usize, usize) {
    let path = input.path(&state.answers);
    let total = path.len();
    match state.step {
        Step::Question => (path.iter().position(|&i| i == state.current).map_or(1, |p| p + 1), total),
        Step::Review | Step::Done => (total, total),
    }
}

/// The progress text: "Question 2 of 4", "Review" or "Done".
#[must_use]
pub fn progress_text(state: &QuestionnaireState, input: &QuestionnaireInput) -> String {
    let (n, total) = progress(state, input);
    match state.step {
        Step::Question => format!("Question {n} of {total}"),
        Step::Review => "Review your answers".to_string(),
        Step::Done => "Done".to_string(),
    }
}

/// The answer as readable text: choice labels for choice questions.
#[must_use]
pub fn answer_text(question: &Question, answers: &Answers) -> String {
    let Some(values) = answers.get(&question.id).filter(|v| !v.is_empty()) else {
        return "Skipped".to_string();
    };
    let choices = match &question.kind {
        Kind::Single(c) | Kind::Multi(c) => c.as_slice(),
        Kind::Text { .. } | Kind::Scale { .. } => &[],
    };
    values
        .iter()
        .map(|v| choices.iter().find(|c| c.value == *v).map_or(v.as_str(), |c| c.label.as_str()))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The message under a required question that was left empty.
#[must_use]
pub const fn required_text() -> &'static str {
    "Answer this to continue."
}

/// Label of the forward button.
#[must_use]
pub fn next_label(state: &QuestionnaireState, input: &QuestionnaireInput) -> &'static str {
    let (n, total) = progress(state, input);
    if n >= total { "Review" } else { "Next" }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> QuestionnaireInput {
        QuestionnaireInput {
            questions: vec![
                Question::single("a", "A?", &[("x", "Ex"), ("y", "Why")]),
                Question::text("b", "B?", false).show_if("a", Test::Equals("y".into())),
            ],
            ..QuestionnaireInput::default()
        }
    }

    #[test]
    fn progress_counts_visible_questions_only() {
        let input = input();
        let mut s = QuestionnaireState::default();
        assert_eq!(progress(&s, &input), (1, 1));
        assert_eq!(next_label(&s, &input), "Review");
        s.answers.insert("a".into(), vec!["y".into()]);
        assert_eq!(progress_text(&s, &input), "Question 1 of 2");
        assert_eq!(next_label(&s, &input), "Next");
    }

    #[test]
    fn answers_read_as_labels() {
        let input = input();
        let mut answers = Answers::new();
        assert_eq!(answer_text(&input.questions[0], &answers), "Skipped");
        answers.insert("a".into(), vec!["y".into()]);
        assert_eq!(answer_text(&input.questions[0], &answers), "Why");
    }
}
