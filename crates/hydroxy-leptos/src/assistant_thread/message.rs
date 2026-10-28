//! The message: avatar, parts, chain of thought and error.

use hydroxy_headless::assistant_thread::{self as h, PartGroup};
use leptos::either::EitherOf3;
use leptos::prelude::*;

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
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn AssistantMessage(
    /// The message.
    #[prop(into)]
    message: Signal<AssistantMessageData>,
    /// A reply is running.
    #[prop(optional, into)]
    running: Signal<bool>,
    /// This is the last message in the thread.
    #[prop(optional, into)]
    is_last: Signal<bool>,
    /// `(zero-based index, count)` when the message has branches.
    #[prop(optional, into)]
    branch: Signal<Option<(usize, usize)>>,
    /// Hover-reveal rule for the action bar.
    #[prop(optional)]
    autohide: Autohide,
    /// Hide the action bar while a reply runs.
    #[prop(optional)]
    hide_when_running: bool,
    /// Called when Reload is pressed.
    #[prop(optional, into)]
    on_reload: Option<Callback<()>>,
    /// Called when Edit is pressed.
    #[prop(optional, into)]
    on_edit: Option<Callback<()>>,
    /// Called when Try again is pressed on the error.
    #[prop(optional, into)]
    on_retry: Option<Callback<()>>,
    /// Called with the vote (`None` when taken back).
    #[prop(optional, into)]
    on_feedback: Option<Callback<Option<Feedback>>>,
    /// Called with the Markdown export.
    #[prop(optional, into)]
    on_export: Option<Callback<String>>,
    /// Called with the branch to show.
    #[prop(optional, into)]
    on_branch_switch: Option<Callback<usize>>,
    /// Called with whether reading aloud should start.
    #[prop(optional, into)]
    on_speak: Option<Callback<bool>>,
) -> impl IntoView {
    let role = Signal::derive(move || message.with(|m| m.role));
    let text = Signal::derive(move || message.with(AssistantMessageData::plain_text));
    let markdown = Signal::derive(move || message.with(h::export_markdown));
    let streaming = Signal::derive(move || message.with(|m| m.streaming));
    view! {
        <article
            data-scope=h::SCOPE_MESSAGE
            data-part="root"
            data-role=move || role.get().as_str()
            data-message-id=move || message.with(|m| m.id.clone())
            data-streaming=move || streaming.get().then_some("")
        >
            {move || {
                (role.get() == Role::Assistant)
                    .then(|| view! { <span data-scope=h::SCOPE_MESSAGE data-part="avatar"><Icon name=IconName::Mark size=28 /></span> })
            }}
            <div data-scope=h::SCOPE_MESSAGE data-part="body">
                // the body rebuilds as parts stream in, so a chain the reader opened by hand closes on the next tick. Keyed parts would keep it.
                {move || message.with(|m| render_parts(m, streaming.get()))}
                {move || {
                    message
                        .with(|m| m.error.clone())
                        .map(|e| view! { <AssistantError message=e on_retry=fwd(on_retry) /> })
                }}
                <div data-scope=h::SCOPE_MESSAGE data-part="footer">
                    {move || {
                        branch
                            .get()
                            .filter(|(_, n)| *n > 1)
                            .map(|(i, n)| {
                                view! {
                                    <AssistantBranchPicker
                                        count=n
                                        index=i
                                        running=running
                                        on_switch=fwd(on_branch_switch)
                                    />
                                }
                            })
                    }}
                    {move || {
                        (!streaming.get())
                            .then(|| {
                                view! {
                                    <AssistantActionBar
                                        role=role.get_untracked()
                                        text=text
                                        markdown=markdown
                                        running=running
                                        is_last=is_last.get_untracked()
                                        autohide=autohide
                                        hide_when_running=hide_when_running
                                        on_reload=fwd(on_reload)
                                        on_edit=fwd(on_edit)
                                        on_feedback=fwd(on_feedback)
                                        on_export=fwd(on_export)
                                        on_speak=fwd(on_speak)
                                    />
                                }
                            })
                    }}
                </div>
            </div>
        </article>
    }
}

fn render_parts(message: &AssistantMessageData, streaming: bool) -> Vec<AnyView> {
    let groups = h::group_parts(&message.parts);
    let last = groups.len().saturating_sub(1);
    let mut out: Vec<AnyView> = Vec::new();
    let mut sources: Vec<(String, String)> = Vec::new();
    let flush = |sources: &mut Vec<(String, String)>, out: &mut Vec<AnyView>| {
        if sources.is_empty() {
            return;
        }
        let chips = std::mem::take(sources)
            .into_iter()
            .map(|(title, _)| view! { <SourceChip>{title}</SourceChip> })
            .collect_view();
        out.push(
            view! {
                <div data-scope=h::SCOPE_PART data-part="root" data-type="source">
                    <SourceRow>{chips}</SourceRow>
                </div>
            }
            .into_any(),
        );
    };
    for (i, group) in groups.into_iter().enumerate() {
        match group {
            PartGroup::Single(Part::Source { title, url }) => sources.push((title, url)),
            PartGroup::Single(Part::Text(t)) => {
                flush(&mut sources, &mut out);
                let cursor = streaming && i == last;
                out.push(
                    view! {
                        <div data-scope=h::SCOPE_PART data-part="root" data-type="text" data-cursor=cursor.then_some("")>
                            <Markdown source=t />
                        </div>
                    }
                    .into_any(),
                );
            }
            PartGroup::Single(Part::Image { src, alt }) => {
                flush(&mut sources, &mut out);
                out.push(
                    view! {
                        <div data-scope=h::SCOPE_PART data-part="root" data-type="image">
                            <img src=src alt=alt loading="lazy" />
                        </div>
                    }
                    .into_any(),
                );
            }
            PartGroup::Single(_) => {}
            PartGroup::Chain(parts) => {
                flush(&mut sources, &mut out);
                let open = parts.iter().any(|p| matches!(p, Part::Tool { status: PartStatus::Running, .. }));
                out.push(view! { <AssistantChainOfThought parts=parts default_open=open /> }.into_any());
            }
        }
    }
    flush(&mut sources, &mut out);
    out
}

// ------------------------------------------------------------ chain of thought

/// A collapsible run of reasoning and tool calls. The trigger reads "3 steps", or
/// "Working, 3 steps so far" while a tool is running.
#[component]
pub fn AssistantChainOfThought(
    /// The reasoning and tool parts.
    parts: Vec<Part>,
    /// Start open.
    #[prop(optional)]
    default_open: bool,
) -> impl IntoView {
    let summary = h::chain_summary(&parts);
    let running = parts.iter().any(|p| matches!(p, Part::Tool { status: PartStatus::Running, .. }));
    let steps = parts
        .into_iter()
        .map(|p| match p {
            Part::Reasoning(t) => EitherOf3::A(view! {
                <li data-scope=h::SCOPE_CHAIN data-part="step" data-type="reasoning">
                    <span data-scope=h::SCOPE_CHAIN data-part="step-label">"Reasoning"</span>
                    <p data-scope=h::SCOPE_CHAIN data-part="step-text">{t}</p>
                </li>
            }),
            Part::Tool { name, args, result, status } => EitherOf3::B(view! {
                <li data-scope=h::SCOPE_CHAIN data-part="step" data-type="tool-call" data-status=status.as_str()>
                    <span data-scope=h::SCOPE_CHAIN data-part="step-label">
                        <code>{name}</code>
                        <span data-scope=h::SCOPE_CHAIN data-part="status">{status.status_text()}</span>
                    </span>
                    <pre data-scope=h::SCOPE_CHAIN data-part="args">{args}</pre>
                    {result.map(|r| view! { <pre data-scope=h::SCOPE_CHAIN data-part="result">{r}</pre> })}
                </li>
            }),
            _ => EitherOf3::C(()),
        })
        .collect_view();
    view! {
        <div data-scope=h::SCOPE_CHAIN data-part="root" data-running=running.then_some("")>
            <Collapsible default_open=default_open>
                <CollapsibleTrigger>{summary}</CollapsibleTrigger>
                <CollapsibleContent>
                    <ol data-scope=h::SCOPE_CHAIN data-part="steps">{steps}</ol>
                </CollapsibleContent>
            </Collapsible>
        </div>
    }
}

// --------------------------------------------------------------------- error

/// A failed run, shown under the message with an optional retry.
#[component]
pub fn AssistantError(
    /// What went wrong.
    #[prop(into)]
    message: String,
    /// Called when Try again is pressed. The button shows only when this is set.
    #[prop(optional, into)]
    on_retry: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div data-scope=h::SCOPE_ERROR data-part="root" role="alert">
            <span data-scope=h::SCOPE_ERROR data-part="icon"><Icon name=IconName::TriangleAlert size=16 /></span>
            <span data-scope=h::SCOPE_ERROR data-part="message">{message}</span>
            {on_retry
                .map(|cb| {
                    view! {
                        <button type="button" data-scope=h::SCOPE_ERROR data-part="retry" on:click=move |_| cb.run(())>
                            "Try again"
                        </button>
                    }
                })}
        </div>
    }
}
