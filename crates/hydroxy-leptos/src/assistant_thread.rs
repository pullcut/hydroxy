//! `<AssistantThread>` and the pieces of one: messages and their parts, the action bar,
//! branch picker, chain of thought, error and the quote-selection toolbar.
//!
//! The owner holds the messages. The thread renders them in a [`MessageScroller`], and
//! leaves a `children` slot below for a composer.

use std::collections::HashMap;
use std::time::Duration;

use hydroxy_headless::assistant_thread::{
    self as h, ActionBarCommand, ActionBarEvent, ActionBarInput, ActionBarMachine, BranchPickerCommand,
    BranchPickerEvent, BranchPickerInput, BranchPickerMachine, PartGroup, SelectionToolbarCommand,
    SelectionToolbarEvent, SelectionToolbarMachine,
};
use hydroxy_statechart::Machine;
use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos::wasm_bindgen::{JsCast, closure::Closure};

use crate::callbacks::fwd;
use crate::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
use crate::drive::drive;
use crate::icon::{Icon, IconName};
use crate::markdown::Markdown;
use crate::message_scroller::MessageScroller;
use crate::source_chip::{SourceChip, SourceRow};

pub use hydroxy_headless::assistant_thread::{
    ActionBarState, AssistantMessageData, Autohide, Feedback, Part, PartStatus, Role, Selection,
};

// ------------------------------------------------------------------- thread

/// A scrolling assistant conversation. Pass the messages and a `running` flag; apply
/// the callbacks to your own data. Children render below the messages (put the composer
/// there).
#[component]
#[allow(
    clippy::too_many_lines,
    clippy::implicit_hasher,
    reason = "one view! tree and a std HashMap prop; splitting or generalising only adds plumbing"
)]
pub fn AssistantThread(
    /// The messages, oldest first.
    #[prop(into)]
    messages: Signal<Vec<AssistantMessageData>>,
    /// A reply is running: shows the typing indicator and disables Copy and branch switching.
    #[prop(optional, into)]
    running: Signal<bool>,
    /// Branch position per message id: `(zero-based index, count)`. Messages with more than one branch show the picker.
    #[prop(optional, into)]
    branches: Signal<HashMap<String, (usize, usize)>>,
    /// Hover-reveal rule for the action bars.
    #[prop(optional)]
    autohide: Autohide,
    /// Hide the action bars while a reply runs.
    #[prop(optional)]
    hide_when_running: bool,
    /// Called with the message id when Reload is pressed.
    #[prop(optional, into)]
    on_reload: Option<Callback<String>>,
    /// Called with the message id when Edit is pressed.
    #[prop(optional, into)]
    on_edit: Option<Callback<String>>,
    /// Called with the message id and the vote (`None` when taken back).
    #[prop(optional, into)]
    on_feedback: Option<Callback<(String, Option<Feedback>)>>,
    /// Called with the message id and its Markdown when Export is pressed.
    #[prop(optional, into)]
    on_export: Option<Callback<(String, String)>>,
    /// Called with the message id and the branch to show.
    #[prop(optional, into)]
    on_branch_switch: Option<Callback<(String, usize)>>,
    /// Called with the message id and whether reading aloud should start.
    #[prop(optional, into)]
    on_speak: Option<Callback<(String, bool)>>,
    /// Called with the message id when Try again is pressed on an error.
    #[prop(optional, into)]
    on_retry: Option<Callback<String>>,
    /// When set, selecting text in a message shows a Quote button that calls this.
    #[prop(optional, into)]
    on_quote: Option<Callback<Selection>>,
    /// Accessible name for the message list.
    #[prop(default = "Conversation".into(), into)]
    label: String,
    /// Text shown while there are no messages.
    #[prop(default = "No messages yet.".into(), into)]
    empty: String,
    /// Below the messages, such as a composer.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let count = Signal::derive(move || messages.with(Vec::len));
    let last_key = Signal::derive(move || messages.with(|m| m.last().map(|m| m.id.clone())));
    let revision = Signal::derive(move || {
        messages.with(|m| {
            m.last().map_or(0, |m| {
                m.parts.iter().map(|p| format!("{p:?}").len()).sum::<usize>() + m.error.as_ref().map_or(0, String::len)
            })
        })
    });
    let ids = Signal::derive(move || messages.with(|m| m.iter().map(|m| m.id.clone()).collect::<Vec<_>>()));
    let typing =
        move || running.get() && messages.with(|m| m.last().is_none_or(|m| m.role == Role::User || m.parts.is_empty()));

    view! {
        <div data-scope=h::SCOPE_THREAD data-part="root" data-running=move || running.get().then_some("")>
            <div data-scope=h::SCOPE_THREAD data-part="scroller">
                <MessageScroller count=count last_key=last_key revision=revision label=label>
                    {move || {
                        messages
                            .with(Vec::is_empty)
                            .then(|| view! { <p data-scope=h::SCOPE_THREAD data-part="empty">{empty.clone()}</p> })
                    }}
                    <For
                        each=move || ids.get()
                        key=Clone::clone
                        children=move |id| {
                            let message = Memo::new({
                                let id = id.clone();
                                move |_| {
                                    messages
                                        .with(|v| v.iter().find(|m| m.id == id).cloned())
                                        .unwrap_or_else(|| AssistantMessageData::text(id.clone(), Role::Assistant, ""))
                                }
                            });
                            let is_last = Signal::derive({
                                let id = id.clone();
                                move || messages.with(|v| v.last().is_some_and(|m| m.id == id))
                            });
                            let branch = Signal::derive({
                                let id = id.clone();
                                move || branches.with(|b| b.get(&id).copied())
                            });
                            let with_id = |cb: Option<Callback<String>>, id: String| {
                                Callback::new(move |(): ()| {
                                    if let Some(cb) = cb {
                                        cb.run(id.clone());
                                    }
                                })
                            };
                            view! {
                                <AssistantMessage
                                    message=message
                                    running=running
                                    is_last=is_last
                                    branch=branch
                                    autohide=autohide
                                    hide_when_running=hide_when_running
                                    on_reload=with_id(on_reload, id.clone())
                                    on_edit=with_id(on_edit, id.clone())
                                    on_retry=with_id(on_retry, id.clone())
                                    on_feedback={
                                        let id = id.clone();
                                        Callback::new(move |v| {
                                            if let Some(cb) = on_feedback {
                                                cb.run((id.clone(), v));
                                            }
                                        })
                                    }
                                    on_export={
                                        let id = id.clone();
                                        Callback::new(move |md| {
                                            if let Some(cb) = on_export {
                                                cb.run((id.clone(), md));
                                            }
                                        })
                                    }
                                    on_branch_switch={
                                        let id = id.clone();
                                        Callback::new(move |i| {
                                            if let Some(cb) = on_branch_switch {
                                                cb.run((id.clone(), i));
                                            }
                                        })
                                    }
                                    on_speak={
                                        let id = id.clone();
                                        Callback::new(move |on| {
                                            if let Some(cb) = on_speak {
                                                cb.run((id.clone(), on));
                                            }
                                        })
                                    }
                                />
                            }
                        }
                    />
                    {move || {
                        typing()
                            .then(|| {
                                view! {
                                    <div data-scope=h::SCOPE_THREAD data-part="typing" role="status" aria-label="Assistant is typing">
                                        <span></span>
                                        <span></span>
                                        <span></span>
                                    </div>
                                }
                            })
                    }}
                </MessageScroller>
            </div>
            {children.map(|c| view! { <div data-scope=h::SCOPE_THREAD data-part="footer">{c()}</div> })}
            {on_quote.map(|cb| view! { <AssistantSelectionToolbar on_quote=cb /> })}
        </div>
    }
}

// ------------------------------------------------------------------ message

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

// ------------------------------------------------------------- branch picker

/// Previous, "2 / 3", next: steps through the branches of one message.
#[component]
pub fn AssistantBranchPicker(
    /// How many branches there are.
    #[prop(into)]
    count: Signal<usize>,
    /// Controlled position (zero-based). Pair with `on_switch`.
    #[prop(optional, into)]
    index: MaybeProp<usize>,
    /// A reply is running: switching is disabled unless `switch_during_run`.
    #[prop(optional, into)]
    running: Signal<bool>,
    /// Allow switching while a reply runs.
    #[prop(optional)]
    switch_during_run: bool,
    /// Called with the zero-based branch to show.
    #[prop(optional, into)]
    on_switch: Option<Callback<usize>>,
) -> impl IntoView {
    let input = Memo::new(move |_| BranchPickerInput {
        count: count.get(),
        index: index.get(),
        running: running.get(),
        switch_during_run,
    });
    let state = RwSignal::new(input.with_untracked(BranchPickerMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| BranchPickerMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let send = move |event: BranchPickerEvent| {
        for BranchPickerCommand::Switch(i) in drive::<BranchPickerMachine, _>(state, input, event) {
            if let Some(cb) = on_switch {
                cb.run(i);
            }
        }
    };
    let prev_ok = move || input.with(|i| state.with(|s| h::can_previous(s, i)));
    let next_ok = move || input.with(|i| state.with(|s| h::can_next(s, i)));
    view! {
        <nav data-scope=h::SCOPE_BRANCH_PICKER data-part="root" aria-label="Branches">
            <button
                type="button"
                data-scope=h::SCOPE_BRANCH_PICKER
                data-part="previous"
                aria-label="Previous branch"
                disabled=move || !prev_ok()
                on:click=move |_| send(BranchPickerEvent::Previous)
            >
                <Icon name=IconName::ChevronRight size=14 />
            </button>
            <span data-scope=h::SCOPE_BRANCH_PICKER data-part="label" aria-live="polite">
                {move || state.with(|s| h::branch_label(s.index.value, count.get()))}
            </span>
            <button
                type="button"
                data-scope=h::SCOPE_BRANCH_PICKER
                data-part="next"
                aria-label="Next branch"
                disabled=move || !next_ok()
                on:click=move |_| send(BranchPickerEvent::Next)
            >
                <Icon name=IconName::ChevronRight size=14 />
            </button>
        </nav>
    }
}

// ----------------------------------------------------------------- action bar

/// Copy, reload, edit, export, read aloud and the two votes for one message. Which
/// buttons show depends on the role.
#[component]
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
pub fn AssistantActionBar(
    /// Whose message this is.
    #[prop(optional)]
    role: Role,
    /// What Copy writes to the clipboard.
    #[prop(into)]
    text: Signal<String>,
    /// What Export hands to `on_export`.
    #[prop(into)]
    markdown: Signal<String>,
    /// A reply is running.
    #[prop(optional, into)]
    running: Signal<bool>,
    /// This is the last message in the thread.
    #[prop(optional)]
    is_last: bool,
    /// Hover-reveal rule.
    #[prop(optional)]
    autohide: Autohide,
    /// Hide the bar while a reply runs.
    #[prop(optional)]
    hide_when_running: bool,
    /// Called when Reload is pressed.
    #[prop(optional, into)]
    on_reload: Option<Callback<()>>,
    /// Called when Edit is pressed.
    #[prop(optional, into)]
    on_edit: Option<Callback<()>>,
    /// Called with the vote (`None` when taken back).
    #[prop(optional, into)]
    on_feedback: Option<Callback<Option<Feedback>>>,
    /// Called with the Markdown export.
    #[prop(optional, into)]
    on_export: Option<Callback<String>>,
    /// Called with whether reading aloud should start.
    #[prop(optional, into)]
    on_speak: Option<Callback<bool>>,
) -> impl IntoView {
    let input = Memo::new(move |_| ActionBarInput { running: running.get(), hide_when_running, autohide, is_last });
    let state = RwSignal::new(ActionBarState::default());
    let send = move |event: ActionBarEvent| {
        for command in drive::<ActionBarMachine, _>(state, input, event) {
            match command {
                ActionBarCommand::Copy => {
                    // Best-effort: without clipboard access the button just confirms anyway.
                    crate::js::copy(&text.get_untracked());
                    set_timeout(
                        move || {
                            drive::<ActionBarMachine, _>(state, input, ActionBarEvent::CopiedReset);
                        },
                        Duration::from_millis(1400),
                    );
                }
                ActionBarCommand::Feedback(v) => {
                    if let Some(cb) = on_feedback {
                        cb.run(v);
                    }
                }
                ActionBarCommand::Reload => {
                    if let Some(cb) = on_reload {
                        cb.run(());
                    }
                }
                ActionBarCommand::Edit => {
                    if let Some(cb) = on_edit {
                        cb.run(());
                    }
                }
                ActionBarCommand::Export => {
                    if let Some(cb) = on_export {
                        cb.run(markdown.get_untracked());
                    }
                }
                ActionBarCommand::Speak => {
                    if let Some(cb) = on_speak {
                        cb.run(true);
                    }
                }
                ActionBarCommand::StopSpeaking => {
                    if let Some(cb) = on_speak {
                        cb.run(false);
                    }
                }
            }
        }
    };

    let buttons = move || {
        h::actions_for(role)
            .iter()
            .map(|&name| {
                let label = move || {
                    state.with(|s| match name {
                        "copy" if s.copied => "Copied",
                        "copy" => "Copy",
                        "reload" => "Reload",
                        "edit" => "Edit",
                        "export" => "Export",
                        "speak" if s.speaking => "Stop",
                        "speak" => "Read aloud",
                        "positive" => "Helpful",
                        _ => "Not helpful",
                    })
                };
                let pressed = move || {
                    state.with(|s| match name {
                        "speak" => Some(s.speaking),
                        "positive" => Some(s.feedback == Some(Feedback::Positive)),
                        "negative" => Some(s.feedback == Some(Feedback::Negative)),
                        _ => None,
                    })
                };
                let event = match name {
                    "copy" => ActionBarEvent::Copy,
                    "reload" => ActionBarEvent::Reload,
                    "edit" => ActionBarEvent::Edit,
                    "export" => ActionBarEvent::Export,
                    "speak" => ActionBarEvent::ToggleSpeak,
                    "positive" => ActionBarEvent::Vote(Feedback::Positive),
                    _ => ActionBarEvent::Vote(Feedback::Negative),
                };
                view! {
                    <button
                        type="button"
                        data-scope=h::SCOPE_ACTION_BAR
                        data-part="action"
                        data-action=name
                        aria-pressed=move || pressed().map(|p| p.to_string())
                        disabled=move || name == "copy" && running.get()
                        on:click=move |_| send(event)
                    >
                        {label}
                    </button>
                }
            })
            .collect_view()
    };

    view! {
        {move || {
            (!input.with(h::action_bar_hidden))
                .then(|| {
                    view! {
                        <div
                            data-scope=h::SCOPE_ACTION_BAR
                            data-part="root"
                            role="toolbar"
                            aria-label="Message actions"
                            aria-live="polite"
                            data-role=role.as_str()
                            data-floating=input.with(h::action_bar_floating).then_some("")
                        >
                            {buttons()}
                        </div>
                    }
                })
        }}
    }
}

// ----------------------------------------------------------- selection toolbar

fn read_selection() -> Option<Selection> {
    let sel = window().get_selection().ok()??;
    if sel.is_collapsed() || sel.range_count() == 0 {
        return None;
    }
    let range = sel.get_range_at(0).ok()?;
    let node = range.common_ancestor_container().ok()?;
    let el = node.dyn_ref::<web_sys::Element>().cloned().or_else(|| node.parent_element())?;
    let message = el.closest("[data-message-id]").ok()??;
    let rect = range.get_bounding_client_rect();
    Some(Selection {
        message_id: message.get_attribute("data-message-id")?,
        text: String::from(sel.to_string()),
        left: rect.left(),
        top: rect.top(),
    })
}

/// A Quote button that floats over text selected inside a message (anything under a
/// `[data-message-id]` element). Rendered by [`AssistantThread`] when `on_quote` is set.
#[component]
pub fn AssistantSelectionToolbar(
    /// Called with the selection when Quote is pressed. The page selection is then cleared.
    on_quote: Callback<Selection>,
) -> impl IntoView {
    let state = RwSignal::new(SelectionToolbarMachine::initial(&()));
    let input = Signal::stored(());
    let send = move |event: SelectionToolbarEvent| {
        for SelectionToolbarCommand::Quote(selection) in drive::<SelectionToolbarMachine, _>(state, input, event) {
            on_quote.run(selection);
            if let Ok(Some(sel)) = window().get_selection() {
                let _ = sel.remove_all_ranges();
            }
        }
    };

    Effect::new(move |_| {
        let listener = Closure::<dyn Fn()>::new(move || {
            send(read_selection().map_or(SelectionToolbarEvent::Cleared, SelectionToolbarEvent::Selected));
        });
        let doc = document();
        let _ = doc.add_event_listener_with_callback("selectionchange", listener.as_ref().unchecked_ref());
        // Closures aren't Send; a local stored value is, and only runs on this thread.
        let held = StoredValue::new_local(listener);
        on_cleanup(move || {
            held.try_with_value(|l| {
                let _ = document().remove_event_listener_with_callback("selectionchange", l.as_ref().unchecked_ref());
            });
        });
    });

    view! {
        {move || {
            state
                .with(|s| s.selection.clone())
                .map(|s| {
                    view! {
                        <div
                            data-scope=h::SCOPE_SELECTION_TOOLBAR
                            data-part="root"
                            role="toolbar"
                            aria-label="Selection"
                            style=format!("left: {}px; top: {}px;", s.left.max(8.0), s.top)
                        >
                            <button
                                type="button"
                                data-scope=h::SCOPE_SELECTION_TOOLBAR
                                data-part="quote"
                                // Keep the page selection alive while the button is pressed.
                                on:mousedown=move |ev| ev.prevent_default()
                                on:click=move |_| send(SelectionToolbarEvent::Quote)
                            >
                                "Quote"
                            </button>
                        </div>
                    }
                })
        }}
    }
}
