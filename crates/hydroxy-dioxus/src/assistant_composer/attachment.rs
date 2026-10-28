//! The [`AssistantAttachment`] file chip.

use dioxus::prelude::*;
use hydroxy_headless::assistant_composer::{self as h, AttachmentItem, AttachmentStatus};
use hydroxy_headless::assistant_thread::SCOPE_ATTACHMENT;
use hydroxy_headless::attachment as att;

use crate::icon::{Icon, IconName};

/// A file chip in the composer or a message: type badge, name, size or upload state, and a
/// remove button when `on_remove` is given.
#[component]
pub fn AssistantAttachment(
    /// The file and its upload state.
    item: AttachmentItem,
    /// Called with the attachment id when remove is pressed. Without it there is no button.
    on_remove: Option<EventHandler<String>>,
) -> Element {
    let kind = att::kind_of(&item.name, &item.mime);
    let meta = h::attachment_meta(&item);
    let progress = match item.status {
        AttachmentStatus::Uploading(p) => Some(p.min(100)),
        _ => None,
    };
    let label = att::remove_label(&item.name);
    let badge = att::badge_text(&item.name);
    let status = item.status.as_str();
    let id = item.id.clone();
    let name = item.name.clone();
    rsx! {
        div {
            "data-scope": SCOPE_ATTACHMENT,
            "data-part": "root",
            "data-status": status,
            "data-kind": kind.as_str(),
            "data-attachment-id": "{item.id}",
            span { "data-scope": SCOPE_ATTACHMENT, "data-part": "thumb", "aria-hidden": "true", "{badge}" }
            span { "data-scope": SCOPE_ATTACHMENT, "data-part": "body",
                span { "data-scope": SCOPE_ATTACHMENT, "data-part": "name", title: "{name}", "{name}" }
                span { "data-scope": SCOPE_ATTACHMENT, "data-part": "meta", "{meta}" }
                if let Some(p) = progress {
                    span {
                        "data-scope": SCOPE_ATTACHMENT,
                        "data-part": "progress",
                        role: "progressbar",
                        "aria-valuemin": "0",
                        "aria-valuemax": "100",
                        "aria-valuenow": "{p}",
                        span { "data-part": "bar", style: "width: {p}%" }
                    }
                }
            }
            if let Some(handler) = on_remove {
                button {
                    r#type: "button",
                    "data-scope": SCOPE_ATTACHMENT,
                    "data-part": "remove",
                    "aria-label": "{label}",
                    title: "{label}",
                    onclick: move |_| handler.call(id.clone()),
                    Icon { name: IconName::X, size: 14 }
                }
            }
        }
    }
}
