use dioxus::prelude::*;
use oxide_dioxus::composer::Composer;

#[component]
pub fn ComposerDemo() -> Element {
    let mut sent = use_signal(Vec::<String>::new);
    rsx! {
        div { style: "display: grid; gap: var(--space-5); max-width: 560px;",
            ul { style: "list-style: none; margin: 0; padding: 0; display: grid; gap: var(--space-3); font: var(--text-sm); color: var(--text-2);",
                if sent.read().is_empty() {
                    li { "Nothing sent yet." }
                }
                for m in sent.read().iter() {
                    li { "Sent: {m}" }
                }
            }
            Composer {
                suggestions: vec!["Summarise the interviews".to_string(), "What changed since August?".to_string()],
                on_send: move |m| sent.write().push(m),
            }
        }
    }
}
