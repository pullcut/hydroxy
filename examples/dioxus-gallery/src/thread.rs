use dioxus::prelude::*;
use hydroxy_dioxus::thread::{Action, Message, MessageRole, Thread};

const REPLY: &str = "Pricing came up in nine of the fourteen interviews, usually as a surprise at renewal rather than at sign-up. Onboarding was the second theme: people who skipped the setup call churned twice as often.";

async fn sleep(ms: u32) {
    let _ = document::eval(&format!("await new Promise((r) => setTimeout(r, {ms}));")).await;
}

#[component]
pub fn ThreadDemo() -> Element {
    let mut messages = use_signal(|| {
        vec![
            Message::new("m1", MessageRole::User, "What are the main churn drivers in the Q3 interviews?").time("09:41"),
            Message::new(
                "m2",
                MessageRole::Assistant,
                "Three themes stand out: pricing surprises at renewal, skipped onboarding, and slow support on integrations.",
            )
            .time("09:41")
            .sources(vec!["q3-interviews.md".to_string(), "support-tickets.csv".to_string()]),
        ]
    });
    let mut streaming = use_signal(|| false);
    // Each reply gets a number; Stop (or a newer reply) bumps it so the old task ends.
    let mut run = use_signal(|| 0_u32);
    let mut next = use_signal(|| 3_u32);

    let mut reply = move || {
        let mine = *run.peek() + 1;
        run.set(mine);
        let id = format!("m{}", *next.peek());
        next += 1;
        streaming.set(true);
        spawn(async move {
            // A pause with the typing indicator, then one word at a time.
            sleep(900).await;
            if *run.peek() != mine {
                return;
            }
            messages.write().push(Message::new(id.clone(), MessageRole::Assistant, ""));
            for word in REPLY.split_inclusive(' ') {
                sleep(60).await;
                if *run.peek() != mine {
                    return;
                }
                if let Some(m) = messages.write().iter_mut().find(|m| m.id == id) {
                    m.text.push_str(word);
                }
            }
            streaming.set(false);
        });
    };

    rsx! {
        div { style: "height: 460px; max-width: 640px;",
            Thread {
                messages,
                streaming,
                suggestions: vec!["What changed since August?".to_string()],
                on_send: move |text: String| {
                    let id = format!("u{}", *next.peek());
                    next += 1;
                    messages.write().push(Message::new(id, MessageRole::User, text));
                    reply();
                },
                on_stop: move |()| {
                    run += 1;
                    streaming.set(false);
                },
                on_action: move |(id, action): (String, Action)| {
                    if action == Action::Retry {
                        messages.write().retain(|m| m.id != id);
                        reply();
                    }
                },
            }
        }
    }
}
