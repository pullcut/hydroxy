use leptos::prelude::*;
use oxide_leptos::carousel::{Carousel, Slide};

fn card(title: &'static str, body: &'static str, tint: &'static str) -> impl IntoView {
    view! {
        <div style=format!("display: grid; align-content: center; gap: var(--space-2); height: 180px; padding: var(--space-8); background: {tint};")>
            <strong style="font: var(--text-h3); color: var(--text-1);">{title}</strong>
            <span style="font: var(--text-sm); color: var(--text-2);">{body}</span>
        </div>
    }
}

pub fn demo() -> AnyView {
    let (index, set_index) = signal(0_usize);

    view! {
        <div style="display: grid; gap: var(--space-8);">
            // Controlled, stops at the ends.
            <div style="display: grid; gap: var(--space-3);">
                <Carousel
                    label="Release highlights"
                    slides=vec![
                        Slide::new(|| card("Faster search", "Results appear as you type.", "var(--accent-soft)")),
                        Slide::new(|| card("Shared boards", "Invite a teammate to any board.", "var(--status-info-soft)")),
                        Slide::new(|| card("Dark mode", "Follows your system setting.", "var(--status-success-soft)")),
                        Slide::new(|| card("Keyboard first", "Every action has a shortcut.", "var(--status-warning-soft)")),
                    ]
                    index=index
                    on_index_change=move |i| set_index.set(i)
                />
                <p style="margin: 0; font: var(--text-label); color: var(--text-3);">{move || format!("Slide {} of 4", index.get() + 1)}</p>
            </div>

            // Looping, with autoplay that pauses on hover and focus.
            <Carousel
                label="Customer stories"
                looping=true
                autoplay_ms=4000
                slides=vec![
                    Slide::new(|| card("“It paid for itself in a week.”", "Maya, ops lead", "var(--surface-sunken)")),
                    Slide::new(|| card("“We stopped using spreadsheets.”", "Jon, finance", "var(--accent-soft)")),
                    Slide::new(|| card("“Setup took ten minutes.”", "Priya, founder", "var(--status-info-soft)")),
                ]
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "carousel",
    name: "Carousel",
    group: "Display",
    summary: "Slides in a scroll-snap strip. Swipe, scroll, use the arrow keys, or press the buttons and dots. Optional autoplay pauses on hover and focus, and never starts under reduced motion.",
    demo,
    leptos: include_str!("carousel.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/carousel.rs"),
    parts: &[
        &["Carousel", "root", "role=region, aria-roledescription=carousel"],
        &["", "viewport", "Scroll-snap strip, tabindex=0. aria-live is off while autoplay is running"],
        &["", "slide", "role=group, aria-roledescription=slide, labelled \"2 of 4\". Slides other than the current one are inert. data-state=current|idle"],
        &["", "controls", "Row holding the buttons and dots"],
        &["", "prev, next", "Buttons. Disabled at the ends unless looping"],
        &["", "dots", "One button per slide, aria-current on the current one"],
        &["", "play", "Pause and play button. Only rendered with autoplay"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Previous or next slide"],
        &["Home / End", "First or last slide"],
        &["Tab", "Moves through the viewport, the buttons and the dots"],
    ],
    props: &[
        &["slides", "Vec<Slide>", "Required. Slide::new(content)"],
        &["label", "String", "Accessible name. Default \"Carousel\""],
        &["index", "Option<usize>", "Controlled slide. Pair with on_index_change"],
        &["default_index", "usize", "Initial slide when uncontrolled"],
        &["looping", "bool", "Wrap past the first and last slide"],
        &["autoplay_ms", "Option<u32>", "Advance this often. Pauses on hover and focus. Autoplay wraps"],
        &["on_index_change", "Callback<usize>", "Called as the slide changes, including after a swipe"],
    ],
};
