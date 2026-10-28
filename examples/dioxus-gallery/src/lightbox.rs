use dioxus::prelude::*;
use oxide_dioxus::button::{Button, ButtonVariant};
use oxide_dioxus::lightbox::{Lightbox, LightboxItem};

#[component]
pub fn LightboxDemo() -> Element {
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
    let mut open = use_signal(|| None::<usize>);

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: var(--space-3);",
            Button { on_click: move |_| open.set(Some(0)), "View listing photos" }
            Button { variant: ButtonVariant::Secondary, on_click: move |_| open.set(Some(2)), "See the terrace" }
        }
        // Controlled: the owner holds which photo is open.
        Lightbox { items, index: open(), on_change: move |i| open.set(i), looping: true, label: "Listing photos" }
    }
}
