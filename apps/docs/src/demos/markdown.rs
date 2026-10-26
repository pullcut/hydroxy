use leptos::prelude::*;
use oxide_leptos::markdown::Markdown;

const NOTES: &str = r"## Onboarding findings

Nine of twelve interviewees raised **onboarding**, and three called it _confusing_.
See the [full notes](https://example.com/notes) or ~~last quarter's~~ this quarter's summary.

- [x] Tag interview transcripts
- [ ] Draft the summary
- [ ] Share with design

| Theme       | Mentions | Share |
|:------------|---------:|------:|
| Onboarding  |        9 |   75% |
| Pricing     |        4 |   33% |

> Raw HTML shows as text: <script>alert(1)</script>

```rust
let flagged = answers.iter().filter(|a| a.flagged).count();
```
";

pub fn demo() -> AnyView {
    view! { <Markdown source=NOTES /> }.into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "markdown",
    name: "Markdown",
    group: "Display",
    summary: "Render a markdown string as styled elements: CommonMark plus GFM tables, task lists and strikethrough. Raw HTML shows as text and unsafe links are dropped, so untrusted input is safe.",
    demo,
    leptos: include_str!("markdown.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/markdown.rs"),
    parts: &[
        &["Markdown", "root", "div[data-scope=markdown] around the rendered blocks"],
        &["", "table", "Scroll wrapper around each table"],
        &["", "—", "Code fences render as CodeBlock. Headings get slug ids. Task items are li[data-task]"],
    ],
    keys: &[],
    props: &[
        &["source", "String", "Required. The markdown. Links keep only http, https, mailto and relative URLs"],
    ],
};
