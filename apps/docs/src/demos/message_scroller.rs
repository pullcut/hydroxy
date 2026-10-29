use hydroxy_leptos::button::{Button, ButtonSize, ButtonVariant};
use hydroxy_leptos::message_scroller::MessageScroller;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    // Messages are numbered; older ones go below zero so prepending is visible.
    let (ids, set_ids) = signal((1..=12).collect::<Vec<i32>>());
    let oldest = StoredValue::new(1_i32);

    view! {
        <div style="display: grid; gap: var(--space-4); max-width: 520px;">
            <div style="display: flex; flex-wrap: wrap; gap: var(--space-3);">
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Sm
                    on_click=move |_| set_ids.update(|v| v.push(v.last().copied().unwrap_or(0) + 1))
                >
                    "New message"
                </Button>
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Sm
                    on_click=move |_| {
                        let first = oldest.get_value();
                        oldest.set_value(first - 5);
                        set_ids.update(|v| {
                            v.splice(0..0, (first - 5)..first);
                        });
                    }
                >
                    "Load 5 older"
                </Button>
            </div>
            <div style="height: 320px; border: 1px solid var(--border-1); border-radius: var(--radius-lg); padding: var(--space-3);">
                <MessageScroller
                    count=Signal::derive(move || ids.with(Vec::len))
                    last_key=Signal::derive(move || ids.with(|v| v.last().map(ToString::to_string)))
                    label="Demo messages"
                >
                    <For
                        each=move || ids.get()
                        key=|id| *id
                        children=|id| {
                            view! {
                                <p style="margin: 0; padding: var(--space-3) var(--space-4); border-radius: var(--radius-md); background: var(--surface-sunken); font: var(--text-sm);">
                                    "Message " {id}
                                </p>
                            }
                        }
                    />
                </MessageScroller>
            </div>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "message-scroller",
    name: "Message scroller",
    group: "Chat",
    summary: "A scrolling list for messages. It sticks to the bottom as messages arrive, lets go when you scroll up, and offers a jump button with a count of what you missed.",
    demo,
    leptos: include_str!("message_scroller.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/message_scroller.rs"),
    parts: &[
        &["MessageScroller", "root", "Positions the jump button. data-state: following or away"],
        &["", "viewport", "The scrolling role=log region. Focusable so the keyboard can scroll it"],
        &["", "jump", "Shown while away from the bottom: \"Jump to latest\" or \"3 new messages\""],
    ],
    keys: &[
        &["Arrow keys / Page keys", "Scroll the focused list natively"],
        &["Enter / Space on the jump button", "Return to the bottom and follow again"],
    ],
    props: &[
        &["count", "usize", "How many messages the list holds"],
        &[
            "last_key",
            "Option<String>",
            "Key of the newest message. A new key means append; the same key with a larger count means older messages were prepended",
        ],
        &["revision", "usize", "Bump when the newest message grows in place, such as streaming text"],
        &["threshold", "f64", "Pixels from the bottom that still count as at the bottom. Default 48"],
        &["label", "String", "Accessible name. Default \"Messages\""],
    ],
    events: &[],
};
