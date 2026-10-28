use hydroxy_leptos::button::{Button, ButtonVariant};
use hydroxy_leptos::lightbox::{Lightbox, LightboxItem};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let items = vec![
        LightboxItem::image("https://picsum.photos/id/1048/1800/1200", "Open-plan studio with a long oak desk")
            .thumb("https://picsum.photos/id/1048/300/300")
            .caption("Studio, ground floor. 64 m², south light."),
        LightboxItem::image("https://picsum.photos/id/1031/1800/1200", "Glass office tower from street level")
            .thumb("https://picsum.photos/id/1031/300/300")
            .caption("The building from Market Street."),
        LightboxItem::image("https://picsum.photos/id/1076/1800/1200", "Rooftop terrace at golden hour")
            .thumb("https://picsum.photos/id/1076/300/300")
            .caption("Shared roof terrace, open 7am to 10pm.")
            .download("https://picsum.photos/id/1076/3600/2400"),
    ];
    let (open, set_open) = signal(None::<usize>);

    view! {
        <div style="display: flex; flex-wrap: wrap; gap: var(--space-3);">
            <Button on_click=move |_| set_open.set(Some(0))>"View listing photos"</Button>
            <Button variant=ButtonVariant::Secondary on_click=move |_| set_open.set(Some(2))>"See the terrace"</Button>
        </div>
        // Controlled: the owner holds which photo is open.
        <Lightbox items=items index=open on_change=move |i| set_open.set(i) looping=true label="Listing photos" />
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "lightbox",
    name: "Lightbox",
    group: "Overlays",
    summary: "A modal viewer for photos and videos. Zoom with the wheel, a pinch or a double-click and drag to pan; swipe or use the arrows to move on.",
    demo,
    leptos: include_str!("lightbox.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/lightbox.rs"),
    parts: &[
        &["Lightbox", "content", "Native modal <dialog>: traps focus, Escape closes, focus returns on close"],
        &["Lightbox", "header / counter", "\"3 of 12\", announced politely as it changes"],
        &["Lightbox", "zoom / download / close", "Icon buttons. Zoom has aria-pressed; download is a link"],
        &["Lightbox", "stage", "The zoom and swipe surface. data-zoomed, data-gesture"],
        &["Lightbox", "media", "The <img> (zoomable) or <video controls>"],
        &["Lightbox", "prev / next", "Icon buttons; disabled at the ends unless looping"],
        &["Lightbox", "caption", "Describes the dialog"],
        &["Lightbox", "thumbs / thumb", "The strip. The current thumb has aria-current"],
        &["Lightbox", "preload", "Hidden neighbour images, so the next one is ready"],
    ],
    keys: &[
        &["← / →", "Previous / next item"],
        &["Home / End", "First / last item"],
        &["+ / -", "Zoom in / out"],
        &["0", "Back to fitted"],
        &["Escape", "Close"],
        &["Double-click, wheel, pinch", "Zoom at the pointer; drag to pan while zoomed"],
        &["Swipe", "Next or previous item when not zoomed"],
    ],
    props: &[
        &["items", "Vec<LightboxItem>", "Required. LightboxItem::image(src, alt) or ::video(src, alt)"],
        &["index", "Option<usize>", "Required. Which item shows; None closes"],
        &["on_change", "Callback<Option<usize>>", "Required. Asks for another item, or None to close"],
        &["looping", "bool", "Wrap from the last item to the first"],
        &["label", "String", "The dialog's accessible name. Default \"Media viewer\""],
    ],
};
