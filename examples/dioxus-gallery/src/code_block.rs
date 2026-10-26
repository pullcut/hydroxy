use dioxus::prelude::*;
use oxide_dioxus::code_block::CodeBlock;

const RUST: &str = r#"use oxide_dioxus::switch::Switch;

// Uncontrolled: the switch keeps its own state.
rsx! { Switch { default_checked: true, "Email me" } }"#;

const SQL: &str = "select project, count(*) as flags
from answers
where flagged = true
group by project; -- 2 rows";

#[component]
pub fn CodeBlockDemo() -> Element {
    rsx! {
        div { style: "display: grid; gap: var(--space-5);",
            CodeBlock { code: RUST, lang: "rust" }
            CodeBlock { code: SQL, lang: "sql" }
        }
    }
}
