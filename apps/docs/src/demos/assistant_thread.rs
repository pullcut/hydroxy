use std::collections::HashMap;
use std::time::Duration;

use hydroxy_leptos::assistant_thread::{
    AssistantMessageData as Msg, AssistantThread, Feedback, Part, PartStatus, Role, Selection,
};
use leptos::prelude::*;

const ANSWER: &str = "Pricing came up in **nine of the fourteen** interviews, usually as a surprise at renewal rather than at sign-up.\n\n- Onboarding was the second theme: people who skipped the setup call churned twice as often.\n- Support wait times on integrations came third.";

fn search_tool(status: PartStatus, done: bool) -> Part {
    Part::Tool {
        name: "search_interviews".into(),
        args: "{ \"query\": \"why did you cancel\", \"quarter\": \"Q3\" }".into(),
        result: done.then(|| "14 transcripts matched".into()),
        status,
    }
}

fn first_answer() -> Vec<Vec<Part>> {
    let sources = |a: &str, b: &str| {
        vec![Part::Source { title: a.into(), url: "#".into() }, Part::Source { title: b.into(), url: "#".into() }]
    };
    let lead = || {
        vec![
            Part::Reasoning(
                "The question is about churn in Q3, so the interview transcripts are the place to start.".into(),
            ),
            search_tool(PartStatus::Complete, true),
            Part::Tool {
                name: "read_file".into(),
                args: "{ \"path\": \"support-tickets.csv\" }".into(),
                result: Some("412 rows".into()),
                status: PartStatus::Complete,
            },
        ]
    };
    let mut a = lead();
    a.push(Part::Text(ANSWER.into()));
    a.extend(sources("q3-interviews.md", "support-tickets.csv"));
    let mut b = lead();
    b.push(Part::Text(
        "Three themes stand out: **pricing surprises at renewal**, skipped onboarding and slow integration support."
            .into(),
    ));
    b.extend(sources("q3-interviews.md", "support-tickets.csv"));
    let c = vec![Part::Text("Short version: pricing, onboarding, support. Pricing leads by a wide margin.".into())];
    vec![a, b, c]
}

fn edit(set: WriteSignal<Vec<Msg>>, id: &str, f: impl FnOnce(&mut Msg)) {
    set.try_update(|v| {
        if let Some(m) = v.iter_mut().find(|m| m.id == id) {
            f(m);
        }
    });
}

#[allow(clippy::too_many_lines, reason = "one demo; splitting it hides the wiring")]
pub fn demo() -> AnyView {
    let alternatives = StoredValue::new(first_answer());
    let (messages, set_messages) = signal(vec![
        Msg::text("m1", Role::User, "What are the main churn drivers in the Q3 interviews?"),
        Msg::new("m2", Role::Assistant, alternatives.with_value(|a| a[0].clone())),
        Msg::text("m3", Role::User, "Can you pull the renewal pricing figures too?"),
        Msg::new("m4", Role::Assistant, vec![search_tool(PartStatus::Failed, false)])
            .error("The billing export timed out before it returned any rows."),
    ]);
    let (branches, set_branches) = signal(HashMap::from([("m2".to_string(), (0_usize, 3_usize))]));
    let (running, set_running) = signal(false);
    let (status, set_status) = signal("Select text in a message to quote it.".to_string());
    let (quote, set_quote) = signal(None::<String>);
    let (draft, set_draft) = signal(String::new());
    // Each reply gets a number; Stop (or a newer reply) bumps it so the old timers end.
    let run = StoredValue::new(0_u32);
    let next = StoredValue::new(5_u32);

    let reply = move || {
        run.update_value(|r| *r += 1);
        let mine = run.get_value();
        let id = format!("m{}", next.get_value());
        next.update_value(|n| *n += 1);
        set_running.set(true);
        let live = move || run.try_get_value() == Some(mine);
        let words: Vec<String> = ANSWER.split_inclusive(' ').map(str::to_string).collect();
        set_timeout(
            move || {
                if !live() {
                    return;
                }
                let first =
                    Part::Reasoning("Checking the interviews for anything about price before answering.".into());
                set_messages.try_update(|m| m.push(Msg::new(id.clone(), Role::Assistant, vec![first]).streaming()));
                let id2 = id.clone();
                set_timeout(
                    move || {
                        if !live() {
                            return;
                        }
                        edit(set_messages, &id2, |m| m.parts.push(search_tool(PartStatus::Running, false)));
                        let id3 = id2.clone();
                        set_timeout(
                            move || {
                                if !live() {
                                    return;
                                }
                                edit(set_messages, &id3, |m| m.parts[1] = search_tool(PartStatus::Complete, true));
                                for (i, w) in words.into_iter().enumerate() {
                                    let id = id3.clone();
                                    set_timeout(
                                        move || {
                                            if !live() {
                                                return;
                                            }
                                            edit(set_messages, &id, |m| match m.parts.last_mut() {
                                                Some(Part::Text(t)) => t.push_str(&w),
                                                _ => m.parts.push(Part::Text(w)),
                                            });
                                        },
                                        Duration::from_millis(50 * (i as u64 + 1)),
                                    );
                                }
                                let id = id3.clone();
                                set_timeout(
                                    move || {
                                        if live() {
                                            edit(set_messages, &id, |m| m.streaming = false);
                                            set_running.set(false);
                                        }
                                    },
                                    Duration::from_millis(50 * (ANSWER.split_inclusive(' ').count() as u64 + 2)),
                                );
                            },
                            Duration::from_millis(900),
                        );
                    },
                    Duration::from_millis(700),
                );
            },
            Duration::from_millis(600),
        );
    };

    let send = move |text: String| {
        let text = text.trim().to_string();
        if text.is_empty() || running.get_untracked() {
            return;
        }
        let text = match quote.get_untracked() {
            Some(q) => format!("> {q}\n\n{text}"),
            None => text,
        };
        set_quote.set(None);
        let id = format!("m{}", next.get_value());
        next.update_value(|n| *n += 1);
        set_messages.update(|m| m.push(Msg::text(id, Role::User, text)));
        reply();
    };
    let stop = move |_| {
        run.update_value(|r| *r += 1);
        set_running.set(false);
        set_messages.update(|v| v.iter_mut().for_each(|m| m.streaming = false));
    };
    let truncate_from = move |id: &str| {
        set_messages.update(|v| {
            if let Some(i) = v.iter().position(|m| m.id == id) {
                v.truncate(i);
            }
        });
    };

    view! {
        <div style="display: grid; gap: var(--space-3); max-width: 760px;">
            <div style="height: 640px; padding: var(--space-4); border-radius: var(--radius-lg); background: var(--surface-sunken);">
                <AssistantThread
                    messages=messages
                    running=running
                    branches=branches
                    autohide=hydroxy_leptos::assistant_thread::Autohide::NotLast
                    on_reload=move |id: String| {
                        truncate_from(&id);
                        reply();
                    }
                    on_retry=move |id: String| {
                        truncate_from(&id);
                        reply();
                    }
                    on_edit=move |id: String| {
                        let text = messages.with_untracked(|v| v.iter().find(|m| m.id == id).map(Msg::plain_text));
                        truncate_from(&id);
                        set_draft.set(text.unwrap_or_default());
                    }
                    on_feedback=move |(_, v): (String, Option<Feedback>)| {
                        set_status
                            .set(
                                match v {
                                    Some(Feedback::Positive) => "Marked helpful.",
                                    Some(Feedback::Negative) => "Marked not helpful.",
                                    None => "Vote taken back.",
                                }
                                    .to_string(),
                            );
                    }
                    on_export=move |(_, md): (String, String)| {
                        set_status.set(format!("Exported {} characters of Markdown.", md.len()));
                    }
                    on_speak=move |(_, on): (String, bool)| {
                        set_status.set(if on { "Reading aloud." } else { "Stopped reading." }.to_string());
                    }
                    on_branch_switch=move |(id, i): (String, usize)| {
                        set_branches.update(|b| {
                            if let Some(e) = b.get_mut(&id) {
                                e.0 = i;
                            }
                        });
                        let parts = alternatives.with_value(|a| a[i].clone());
                        edit(set_messages, &id, |m| m.parts = parts);
                    }
                    on_quote=move |s: Selection| set_quote.set(Some(s.text))
                >
                    {move || {
                        quote
                            .get()
                            .map(|q| {
                                view! {
                                    <p style="margin: 0; padding: 6px 10px; border-left: 2px solid var(--accent); font: var(--text-sm); color: var(--text-2);">
                                        "Quoting: " {q}
                                        " "
                                        <button type="button" on:click=move |_| set_quote.set(None)>"Clear"</button>
                                    </p>
                                }
                            })
                    }}
                    <form
                        style="display: flex; gap: var(--space-2);"
                        on:submit=move |ev| {
                            ev.prevent_default();
                            send(draft.get_untracked());
                            set_draft.set(String::new());
                        }
                    >
                        <input
                            type="text"
                            aria-label="Message"
                            placeholder="Ask a follow-up"
                            style="flex: 1; padding: 8px 12px; border: 1px solid var(--border-strong); border-radius: var(--radius-md); background: var(--surface-card); color: var(--text-1); font: var(--text-ui);"
                            prop:value=draft
                            on:input=move |ev| set_draft.set(event_target_value(&ev))
                        />
                        {move || {
                            if running.get() {
                                view! { <button type="button" on:click=stop>"Stop"</button> }.into_any()
                            } else {
                                view! { <button type="submit">"Send"</button> }.into_any()
                            }
                        }}
                    </form>
                </AssistantThread>
            </div>
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);" role="status">{status}</p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "assistant-thread",
    name: "Assistant thread",
    group: "Chat",
    summary: "A conversation with an assistant. Messages hold text, images, sources and runs of reasoning and tool calls that fold into a chain of thought. Each message has an action bar, a branch picker when it has alternatives, and an error line when a run fails. The footer slot takes a composer.",
    demo,
    leptos: include_str!("assistant_thread.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/assistant_thread.rs"),
    parts: &[
        &[
            "AssistantThread",
            "root, scroller, footer, empty, typing",
            "Scrolling list over a footer slot. data-running while a reply runs",
        ],
        &["AssistantMessage", "root, avatar, body, footer", "One message. data-role, data-message-id, data-streaming"],
        &["AssistantMessagePart", "root", "A text, image or source part. data-type, data-cursor on streaming text"],
        &[
            "AssistantChainOfThought",
            "root, steps, step, step-label, status, step-text, args, result",
            "Collapsible run of reasoning and tool calls. data-running, data-status on a step",
        ],
        &["AssistantError", "root, icon, message, retry", "role=alert line under a failed message"],
        &["AssistantBranchPicker", "root, previous, next, label", "A nav with the position as 2 / 3"],
        &[
            "AssistantActionBar",
            "root, action",
            "A toolbar. data-action names the button; data-floating reveals on hover",
        ],
        &["AssistantSelectionToolbar", "root, quote", "Fixed above the selected text inside a message"],
    ],
    keys: &[
        &["Tab", "Move between the action bar buttons and the branch picker"],
        &["Enter / Space", "Press the focused button"],
    ],
    props: &[
        &["messages", "Vec<AssistantMessageData>", "Oldest first. A message has id, role, parts, error and streaming"],
        &["running", "bool", "A reply is running: typing indicator, Copy and branch switching disabled"],
        &[
            "branches",
            "HashMap<String, (usize, usize)>",
            "Message id to (index, count). Shows the picker when count is above one",
        ],
        &[
            "autohide / hide_when_running",
            "Autohide / bool",
            "Never, Always or NotLast; hide action bars while a reply runs",
        ],
        &["on_reload / on_edit / on_retry", "Callback<String>", "Message id. You change the messages"],
        &["on_feedback", "Callback<(String, Option<Feedback>)>", "Message id and the vote; None when taken back"],
        &["on_export", "Callback<(String, String)>", "Message id and its Markdown"],
        &["on_branch_switch", "Callback<(String, usize)>", "Message id and the branch to show"],
        &["on_speak", "Callback<(String, bool)>", "Message id and whether reading should start"],
        &["on_quote", "Callback<Selection>", "When set, selecting message text shows a Quote button"],
        &["children", "Children", "Footer slot below the messages, such as a composer"],
    ],
    events: &[],
};
