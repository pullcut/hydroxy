//! Wizard connector. Anatomy: root, steps (an `<ol>`), step (an `<li>`),
//! trigger (the step's button), indicator, label, panel, footer, back and
//! next. Every part carries `data-scope="wizard"` and its `data-part`; steps
//! carry `data-state="complete"`, `"current"` or `"upcoming"`.
//!
//! Step triggers are native buttons, disabled while unreachable, so Tab, Enter
//! and Space work without a key map. The current one has `aria-current="step"`.

pub use hydroxy_machines::wizard::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "wizard";

/// Attributes for one step's trigger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepAttrs {
    /// `id`, which labels the panel while current.
    pub id: String,
    /// `aria-current="step"` on the current step.
    pub aria_current: Option<&'static str>,
    /// Native `disabled`: the step cannot be visited yet.
    pub disabled: bool,
    /// `"complete"`, `"current"` or `"upcoming"`.
    pub data_state: &'static str,
}

/// The trigger `id` for step `index`.
#[must_use]
pub fn step_id(root: &str, index: usize) -> String {
    format!("{root}-step-{index}")
}

/// The panel `id`.
#[must_use]
pub fn panel_id(root: &str) -> String {
    format!("{root}-panel")
}

/// `data-state` for step `index`. The current step wins over complete.
#[must_use]
pub fn step_state(state: &WizardState, index: usize) -> &'static str {
    if index == state.current.value {
        "current"
    } else if state.is_complete(index) {
        "complete"
    } else {
        "upcoming"
    }
}

/// Attributes for step `index`.
#[must_use]
pub fn step_attrs(root: &str, state: &WizardState, input: &WizardInput, index: usize) -> StepAttrs {
    StepAttrs {
        id: step_id(root, index),
        aria_current: (index == state.current.value).then_some("step"),
        disabled: !state.can_visit(index, input),
        data_state: step_state(state, index),
    }
}

/// Screen reader suffix for a step: `", current"`, `", done"` or nothing.
#[must_use]
pub fn step_sr(state: &WizardState, index: usize) -> &'static str {
    match step_state(state, index) {
        "current" => ", current",
        "complete" => ", done",
        _ => "",
    }
}

/// Whether the current step is the last.
#[must_use]
pub fn is_last(state: &WizardState, input: &WizardInput) -> bool {
    state.current.value + 1 >= input.count
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::Controllable;

    #[test]
    fn steps_reflect_progress_and_reachability() {
        let input = WizardInput { count: 3, linear: true, ..WizardInput::default() };
        let state = WizardState { current: Controllable::new(1, false), completed: vec![true, false, false] };
        assert_eq!(step_state(&state, 0), "complete");
        let current = step_attrs("w", &state, &input, 1);
        assert_eq!((current.aria_current, current.data_state, current.disabled), (Some("step"), "current", false));
        assert!(step_attrs("w", &state, &input, 2).disabled);
        assert_eq!(step_sr(&state, 0), ", done");
        assert!(!is_last(&state, &input));
    }
}
