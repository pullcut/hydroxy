//! One message and its parts: text, reasoning, tool calls, sources, chain of thought and
//! the error banner.

use dioxus::prelude::*;
use hydroxy_headless::assistant_thread::{self as h, PartGroup};

use super::{
    AssistantActionBar, AssistantBranchPicker, AssistantMessageData, Autohide, Feedback, Part, PartStatus, Role,
};
use crate::callbacks::fwd;
use crate::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
use crate::icon::{Icon, IconName};
use crate::markdown::Markdown;
use crate::source_chip::{SourceChip, SourceRow};

/// One message: avatar, parts, error, branch picker and action bar. Text parts render as
/// Markdown; consecutive reasoning and tool parts fold into a chain of thought.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn AssistantMessage(
    /// The message.
    message: AssistantMessageData,
    /// A reply is running.
    #[props(default)]
    running: bool,
    /// This is the last message in the thread.
    #[props(default)]
    is_last: bool,
    /// `(zero-based index, count)` when the message has branches.
    #[props(default)]
    branch: Option<(usize, usize)>,
    /// Hover-reveal rule for the action bar.
    #[props(default)]
    autohide: Autohide,
    /// Hide the action bar while a reply runs.
    #[props(default)]
    hide_when_running: bool,
    /// Called when Reload is pressed.
    on_reload: Option<EventHandler<()>>,
    /// Called when Edit is pressed.
    on_edit: Option<EventHandler<()>>,
    /// Called when Try again is pressed on the error.
    on_retry: Option<EventHandler<()>>,
    /// Called with the vote (`None` when taken back).
    on_feedback: Option<EventHandler<Option<Feedback>>>,
    /// Called with the Markdown export.
    on_export: Option<EventHandler<String>>,
    /// Called with the branch to show.
    on_branch_switch: Option<EventHandler<usize>>,
    /// Called with whether reading aloud should start.
    on_speak: Option<EventHandler<bool>>,
) -> Element {
    let groups = h::group_parts(&message.parts);
    let last = groups.len().saturating_sub(1);
    let text = message.plain_text();
    let markdown = h::export_markdown(&message);

    rsx! {
        article {
            "data-scope": h::SCOPE_MESSAGE,
            "data-part": "root",
            "data-role": message.role.as_str(),
            "data-message-id": "{message.id}",
            "data-streaming": message.streaming.then_some(""),
            if message.role == Role::Assistant {
                span { "data-scope": h::SCOPE_MESSAGE, "data-part": "avatar",
                    Icon { name: IconName::Mark, size: 28 }
                }
            }
            div { "data-scope": h::SCOPE_MESSAGE, "data-part": "body",
                {render_groups(&groups, message.streaming, last)}
                if let Some(error) = message.error.clone() {
                    AssistantError { message: error, on_retry: fwd(on_retry) }
                }
                div { "data-scope": h::SCOPE_MESSAGE, "data-part": "footer",
                    if let Some((index, count)) = branch.filter(|(_, n)| *n > 1) {
                        AssistantBranchPicker { count, index, running, on_switch: fwd(on_branch_switch) }
                    }
                    if !message.streaming {
                        AssistantActionBar {
                            role: message.role,
                            text,
                            markdown,
                            running,
                            is_last,
                            autohide,
                            hide_when_running,
                            on_reload: fwd(on_reload),
                            on_edit: fwd(on_edit),
                            on_feedback: fwd(on_feedback),
                            on_export: fwd(on_export),
                            on_speak: fwd(on_speak),
                        }
                    }
                }
            }
        }
    }
}

fn render_groups(groups: &[PartGroup], streaming: bool, last: usize) -> Element {
    // Consecutive sources share one row.
    let mut blocks: Vec<Element> = Vec::new();
    let mut sources: Vec<String> = Vec::new();
    let flush = |sources: &mut Vec<String>, blocks: &mut Vec<Element>| {
        if sources.is_empty() {
            return;
        }
        let titles = std::mem::take(sources);
        blocks.push(rsx! {
            div { "data-scope": h::SCOPE_PART, "data-part": "root", "data-type": "source",
                SourceRow {
                    for t in titles {
                        SourceChip { "{t}" }
                    }
                }
            }
        });
    };
    for (i, group) in groups.iter().enumerate() {
        match group {
            PartGroup::Single(Part::Source { title, .. }) => sources.push(title.clone()),
            PartGroup::Single(Part::Text(t)) => {
                flush(&mut sources, &mut blocks);
                let cursor = streaming && i == last;
                blocks.push(rsx! {
                    div {
                        "data-scope": h::SCOPE_PART,
                        "data-part": "root",
                        "data-type": "text",
                        "data-cursor": cursor.then_some(""),
                        Markdown { source: t.clone() }
                    }
                });
            }
            PartGroup::Single(Part::Image { src, alt }) => {
                flush(&mut sources, &mut blocks);
                blocks.push(rsx! {
                    div { "data-scope": h::SCOPE_PART, "data-part": "root", "data-type": "image",
                        img { src: "{src}", alt: "{alt}", loading: "lazy" }
                    }
                });
            }
            PartGroup::Single(_) => {}
            PartGroup::Chain(parts) => {
                flush(&mut sources, &mut blocks);
                let open = parts.iter().any(|p| matches!(p, Part::Tool { status: PartStatus::Running, .. }));
                blocks.push(rsx! {
                    AssistantChainOfThought { parts: parts.clone(), default_open: open }
                });
            }
        }
    }
    flush(&mut sources, &mut blocks);
    rsx! {
        for (i , b) in blocks.into_iter().enumerate() {
            Fragment { key: "{i}", {b} }
        }
    }
}

// ------------------------------------------------------------ chain of thought

/// A collapsible run of reasoning and tool calls. The trigger reads "3 steps", or
/// "Working, 3 steps so far" while a tool is running.
#[component]
pub fn AssistantChainOfThought(
    /// The reasoning and tool parts.
    parts: Vec<Part>,
    /// Start open.
    #[props(default)]
    default_open: bool,
) -> Element {
    let summary = h::chain_summary(&parts);
    let running = parts.iter().any(|p| matches!(p, Part::Tool { status: PartStatus::Running, .. }));
    rsx! {
        div { "data-scope": h::SCOPE_CHAIN, "data-part": "root", "data-running": running.then_some(""),
            Collapsible { default_open,
                CollapsibleTrigger { "{summary}" }
                CollapsibleContent {
                    ol { "data-scope": h::SCOPE_CHAIN, "data-part": "steps",
                        for (i , p) in parts.iter().enumerate() {
                            match p {
                                Part::Reasoning(t) => rsx! {
                                    li { key: "{i}", "data-scope": h::SCOPE_CHAIN, "data-part": "step", "data-type": "reasoning",
                                        span { "data-scope": h::SCOPE_CHAIN, "data-part": "step-label", "Reasoning" }
                                        p { "data-scope": h::SCOPE_CHAIN, "data-part": "step-text", "{t}" }
                                    }
                                },
                                Part::Tool { name, args, result, status } => rsx! {
                                    li {
                                        key: "{i}",
                                        "data-scope": h::SCOPE_CHAIN,
                                        "data-part": "step",
                                        "data-type": "tool-call",
                                        "data-status": status.as_str(),
                                        span { "data-scope": h::SCOPE_CHAIN, "data-part": "step-label",
                                            code { "{name}" }
                                            span { "data-scope": h::SCOPE_CHAIN, "data-part": "status", "{status.status_text()}" }
                                        }
                                        pre { "data-scope": h::SCOPE_CHAIN, "data-part": "args", "{args}" }
                                        if let Some(r) = result {
                                            pre { "data-scope": h::SCOPE_CHAIN, "data-part": "result", "{r}" }
                                        }
                                    }
                                },
                                _ => rsx! {},
                            }
                        }
                    }
                }
            }
        }
    }
}

// --------------------------------------------------------------------- error

/// A failed run, shown under the message with an optional retry.
#[component]
pub fn AssistantError(
    /// What went wrong.
    message: String,
    /// Called when Try again is pressed. The button shows only when this is set.
    on_retry: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        div { "data-scope": h::SCOPE_ERROR, "data-part": "root", role: "alert",
            span { "data-scope": h::SCOPE_ERROR, "data-part": "icon",
                Icon { name: IconName::TriangleAlert, size: 16 }
            }
            span { "data-scope": h::SCOPE_ERROR, "data-part": "message", "{message}" }
            if let Some(handler) = on_retry {
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE_ERROR,
                    "data-part": "retry",
                    onclick: move |_| handler.call(()),
                    "Try again"
                }
            }
        }
    }
}
