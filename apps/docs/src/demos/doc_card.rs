use leptos::prelude::*;
use oxide_leptos::avatar::Avatar;
use oxide_leptos::doc_card::{DocCard, Jewel};
use oxide_leptos::icon::IconName;

pub fn demo() -> AnyView {
    let (pinned, set_pinned) = signal(true);
    let (opened, set_opened) = signal(String::from("nothing yet"));
    view! {
        <div style="display: grid; gap: var(--space-5);">
            <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: var(--space-5);">
                <DocCard
                    title="Q3 customer research"
                    type_label="Summary"
                    snippet="Nine of twelve interviewees raised onboarding. Two items need your judgement before this is shared."
                    pinned=pinned
                    on_toggle_pin=move || set_pinned.update(|p| *p = !*p)
                    on_click=move || set_opened.set("Q3 customer research".into())
                >
                    <Avatar initials="JM" title="Jo Marsh" />
                    <span style="font: var(--text-xs); color: var(--text-3);">"Edited 26 October"</span>
                </DocCard>
                <DocCard
                    title="Support tickets, September"
                    type_label="Spreadsheet"
                    icon=IconName::Inbox
                    jewel=Jewel::Sapphire
                    snippet="410 fewer tickets than the August summary. The export may be missing a week."
                    on_click=move || set_opened.set("Support tickets, September".into())
                />
            </div>
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">"Opened: " {opened}</p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "doc-card",
    name: "Doc card",
    group: "Display",
    summary: "A document in a grid: type tile, title, a three-line snippet, an optional pin and a footer. The whole card opens the document.",
    demo,
    leptos: include_str!("doc_card.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/doc_card.rs"),
    parts: &[
        &["DocCard", "—", "article; role=button when on_click is set"],
        &["", "icon", "Type tile tinted with the jewel"],
        &["", "pin", "Button with aria-pressed; doesn't open the card"],
        &["", "title", "h3, balanced across lines"],
        &["", "snippet", "Clamped to three lines"],
        &["", "footer", "Your content, above a hairline"],
    ],
    keys: &[&["Enter / Space", "Opens the document when the card has focus"]],
    props: &[
        &["title", "String", "Required"],
        &["type_label", "String", "What kind of document, such as Summary"],
        &["icon", "IconName", "Type tile icon. Default FileText"],
        &["jewel", "Jewel", "Type tile colour. Default Amethyst"],
        &["snippet", "String", "A few lines of the document"],
        &["pinned", "bool", "Pinned state"],
        &["on_toggle_pin", "Callback<()>", "Shows the pin button"],
        &["on_click", "Callback<()>", "Opens the document"],
        &["children", "View", "Footer content (footer prop in Dioxus)"],
    ],
};
