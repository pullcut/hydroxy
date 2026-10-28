//! `ToolTrace`: the steps of an agent run as collapsible rows: thinking, tool
//! calls with their arguments and result, messages, errors, and sub-agents with
//! their own steps nested inside. The host appends and updates steps as the run
//! goes; rows are keyed by id so they keep their place and focus, and finished
//! steps are announced.

use dioxus::prelude::*;
use hydroxy_headless::tool_trace::{self as h, ToolTraceEvent, ToolTraceMachine, ToolTraceState};
use hydroxy_statechart::Machine;

use crate::code_block::CodeBlock;
use crate::icon::{Icon, IconName};
use crate::json_inspector::JsonInspector;

pub use hydroxy_headless::tool_trace::{StepKind, StepStatus, TraceStep};

#[derive(Clone, Copy)]
struct Ctx {
    root: CopyValue<String>,
    state: Signal<ToolTraceState>,
}

impl Ctx {
    fn send(mut self, event: ToolTraceEvent) {
        let step = ToolTraceMachine::transition(&self.state.peek(), event, &());
        if let Some(next) = step.state {
            self.state.set(next);
        }
    }
}

/// An agent run's steps.
#[component]
pub fn ToolTrace(
    /// The steps, in order. Update the vector as the run progresses.
    steps: ReadSignal<Vec<TraceStep>>,
    /// Accessible name for the trace. Default "Agent run".
    #[props(default = "Agent run".to_string())]
    label: String,
) -> Element {
    let ctx = use_context_provider(|| Ctx {
        root: CopyValue::new(hydroxy_headless::next_id(h::SCOPE)),
        state: Signal::new(ToolTraceMachine::initial(&())),
    });
    let mut announce = use_signal(String::new);
    let mut previous = use_signal(|| None::<Vec<TraceStep>>);
    use_effect(move || {
        let now = steps.read().clone();
        if let Some(text) = previous.peek().as_ref().and_then(|p| h::announcement(p, &now)) {
            announce.set(text);
        }
        previous.set(Some(now));
    });

    let list = steps.read().clone();
    let running = list.iter().flat_map(TraceStep::walk).any(|s| s.status == StepStatus::Running);
    let all_open = h::all_open(&ctx.state.read(), &list);
    let all_ids = h::all_ids(&list);

    rsx! {
        section {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "aria-label": label,
            "data-running": running.then_some(""),
            div { "data-scope": h::SCOPE, "data-part": "header",
                span { "data-scope": h::SCOPE, "data-part": "summary", {h::summary(&list)} }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "toggle-all",
                    onclick: move |_| ctx.send(ToolTraceEvent::SetAll { ids: all_ids.clone(), open: !all_open }),
                    if all_open { "Collapse all" } else { "Expand all" }
                }
            }
            StepList { steps: list }
            span { "data-scope": h::SCOPE, "data-part": "live", role: "status", "{announce}" }
        }
    }
}

/// One level of steps.
#[component]
fn StepList(steps: Vec<TraceStep>) -> Element {
    rsx! {
        ol { "data-scope": h::SCOPE, "data-part": "steps",
            for step in steps {
                StepView { key: "{step.id}", step }
            }
        }
    }
}

#[component]
fn StepView(step: TraceStep) -> Element {
    let ctx = try_use_context::<Ctx>().expect("steps render inside ToolTrace");
    let root = ctx.root.peek().clone();
    let open = h::is_open(&ctx.state.read(), &step);
    let (header_id, body_id) = (h::header_id(&root, &step.id), h::body_id(&root, &step.id));
    let id = step.id.clone();
    let running = step.status == StepStatus::Running;
    let body = match step.kind.clone() {
        StepKind::Thinking { text } | StepKind::Message { text } => rsx! {
            p { "data-scope": h::SCOPE, "data-part": "text", "{text}" }
        },
        StepKind::Error { message } => rsx! {
            p { "data-scope": h::SCOPE, "data-part": "text", "data-tone": "danger", "{message}" }
        },
        StepKind::ToolCall { arguments, result, .. } => rsx! {
            Section { title: "Arguments", text: arguments }
            if let Some(r) = result {
                Section { title: "Result", text: r }
            } else if running {
                p { "data-scope": h::SCOPE, "data-part": "waiting", "Waiting for the result" }
            }
        },
        StepKind::Agent { .. } => rsx! {},
    };

    rsx! {
        li {
            "data-scope": h::SCOPE,
            "data-part": "step",
            "data-kind": step.kind.as_str(),
            "data-status": step.status.as_str(),
            "aria-busy": (step.status == StepStatus::Running).then_some("true"),
            button {
                r#type: "button",
                id: header_id,
                "aria-expanded": open.to_string(),
                "aria-controls": body_id.clone(),
                "data-scope": h::SCOPE,
                "data-part": "step-header",
                onclick: move |_| ctx.send(ToolTraceEvent::Toggle { id: id.clone(), was_open: open }),
                span { "data-scope": h::SCOPE, "data-part": "chevron", "aria-hidden": "true",
                    Icon { name: IconName::ChevronRight, size: 14 }
                }
                svg {
                    "data-scope": h::SCOPE,
                    "data-part": "icon",
                    view_box: "0 0 24 24",
                    width: "15",
                    height: "15",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "1.75",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    "aria-hidden": "true",
                    dangerous_inner_html: h::kind_icon(&step.kind),
                }
                span { "data-scope": h::SCOPE, "data-part": "title", {step.title().to_string()} }
                span { "data-scope": h::SCOPE, "data-part": "status", {h::status_text(&step)} }
                span { "data-scope": h::SCOPE, "data-part": "duration", "aria-hidden": "true",
                    {step.duration_ms.map(h::format_duration)}
                }
            }
            span { "data-scope": h::SCOPE, "data-part": "progress", "aria-hidden": "true" }
            div { "data-scope": h::SCOPE, "data-part": "body", id: body_id, hidden: !open,
                {body}
                if !step.children.is_empty() {
                    StepList { steps: step.children.clone() }
                }
            }
        }
    }
}

/// A labelled block of arguments or result. JSON gets the inspector and a copy
/// button (copying it pretty-printed); anything else a code block, which has its own.
#[component]
fn Section(title: &'static str, text: String) -> Element {
    let mut copied = use_signal(|| false);
    if !h::is_json(&text) {
        return rsx! {
            div { "data-scope": h::SCOPE, "data-part": "section",
                span { "data-scope": h::SCOPE, "data-part": "section-title", "{title}" }
                CodeBlock { code: text }
            }
        };
    }
    let pretty = h::pretty(&text);
    let copy = move |_| {
        // Clipboard and the "Copied" reset both run through eval, so this works on web and desktop.
        let mut wait = document::eval(&format!(
            "try {{ await navigator.clipboard.writeText({pretty:?}); }} catch (_) {{}} await new Promise(r => setTimeout(r, 1200)); dioxus.send(true);"
        ));
        copied.set(true);
        spawn(async move {
            let _ = wait.recv::<bool>().await;
            copied.set(false);
        });
    };
    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "section",
            div { "data-scope": h::SCOPE, "data-part": "section-head",
                span { "data-scope": h::SCOPE, "data-part": "section-title", "{title}" }
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "copy",
                    "aria-label": "Copy {title.to_lowercase()}",
                    onclick: copy,
                    if copied() {
                        Icon { name: IconName::Check, size: 13 }
                        span { "aria-live": "polite", "Copied" }
                    } else {
                        Icon { name: IconName::Copy, size: 13 }
                        span { "aria-live": "polite", "Copy" }
                    }
                }
            }
            JsonInspector { json: text, label: title.to_string(), expand_depth: 2 }
        }
    }
}
