use dioxus::prelude::*;
use hydroxy_dioxus::lightbox::LightboxItem;
use hydroxy_dioxus::media_gallery::MediaGallery;

fn photo(id: u32, alt: &str, caption: &str) -> LightboxItem {
    LightboxItem::image(format!("https://picsum.photos/id/{id}/1600/1067"), alt)
        .thumb(format!("https://picsum.photos/id/{id}/400/400"))
        .caption(caption)
        .download(format!("https://picsum.photos/id/{id}/3200/2134"))
}

#[component]
pub fn MediaGalleryDemo() -> Element {
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

    rsx! { MediaGallery { items, label: "Trip photos" } }
}
