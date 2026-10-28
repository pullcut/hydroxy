use hydroxy_leptos::card::{Card, CardTone, Jewel};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (opened, set_opened) = signal(0);

    view! {
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: var(--space-5);">
            // Default surface.
            <Card>
                <p style="margin: 0 0 4px; font: var(--text-label); letter-spacing: var(--tracking-label); text-transform: uppercase; color: var(--text-3);">"Project"</p>
                <p style="margin: 0; font: var(--text-h4);">"Q3 customer research"</p>
                <p style="margin: 8px 0 0; font: var(--text-sm); color: var(--text-2);">"12 files · 3 summaries"</p>
            </Card>

            // Interactive: lifts on hover, opens on click, Enter or Space.
            <Card on_click=move |()| set_opened.update(|n| *n += 1)>
                <p style="margin: 0; font: var(--text-h4);">"Onboarding interviews"</p>
                <p style="margin: 8px 0 0; font: var(--text-sm); color: var(--text-2);">
                    {move || format!("Opened {} times", opened.get())}
                </p>
            </Card>

            <Card tone=CardTone::Sunken>
                <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">"Drafts stay here until you share them."</p>
            </Card>

            <Card tone=CardTone::Inverse>
                <p style="margin: 0; font: var(--text-figure);">"22 of 24"</p>
                <p style="margin: 8px 0 0; font: var(--text-sm);">"Interviews read"</p>
            </Card>

            // Jewel cards: feature and highlight surfaces. Citrine takes ink text.
            <Card tone=CardTone::Jewel(Jewel::Amethyst)>
                <p style="margin: 0; font: var(--text-h4);">"Read"</p>
                <p style="margin: 8px 0 0; font: var(--text-sm);">"Every file, with the page for each figure."</p>
            </Card>
            <Card tone=CardTone::Jewel(Jewel::Citrine)>
                <p style="margin: 0; font: var(--text-h4);">"Flag"</p>
                <p style="margin: 8px 0 0; font: var(--text-sm);">"Two items need your judgement."</p>
            </Card>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "card",
    name: "Card",
    group: "Display",
    summary: "A surface for grouping content. Jewel tones are for feature and highlight cards; interactive cards lift on hover.",
    demo,
    leptos: include_str!("card.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/card.rs"),
    parts: &[&["Card", "root", "<div>, data-tone, data-interactive. With on_click: role=button, tabindex=0"]],
    keys: &[&["Enter / Space", "Runs on_click when the card is clickable"]],
    props: &[
        &["tone", "CardTone", "Default, Sunken, Inverse or Jewel(Jewel). Default Default"],
        &["padding", "u16", "Inner padding in px. Default 24"],
        &["interactive", "bool", "Lift 2px on hover. Implied by on_click"],
        &["on_click", "Callback<()>", "Makes the card a button"],
        &["children", "Children", "Card content"],
    ],
};
