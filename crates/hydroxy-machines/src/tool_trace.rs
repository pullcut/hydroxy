//! Tool trace: the steps of an agent run (thinking, tool calls, messages, errors),
//! nested where a step started a sub-agent. The host appends and updates steps as
//! the run goes; this module decides which steps are open, sums up the run, and
//! words the status changes screen readers hear.
//!
//! Running and failed steps open by themselves until the user opens or closes them;
//! after that, the user's choice sticks.

use std::collections::HashMap;

use hydroxy_statechart::{Machine, Transition};

/// How far a step has got.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StepStatus {
    /// Still going.
    #[default]
    Running,
    /// Finished.
    Done,
    /// Finished with an error.
    Failed,
}

impl StepStatus {
    /// The `data-status` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            StepStatus::Running => "running",
            StepStatus::Done => "done",
            StepStatus::Failed => "failed",
        }
    }

    /// The visible and spoken word.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            StepStatus::Running => "Running",
            StepStatus::Done => "Done",
            StepStatus::Failed => "Failed",
        }
    }
}

/// What a step is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepKind {
    /// Reasoning text.
    Thinking {
        /// The reasoning.
        text: String,
    },
    /// A tool call.
    ToolCall {
        /// Tool name, such as `search_docs`.
        name: String,
        /// Arguments, usually JSON.
        arguments: String,
        /// The result once there is one, usually JSON.
        result: Option<String>,
    },
    /// Text the agent wrote.
    Message {
        /// The text.
        text: String,
    },
    /// Something went wrong.
    Error {
        /// What went wrong.
        message: String,
    },
    /// A sub-agent; its steps are the children.
    Agent {
        /// The sub-agent's name or task.
        name: String,
    },
}

impl StepKind {
    /// The `data-kind` value.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            StepKind::Thinking { .. } => "thinking",
            StepKind::ToolCall { .. } => "tool-call",
            StepKind::Message { .. } => "message",
            StepKind::Error { .. } => "error",
            StepKind::Agent { .. } => "agent",
        }
    }
}

/// One step of a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceStep {
    /// Stable id, unique within the trace.
    pub id: String,
    /// What it is.
    pub kind: StepKind,
    /// How far it has got.
    pub status: StepStatus,
    /// How long it took, once known.
    pub duration_ms: Option<u64>,
    /// Steps it started (a sub-agent's steps).
    pub children: Vec<TraceStep>,
}

impl TraceStep {
    /// A running step with no children.
    #[must_use]
    pub fn new(id: impl Into<String>, kind: StepKind) -> Self {
        Self { id: id.into(), kind, status: StepStatus::Running, duration_ms: None, children: Vec::new() }
    }

    /// The same step, finished in `ms`.
    #[must_use]
    pub fn done(mut self, ms: u64) -> Self {
        self.status = StepStatus::Done;
        self.duration_ms = Some(ms);
        self
    }

    /// The same step, failed after `ms`.
    #[must_use]
    pub fn failed(mut self, ms: u64) -> Self {
        self.status = StepStatus::Failed;
        self.duration_ms = Some(ms);
        self
    }

    /// The same step with these children.
    #[must_use]
    pub fn with_children(mut self, children: Vec<TraceStep>) -> Self {
        self.children = children;
        self
    }

    /// The step's title: "Thinking", the tool or agent name, "Message" or "Error".
    #[must_use]
    pub fn title(&self) -> &str {
        match &self.kind {
            StepKind::Thinking { .. } => "Thinking",
            StepKind::ToolCall { name, .. } | StepKind::Agent { name } => name,
            StepKind::Message { .. } => "Message",
            StepKind::Error { .. } => "Error",
        }
    }

    /// The step and every step below it, depth first.
    pub fn walk(&self) -> Box<dyn Iterator<Item = &TraceStep> + '_> {
        Box::new(std::iter::once(self).chain(self.children.iter().flat_map(TraceStep::walk)))
    }
}

/// Find a step anywhere in `steps` by id.
#[must_use]
pub fn find<'a>(steps: &'a [TraceStep], id: &str) -> Option<&'a TraceStep> {
    steps.iter().flat_map(TraceStep::walk).find(|s| s.id == id)
}

/// A duration as people read it: "840 ms", "4.2 s", "2 m 05 s".
#[must_use]
pub fn format_duration(ms: u64) -> String {
    match ms {
        0..1_000 => format!("{ms} ms"),
        1_000..60_000 => {
            let tenths = (ms + 50) / 100;
            format!("{}.{} s", tenths / 10, tenths % 10)
        }
        _ => {
            let s = (ms + 500) / 1_000;
            format!("{} m {:02} s", s / 60, s % 60)
        }
    }
}

/// The trace's heading: "6 steps, 3 tool calls, 4.2 s" or "Running, 6 steps so far".
#[must_use]
pub fn summary(steps: &[TraceStep]) -> String {
    let all: Vec<&TraceStep> = steps.iter().flat_map(TraceStep::walk).collect();
    let n = all.len();
    let noun = if n == 1 { "step" } else { "steps" };
    if all.iter().any(|s| s.status == StepStatus::Running) {
        return format!("Running, {n} {noun} so far");
    }
    let tools = all.iter().filter(|s| matches!(s.kind, StepKind::ToolCall { .. })).count();
    let failed = all.iter().filter(|s| s.status == StepStatus::Failed).count();
    let total: u64 = steps.iter().filter_map(|s| s.duration_ms).sum();
    let mut parts = vec![format!("{n} {noun}")];
    if tools > 0 {
        parts.push(format!("{tools} tool call{}", if tools == 1 { "" } else { "s" }));
    }
    if failed > 0 {
        parts.push(format!("{failed} failed"));
    }
    if total > 0 {
        parts.push(format_duration(total));
    }
    parts.join(", ")
}

/// What to announce after the host updated the steps: steps that finished or failed
/// since `before`, such as "`search_docs` done. Error failed". `None` when nothing did.
#[must_use]
pub fn announcement(before: &[TraceStep], after: &[TraceStep]) -> Option<String> {
    let old: HashMap<&str, StepStatus> = before.iter().flat_map(TraceStep::walk).map(|s| (s.id.as_str(), s.status)).collect();
    let news: Vec<String> = after
        .iter()
        .flat_map(TraceStep::walk)
        .filter(|s| s.status != StepStatus::Running && old.get(s.id.as_str()).is_none_or(|o| *o != s.status))
        .map(|s| format!("{} {}", s.title(), s.status.label().to_lowercase()))
        .collect();
    (!news.is_empty()).then(|| news.join(". "))
}

/// Props for [`ToolTraceMachine`]: none beyond the steps the adapter passes per event.
pub type ToolTraceInput = ();

/// What [`ToolTraceMachine`] remembers: the user's open or closed choice per step.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolTraceState {
    /// Steps the user opened (`true`) or closed (`false`).
    pub chosen: HashMap<String, bool>,
}

/// Whether `step` shows its body: the user's choice, else open while running or failed.
#[must_use]
pub fn is_open(state: &ToolTraceState, step: &TraceStep) -> bool {
    state.chosen.get(&step.id).copied().unwrap_or(step.status != StepStatus::Done)
}

/// Inputs to [`ToolTraceMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolTraceEvent {
    /// The step's header was pressed. Carries whether it was open.
    Toggle {
        /// The step.
        id: String,
        /// Whether it showed its body before the press.
        was_open: bool,
    },
    /// Open or close every step at once.
    SetAll {
        /// Every step id in the trace.
        ids: Vec<String>,
        /// Open (`true`) or close.
        open: bool,
    },
}

/// Which steps are open.
#[derive(Debug, Clone, Copy)]
pub struct ToolTraceMachine;

impl Machine for ToolTraceMachine {
    type Input = ToolTraceInput;
    type State = ToolTraceState;
    type Event = ToolTraceEvent;
    type Command = ();

    fn initial((): &()) -> ToolTraceState {
        ToolTraceState::default()
    }

    fn transition(state: &ToolTraceState, event: ToolTraceEvent, (): &()) -> Transition<ToolTraceState, ()> {
        let mut next = state.clone();
        match event {
            ToolTraceEvent::Toggle { id, was_open } => {
                next.chosen.insert(id, !was_open);
            }
            ToolTraceEvent::SetAll { ids, open } => {
                next.chosen = ids.into_iter().map(|id| (id, open)).collect();
            }
        }
        Transition::to(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(id: &str) -> TraceStep {
        TraceStep::new(id, StepKind::ToolCall { name: format!("tool_{id}"), arguments: "{}".into(), result: None })
    }

    #[test]
    fn durations_read_naturally() {
        assert_eq!(format_duration(840), "840 ms");
        assert_eq!(format_duration(4_249), "4.2 s");
        assert_eq!(format_duration(59_990), "60.0 s");
        assert_eq!(format_duration(125_000), "2 m 05 s");
    }

    #[test]
    fn summary_counts_nested_steps() {
        let steps = vec![
            TraceStep::new("t", StepKind::Thinking { text: "plan".into() }).done(300),
            TraceStep::new("a", StepKind::Agent { name: "researcher".into() })
                .with_children(vec![tool("1").done(500), tool("2").failed(200)])
                .failed(900),
        ];
        assert_eq!(summary(&steps), "4 steps, 2 tool calls, 2 failed, 1.2 s");
        assert_eq!(summary(&[tool("x")]), "Running, 1 step so far");
        assert!(find(&steps, "2").is_some());
    }

    #[test]
    fn running_and_failed_open_until_the_user_chooses() {
        let s = ToolTraceState::default();
        assert!(is_open(&s, &tool("x")));
        assert!(!is_open(&s, &tool("x").done(1)));
        assert!(is_open(&s, &tool("x").failed(1)));
        let t = ToolTraceMachine::transition(&s, ToolTraceEvent::Toggle { id: "x".into(), was_open: true }, &()).resolve(&s);
        assert!(!is_open(&t.state, &tool("x")), "the user's close sticks while it runs");
        let all = ToolTraceMachine::transition(&t.state, ToolTraceEvent::SetAll { ids: vec!["x".into()], open: true }, &()).resolve(&t.state);
        assert!(is_open(&all.state, &tool("x").done(1)));
    }

    #[test]
    fn announces_only_new_outcomes() {
        let before = vec![tool("1"), tool("2").done(5)];
        let after = vec![tool("1").done(9), tool("2").done(5), TraceStep::new("3", StepKind::Error { message: "x".into() }).failed(1)];
        assert_eq!(announcement(&before, &after).as_deref(), Some("tool_1 done. Error failed"));
        assert_eq!(announcement(&after, &after), None);
    }
}
