use hydroxy_leptos::icon::IconName;
use hydroxy_leptos::space_icon::{Jewel, SpaceIcon};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let spaces = [
        (IconName::FileText, Jewel::Amethyst, "Research"),
        (IconName::Inbox, Jewel::Sapphire, "Support"),
        (IconName::Lightbulb, Jewel::Citrine, "Product ideas"),
        (IconName::User, Jewel::Emerald, "People"),
    ];
    view! {
        <div style="display: grid; gap: var(--space-7);">
            <ul style="list-style: none; margin: 0; padding: 0; display: grid; gap: var(--space-4);">
                {spaces
                    .into_iter()
                    .map(|(icon, jewel, name)| {
                        view! {
                            <li style="display: flex; align-items: center; gap: 10px; font: var(--text-ui);">
                                <SpaceIcon icon jewel />
                                {name}
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
            <div style="display: flex; align-items: center; gap: var(--space-4);">
                <SpaceIcon icon=IconName::FileText size=20 />
                <SpaceIcon icon=IconName::FileText size=32 />
                <SpaceIcon icon=IconName::FileText size=48 title="Research team" />
            </div>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "space-icon",
    name: "Space icon",
    group: "Display",
    summary: "A rounded jewel tile with a line icon that identifies a team, workspace or space. Citrine tiles take ink icons; the rest take white.",
    demo,
    leptos: include_str!("space_icon.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/space_icon.rs"),
    parts: &[&["SpaceIcon", "—", "span[data-scope=space-icon][data-jewel], decorative unless titled"]],
    keys: &[],
    props: &[
        &["icon", "IconName", "Required. The icon"],
        &["jewel", "Jewel", "Tile colour. Default Amethyst"],
        &["size", "u16", "Side in px. Default 24; corners and icon scale with it"],
        &["title", "String", "Tooltip and accessible name"],
    ],
};
