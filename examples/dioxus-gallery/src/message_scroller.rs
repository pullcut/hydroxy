use dioxus::prelude::*;
use oxide_dioxus::button::{Button, ButtonSize, ButtonVariant};
use oxide_dioxus::message_scroller::MessageScroller;

#[component]
pub fn MessageScrollerDemo() -> Element {
    // Messages are numbered; older ones go below zero so prepending is visible.
    let mut ids = use_signal(|| (1..=12).collect::<Vec<i32>>());
    let mut oldest = use_signal(|| 1_i32);

    rsx! {
        div { style: "display: grid; gap: var(--space-4); max-width: 520px;",
            div { style: "display: flex; flex-wrap: wrap; gap: var(--space-3);",
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    on_click: move |_| {
                        let next = ids.peek().last().copied().unwrap_or(0) + 1;
                        ids.write().push(next);
                    },
                    "New message"
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    on_click: move |_| {
                        let first = *oldest.peek();
                        oldest.set(first - 5);
                        ids.write().splice(0..0, (first - 5)..first);
                    },
                    "Load 5 older"
                }
            }
            div { style: "height: 320px; border: 1px solid var(--border-1); border-radius: var(--radius-lg); padding: var(--space-3);",
                MessageScroller {
                    count: ids.read().len(),
                    last_key: ids.read().last().map(ToString::to_string),
                    label: "Demo messages",
                    for id in ids.read().iter() {
                        p {
                            key: "{id}",
                            style: "margin: 0; padding: var(--space-3) var(--space-4); border-radius: var(--radius-md); background: var(--surface-sunken); font: var(--text-sm);",
                            "Message {id}"
                        }
                    }
                }
            }
        }
    }
}
