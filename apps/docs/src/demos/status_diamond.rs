use leptos::prelude::*;
use oxide_leptos::status_diamond::{Jewel, StatusDiamond};

pub fn demo() -> AnyView {
    // The four lifecycle stages always use these jewels, in this order.
    let stages = [
        (Some(Jewel::Garnet), "Waiting"),
        (Some(Jewel::Citrine), "In review"),
        (Some(Jewel::Emerald), "Ready"),
        (Some(Jewel::Sapphire), "Shared"),
        (None, "Not started"),
    ];
    view! {
        <ul style="list-style: none; margin: 0; padding: 0; display: grid; gap: var(--space-4);">
            {stages
                .into_iter()
                .map(|(jewel, name)| {
                    view! {
                        <li style="display: flex; align-items: center; gap: 10px; font: var(--text-sm);">
                            <StatusDiamond jewel=jewel />
                            {name}
                        </li>
                    }
                })
                .collect_view()}
            <li style="display: flex; align-items: center; gap: 10px; font: var(--text-sm);">
                <StatusDiamond jewel=Jewel::Amethyst hollow=true size=11 title="Draft" />
                "Hollow, with an accessible name"
            </li>
        </ul>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "status-diamond",
    name: "Status diamond",
    group: "Display",
    summary: "The rotated-square marker for workflow stages, label colours and legends. Pair it with text; on its own it needs a title.",
    demo,
    leptos: include_str!("status_diamond.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/status_diamond.rs"),
    parts: &[&["StatusDiamond", "—", "span[data-scope=status-diamond], decorative unless titled"]],
    keys: &[],
    props: &[
        &["jewel", "Option<Jewel>", "Colour. None draws a hollow grey diamond"],
        &["size", "u16", "Side in px before rotation. Default 9"],
        &["hollow", "bool", "Outline only, in the jewel colour"],
        &["title", "String", "Tooltip and accessible name"],
    ],
};
