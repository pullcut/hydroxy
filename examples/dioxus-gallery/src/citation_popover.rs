use dioxus::prelude::*;
use hydroxy_dioxus::citation_popover::{CitedSpan, CitedText, Source};
use hydroxy_dioxus::source_chip::{SourceChip, SourceRow};

const ANSWER: &str = "Rust 2024 makes `unsafe extern` blocks required, so every foreign declaration says it trusts the other side. The `gen` keyword is reserved for generators. Both changes are opt-in per crate through the edition field in Cargo.toml.";

fn sources() -> Vec<Source> {
    vec![
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
    ]
}

#[component]
pub fn CitationPopoverDemo() -> Element {
    let sources = sources();
    let spans: Vec<CitedSpan> = [
        ("Rust 2024 makes `unsafe extern` blocks required, so every foreign declaration says it trusts the other side.", vec![0]),
        ("The `gen` keyword is reserved for generators.", vec![1]),
        ("Both changes are opt-in per crate through the edition field in Cargo.toml.", vec![0, 2]),
    ]
    .iter()
    .filter_map(|(s, src)| CitedSpan::find(ANSWER, s, src))
    .collect();
    #[allow(clippy::needless_collect, reason = "names must not borrow `sources`, which the popover takes")]
    let names: Vec<String> =
        sources.iter().map(|s| s.title.split(" - ").next().unwrap_or_default().to_string()).collect();

    rsx! {
        div { style: "display: grid; gap: var(--space-4); max-width: 560px;",
            p { style: "margin: 0; font: var(--text-body); color: var(--text-1);",
                CitedText { text: ANSWER, spans, sources }
            }
            SourceRow {
                for name in names {
                    SourceChip { "{name}" }
                }
            }
        }
    }
}
