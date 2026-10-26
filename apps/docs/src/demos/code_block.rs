use leptos::prelude::*;
use oxide_leptos::code_block::CodeBlock;

const RUST: &str = r#"use oxide_leptos::switch::Switch;

// Uncontrolled: the switch keeps its own state.
let view = view! { <Switch default_checked=true>"Email me"</Switch> };"#;

const SQL: &str = "select project, count(*) as flags
from answers
where flagged = true
group by project; -- 2 rows";

pub fn demo() -> AnyView {
    view! {
        <div style="display: grid; gap: var(--space-5);">
            <CodeBlock code=RUST lang="rust" />
            <CodeBlock code=SQL lang="sql" />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "code-block",
    name: "Code block",
    group: "Display",
    summary: "An ink code panel with a language label, a copy button and light syntax colouring. Colouring runs on the server, so code arrives coloured.",
    demo,
    leptos: include_str!("code_block.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/code_block.rs"),
    parts: &[
        &["CodeBlock", "—", "figure[data-scope=code-block]"],
        &["", "head", "Language label and copy button"],
        &["", "copy", "Copies the code; shows Copied for a moment"],
    ],
    keys: &[],
    props: &[
        &["code", "String", "Required. The code"],
        &["lang", "String", "rust, ts, js, json, sql, bash, yaml or toml. Others stay plain"],
    ],
};
