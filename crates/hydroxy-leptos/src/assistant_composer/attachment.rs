//! The [`AssistantAttachment`] file chip.

use hydroxy_headless::assistant_composer::{self as h, AttachmentItem, AttachmentStatus};
use hydroxy_headless::assistant_thread::SCOPE_ATTACHMENT;
use hydroxy_headless::attachment as att;
use leptos::prelude::*;

use crate::icon::{Icon, IconName};

/// A file chip in the composer or a message: type badge, name, size or upload state, and a
/// remove button when `on_remove` is given.
#[component]
pub fn AssistantAttachment(
    /// The file and its upload state.
    item: AttachmentItem,
    /// Called with the attachment id when remove is pressed. Without it there is no button.
    #[prop(optional, into)]
    on_remove: Option<Callback<String>>,
) -> impl IntoView {
    let kind = att::kind_of(&item.name, &item.mime);
    let meta = h::attachment_meta(&item);
    let progress = match item.status {
        AttachmentStatus::Uploading(p) => Some(p.min(100)),
        _ => None,
    };
    let label = att::remove_label(&item.name);
    let AttachmentItem { id, name, status, .. } = item;
    let remove_id = id.clone();
    let title = name.clone();
    view! {
        <div
            data-scope=SCOPE_ATTACHMENT
            data-part="root"
            data-status=status.as_str()
            data-kind=kind.as_str()
            data-attachment-id=id
        >
            <span data-scope=SCOPE_ATTACHMENT data-part="thumb" aria-hidden="true">{att::badge_text(&name)}</span>
            <span data-scope=SCOPE_ATTACHMENT data-part="body">
                <span data-scope=SCOPE_ATTACHMENT data-part="name" title=title>{name}</span>
                <span data-scope=SCOPE_ATTACHMENT data-part="meta">{meta}</span>
                {progress
                    .map(|p| {
                        view! {
                            <span
                                data-scope=SCOPE_ATTACHMENT
                                data-part="progress"
                                role="progressbar"
                                aria-valuemin="0"
                                aria-valuemax="100"
                                aria-valuenow=p.to_string()
                            >
                                <span data-part="bar" style=format!("width: {p}%")></span>
                            </span>
                        }
                    })}
            </span>
            {on_remove
                .map(|cb| {
                    view! {
                        <button
                            type="button"
                            data-scope=SCOPE_ATTACHMENT
                            data-part="remove"
                            aria-label=label.clone()
                            title=label
                            on:click=move |_| cb.run(remove_id.clone())
                        >
                            <Icon name=IconName::X size=14 />
                        </button>
                    }
                })}
        </div>
    }
}
