use leptos::prelude::*;
use hydroxy_leptos::media_gallery::MediaGallery;
use hydroxy_leptos::lightbox::LightboxItem;

fn photo(id: u32, alt: &str, caption: &str) -> LightboxItem {
    LightboxItem::image(format!("https://picsum.photos/id/{id}/1600/1067"), alt)
        .thumb(format!("https://picsum.photos/id/{id}/400/400"))
        .caption(caption)
        .download(format!("https://picsum.photos/id/{id}/3200/2134"))
}

pub fn demo() -> AnyView {
    let items = vec![
        photo(1015, "River winding through a green valley", "Day two: the valley trail above the river."),
        photo(1018, "Mountain ridge under low cloud", "The ridge just after sunrise."),
        photo(1036, "Snowy forest at dusk", "Last light over the pines."),
        photo(1043, "Wooden pier stretching into a lake", "Breakfast spot on the north shore."),
        LightboxItem::video("https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.mp4", "A bee landing on a flower")
            .poster("https://picsum.photos/id/152/1600/1067")
            .thumb("https://picsum.photos/id/152/400/400")
            .caption("Slow motion from the meadow, 12 seconds."),
        photo(1050, "Waves breaking against coastal rocks", "Wind picked up in the afternoon."),
        photo(1056, "Footpath through tall grass", "The way back down."),
        photo(1067, "City skyline seen from the hills", "Home, from the last viewpoint."),
    ];

    view! { <MediaGallery items=items label="Trip photos" /> }.into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "media-gallery",
    name: "Media gallery",
    group: "Display",
    summary: "A grid of photo and video thumbnails that opens each one in a lightbox. The picture grows out of its thumbnail when motion is allowed.",
    demo,
    leptos: include_str!("media_gallery.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/media_gallery.rs"),
    parts: &[
        &["MediaGallery", "root", "<ul> grid of square thumbnails. Off-screen rows skip rendering"],
        &["MediaGallery", "item", "<li>"],
        &["MediaGallery", "thumb", "<button> named \"Open image: …\" or \"Play video: …\". data-kind"],
        &["MediaGallery", "image", "Lazy-loaded thumbnail <img>"],
        &["MediaGallery", "badge", "Play mark on videos"],
        &["Lightbox", "content …", "The viewer it opens; see Lightbox"],
    ],
    keys: &[
        &["Tab", "Move between thumbnails"],
        &["Enter / Space", "Open the lightbox on this item"],
        &["Escape", "In the lightbox: close and return to the thumbnail last shown"],
    ],
    props: &[
        &["items", "Vec<LightboxItem>", "Required. LightboxItem::image(src, alt) or ::video(src, alt)"],
        &[".thumb / .caption / .poster / .download", "String", "Smaller source, caption, video still, download link"],
        &["looping", "bool", "Wrap from the last item to the first"],
        &["label", "String", "The lightbox's accessible name. Default \"Media viewer\""],
    ],
};
