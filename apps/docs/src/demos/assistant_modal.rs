use hydroxy_leptos::assistant_modal::AssistantModal;
use hydroxy_leptos::assistant_thread::{AssistantMessageData as Msg, AssistantThread, Role};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (open, set_open) = signal(false);
    let (messages, set_messages) =
        signal(vec![Msg::text("m1", Role::Assistant, "Hi. Ask me about your Q3 interviews, or the renewal figures.")]);
    let (draft, set_draft) = signal(String::new());
    let next = StoredValue::new(2_u32);

    view! {
        <div style="display: grid; gap: var(--space-3); max-width: 760px;">
            <div style="position: relative; height: 640px; border: 1px solid var(--border-1); border-radius: var(--radius-lg); background: var(--surface-sunken); padding: var(--space-5);">
                <p style="margin: 0; font: var(--text-body); color: var(--text-2); max-width: 44ch;">
                    "Your page goes here. The assistant opens from the round button at the bottom-right and floats over it."
                </p>
                <AssistantModal on_open_change=move |o: bool| set_open.set(o) title="Ask hydroxy" trigger_label="Open assistant">
                    <AssistantThread messages=messages empty="Ask anything.">
                        <form
                            style="display: flex; gap: var(--space-2);"
                            on:submit=move |ev| {
                                ev.prevent_default();
                                let text = draft.get_untracked().trim().to_string();
                                if text.is_empty() {
                                    return;
                                }
                                set_draft.set(String::new());
                                let n = next.get_value();
                                next.set_value(n + 2);
                                set_messages.update(|m| m.push(Msg::text(format!("m{n}"), Role::User, text)));
                                set_timeout(
                                    move || {
                                        set_messages
                                            .try_update(|m| {
                                                m.push(Msg::text(format!("m{}", n + 1), Role::Assistant, "Pricing came up in nine of fourteen interviews."));
                                            });
                                    },
                                    std::time::Duration::from_millis(700),
                                );
                            }
                        >
                            <input
                                type="text"
                                aria-label="Message"
                                placeholder="Ask anything"
                                style="flex: 1; min-width: 0; padding: 8px 12px; border: 1px solid var(--border-strong); border-radius: var(--radius-md); background: var(--surface-card); color: var(--text-1); font: var(--text-ui);"
                                prop:value=draft
                                on:input=move |ev| set_draft.set(event_target_value(&ev))
                            />
                            <button type="submit">"Send"</button>
                        </form>
                    </AssistantThread>
                </AssistantModal>
            </div>
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);" role="status">
                {move || if open.get() { "Panel open. Escape closes it." } else { "Panel closed." }}
            </p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "assistant-modal",
    name: "Assistant modal",
    group: "Chat",
    summary: "A floating chat panel. A round button at the bottom-right of its container opens a panel above it; put a thread and composer inside. The panel stays mounted while closed, so the conversation keeps its place.",
    demo,
    leptos: include_str!("assistant_modal.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/assistant_modal.rs"),
    parts: &[
        &["AssistantModal", "root", "Positioned at the bottom-right of the nearest positioned ancestor. data-state"],
        &["", "panel", "role=dialog, labelled by the title. hidden while closed"],
        &["", "header / title / close", "Heading row with a close button"],
        &["", "body", "Where children render"],
        &["", "trigger", "Round button with aria-expanded and aria-controls"],
    ],
    keys: &[
        &["Escape", "Close the panel and return focus to the round button"],
        &["Enter / Space", "On the round button: open or close"],
    ],
    props: &[
        &["open", "Option<bool>", "Controlled open state. Pair with on_open_change"],
        &["default_open", "bool", "Open at first when uncontrolled"],
        &["on_open_change", "Callback<bool>", "Called whenever it opens or closes"],
        &["title", "String", "Panel heading and accessible name. Default Assistant"],
        &["trigger_label", "String", "Accessible name of the round button while closed"],
        &["children", "Children", "The panel contents"],
    ],
};
