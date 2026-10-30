use std::time::Duration;

use hydroxy_leptos::assistant_composer::{
    AssistantComposer, AssistantSuggestions, AttachmentFile, AttachmentItem, AttachmentStatus, Suggestion,
};
use hydroxy_leptos::chat_bubble::{ChatBubble, ChatRole};
use leptos::prelude::*;

const REPLIES: [&str; 3] = [
    "Pricing came up in nine of the fourteen interviews, usually as a surprise at renewal rather than at sign-up.",
    "Onboarding was the second theme: people who skipped the setup call churned twice as often.",
    "Support was slowest on integrations. Median first reply was 19 hours, against 3 hours for billing.",
];

// A fake upload: 20% every quarter second.
fn advance(files: RwSignal<Vec<AttachmentItem>>, id: String, progress: u8) {
    files.try_update(|all| {
        if let Some(f) = all.iter_mut().find(|f| f.id == id)
            && matches!(f.status, AttachmentStatus::Uploading(_))
        {
            f.status = if progress >= 100 { AttachmentStatus::Ready } else { AttachmentStatus::Uploading(progress) };
        }
    });
    if progress < 100 {
        set_timeout(move || advance(files, id, progress + 20), Duration::from_millis(250));
    }
}

#[allow(clippy::too_many_lines, reason = "one demo view")]
#[component]
pub fn Demo() -> impl IntoView {
    let (messages, set_messages) = signal(Vec::<(u32, ChatRole, String)>::new());
    let (running, set_running) = signal(false);
    let files_rw = RwSignal::new(vec![AttachmentItem::new("f0", "q3-interviews.pdf", 2_516_582)]);
    let (files, set_files) = files_rw.split();
    let (quote, set_quote) = signal(None::<String>);
    // Each reply gets a number; Stop or a steer bumps it so the old timer ends.
    let run = StoredValue::new(0_u32);
    let next = StoredValue::new(1_u32);
    let upload = StoredValue::new(1_u32);

    let push = move |role: ChatRole, text: String| {
        let id = next.get_value();
        next.update_value(|n| *n += 1);
        set_messages.update(|m| m.push((id, role, text)));
    };
    let start = move |text: String| {
        let n = files.with_untracked(Vec::len);
        let shown = if n > 0 && !text.is_empty() { format!("{text} ({n} attached)") } else { text };
        push(ChatRole::User, shown);
        set_files.set(Vec::new());
        set_quote.set(None);
        run.update_value(|r| *r += 1);
        let mine = run.get_value();
        set_running.set(true);
        set_timeout(
            move || {
                if run.try_get_value() != Some(mine) {
                    return;
                }
                push(ChatRole::Assistant, REPLIES[mine as usize % REPLIES.len()].to_string());
                set_running.set(false);
            },
            Duration::from_secs(3),
        );
    };

    let on_files = move |picked: Vec<AttachmentFile>| {
        for f in picked {
            let id = format!("u{}", upload.get_value());
            upload.update_value(|n| *n += 1);
            // Anything over 10 MB fails, so the failed chip is visible.
            if f.size > 10_000_000 {
                let item =
                    AttachmentItem::new(id, f.name, f.size).status(AttachmentStatus::Failed("over 10 MB".to_string()));
                set_files.update(|all| all.push(item));
            } else {
                let item = AttachmentItem::new(id.clone(), f.name, f.size).status(AttachmentStatus::Uploading(0));
                set_files.update(|all| all.push(item));
                advance(files_rw, id, 20);
            }
        }
    };

    view! {
        <div style="display: grid; gap: var(--space-4); max-width: 640px;">
            <div style="display: grid; gap: var(--space-4); min-height: 120px;">
                {move || {
                    if messages.with(Vec::is_empty) {
                        view! {
                            <AssistantSuggestions
                                items=vec![
                                    Suggestion::new("Summarise the interviews", "Themes across all fourteen", "Summarise the Q3 interviews by theme."),
                                    Suggestion::new("Why did churn rise?", "Look at pricing and onboarding", "Why did churn rise in Q3?"),
                                ]
                                on_pick=start
                            />
                        }
                            .into_any()
                    } else {
                        view! {
                            <For
                                each=move || messages.get()
                                key=|m| m.0
                                children=move |(_, role, text)| {
                                    let reply = role == ChatRole::Assistant;
                                    view! {
                                        <ChatBubble role=role>
                                            {text.clone()}
                                            {reply
                                                .then(|| {
                                                    let t = text.clone();
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class="ox-button"
                                                            data-variant="ghost"
                                                            data-size="sm"
                                                            on:click=move |_| set_quote.set(Some(t.clone()))
                                                        >
                                                            "Quote"
                                                        </button>
                                                    }
                                                })}
                                        </ChatBubble>
                                    }
                                }
                            />
                        }
                            .into_any()
                    }
                }}
            </div>
            <AssistantComposer
                running=running
                placeholder="Ask about the interviews. Type @ for people, / for commands."
                mentions=vec!["Priya Nair".to_string(), "Tom Okafor".to_string(), "Sam Whitlock".to_string()]
                commands=vec!["summarise".to_string(), "compare".to_string(), "cite-sources".to_string()]
                accept="image/*,.pdf,.csv,.md"
                attachments=files
                quote=quote
                on_quote_dismiss=move |()| set_quote.set(None)
                on_send=start
                on_steer=move |text: String| {
                    push(ChatRole::Assistant, "Stopped to read your new message.".to_string());
                    start(text);
                }
                on_cancel=move |()| {
                    run.update_value(|r| *r += 1);
                    set_running.set(false);
                    push(ChatRole::Assistant, "Stopped.".to_string());
                }
                on_add_files=on_files
                on_remove_attachment=move |id: String| set_files.update(|all| all.retain(|f| f.id != id))
            />
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">
                "Send starts a three second reply. Send again to queue, Ctrl+Enter to steer, Escape to stop, Up to recall."
            </p>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "assistant-composer",
    name: "Assistant composer",
    group: "Chat",
    summary: "The assistant's message box. Messages written while a reply runs wait in a queue and send in order when it ends. Up recalls earlier messages, @ and / open a picker, and files attach as chips with upload progress. Suggestion cards fill an empty thread.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("assistant_composer.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/assistant_composer.rs"),
    parts: &[
        &[
            "AssistantComposer",
            "root",
            "The card. data-running, data-dragging while files are dragged over, data-compact",
        ],
        &["", "quote / quote-text / quote-dismiss", "Quoted text above the field, with a button to remove it"],
        &["", "attachments", "Row of AssistantAttachment chips"],
        &["", "field / input", "Native textarea (role combobox) that grows with its text"],
        &["", "popover / option", "Listbox for @ and /. data-highlighted on the active option"],
        &["", "queue", "List of AssistantQueueItem rows"],
        &["", "actions", "Row holding the buttons below"],
        &["", "add-attachment / file-input", "Label around a native file input"],
        &["", "cancel / steer / send", "Stop and Steer show while running; send reads Queue"],
        &["AssistantQueueItem", "root / text / steer / remove", "A queued message with Send now and remove"],
        &[
            "AssistantAttachment",
            "root / thumb / body / name / meta / progress / remove",
            "A file chip. data-status is uploading, ready or failed; data-kind is the file type",
        ],
        &["AssistantSuggestions", "grid", "Wraps the cards"],
        &["AssistantSuggestion", "root / trigger / title / description", "One card; the trigger is a button"],
    ],
    keys: &[
        &["Enter", "Send, or queue while a reply runs. In the popover: pick the highlighted option"],
        &["Shift+Enter", "Add a line"],
        &["Ctrl/Cmd+Enter", "Send now, interrupting the running reply"],
        &["Escape", "Close the popover, or stop the running reply"],
        &["Up / Down", "Move through the popover. In an empty field: recall earlier messages"],
        &["Tab", "Pick the highlighted option"],
    ],
    props: &[
        &[
            "running",
            "Signal<bool>",
            "A reply is running. Sends queue; when it turns false the next queued message goes to on_send",
        ],
        &["on_send", "Callback<String>", "Message to send now. Start a reply and set running"],
        &["on_steer", "Callback<String>", "Message that should interrupt the running reply. Falls back to on_send"],
        &["on_cancel", "Callback<()>", "Stop was pressed"],
        &[
            "on_add_files",
            "Callback<Vec<AttachmentFile>>",
            "Names, sizes and types of picked or dropped files. Without it there is no add button",
        ],
        &[
            "attachments / on_remove_attachment",
            "Vec<AttachmentItem> / Callback<String>",
            "Chips above the field. An item has id, name, size, mime and a status of Uploading(percent), Ready or Failed(reason)",
        ],
        &["quote / on_quote_dismiss", "Option<String> / Callback<()>", "Quoted text above the field"],
        &["mentions / commands", "Vec<String>", "Options after @, and after a leading /"],
        &["accept / multiple", "String / bool", "File picker filter, and whether several files can be picked"],
        &["placeholder / label / compact", "String / String / bool", "Hint text, accessible name, tighter padding"],
        &[
            "AssistantSuggestions items / on_pick",
            "Vec<Suggestion> / Callback<String>",
            "Cards, and the text to send when one is pressed",
        ],
    ],
    events: &[],
};
