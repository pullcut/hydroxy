use std::time::Duration;

use leptos::prelude::*;
use hydroxy_leptos::tool_trace::{StepKind, ToolTrace, TraceStep};

fn tool(id: &str, name: &str, arguments: &str, result: Option<&str>) -> TraceStep {
    TraceStep::new(
        id,
        StepKind::ToolCall { name: name.into(), arguments: arguments.into(), result: result.map(Into::into) },
    )
}

/// The run as it stands after `n` updates, the way a streaming agent API would deliver it.
fn run(n: usize) -> Vec<TraceStep> {
    let mut steps = vec![TraceStep::new("think", StepKind::Thinking {
        text: "The user wants last quarter's churn by plan. I'll query the warehouse, then ask a sub-agent to check the numbers against the finance export.".into(),
    })];
    if n == 0 {
        return steps;
    }
    steps[0] = steps[0].clone().done(1_340);
    let query = r#"{"sql":"select plan, count(*) filter (where churned) * 1.0 / count(*) as churn from accounts where quarter = '2026-Q3' group by plan","timeout_s":30}"#;
    steps.push(if n == 1 {
        tool("sql", "run_sql", query, None)
    } else {
        tool("sql", "run_sql", query, Some(r#"{"rows":[{"plan":"Starter","churn":0.061},{"plan":"Team","churn":0.034},{"plan":"Enterprise","churn":0.012}],"elapsed_ms":812}"#)).done(2_210)
    });
    if n < 3 {
        return steps;
    }
    let mut checker = vec![tool("fetch", "read_file", r#"{"path":"finance/q3-churn.csv"}"#, Some("plan,churn\nStarter,0.062\nTeam,0.034\nEnterprise,0.012")).done(420)];
    if n >= 4 {
        checker.push(TraceStep::new("cmp", StepKind::Message { text: "Starter differs by 0.1 points (rounding in the finance export). The rest match.".into() }).done(900));
    }
    let agent = TraceStep::new("verify", StepKind::Agent { name: "verify-numbers".into() }).with_children(checker);
    steps.push(if n >= 4 { agent.done(1_620) } else { agent });
    if n >= 5 {
        steps.push(tool("chart", "render_chart", r#"{"type":"bar","series":"churn"}"#, None).failed(180));
        steps.push(TraceStep::new("err", StepKind::Error { message: "render_chart: the chart service is not reachable (timeout after 180 ms).".into() }).failed(0));
        steps.push(TraceStep::new("msg", StepKind::Message { text: "Q3 churn: Starter 6.1%, Team 3.4%, Enterprise 1.2%. Verified against finance; I couldn't draw the chart.".into() }).done(640));
    }
    steps
}

pub fn demo() -> AnyView {
    let n = RwSignal::new(0_usize);
    let steps = Signal::derive(move || run(n.get()));
    // Replay the run: one update every 1.2 s until it finishes.
    let play = move |_| {
        n.set(0);
        for i in 1..=5_u64 {
            set_timeout(move || n.set(usize::try_from(i).unwrap_or(5)), Duration::from_millis(1_200 * i));
        }
    };

    view! {
        <div style="display: grid; gap: var(--space-3); max-width: 640px;">
            <ToolTrace steps=steps label="Churn analysis run" />
            <button type="button" class="ox-button" data-variant="secondary" data-size="sm" style="justify-self: start;" on:click=play>
                "Replay run"
            </button>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "tool-trace",
    name: "Tool trace",
    group: "Chat",
    summary: "An agent run step by step: thinking, tool calls with arguments and results, sub-agents, errors. Updates live as steps arrive.",
    demo,
    leptos: include_str!("tool_trace.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/tool_trace.rs"),
    parts: &[
        &["ToolTrace", "root", "<section> named by label. data-running while any step runs"],
        &["ToolTrace", "header", "Summary (\"6 steps, 3 tool calls, 1 failed, 7.3 s\") and Expand all"],
        &["ToolTrace", "step", "<li> with data-kind and data-status; aria-busy while running"],
        &["ToolTrace", "step-header", "Disclosure button (aria-expanded) with icon, title, status and duration"],
        &["ToolTrace", "progress", "Indeterminate bar under a running step"],
        &["ToolTrace", "body", "Text, or Arguments and Result sections. JSON opens in JsonInspector with Copy"],
        &["ToolTrace", "steps", "Nested <ol> for a sub-agent's steps"],
        &["ToolTrace", "live", "Polite live region: \"run_sql done\""],
    ],
    keys: &[
        &["Tab", "Move between step headers and the controls inside open steps"],
        &["Enter / Space", "Open or close a step"],
    ],
    props: &[
        &["steps", "Vec<TraceStep>", "Required. Id, kind, status, duration, children. Update it as the run goes"],
        &["label", "String", "Accessible name. Default \"Agent run\""],
    ],
};
