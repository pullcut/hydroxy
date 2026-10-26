use dioxus::prelude::*;
use oxide_dioxus::carousel::{Carousel, Slide};

fn card(title: &'static str, body: &'static str, tint: &'static str) -> Element {
    rsx! {
        div { style: "display: grid; align-content: center; gap: var(--space-2); height: 180px; padding: var(--space-8); background: {tint};",
            strong { style: "font: var(--text-h3); color: var(--text-1);", {title} }
            span { style: "font: var(--text-sm); color: var(--text-2);", {body} }
        }
    }
}

#[component]
pub fn CarouselDemo() -> Element {
    let mut index = use_signal(|| 0_usize);

    rsx! {
        div { style: "display: grid; gap: var(--space-8);",
            div { style: "display: grid; gap: var(--space-3);",
                Carousel {
                    label: "Release highlights",
                    slides: vec![
                        Slide::new(card("Faster search", "Results appear as you type.", "var(--accent-soft)")),
                        Slide::new(card("Shared boards", "Invite a teammate to any board.", "var(--status-info-soft)")),
                        Slide::new(card("Dark mode", "Follows your system setting.", "var(--status-success-soft)")),
                        Slide::new(card("Keyboard first", "Every action has a shortcut.", "var(--status-warning-soft)")),
                    ],
                    index: Some(index()),
                    on_index_change: move |i| index.set(i),
                }
                p { style: "margin: 0; font: var(--text-label); color: var(--text-3);", "Slide {index() + 1} of 4" }
            }
            Carousel {
                label: "Customer stories",
                looping: true,
                autoplay_ms: 4000,
                slides: vec![
                    Slide::new(card("“It paid for itself in a week.”", "Maya, ops lead", "var(--surface-sunken)")),
                    Slide::new(card("“We stopped using spreadsheets.”", "Jon, finance", "var(--accent-soft)")),
                    Slide::new(card("“Setup took ten minutes.”", "Priya, founder", "var(--status-info-soft)")),
                ],
            }
        }
    }
}
