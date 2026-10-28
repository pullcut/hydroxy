use dioxus::prelude::*;
use hydroxy_dioxus::assistant_modal::AssistantModal;
use hydroxy_dioxus::assistant_thread::{AssistantMessageData as Msg, AssistantThread, Role};

#[component]
pub fn AssistantModalDemo() -> Element {
    let mut open = use_signal(|| false);
    let mut messages = use_signal(|| {
        vec![Msg::text("m1", Role::Assistant, "Hi. Ask me about your Q3 interviews, or the renewal figures.")]
    });
    let mut draft = use_signal(String::new);
    let mut next = use_signal(|| 2_u32);

    rsx! {
        div { style: "display: grid; gap: var(--space-3); max-width: 760px;",
            div { style: "position: relative; height: 640px; border: 1px solid var(--border-1); border-radius: var(--radius-lg); background: var(--surface-sunken); padding: var(--space-5);",
                p { style: "margin: 0; font: var(--text-body); color: var(--text-2); max-width: 44ch;",
                    "Your page goes here. The assistant opens from the round button at the bottom-right and floats over it."
                }
                AssistantModal {
                    on_open_change: move |o: bool| open.set(o),
                    title: "Ask hydroxy",
                    trigger_label: "Open assistant",
                    AssistantThread { messages, empty: "Ask anything.",
                        form {
                            style: "display: flex; gap: var(--space-2);",
                            onsubmit: move |e| {
                                e.prevent_default();
                                let text = draft().trim().to_string();
                                if text.is_empty() {
                                    return;
                                }
                                draft.set(String::new());
                                let n = *next.peek();
                                next += 2;
                                messages.write().push(Msg::text(format!("m{n}"), Role::User, text));
                                spawn(async move {
                                    let _ = document::eval("await new Promise((r) => setTimeout(r, 700));").await;
                                    messages
                                        .write()
                                        .push(Msg::text(format!("m{}", n + 1), Role::Assistant, "Pricing came up in nine of fourteen interviews."));
                                });
                            },
                            input {
                                r#type: "text",
                                "aria-label": "Message",
                                placeholder: "Ask anything",
                                style: "flex: 1; min-width: 0; padding: 8px 12px; border: 1px solid var(--border-strong); border-radius: var(--radius-md); background: var(--surface-card); color: var(--text-1); font: var(--text-ui);",
                                value: "{draft}",
                                oninput: move |e| draft.set(e.value()),
                            }
                            button { r#type: "submit", "Send" }
                        }
                    }
                }
            }
            p { style: "margin: 0; font: var(--text-sm); color: var(--text-2);", role: "status",
                if open() {
                    "Panel open. Escape closes it."
                } else {
                    "Panel closed."
                }
            }
        }
    }
}
