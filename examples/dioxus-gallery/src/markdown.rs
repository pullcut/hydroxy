use dioxus::prelude::*;
use hydroxy_dioxus::markdown::Markdown;

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

#[component]
pub fn MarkdownDemo() -> Element {
    rsx! {
        Markdown { source: NOTES }
    }
}
