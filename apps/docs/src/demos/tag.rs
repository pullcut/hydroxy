use hydroxy_leptos::tag::{Jewel, Tag};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (interviews, set_interviews) = signal(true);
    let (tickets, set_tickets) = signal(false);
    let (sources, set_sources) = signal(vec!["interviews-q3.pdf", "tickets-sept.csv", "nps-august.xlsx"]);

    view! {
        <div style="display: grid; gap: var(--space-5);">
            // Toggle filters: a clickable tag is a button with aria-pressed.
            <div style="display: flex; flex-wrap: wrap; gap: var(--space-3);">
                <Tag jewel=Jewel::Amethyst selected=interviews on_click=move |()| set_interviews.update(|on| *on = !*on)>
                    "Interviews"
                </Tag>
                <Tag jewel=Jewel::Sapphire selected=tickets on_click=move |()| set_tickets.update(|on| *on = !*on)>
                    "Support tickets"
                </Tag>
                <Tag jewel=Jewel::Citrine>"Q3 research"</Tag>
            </div>

            // Removable.
            <div style="display: flex; flex-wrap: wrap; gap: var(--space-3);">
                <For
                    each=move || sources.get()
                    key=|source| *source
                    children=move |source| {
                        view! {
                            <Tag
                                jewel=Jewel::Emerald
                                on_remove=move |()| set_sources.update(|list| list.retain(|s| *s != source))
                            >
                                {source}
                            </Tag>
                        }
                    }
                />
            </div>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "tag",
    name: "Tag",
    group: "Display",
    summary: "A pill with a jewel diamond, for filters, categories and chat suggestions. Clickable tags toggle; removable tags get a remove button.",
    demo,
    leptos: include_str!("tag.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/tag.rs"),
    parts: &[
        &["Tag", "(root)", "<span data-scope=tag data-jewel>, data-selected, data-clickable"],
        &["(inside)", "label", "<button aria-pressed> when on_click is set, else <span>. Covers the whole pill"],
        &["(inside)", "diamond", "8px rotated square in the jewel colour, aria-hidden"],
        &["(inside)", "remove", "<button aria-label=Remove> with an x icon, when on_remove is set"],
    ],
    keys: &[
        &["Enter / Space", "Toggles a clickable tag, or removes when the remove button has focus"],
        &["Tab", "Moves between the tag and its remove button"],
    ],
    props: &[
        &["jewel", "Jewel", "Diamond colour, and the border and tint when selected. Default Amethyst"],
        &["selected", "bool", "Selected look. Reported as aria-pressed when clickable"],
        &["on_click", "Callback<()>", "Makes the tag a toggle button"],
        &["on_remove", "Callback<()>", "Adds a remove button"],
        &["children", "View", "The label"],
    ],
};
