use std::collections::HashMap;

use dioxus::prelude::*;
use hydroxy_dioxus::assistant_thread::{
    AssistantMessageData as Msg, AssistantThread, Autohide, Feedback, Part, PartStatus, Role, Selection,
};

const ANSWER: &str = "Pricing came up in **nine of the fourteen** interviews, usually as a surprise at renewal rather than at sign-up.\n\n- Onboarding was the second theme: people who skipped the setup call churned twice as often.\n- Support wait times on integrations came third.";

async fn sleep(ms: u32) {
    let _ = document::eval(&format!("await new Promise((r) => setTimeout(r, {ms}));")).await;
}

fn search_tool(status: PartStatus, done: bool) -> Part {
    Part::Tool {
        name: "search_interviews".into(),
        args: "{ \"query\": \"why did you cancel\", \"quarter\": \"Q3\" }".into(),
        result: done.then(|| "14 transcripts matched".into()),
        status,
    }
}

fn alternatives() -> Vec<Vec<Part>> {
    let sources = || {
        vec![
            Part::Source { title: "q3-interviews.md".into(), url: "#".into() },
            Part::Source { title: "support-tickets.csv".into(), url: "#".into() },
        ]
    };
    let lead = || {
        vec![
            Part::Reasoning("The question is about churn in Q3, so the interview transcripts are the place to start.".into()),
            search_tool(PartStatus::Complete, true),
        ]
    };
    let mut a = lead();
    a.push(Part::Text(ANSWER.into()));
    a.extend(sources());
    let mut b = lead();
    b.push(Part::Text("Three themes stand out: **pricing surprises at renewal**, skipped onboarding and slow integration support.".into()));
    b.extend(sources());
    vec![a, b, vec![Part::Text("Short version: pricing, onboarding, support.".into())]]
}

#[component]
pub fn AssistantThreadDemo() -> Element {
    let mut messages = use_signal(|| {
        vec![
            Msg::text("m1", Role::User, "What are the main churn drivers in the Q3 interviews?"),
            Msg::new("m2", Role::Assistant, alternatives().remove(0)),
            Msg::text("m3", Role::User, "Can you pull the renewal pricing figures too?"),
            Msg::new("m4", Role::Assistant, vec![search_tool(PartStatus::Failed, false)])
                .error("The billing export timed out before it returned any rows."),
        ]
    });
    let mut branches = use_signal(|| HashMap::from([("m2".to_string(), (0_usize, 3_usize))]));
    let mut running = use_signal(|| false);
    let mut status = use_signal(|| "Select text in a message to quote it.".to_string());
    let mut quote = use_signal(|| None::<String>);
    let mut draft = use_signal(String::new);
    // Each reply gets a number; Stop (or a newer reply) bumps it so the old task ends.
    let mut run = use_signal(|| 0_u32);
    let mut next = use_signal(|| 5_u32);

    let mut reply = move || {
        let mine = *run.peek() + 1;
        run.set(mine);
        let id = format!("m{}", *next.peek());
        next += 1;
        running.set(true);
        spawn(async move {
            sleep(600).await;
            if *run.peek() != mine {
                return;
            }
            let first = Part::Reasoning("Checking the interviews for anything about price before answering.".into());
            messages.write().push(Msg::new(id.clone(), Role::Assistant, vec![first]).streaming());
            sleep(700).await;
            if *run.peek() != mine {
                return;
            }
            if let Some(m) = messages.write().iter_mut().find(|m| m.id == id) {
                m.parts.push(search_tool(PartStatus::Running, false));
            }
            sleep(900).await;
            if *run.peek() != mine {
                return;
            }
            if let Some(m) = messages.write().iter_mut().find(|m| m.id == id) {
                m.parts[1] = search_tool(PartStatus::Complete, true);
            }
            for word in ANSWER.split_inclusive(' ') {
                sleep(50).await;
                if *run.peek() != mine {
                    return;
                }
                if let Some(m) = messages.write().iter_mut().find(|m| m.id == id) {
                    match m.parts.last_mut() {
                        Some(Part::Text(t)) => t.push_str(word),
                        _ => m.parts.push(Part::Text(word.to_string())),
                    }
                }
            }
            if let Some(m) = messages.write().iter_mut().find(|m| m.id == id) {
                m.streaming = false;
            }
            running.set(false);
        });
    };
    let mut truncate_from = move |id: &str| {
        let mut v = messages.write();
        if let Some(i) = v.iter().position(|m| m.id == id) {
            v.truncate(i);
        }
    };

    rsx! {
        div { style: "display: grid; gap: var(--space-3); max-width: 760px;",
            div { style: "height: 640px; padding: var(--space-4); border: 1px solid var(--border-1); border-radius: var(--radius-lg); background: var(--surface-card);",
                AssistantThread {
                    messages,
                    running,
                    branches,
                    autohide: Autohide::NotLast,
                    on_reload: move |id: String| {
                        truncate_from(&id);
                        reply();
                    },
                    on_retry: move |id: String| {
                        truncate_from(&id);
                        reply();
                    },
                    on_edit: move |id: String| {
                        let text = messages.peek().iter().find(|m| m.id == id).map(Msg::plain_text);
                        truncate_from(&id);
                        draft.set(text.unwrap_or_default());
                    },
                    on_feedback: move |(_, v): (String, Option<Feedback>)| {
                        status.set(
                            match v {
                                Some(Feedback::Positive) => "Marked helpful.",
                                Some(Feedback::Negative) => "Marked not helpful.",
                                None => "Vote taken back.",
                            }
                                .to_string(),
                        );
                    },
                    on_export: move |(_, md): (String, String)| {
                        status.set(format!("Exported {} characters of Markdown.", md.len()));
                    },
                    on_speak: move |(_, on): (String, bool)| {
                        status.set(if on { "Reading aloud." } else { "Stopped reading." }.to_string());
                    },
                    on_branch_switch: move |(id, i): (String, usize)| {
                        if let Some(e) = branches.write().get_mut(&id) {
                            e.0 = i;
                        }
                        let parts = alternatives().remove(i);
                        if let Some(m) = messages.write().iter_mut().find(|m| m.id == id) {
                            m.parts = parts;
                        }
                    },
                    on_quote: move |s: Selection| quote.set(Some(s.text)),
                    if let Some(q) = quote() {
                        p { style: "margin: 0; padding: 6px 10px; border-left: 2px solid var(--accent); font: var(--text-sm); color: var(--text-2);",
                            "Quoting: {q} "
                            button { r#type: "button", onclick: move |_| quote.set(None), "Clear" }
                        }
                    }
                    form {
                        style: "display: flex; gap: var(--space-2);",
                        onsubmit: move |e| {
                            e.prevent_default();
                            let text = draft().trim().to_string();
                            if text.is_empty() || running() {
                                return;
                            }
                            let text = match quote() {
                                Some(q) => format!("> {q}\n\n{text}"),
                                None => text,
                            };
                            quote.set(None);
                            draft.set(String::new());
                            let id = format!("m{}", *next.peek());
                            next += 1;
                            messages.write().push(Msg::text(id, Role::User, text));
                            reply();
                        },
                        input {
                            r#type: "text",
                            "aria-label": "Message",
                            placeholder: "Ask a follow-up",
                            style: "flex: 1; padding: 8px 12px; border: 1px solid var(--border-strong); border-radius: var(--radius-md); background: var(--surface-card); color: var(--text-1); font: var(--text-ui);",
                            value: "{draft}",
                            oninput: move |e| draft.set(e.value()),
                        }
                        if running() {
                            button {
                                r#type: "button",
                                onclick: move |_| {
                                    run += 1;
                                    running.set(false);
                                    messages.write().iter_mut().for_each(|m| m.streaming = false);
                                },
                                "Stop"
                            }
                        } else {
                            button { r#type: "submit", "Send" }
                        }
                    }
                }
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);", role: "status", "{status}" }
        }
    }
}
