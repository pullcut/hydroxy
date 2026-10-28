//! `AssistantThread` and the pieces of one: messages and their parts, the action bar,
//! branch picker, chain of thought, error and the quote-selection toolbar.
//!
//! The owner holds the messages. The thread renders them in a `MessageScroller`, and
//! leaves a `children` slot below for a composer.

use std::collections::HashMap;

use dioxus::prelude::*;
use hydroxy_headless::assistant_thread::{
    self as h, ActionBarCommand, ActionBarEvent, ActionBarInput, ActionBarMachine, BranchPickerCommand,
    BranchPickerEvent, BranchPickerInput, BranchPickerMachine, PartGroup, SelectionToolbarCommand,
    SelectionToolbarEvent, SelectionToolbarMachine,
};
use hydroxy_statechart::Machine;

use crate::callbacks::fwd;
use crate::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
use crate::drive::drive;
use crate::icon::{Icon, IconName};
use crate::js;
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
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn AssistantThread(
    /// The messages, oldest first.
    messages: ReadSignal<Vec<AssistantMessageData>>,
    /// A reply is running: shows the typing indicator and disables Copy and branch switching.
    #[props(default)]
    running: ReadSignal<bool>,
    /// Branch position per message id: `(zero-based index, count)`. Messages with more than one branch show the picker.
    #[props(default)]
    branches: ReadSignal<HashMap<String, (usize, usize)>>,
    /// Hover-reveal rule for the action bars.
    #[props(default)]
    autohide: Autohide,
    /// Hide the action bars while a reply runs.
    #[props(default)]
    hide_when_running: bool,
    /// Called with the message id when Reload is pressed.
    on_reload: Option<EventHandler<String>>,
    /// Called with the message id when Edit is pressed.
    on_edit: Option<EventHandler<String>>,
    /// Called with the message id and the vote (`None` when taken back).
    on_feedback: Option<EventHandler<(String, Option<Feedback>)>>,
    /// Called with the message id and its Markdown when Export is pressed.
    on_export: Option<EventHandler<(String, String)>>,
    /// Called with the message id and the branch to show.
    on_branch_switch: Option<EventHandler<(String, usize)>>,
    /// Called with the message id and whether reading aloud should start.
    on_speak: Option<EventHandler<(String, bool)>>,
    /// Called with the message id when Try again is pressed on an error.
    on_retry: Option<EventHandler<String>>,
    /// When set, selecting text in a message shows a Quote button that calls this.
    on_quote: Option<EventHandler<Selection>>,
    /// Accessible name for the message list.
    #[props(default = "Conversation".to_string())]
    label: String,
    /// Text shown while there are no messages.
    #[props(default = "No messages yet.".to_string())]
    empty: String,
    /// Below the messages, such as a composer.
    children: Element,
) -> Element {
    let msgs = messages.read();
    let branch_map = branches.read();
    let last_key = msgs.last().map(|m| m.id.clone());
    let revision = msgs.last().map_or(0, |m| {
        m.parts.iter().map(|p| format!("{p:?}").len()).sum::<usize>() + m.error.as_ref().map_or(0, String::len)
    });
    let is_running = running();
    let typing = is_running && msgs.last().is_none_or(|m| m.role == Role::User || m.parts.is_empty());
    let last_index = msgs.len().saturating_sub(1);

    rsx! {
        div { "data-scope": h::SCOPE_THREAD, "data-part": "root", "data-running": is_running.then_some(""),
            div { "data-scope": h::SCOPE_THREAD, "data-part": "scroller",
                MessageScroller { count: msgs.len(), last_key, revision, label,
                    if msgs.is_empty() {
                        p { "data-scope": h::SCOPE_THREAD, "data-part": "empty", "{empty}" }
                    }
                    for (i , m) in msgs.iter().enumerate() {
                        {
                            let (reload_id, edit_id, retry_id, fb_id, export_id, switch_id, speak_id) = (
                                m.id.clone(),
                                m.id.clone(),
                                m.id.clone(),
                                m.id.clone(),
                                m.id.clone(),
                                m.id.clone(),
                                m.id.clone(),
                            );
                            rsx! {
                                AssistantMessage {
                                    key: "{m.id}",
                                    message: m.clone(),
                                    running: is_running,
                                    is_last: i == last_index,
                                    branch: branch_map.get(&m.id).copied(),
                                    autohide,
                                    hide_when_running,
                                    on_reload: move |()| {
                                        if let Some(h) = on_reload {
                                            h.call(reload_id.clone());
                                        }
                                    },
                                    on_edit: move |()| {
                                        if let Some(h) = on_edit {
                                            h.call(edit_id.clone());
                                        }
                                    },
                                    on_retry: move |()| {
                                        if let Some(h) = on_retry {
                                            h.call(retry_id.clone());
                                        }
                                    },
                                    on_feedback: move |v| {
                                        if let Some(h) = on_feedback {
                                            h.call((fb_id.clone(), v));
                                        }
                                    },
                                    on_export: move |md| {
                                        if let Some(h) = on_export {
                                            h.call((export_id.clone(), md));
                                        }
                                    },
                                    on_branch_switch: move |i| {
                                        if let Some(h) = on_branch_switch {
                                            h.call((switch_id.clone(), i));
                                        }
                                    },
                                    on_speak: move |on| {
                                        if let Some(h) = on_speak {
                                            h.call((speak_id.clone(), on));
                                        }
                                    },
                                }
                            }
                        }
                    }
                    if typing {
                        div {
                            "data-scope": h::SCOPE_THREAD,
                            "data-part": "typing",
                            role: "status",
                            "aria-label": "Assistant is typing",
                            span {}
                            span {}
                            span {}
                        }
                    }
                }
            }
            div { "data-scope": h::SCOPE_THREAD, "data-part": "footer", {children} }
            if let Some(on_quote) = on_quote {
                AssistantSelectionToolbar { on_quote }
            }
        }
    }
}

// ------------------------------------------------------------------ message

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

// ------------------------------------------------------------- branch picker

/// Previous, "2 / 3", next: steps through the branches of one message.
#[component]
pub fn AssistantBranchPicker(
    /// How many branches there are.
    count: usize,
    /// Controlled position (zero-based). Pair with `on_switch`.
    index: Option<usize>,
    /// A reply is running: switching is disabled unless `switch_during_run`.
    #[props(default)]
    running: bool,
    /// Allow switching while a reply runs.
    #[props(default)]
    switch_during_run: bool,
    /// Called with the zero-based branch to show.
    on_switch: Option<EventHandler<usize>>,
) -> Element {
    let input = BranchPickerInput { count, index, running, switch_during_run };
    let mut state = use_signal(|| BranchPickerMachine::initial(&input));
    // Follow the owner's index and count on every render.
    let current = BranchPickerMachine::sync(&state.read(), &input).unwrap_or_else(|| state.read().clone());
    let mut send = move |event: BranchPickerEvent| {
        let synced = BranchPickerMachine::sync(&state.peek(), &input);
        let base = synced.clone().unwrap_or_else(|| state.peek().clone());
        let step = BranchPickerMachine::transition(&base, event, &input);
        if let Some(next) = step.state.or(synced) {
            state.set(next);
        }
        for BranchPickerCommand::Switch(i) in step.commands {
            if let Some(handler) = on_switch {
                handler.call(i);
            }
        }
    };
    rsx! {
        nav { "data-scope": h::SCOPE_BRANCH_PICKER, "data-part": "root", "aria-label": "Branches",
            button {
                r#type: "button",
                "data-scope": h::SCOPE_BRANCH_PICKER,
                "data-part": "previous",
                "aria-label": "Previous branch",
                disabled: !h::can_previous(&current, &input),
                onclick: move |_| send(BranchPickerEvent::Previous),
                Icon { name: IconName::ChevronRight, size: 14 }
            }
            span { "data-scope": h::SCOPE_BRANCH_PICKER, "data-part": "label", "aria-live": "polite",
                "{h::branch_label(current.index.value, count)}"
            }
            button {
                r#type: "button",
                "data-scope": h::SCOPE_BRANCH_PICKER,
                "data-part": "next",
                "aria-label": "Next branch",
                disabled: !h::can_next(&current, &input),
                onclick: move |_| send(BranchPickerEvent::Next),
                Icon { name: IconName::ChevronRight, size: 14 }
            }
        }
    }
}

// ----------------------------------------------------------------- action bar

/// Copy, reload, edit, export, read aloud and the two votes for one message. Which
/// buttons show depends on the role.
#[component]
pub fn AssistantActionBar(
    /// Whose message this is.
    #[props(default)]
    role: Role,
    /// What Copy writes to the clipboard.
    text: String,
    /// What Export hands to `on_export`.
    markdown: String,
    /// A reply is running.
    #[props(default)]
    running: bool,
    /// This is the last message in the thread.
    #[props(default)]
    is_last: bool,
    /// Hover-reveal rule.
    #[props(default)]
    autohide: Autohide,
    /// Hide the bar while a reply runs.
    #[props(default)]
    hide_when_running: bool,
    /// Called when Reload is pressed.
    on_reload: Option<EventHandler<()>>,
    /// Called when Edit is pressed.
    on_edit: Option<EventHandler<()>>,
    /// Called with the vote (`None` when taken back).
    on_feedback: Option<EventHandler<Option<Feedback>>>,
    /// Called with the Markdown export.
    on_export: Option<EventHandler<String>>,
    /// Called with whether reading aloud should start.
    on_speak: Option<EventHandler<bool>>,
) -> Element {
    let input = ActionBarInput { running, hide_when_running, autohide, is_last };
    // Props aren't reactive inside handlers; refresh the value they read each render.
    let mut input_cell = use_hook(|| CopyValue::new(input));
    *input_cell.write() = input;
    let mut state = use_signal(ActionBarState::default);
    // Strings aren't Copy; hold them where the handlers can all reach them.
    let mut payload = use_hook(|| CopyValue::new((String::new(), String::new())));
    *payload.write() = (text, markdown);
    let mut send = move |event: ActionBarEvent| {
        for command in drive::<ActionBarMachine>(state, input_cell, event) {
            match command {
                ActionBarCommand::Copy => {
                    let text = payload.read().0.clone();
                    spawn(async move {
                        // Best-effort: without clipboard access the button just confirms anyway.
                        js::copy(&text);
                        js::sleep(1400).await;
                        drive::<ActionBarMachine>(state, input_cell, ActionBarEvent::CopiedReset);
                    });
                }
                ActionBarCommand::Feedback(v) => {
                    if let Some(handler) = on_feedback {
                        handler.call(v);
                    }
                }
                ActionBarCommand::Reload => {
                    if let Some(handler) = on_reload {
                        handler.call(());
                    }
                }
                ActionBarCommand::Edit => {
                    if let Some(handler) = on_edit {
                        handler.call(());
                    }
                }
                ActionBarCommand::Export => {
                    if let Some(handler) = on_export {
                        handler.call(payload.read().1.clone());
                    }
                }
                ActionBarCommand::Speak => {
                    if let Some(handler) = on_speak {
                        handler.call(true);
                    }
                }
                ActionBarCommand::StopSpeaking => {
                    if let Some(handler) = on_speak {
                        handler.call(false);
                    }
                }
            }
        }
    };
    if h::action_bar_hidden(&input) {
        return rsx! {};
    }
    let s = state.read().clone();
    rsx! {
        div {
            "data-scope": h::SCOPE_ACTION_BAR,
            "data-part": "root",
            role: "toolbar",
            "aria-label": "Message actions",
            "aria-live": "polite",
            "data-role": role.as_str(),
            "data-floating": h::action_bar_floating(&input).then_some(""),
            for name in h::actions_for(role).iter().copied() {
                {
                    let label = match name {
                        "copy" if s.copied => "Copied",
                        "copy" => "Copy",
                        "reload" => "Reload",
                        "edit" => "Edit",
                        "export" => "Export",
                        "speak" if s.speaking => "Stop",
                        "speak" => "Read aloud",
                        "positive" => "Helpful",
                        _ => "Not helpful",
                    };
                    let pressed = match name {
                        "speak" => Some(s.speaking),
                        "positive" => Some(s.feedback == Some(Feedback::Positive)),
                        "negative" => Some(s.feedback == Some(Feedback::Negative)),
                        _ => None,
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
                    rsx! {
                        button {
                            key: "{name}",
                            r#type: "button",
                            "data-scope": h::SCOPE_ACTION_BAR,
                            "data-part": "action",
                            "data-action": name,
                            "aria-pressed": pressed.map(|p| p.to_string()),
                            disabled: name == "copy" && running,
                            onclick: move |_| send(event),
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}

// ----------------------------------------------------------- selection toolbar

const WATCH_SELECTION: &str = "
if (window.__hydroxySelection) document.removeEventListener('selectionchange', window.__hydroxySelection);
window.__hydroxySelection = () => {
  const s = window.getSelection();
  let out = '';
  if (s && !s.isCollapsed && s.rangeCount) {
    const r = s.getRangeAt(0);
    const n = r.commonAncestorContainer;
    const el = n.nodeType === 1 ? n : n.parentElement;
    const m = el && el.closest('[data-message-id]');
    if (m) {
      const b = r.getBoundingClientRect();
      out = [m.dataset.messageId, b.left, b.top, s.toString()].join('\\u0001');
    }
  }
  dioxus.send(out);
};
document.addEventListener('selectionchange', window.__hydroxySelection);
await new Promise(() => {});
";

fn parse_selection(raw: &str) -> Option<Selection> {
    let mut it = raw.splitn(4, '\u{1}');
    Some(Selection {
        message_id: it.next()?.to_string(),
        left: it.next()?.parse().ok()?,
        top: it.next()?.parse().ok()?,
        text: it.next()?.to_string(),
    })
}

/// A Quote button that floats over text selected inside a message (anything under a
/// `[data-message-id]` element). Rendered by `AssistantThread` when `on_quote` is set.
#[component]
pub fn AssistantSelectionToolbar(
    /// Called with the selection when Quote is pressed. The page selection is then cleared.
    on_quote: EventHandler<Selection>,
) -> Element {
    let state = use_signal(|| SelectionToolbarMachine::initial(&()));
    let unit = use_signal(|| ());
    let send = move |event: SelectionToolbarEvent| {
        for SelectionToolbarCommand::Quote(selection) in drive::<SelectionToolbarMachine>(state, unit, event) {
            on_quote.call(selection);
            document::eval("window.getSelection()?.removeAllRanges();");
        }
    };
    use_hook(|| {
        spawn(async move {
            let mut watcher = document::eval(WATCH_SELECTION);
            while let Ok(raw) = watcher.recv::<String>().await {
                send(parse_selection(&raw).map_or(SelectionToolbarEvent::Cleared, SelectionToolbarEvent::Selected));
            }
        });
    });
    use_drop(|| {
        document::eval(
            "if (window.__hydroxySelection) { document.removeEventListener('selectionchange', window.__hydroxySelection); window.__hydroxySelection = null; }",
        );
    });

    let selection = state.read().selection.clone();
    rsx! {
        if let Some(s) = selection {
            div {
                "data-scope": h::SCOPE_SELECTION_TOOLBAR,
                "data-part": "root",
                role: "toolbar",
                "aria-label": "Selection",
                style: "left: {s.left.max(8.0)}px; top: {s.top}px;",
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE_SELECTION_TOOLBAR,
                    "data-part": "quote",
                    // Keep the page selection alive while the button is pressed.
                    onmousedown: move |e| e.prevent_default(),
                    onclick: move |_| send(SelectionToolbarEvent::Quote),
                    "Quote"
                }
            }
        }
    }
}
