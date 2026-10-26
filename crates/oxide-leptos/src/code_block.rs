//! `<CodeBlock>`: an ink code panel with a language label, a copy button and light
//! syntax colouring (Rust, TypeScript/JavaScript, JSON, SQL, Bash, YAML, TOML).

use leptos::prelude::*;
use oxide_headless::code::highlight;

use crate::icon::{Icon, IconName};

/// A block of code. Highlighting runs on the server too, so it arrives coloured.
#[component]
pub fn CodeBlock(
    /// The code.
    #[prop(into)]
    code: String,
    /// Language name for the label and colouring, such as "rust" or "sql".
    #[prop(optional, into)]
    lang: Option<String>,
) -> impl IntoView {
    let copied = RwSignal::new(false);
    let label = lang.unwrap_or_else(|| "text".into());
    let runs = highlight(&code, &label)
        .into_iter()
        .map(|(token, text)| match token {
            Some(t) => view! { <span data-hl=t.as_str()>{text.to_string()}</span> }.into_any(),
            None => text.to_string().into_any(),
        })
        .collect_view();
    let copy = move |_| {
        // Best-effort: without clipboard access the button just doesn't confirm.
        let _ = window().navigator().clipboard().write_text(&code);
        copied.set(true);
        set_timeout(move || copied.set(false), std::time::Duration::from_millis(1200));
    };
    view! {
        <figure data-scope="code-block">
            <div data-part="head">
                <span>{label}</span>
                <button type="button" data-part="copy" on:click=copy aria-live="polite">
                    {move || {
                        if copied.get() {
                            view! { <Icon name=IconName::Check size=13 /> "Copied" }.into_any()
                        } else {
                            view! { <Icon name=IconName::Copy size=13 /> "Copy" }.into_any()
                        }
                    }}
                </button>
            </div>
            <pre><code>{runs}</code></pre>
        </figure>
    }
}
