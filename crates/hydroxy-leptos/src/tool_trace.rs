//! `<ToolTrace>`: the steps of an agent run as collapsible rows: thinking, tool
//! calls with their arguments and result, messages, errors, and sub-agents with
//! their own steps nested inside. The host appends and updates steps as the run
//! goes; rows keep their place and focus, and finished steps are announced.

use std::time::Duration;

use leptos::prelude::*;
use hydroxy_headless::tool_trace::{self as h, ToolTraceEvent, ToolTraceMachine, ToolTraceState};
use hydroxy_statechart::Machine;

use crate::code_block::CodeBlock;
use crate::icon::{Icon, IconName};
use crate::json_inspector::JsonInspector;

pub use hydroxy_headless::tool_trace::{StepKind, StepStatus, TraceStep};

#[derive(Clone, Copy)]
struct Ctx {
    root: StoredValue<String>,
    state: RwSignal<ToolTraceState>,
}

impl Ctx {
    fn send(self, event: ToolTraceEvent) {
        if let Some(step) = self.state.try_with_untracked(|s| ToolTraceMachine::transition(s, event, &()))
            && let Some(next) = step.state {
                self.state.set(next);
            }
    }
}

/// An agent run's steps.
#[component]
pub fn ToolTrace(
    /// The steps, in order. Update the vector as the run progresses.
    #[prop(into)]
    steps: Signal<Vec<TraceStep>>,
    /// Accessible name for the trace. Default "Agent run".
    #[prop(default = "Agent run".into(), into)]
    label: String,
) -> impl IntoView {
    let ctx = Ctx { root: StoredValue::new(crate::id::next_id(h::SCOPE)), state: RwSignal::new(ToolTraceMachine::initial(&())) };
    let announce = RwSignal::new(String::new());
    Effect::new(move |prev: Option<Vec<TraceStep>>| {
        let now = steps.get();
        if let Some(text) = prev.and_then(|p| h::announcement(&p, &now)) {
            announce.set(text);
        }
        now
    });
    let running = move || steps.with(|s| s.iter().flat_map(TraceStep::walk).any(|s| s.status == StepStatus::Running));
    let all_open = move || ctx.state.with(|st| steps.with(|s| h::all_open(st, s)));

    view! {
        <section
            data-scope=h::SCOPE
            data-part="root"
            aria-label=label
            data-running=move || running().then_some("")
        >
            <div data-scope=h::SCOPE data-part="header">
                <span data-scope=h::SCOPE data-part="summary">{move || steps.with(|s| h::summary(s))}</span>
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="toggle-all"
                    on:click=move |_| {
                        let open = !all_open();
                        ctx.send(ToolTraceEvent::SetAll { ids: steps.with_untracked(|s| h::all_ids(s)), open });
                    }
                >
                    {move || if all_open() { "Collapse all" } else { "Expand all" }}
                </button>
            </div>
            {step_list(ctx, steps)}
            <span data-scope=h::SCOPE data-part="live" role="status">{move || announce.get()}</span>
        </section>
    }
}

/// One level of steps. Keyed by id, so updates keep rows (and focus) in place.
fn step_list(ctx: Ctx, list: Signal<Vec<TraceStep>>) -> AnyView {
    view! {
        <ol data-scope=h::SCOPE data-part="steps">
            <For
                each=move || list.with(|l| l.iter().map(|s| s.id.clone()).collect::<Vec<_>>())
                key=String::clone
                children=move |id| step_view(ctx, list, &id)
            />
        </ol>
    }
    .into_any()
}

fn step_view(ctx: Ctx, list: Signal<Vec<TraceStep>>, id: &str) -> AnyView {
    let step = Memo::new({
        let id = id.to_owned();
        move |_| {
            list.with(|l| l.iter().find(|s| s.id == id).cloned())
                .unwrap_or_else(|| TraceStep::new(id.clone(), StepKind::Message { text: String::new() }))
        }
    });
    let kind = Memo::new(move |_| step.with(|s| s.kind.clone()));
    let status = Memo::new(move |_| step.with(|s| s.status));
    let children = Signal::derive(move || step.with(|s| s.children.clone()));
    let has_children = Memo::new(move |_| step.with(|s| !s.children.is_empty()));
    let open = Memo::new(move |_| ctx.state.with(|st| step.with(|s| h::is_open(st, s))));
    let root = ctx.root.get_value();
    let (header_id, body_id) = (h::header_id(&root, id), h::body_id(&root, id));

    view! {
        <li
            data-scope=h::SCOPE
            data-part="step"
            data-kind=move || kind.with(StepKind::as_str)
            data-status=move || status.get().as_str()
            aria-busy=move || (status.get() == StepStatus::Running).then_some("true")
        >
            <button
                type="button"
                id=header_id
                aria-expanded=move || open.get().to_string()
                aria-controls=body_id.clone()
                data-scope=h::SCOPE
                data-part="step-header"
                on:click={
                    let id = id.to_owned();
                    move |_| ctx.send(ToolTraceEvent::Toggle { id: id.clone(), was_open: open.get_untracked() })
                }
            >
                <span data-scope=h::SCOPE data-part="chevron" aria-hidden="true">
                    <Icon name=IconName::ChevronRight size=14 />
                </span>
                <svg
                    data-scope=h::SCOPE
                    data-part="icon"
                    viewBox="0 0 24 24"
                    width="15"
                    height="15"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.75"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                    inner_html=kind.with_untracked(h::kind_icon)
                ></svg>
                <span data-scope=h::SCOPE data-part="title">{move || step.with(|s| s.title().to_string())}</span>
                <span data-scope=h::SCOPE data-part="status">{move || step.with(h::status_text)}</span>
                <span data-scope=h::SCOPE data-part="duration" aria-hidden="true">
                    {move || step.with(|s| s.duration_ms.map(h::format_duration))}
                </span>
            </button>
            <span data-scope=h::SCOPE data-part="progress" aria-hidden="true"></span>
            <div data-scope=h::SCOPE data-part="body" id=body_id hidden=move || !open.get()>
                {move || body(kind.get(), status.get())}
                {move || has_children.get().then(|| step_list(ctx, children))}
            </div>
        </li>
    }
    .into_any()
}

fn body(kind: StepKind, status: StepStatus) -> AnyView {
    match kind {
        StepKind::Thinking { text } | StepKind::Message { text } => {
            view! { <p data-scope=h::SCOPE data-part="text">{text}</p> }.into_any()
        }
        StepKind::Error { message } => {
            view! { <p data-scope=h::SCOPE data-part="text" data-tone="danger">{message}</p> }.into_any()
        }
        StepKind::ToolCall { arguments, result, .. } => {
            let result = match (result, status) {
                (Some(r), _) => section("Result", r).into_any(),
                (None, StepStatus::Running) => {
                    view! { <p data-scope=h::SCOPE data-part="waiting">"Waiting for the result"</p> }.into_any()
                }
                (None, _) => ().into_any(),
            };
            view! {
                {section("Arguments", arguments)}
                {result}
            }
            .into_any()
        }
        StepKind::Agent { .. } => ().into_any(),
    }
}

/// A labelled block of arguments or result. JSON gets the inspector and a copy
/// button (copying it pretty-printed); anything else a code block, which has its own.
fn section(title: &'static str, text: String) -> impl IntoView {
    if !h::is_json(&text) {
        return view! {
            <div data-scope=h::SCOPE data-part="section">
                <span data-scope=h::SCOPE data-part="section-title">{title}</span>
                <CodeBlock code=text />
            </div>
        }
        .into_any();
    }
    let copied = RwSignal::new(false);
    let pretty = h::pretty(&text);
    let copy = move |_| {
        // Best-effort: without clipboard access the button just doesn't confirm.
        let _ = window().navigator().clipboard().write_text(&pretty);
        copied.set(true);
        set_timeout(
            move || {
                copied.try_set(false);
            },
            Duration::from_millis(1_200),
        );
    };
    view! {
        <div data-scope=h::SCOPE data-part="section">
            <div data-scope=h::SCOPE data-part="section-head">
                <span data-scope=h::SCOPE data-part="section-title">{title}</span>
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="copy"
                    aria-label=format!("Copy {}", title.to_lowercase())
                    on:click=copy
                >
                    {move || {
                        let (icon, text) = if copied.get() { (IconName::Check, "Copied") } else { (IconName::Copy, "Copy") };
                        view! { <Icon name=icon size=13 /> <span aria-live="polite">{text}</span> }
                    }}
                </button>
            </div>
            <JsonInspector json=text label=title expand_depth=2 />
        </div>
    }
    .into_any()
}
