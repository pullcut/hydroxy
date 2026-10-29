//! `CodeBlock`: an ink code panel with a language label, a copy button and light
//! syntax colouring (Rust, TypeScript/JavaScript, JSON, SQL, Bash, YAML, TOML).

use dioxus::prelude::*;

use hydroxy_headless::code::{Token, highlight};

use crate::icon::{Icon, IconName};
use crate::js;

/// A block of code.
#[component]
pub fn CodeBlock(
    /// The code.
    code: String,
    /// Language name for the label and colouring, such as "rust" or "sql".
    lang: Option<String>,
) -> Element {
    let mut copied = use_signal(|| false);
    let label = lang.unwrap_or_else(|| "text".into());
    #[allow(clippy::needless_collect, reason = "runs borrow `code`, which the copy handler then takes")]
    let runs: Vec<(Option<&'static str>, String)> =
        highlight(&code, &label).into_iter().map(|(t, s)| (t.map(Token::as_str), s.to_string())).collect();
    let copy = move |_| {
        js::copy(&code);
        copied.set(true);
        spawn(async move {
            js::sleep(1200).await;
            copied.set(false);
        });
    };
    rsx! {
        figure { "data-scope": "code-block",
            div { "data-part": "head",
                span { "{label}" }
                button { r#type: "button", "data-part": "copy", "aria-live": "polite", onclick: copy,
                    if copied() {
                        Icon { name: IconName::Check, size: 13 }
                        "Copied"
                    } else {
                        Icon { name: IconName::Copy, size: 13 }
                        "Copy"
                    }
                }
            }
            pre {
                code {
                    for (token, text) in runs {
                        if let Some(t) = token {
                            span { "data-hl": t, "{text}" }
                        } else {
                            "{text}"
                        }
                    }
                }
            }
        }
    }
}
