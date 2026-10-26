//! `CodeBlock`: an ink code panel with a language label, a copy button and light
//! syntax colouring (Rust, TypeScript/JavaScript, JSON, SQL, Bash, YAML, TOML).

use dioxus::prelude::*;
use std::fmt::Write as _;

use oxide_headless::code::{Token, highlight};

use crate::icon::{Icon, IconName};

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
    let runs: Vec<(Option<&'static str>, String)> =
        highlight(&code, &label).into_iter().map(|(t, s)| (t.map(Token::as_str), s.to_string())).collect();
    let copy = move |_| {
        // Clipboard and the "Copied" reset both run through eval, so this works in the
        // web and desktop renderers alike.
        let text = js_string(&code);
        let mut wait = document::eval(&format!(
            "try {{ await navigator.clipboard.writeText({text}); }} catch (_) {{}} await new Promise(r => setTimeout(r, 1200)); dioxus.send(true);"
        ));
        copied.set(true);
        spawn(async move {
            let _ = wait.recv::<bool>().await;
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

/// `s` as a JavaScript string literal.
fn js_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if c < ' ' => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
