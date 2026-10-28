//! `<MediaGallery>`: a grid of image and video thumbnails that opens a
//! [`Lightbox`]. With motion allowed, the media grows out of the thumbnail it
//! was opened from; on close, focus lands on the thumbnail of the item last shown.

use leptos::prelude::*;
use leptos::wasm_bindgen::JsValue;
use oxide_headless::media_gallery::{self as h, MediaItem, MediaKind};

use crate::focus::focus_id;
use crate::js;
use crate::lightbox::Lightbox;

/// A play mark for video thumbnails.
fn play_badge() -> impl IntoView {
    view! {
        <span data-scope=h::SCOPE data-part="badge" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M8 5.5v13l10.5-6.5z" /></svg>
        </span>
    }
}

/// A thumbnail grid with a built-in lightbox.
#[component]
pub fn MediaGallery(
    /// The media, in order.
    #[prop(into)]
    items: Signal<Vec<MediaItem>>,
    /// Wrap from the last item to the first in the lightbox.
    #[prop(optional)]
    looping: bool,
    /// The lightbox's accessible name. Default "Media viewer".
    #[prop(default = "Media viewer".into(), into)]
    label: String,
) -> impl IntoView {
    let root = StoredValue::new(crate::id::next_id(h::SCOPE));
    let open = RwSignal::new(None::<usize>);
    let last = StoredValue::new(0usize);

    let on_change = move |to: Option<usize>| {
        if let Some(i) = to {
            last.set_value(i);
        } else {
            // After the dialog's own focus return, land on the item last shown.
            let id = h::thumb_id(&root.get_value(), last.get_value());
            set_timeout(move || focus_id(&id), std::time::Duration::ZERO);
        }
        open.set(to);
    };

    let grid = move || {
        items
            .get()
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let id = h::thumb_id(&root.get_value(), i);
                let thumb = id.clone();
                view! {
                    <li data-scope=h::SCOPE data-part="item">
                        <button
                            type="button"
                            id=id
                            data-scope=h::SCOPE
                            data-part="thumb"
                            data-kind=if item.kind == MediaKind::Video { "video" } else { "image" }
                            aria-label=h::thumb_label(&item)
                            on:click=move |_| {
                                last.set_value(i);
                                open.set(Some(i));
                                js::call("thumb", h::ZOOM_FROM_JS, &[JsValue::from(thumb.as_str())]);
                            }
                        >
                            <img
                                data-scope=h::SCOPE
                                data-part="image"
                                src=item.thumb_src().to_string()
                                alt=""
                                loading="lazy"
                                decoding="async"
                                draggable="false"
                            />
                            {(item.kind == MediaKind::Video).then(play_badge)}
                        </button>
                    </li>
                }
            })
            .collect_view()
    };

    view! {
        <ul data-scope=h::SCOPE data-part="root" role="list">{grid}</ul>
        <Lightbox items=items index=open on_change=on_change looping=looping label=label />
    }
}
