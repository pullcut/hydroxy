//! The [`AssistantSuggestion`] card and [`AssistantSuggestions`] grid.

use dioxus::prelude::*;
use hydroxy_headless::assistant_thread::{SCOPE_SUGGESTION, Suggestion};

/// A suggestion card for an empty thread: title, description and a button that reports the
/// text to send.
#[component]
pub fn AssistantSuggestion(
    /// The suggestion.
    suggestion: Suggestion,
    /// Called with [`Suggestion::text`] when the card is pressed.
    on_pick: Option<EventHandler<String>>,
) -> Element {
    let text = suggestion.text();
    rsx! {
        div { "data-scope": SCOPE_SUGGESTION, "data-part": "root",
            button {
                r#type: "button",
                "data-scope": SCOPE_SUGGESTION,
                "data-part": "trigger",
                onclick: move |_| {
                    if let Some(handler) = on_pick {
                        handler.call(text.clone());
                    }
                },
                span { "data-scope": SCOPE_SUGGESTION, "data-part": "title", "{suggestion.title}" }
                span { "data-scope": SCOPE_SUGGESTION, "data-part": "description", "{suggestion.description}" }
            }
        }
    }
}

/// A grid of [`AssistantSuggestion`] cards.
#[component]
pub fn AssistantSuggestions(
    /// The suggestions, in display order.
    items: Vec<Suggestion>,
    /// Called with [`Suggestion::text`] when a card is pressed.
    on_pick: Option<EventHandler<String>>,
    /// Accessible name for the group.
    #[props(default = "Suggestions".to_string())]
    label: String,
) -> Element {
    rsx! {
        div { "data-scope": SCOPE_SUGGESTION, "data-part": "grid", role: "group", "aria-label": "{label}",
            for (i, s) in items.into_iter().enumerate() {
                AssistantSuggestion { key: "{i}", suggestion: s, on_pick }
            }
        }
    }
}
