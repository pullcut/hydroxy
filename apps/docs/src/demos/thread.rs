use std::time::Duration;

use hydroxy_leptos::thread::{Action, Message, MessageRole, Thread};
use leptos::prelude::*;

const REPLY: &str = "Pricing came up in nine of the fourteen interviews, usually as a surprise at renewal rather than at sign-up. Onboarding was the second theme: people who skipped the setup call churned twice as often.";

pub fn demo() -> AnyView {
    let (messages, set_messages) = signal(vec![
        Message::new("m1", MessageRole::User, "What are the main churn drivers in the Q3 interviews?").time("09:41"),
        Message::new("m2", MessageRole::Assistant, "Three themes stand out: pricing surprises at renewal, skipped onboarding, and slow support on integrations.")
            .time("09:41")
            .sources(vec!["q3-interviews.md".to_string(), "support-tickets.csv".to_string()]),
    ]);
    let (streaming, set_streaming) = signal(false);
    // Each reply gets a number; Stop (or a newer reply) bumps it so the old timer ends.
    let run = StoredValue::new(0_u32);
    let next = StoredValue::new(3_u32);

    let reply = move || {
        run.update_value(|r| *r += 1);
        let mine = run.get_value();
        let id = format!("m{}", next.get_value());
        next.update_value(|n| *n += 1);
        set_streaming.set(true);
        let words: Vec<&str> = REPLY.split_inclusive(' ').collect();
        // A pause with the typing indicator, then one word at a time.
        set_timeout(
            move || {
                if run.try_get_value() != Some(mine) {
                    return;
                }
                set_messages.try_update(|m| m.push(Message::new(id.clone(), MessageRole::Assistant, "")));
                for (i, w) in words.iter().enumerate() {
                    let (word, id) = ((*w).to_string(), format!("m{}", next.try_get_value().unwrap_or(1) - 1));
                    let last = i == words.len() - 1;
                    set_timeout(
                        move || {
                            if run.try_get_value() != Some(mine) {
                                return;
                            }
                            set_messages.try_update(|m| {
                                if let Some(msg) = m.iter_mut().find(|m| m.id == id) {
                                    msg.text.push_str(&word);
                                }
                            });
                            if last {
                                set_streaming.try_set(false);
                            }
                        },
                        Duration::from_millis(60 * (i as u64 + 1)),
                    );
                }
            },
            Duration::from_millis(900),
        );
    };

    view! {
        <div style="height: 460px; max-width: 640px;">
            <Thread
                messages=messages
                streaming=streaming
                suggestions=vec!["What changed since August?".to_string()]
                on_send=move |text: String| {
                    let id = format!("u{}", next.get_value());
                    next.update_value(|n| *n += 1);
                    set_messages.update(|m| m.push(Message::new(id, MessageRole::User, text)));
                    reply();
                }
                on_stop=move |()| {
                    run.update_value(|r| *r += 1);
                    set_streaming.set(false);
                }
                on_action=move |(id, action): (String, Action)| {
                    if action == Action::Retry {
                        set_messages.update(|m| m.retain(|m| m.id != id));
                        reply();
                    }
                }
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "thread",
    name: "Thread",
    group: "Chat",
    summary: "A whole conversation: chat bubbles in a message scroller over a composer, with a typing indicator, streaming text, stop, and copy and retry on each message.",
    demo,
    leptos: include_str!("thread.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/thread.rs"),
    parts: &[
        &["Thread", "root", "Column of scroller and composer. data-streaming while a reply streams"],
        &["", "scroller", "Wraps the message scroller"],
        &["", "message", "One message: bubble plus actions. data-role: user or assistant"],
        &["", "text", "The message text, line breaks kept"],
        &["", "actions", "Copy, and Retry on the last assistant message"],
        &["", "typing", "Three dots while a reply is coming and has no text"],
        &["", "stop", "Stop button, shown while streaming when on_stop is set"],
        &["", "composer", "The Composer and the stop button"],
    ],
    keys: &[
        &["Enter", "Sends the message"],
        &["Shift + Enter", "Adds a new line"],
        &["Tab", "Moves through the list, the actions, then the composer"],
    ],
    props: &[
        &["messages", "Vec<ThreadMessage>", "Oldest first. A message has id, role, text, optional time and sources"],
        &[
            "streaming",
            "bool",
            "A reply is being written. Shows the typing indicator, then follows the text as it grows",
        ],
        &["on_send", "Callback<String>", "Called with the trimmed text. You append the message"],
        &["on_stop", "Callback<()>", "Stop button handler. The button shows only while streaming"],
        &[
            "on_action",
            "Callback<(String, MessageAction)>",
            "Message id and Copy or Retry. Copy also writes to the clipboard",
        ],
        &["suggestions", "Vec<String>", "Tags above the composer"],
        &["label / placeholder / empty", "String", "Accessible name, composer placeholder, empty text"],
    ],
    events: &[],
};
