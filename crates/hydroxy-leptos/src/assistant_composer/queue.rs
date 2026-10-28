//! The [`AssistantQueueItem`] row.

use hydroxy_headless::assistant_thread::SCOPE_QUEUE_ITEM;
use leptos::prelude::*;

use crate::icon::{Icon, IconName};

/// One message waiting for the running reply to end, with "Send now" and remove buttons.
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn AssistantQueueItem(
    /// The queued message.
    #[prop(into)]
    text: String,
    /// Called when "Send now" is pressed: interrupt the reply and send this message.
    #[prop(optional, into)]
    on_steer: Option<Callback<()>>,
    /// Called when remove is pressed.
    #[prop(optional, into)]
    on_remove: Option<Callback<()>>,
) -> impl IntoView {
    let title = text.clone();
    view! {
        <li data-scope=SCOPE_QUEUE_ITEM data-part="root">
            <span data-scope=SCOPE_QUEUE_ITEM data-part="text" title=title>{text}</span>
            <button
                type="button"
                data-scope=SCOPE_QUEUE_ITEM
                data-part="steer"
                aria-label="Send queued message now"
                on:click=move |_| {
                    if let Some(cb) = on_steer {
                        cb.run(());
                    }
                }
            >
                "Send now"
            </button>
            <button
                type="button"
                data-scope=SCOPE_QUEUE_ITEM
                data-part="remove"
                aria-label="Remove queued message"
                title="Remove queued message"
                on:click=move |_| {
                    if let Some(cb) = on_remove {
                        cb.run(());
                    }
                }
            >
                <Icon name=IconName::X size=14 />
            </button>
        </li>
    }
}
