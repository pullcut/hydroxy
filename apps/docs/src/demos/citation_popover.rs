use hydroxy_leptos::citation_popover::{CitedSpan, CitedText, Source};
use hydroxy_leptos::source_chip::{SourceChip, SourceRow};
use leptos::prelude::*;

const ANSWER: &str = "Rust 2024 makes `unsafe extern` blocks required, so every foreign declaration says it trusts the other side. The `gen` keyword is reserved for generators. Both changes are opt-in per crate through the edition field in Cargo.toml.";

pub fn demo() -> AnyView {
    let sources = vec![
        Source::new(
            "Unsafe extern blocks - The Rust Edition Guide",
            "https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html",
            "Rust 2024 requires extern blocks to be marked with unsafe. This makes it clear that it is the responsibility of the author to ensure the signatures are correct.",
        )
        .quote("requires extern blocks to be marked with unsafe"),
        Source::new(
            "gen keyword - The Rust Edition Guide",
            "https://doc.rust-lang.org/edition-guide/rust-2024/gen-keyword.html",
            "The gen keyword has been reserved as part of RFC 3513 to introduce gen blocks in a future release of Rust.",
        )
        .quote("The gen keyword has been reserved"),
        Source::new(
            "Editions - The Cargo Book",
            "https://doc.rust-lang.org/cargo/reference/manifest.html#the-edition-field",
            "The edition key is an optional key that affects which Rust Edition your package is compiled with. Setting it in Cargo.toml opts the whole package in.",
        )
        .quote("affects which Rust Edition your package is compiled with"),
    ];
    let spans: Vec<CitedSpan> = [
        ("Rust 2024 makes `unsafe extern` blocks required, so every foreign declaration says it trusts the other side.", vec![0]),
        ("The `gen` keyword is reserved for generators.", vec![1]),
        ("Both changes are opt-in per crate through the edition field in Cargo.toml.", vec![0, 2]),
    ]
    .iter()
    .filter_map(|(s, src)| CitedSpan::find(ANSWER, s, src))
    .collect();
    let chips = sources
        .iter()
        .map(|s| s.title.split(" - ").next().unwrap_or_default().to_string())
        .map(|title| view! { <SourceChip>{title}</SourceChip> })
        .collect_view();

    view! {
        <div style="display: grid; gap: var(--space-4); max-width: 560px;">
            <p style="margin: 0; font: var(--text-body); color: var(--text-1);">
                <CitedText text=ANSWER spans=spans sources=sources />
            </p>
            <SourceRow>{chips}</SourceRow>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "citation-popover",
    name: "Citation popover",
    group: "Chat",
    summary: "Numbered markers after cited text. Hover or focus one to see the source, the quoted passage highlighted, and a link out.",
    demo,
    leptos: include_str!("citation_popover.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/citation_popover.rs"),
    parts: &[
        &["CitedText", "text", "Prose split into runs from byte-offset spans"],
        &["CitedText", "cited", "A cited run, dotted underline"],
        &[
            "CitationMarker",
            "marker",
            "Link to the source, \"[n]\", named \"Source n: title\", described by the snippet",
        ],
        &["CitationMarker", "card", "Manual popover anchored to the marker; stays open while hovered or focused"],
        &["CitationMarker", "favicon", "The source's icon, or its host's first letter"],
        &["CitationMarker", "snippet", "The passage, with the quote in a <mark>"],
        &["CitationMarker", "open", "Link that opens the source in a new tab"],
    ],
    keys: &[
        &["Tab", "Focus a marker to open its card; Tab again reaches Open source"],
        &["Escape", "Close the card and return to the marker"],
        &["Enter", "On the marker: open the source"],
    ],
    props: &[
        &["text", "String", "CitedText. The prose"],
        &[
            "spans",
            "Vec<CitedSpan>",
            "CitedText. Byte ranges and the sources behind each; CitedSpan::find builds one from a substring",
        ],
        &["sources", "Vec<Source>", "Title, URL, snippet, optional quote and favicon. Numbered from 1"],
        &["index / source", "usize / Source", "CitationMarker on its own, for custom layouts"],
    ],
    events: &[],
};
