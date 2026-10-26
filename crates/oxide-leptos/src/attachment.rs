//! `<Attachment>`: a file as a chip or a card. Shows a thumbnail when it has
//! one, or a badge with the file type, plus name, size and an optional remove button.

use leptos::either::Either;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::attachment::{self as h, Kind};

use crate::icon::{Icon, IconName};

pub use oxide_headless::attachment::Variant as AttachmentVariant;

/// A file with its name, size and type. Give `on_remove` to show the remove button.
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn Attachment(
    /// File name. Also decides the badge when `mime` is unset.
    #[prop(into)]
    name: String,
    /// Size in bytes.
    #[prop(optional)]
    size: Option<u64>,
    /// MIME type, such as `application/pdf`.
    #[prop(optional, into)]
    mime: String,
    /// URL of a preview image. Shown instead of the badge.
    #[prop(optional, into)]
    thumbnail: Option<String>,
    /// Chip (default) or card.
    #[prop(optional)]
    variant: AttachmentVariant,
    /// Called when the remove button is pressed. Without it there is no button.
    #[prop(optional, into)]
    on_remove: Option<Callback<()>>,
) -> impl IntoView {
    let kind: Kind = h::kind_of(&name, &mime);
    let remove_label = h::remove_label(&name);
    let lead = match thumbnail {
        Some(src) => Either::Left(view! { <img data-scope=h::SCOPE data-part="thumbnail" src=src alt="" loading="lazy" /> }),
        None => Either::Right(view! { <span data-scope=h::SCOPE data-part="badge" aria-hidden="true">{h::badge_text(&name)}</span> }),
    };
    let meta = h::meta_text(kind, size);
    let title = name.clone();

    view! {
        <div data-scope=h::SCOPE data-part="root" data-variant=variant.as_str() data-kind=kind.as_str()>
            {lead}
            <span data-scope=h::SCOPE data-part="body">
                <span data-scope=h::SCOPE data-part="name" title=title>{name}</span>
                <span data-scope=h::SCOPE data-part="meta">{meta}</span>
            </span>
            {on_remove.map(|cb| view! {
                <button
                    type="button"
                    aria-label=remove_label.clone()
                    title=remove_label
                    data-scope=h::SCOPE
                    data-part="remove"
                    on:click=move |ev| {
                        // The chip usually leaves with this press; hand focus to a neighbour's button.
                        let root = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()).and_then(|e| e.closest("[data-part=root]").ok().flatten());
                        let near = root.and_then(|r| r.next_element_sibling().or_else(|| r.previous_element_sibling()));
                        if let Some(b) = near.and_then(|n| n.query_selector("[data-part=remove]").ok().flatten()) {
                            let _ = b.unchecked_into::<web_sys::HtmlElement>().focus();
                        }
                        cb.run(());
                    }
                >
                    <Icon name=IconName::X size=14 />
                </button>
            })}
        </div>
    }
}
