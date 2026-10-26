use dioxus::prelude::*;
use oxide_dioxus::assistant_composer::{
    AssistantComposer, AssistantSuggestions, AttachmentFile, AttachmentItem, AttachmentStatus, Suggestion,
};
use oxide_dioxus::chat_bubble::{ChatBubble, ChatRole};

const REPLIES: [&str; 3] = [
    "Pricing came up in nine of the fourteen interviews, usually as a surprise at renewal rather than at sign-up.",
    "Onboarding was the second theme: people who skipped the setup call churned twice as often.",
    "Support was slowest on integrations. Median first reply was 19 hours, against 3 hours for billing.",
];

async fn sleep(ms: u32) {
    let _ = document::eval(&format!("await new Promise((r) => setTimeout(r, {ms}));")).await;
}

#[component]
pub fn AssistantComposerDemo() -> Element {
    let mut messages = use_signal(Vec::<(u32, ChatRole, String)>::new);
    let mut running = use_signal(|| false);
    let mut files = use_signal(|| vec![AttachmentItem::new("f0", "q3-interviews.pdf", 2_516_582)]);
    let mut quote = use_signal(|| None::<String>);
    // Each reply gets a number; Stop or a steer bumps it so the old task ends.
    let mut run = use_signal(|| 0_u32);
    let mut next = use_signal(|| 1_u32);
    let mut upload = use_signal(|| 1_u32);

    let mut push = move |role: ChatRole, text: String| {
        let id = *next.peek();
        next += 1;
        messages.write().push((id, role, text));
    };
    let mut start = move |text: String| {
        let n = files.peek().len();
        let shown = if n > 0 && !text.is_empty() { format!("{text} ({n} attached)") } else { text };
        push(ChatRole::User, shown);
        files.set(Vec::new());
        quote.set(None);
        let mine = *run.peek() + 1;
        run.set(mine);
        running.set(true);
        spawn(async move {
            sleep(3000).await;
            if *run.peek() != mine {
                return;
            }
            push(ChatRole::Assistant, REPLIES[mine as usize % REPLIES.len()].to_string());
            running.set(false);
        });
    };

    rsx! {
        div { style: "display: grid; gap: var(--space-4); max-width: 640px;",
            div { style: "display: grid; gap: var(--space-4); min-height: 120px;",
                if messages.read().is_empty() {
                    AssistantSuggestions {
                        items: vec![
                            Suggestion::new("Summarise the interviews", "Themes across all fourteen", "Summarise the Q3 interviews by theme."),
                            Suggestion::new("Why did churn rise?", "Look at pricing and onboarding", "Why did churn rise in Q3?"),
                        ],
                        on_pick: move |text: String| start(text),
                    }
                } else {
                    for (id , role , text) in messages.read().clone() {
                        ChatBubble { key: "{id}", role, "{text}" }
                    }
                }
            }
            AssistantComposer {
                running,
                placeholder: "Ask about the interviews. Type @ for people, / for commands.",
                mentions: vec!["Priya Nair".to_string(), "Tom Okafor".to_string(), "Sam Whitlock".to_string()],
                commands: vec!["summarise".to_string(), "compare".to_string(), "cite-sources".to_string()],
                accept: "image/*,.pdf,.csv,.md",
                attachments: files,
                quote,
                on_quote_dismiss: move |()| quote.set(None),
                on_send: move |text: String| start(text),
                on_steer: move |text: String| {
                    push(ChatRole::Assistant, "Stopped to read your new message.".to_string());
                    start(text);
                },
                on_cancel: move |()| {
                    run += 1;
                    running.set(false);
                    push(ChatRole::Assistant, "Stopped.".to_string());
                },
                on_add_files: move |picked: Vec<AttachmentFile>| {
                    for f in picked {
                        let id = format!("u{}", *upload.peek());
                        upload += 1;
                        // Anything over 10 MB fails, so the failed chip is visible.
                        if f.size > 10_000_000 {
                            let item = AttachmentItem::new(id, f.name, f.size)
                                .status(AttachmentStatus::Failed("over 10 MB".to_string()));
                            files.write().push(item);
                        } else {
                            files.write().push(AttachmentItem::new(id.clone(), f.name, f.size).status(AttachmentStatus::Uploading(0)));
                            spawn(async move {
                                for p in [20_u8, 40, 60, 80, 100] {
                                    sleep(250).await;
                                    if let Some(a) = files.write().iter_mut().find(|a| a.id == id) {
                                        a.status = if p == 100 { AttachmentStatus::Ready } else { AttachmentStatus::Uploading(p) };
                                    }
                                }
                            });
                        }
                    }
                },
                on_remove_attachment: move |id: String| files.write().retain(|f| f.id != id),
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);",
                "Send starts a three second reply. Send again to queue, Ctrl+Enter to steer, Escape to stop, Up to recall."
            }
        }
    }
}
