//! The [`AssistantQueueItem`] row.

use dioxus::prelude::*;
use hydroxy_headless::assistant_thread::SCOPE_QUEUE_ITEM;

use crate::icon::{Icon, IconName};

/// One message waiting for the running reply to end, with "Send now" and remove buttons.
#[component]
pub fn AssistantQueueItem(
    /// The queued message.
    text: String,
    /// Called when "Send now" is pressed: interrupt the reply and send this message.
    on_steer: Option<EventHandler<()>>,
    /// Called when remove is pressed.
    on_remove: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        li { "data-scope": SCOPE_QUEUE_ITEM, "data-part": "root",
            span { "data-scope": SCOPE_QUEUE_ITEM, "data-part": "text", title: "{text}", "{text}" }
            button {
                r#type: "button",
                "data-scope": SCOPE_QUEUE_ITEM,
                "data-part": "steer",
                "aria-label": "Send queued message now",
                onclick: move |_| {
                    if let Some(handler) = on_steer {
                        handler.call(());
                    }
                },
                "Send now"
            }
            button {
                r#type: "button",
                "data-scope": SCOPE_QUEUE_ITEM,
                "data-part": "remove",
                "aria-label": "Remove queued message",
                title: "Remove queued message",
                onclick: move |_| {
                    if let Some(handler) = on_remove {
                        handler.call(());
                    }
                },
                Icon { name: IconName::X, size: 14 }
            }
        }
    }
}
