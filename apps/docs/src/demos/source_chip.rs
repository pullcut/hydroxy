use leptos::prelude::*;
use oxide_leptos::icon::IconName;
use oxide_leptos::source_chip::{SourceChip, SourceRow};

pub fn demo() -> AnyView {
    let (opened, set_opened) = signal(String::from("nothing yet"));
    view! {
        <div style="display: grid; gap: var(--space-5);">
            <p style="margin: 0; font: var(--text-body);">"Nine of twelve interviewees raised onboarding."</p>
            <SourceRow>
                <SourceChip on_click=move || set_opened.set("interviews-q3.pdf".into())>"interviews-q3.pdf"</SourceChip>
                <SourceChip on_click=move || set_opened.set("tickets-sept.csv".into())>"tickets-sept.csv · p. 4"</SourceChip>
                <SourceChip icon=IconName::Inbox>"support inbox"</SourceChip>
            </SourceRow>
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">"Opened: " {opened}</p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "source-chip",
    name: "Source chip",
    group: "Display",
    summary: "Mono chips that name the files behind an answer or summary. With on_click they become buttons that open the source.",
    demo,
    leptos: include_str!("source_chip.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/source_chip.rs"),
    parts: &[
        &["SourceChip", "—", "span, or a button with on_click"],
        &["SourceRow", "—", "Wrapping group, labelled \"Sources\""],
    ],
    keys: &[],
    props: &[
        &["children", "View", "The source's name"],
        &["icon", "IconName", "Leading icon. Default FileText"],
        &["on_click", "Callback<()>", "Opens the source"],
    ],
};
