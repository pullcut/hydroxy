//! `<CodeBlock>`: an ink code panel with a language label, a copy button and light
//! syntax colouring (Rust, TypeScript/JavaScript, JSON, SQL, Bash, YAML, TOML).

use leptos::either::Either;
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
            Some(t) => Either::Left(view! { <span data-hl=t.as_str()>{text.to_string()}</span> }),
            None => Either::Right(text.to_string()),
        })
        .collect_view();
    let copy = move |_| {
        // Best-effort: without clipboard access the button just doesn't confirm.
        let _ = window().navigator().clipboard().write_text(&code);
        copied.set(true);
        set_timeout(move || {
            copied.try_set(false);
        }, std::time::Duration::from_millis(1200));
    };
    view! {
        <figure data-scope="code-block">
            <div data-part="head">
                <span>{label}</span>
                <button type="button" data-part="copy" on:click=copy aria-live="polite">
                    {move || {
                        let (icon, text) = if copied.get() { (IconName::Check, "Copied") } else { (IconName::Copy, "Copy") };
                        view! { <Icon name=icon size=13 /> {text} }
                    }}
                </button>
            </div>
            <pre><code>{runs}</code></pre>
        </figure>
    }
}
