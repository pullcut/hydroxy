//! `MediaGallery`: a grid of image and video thumbnails that opens a
//! [`Lightbox`]. With motion allowed, the media grows out of the thumbnail it
//! was opened from; on close, focus lands on the thumbnail of the item last shown.

use dioxus::prelude::*;
use hydroxy_headless::media_gallery::{self as h, MediaItem, MediaKind};

use crate::focus::focus_soon;
use crate::icon::{Icon, IconName};
use crate::js;
use crate::lightbox::Lightbox;

/// A thumbnail grid with a built-in lightbox.
#[component]
pub fn MediaGallery(
    /// The media, in order.
    items: ReadSignal<Vec<MediaItem>>,
    /// Wrap from the last item to the first in the lightbox.
    #[props(default)]
    looping: bool,
    /// The lightbox's accessible name. Default "Media viewer".
    #[props(default = "Media viewer".to_string())]
    label: String,
) -> Element {
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let mut open = use_signal(|| None::<usize>);
    let mut last = use_signal(|| 0usize);
    let close_root = root.clone();

    rsx! {
        ul { "data-scope": h::SCOPE, "data-part": "root", role: "list",
            for (i, item) in items().into_iter().enumerate() {
                li { key: "{i}", "data-scope": h::SCOPE, "data-part": "item",
                    button {
                        r#type: "button",
                        id: h::thumb_id(&root, i),
                        "data-scope": h::SCOPE,
                        "data-part": "thumb",
                        "data-kind": if item.kind == MediaKind::Video { "video" } else { "image" },
                        "aria-label": h::thumb_label(&item),
                        onclick: {
                            let thumb = h::thumb_id(&root, i);
                            move |_| {
                                last.set(i);
                                open.set(Some(i));
                                js::run(&[("thumb", &thumb)], h::ZOOM_FROM_JS);
                            }
                        },
                        img {
                            "data-scope": h::SCOPE,
                            "data-part": "image",
                            src: item.thumb_src().to_string(),
                            alt: "",
                            loading: "lazy",
                            decoding: "async",
                            draggable: "false",
                        }
                        if item.kind == MediaKind::Video {
                            span { "data-scope": h::SCOPE, "data-part": "badge", "aria-hidden": "true",
                                Icon { name: IconName::PlayFilled, size: 14 }
                            }
                        }
                    }
                }
            }
        }
        Lightbox {
            items,
            index: open(),
            looping,
            label,
            on_change: move |to: Option<usize>| {
                match to {
                    Some(i) => last.set(i),
                    // After the dialog's own focus return, land on the item last shown.
                    None => focus_soon(&h::thumb_id(&close_root, *last.peek())),
                }
                open.set(to);
            },
        }
    }
}
