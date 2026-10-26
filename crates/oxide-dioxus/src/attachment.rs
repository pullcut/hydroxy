//! `Attachment`: a file as a chip or a card. Shows a thumbnail when it has
//! one, or a badge with the file type, plus name, size and an optional remove button.

use dioxus::prelude::*;
use oxide_headless::attachment::{self as h, Kind};

use crate::icon::{Icon, IconName};

pub use oxide_headless::attachment::Variant as AttachmentVariant;

/// A file with its name, size and type. Give `on_remove` to show the remove button.
#[component]
pub fn Attachment(
    /// File name. Also decides the badge when `mime` is unset.
    name: String,
    /// Size in bytes.
    size: Option<u64>,
    /// MIME type, such as `application/pdf`.
    #[props(default)]
    mime: String,
    /// URL of a preview image. Shown instead of the badge.
    thumbnail: Option<String>,
    /// Chip (default) or card.
    #[props(default)]
    variant: AttachmentVariant,
    /// Called when the remove button is pressed. Without it there is no button.
    on_remove: Option<EventHandler<()>>,
) -> Element {
    let kind: Kind = h::kind_of(&name, &mime);
    let remove_label = h::remove_label(&name);
    let meta = h::meta_text(kind, size);

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-variant": variant.as_str(), "data-kind": kind.as_str(),
            if let Some(src) = thumbnail {
                img { "data-scope": h::SCOPE, "data-part": "thumbnail", src, alt: "", loading: "lazy" }
            } else {
                span { "data-scope": h::SCOPE, "data-part": "badge", "aria-hidden": "true", {h::badge_text(&name)} }
            }
            span { "data-scope": h::SCOPE, "data-part": "body",
                span { "data-scope": h::SCOPE, "data-part": "name", title: name.clone(), "{name}" }
                span { "data-scope": h::SCOPE, "data-part": "meta", "{meta}" }
            }
            if let Some(handler) = on_remove {
                button {
                    r#type: "button",
                    "aria-label": remove_label.clone(),
                    title: remove_label,
                    "data-scope": h::SCOPE,
                    "data-part": "remove",
                    onclick: move |_| handler.call(()),
                    Icon { name: IconName::X, size: 14 }
                }
            }
        }
    }
}
