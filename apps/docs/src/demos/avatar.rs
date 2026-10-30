use hydroxy_leptos::avatar::{Avatar, Jewel};
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    view! {
        <div style="display: grid; gap: var(--space-5);">
            // People, an agent and an unassigned slot at the default 22px.
            <div style="display: flex; align-items: center; gap: var(--space-3);">
                <Avatar initials="JM" jewel=Jewel::Sapphire title="Jo Marsh" />
                <Avatar initials="AK" jewel=Jewel::Emerald title="Ana Kovac" />
                <Avatar initials="RT" jewel=Jewel::Garnet title="Rui Tavares" />
                <Avatar initials="hy" agent=true title="hydroxy assistant" />
                <Avatar />
            </div>

            // Larger sizes.
            <div style="display: flex; align-items: center; gap: var(--space-3);">
                <Avatar initials="JM" jewel=Jewel::Sapphire size=32 title="Jo Marsh" />
                <Avatar initials="hy" agent=true size=32 title="hydroxy assistant" />
                <Avatar initials="AK" jewel=Jewel::Emerald size=40 title="Ana Kovac" />
                <Avatar size=40 />
            </div>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "avatar",
    name: "Avatar",
    group: "Display",
    summary: "A person or agent. People are jewel-tinted circles, agents are hexagons, and no initials shows an unassigned placeholder.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("avatar.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/avatar.rs"),
    parts: &[&[
        "Avatar",
        "(root)",
        "<span role=img aria-label> with data-kind=person|agent|none and data-jewel. Size set inline",
    ]],
    keys: &[],
    props: &[
        &["initials", "Option<String>", "One or two letters. None shows the unassigned placeholder"],
        &["jewel", "Jewel", "Tint for people. Default Amethyst"],
        &["agent", "bool", "Draw as an ink hexagon. Default false"],
        &["size", "u16", "Width and height in px. Default 22"],
        &["title", "Option<String>", "Accessible name. Falls back to the initials, then Unassigned"],
    ],
    events: &[],
};
