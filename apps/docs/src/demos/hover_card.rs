use hydroxy_leptos::hover_card::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    view! {
        // A <div>, not a <p>: the card is a block element.
        <div>
            "Summary drafted by "
            <HoverCard>
                <HoverCardTrigger href="#">"@maya"</HoverCardTrigger>
                <HoverCardContent>
                    <div style="display: grid; gap: var(--space-2);">
                        <strong>"Maya Okafor"</strong>
                        <span style="color: var(--text-2);">"Research lead. Owns the Q3 customer interviews and 4 summaries."</span>
                    </div>
                </HoverCardContent>
            </HoverCard>
            " from 18 interviews."
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "hover-card",
    name: "Hover card",
    group: "Overlays",
    summary: "A preview card for a link. It opens after the pointer rests on the link, or at once on focus, and stays open while the pointer moves onto the card.",
    demo,
    leptos: include_str!("hover_card.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/hover_card.rs"),
    parts: &[
        &["HoverCard", "-", "Provides the machine. Renders no element"],
        &["HoverCardTrigger", "trigger", "a href, aria-expanded, aria-controls, CSS anchor"],
        &["HoverCardContent", "content", "popover=manual, anchored with flip, data-side, data-state"],
    ],
    keys: &[&["Tab", "Focusing the link opens the card; moving focus away closes it"], &["Escape", "Closes the card"]],
    props: &[
        &["open", "Option<bool>", "Controlled open state. Pair with on_open_change"],
        &["default_open", "bool", "Initial open state when uncontrolled"],
        &["open_delay_ms", "u32", "Hover time before opening. Default 300"],
        &["close_delay_ms", "u32", "Grace time after the pointer leaves. Default 150"],
        &["placement", "Placement", "Side and alignment. Default below, centred"],
        &["on_open_change", "Callback<bool>", "Called when the card opens or closes"],
        &["href", "String", "On HoverCardTrigger: where the link goes"],
    ],
};
