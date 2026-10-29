//! The [`AssistantSuggestion`] card and the [`AssistantSuggestions`] grid.

use hydroxy_headless::assistant_thread::SCOPE_SUGGESTION;
use leptos::prelude::*;

use super::Suggestion;

/// A suggestion card for an empty thread: title, description and a button that reports the
/// text to send.
#[component]
pub fn AssistantSuggestion(
    /// The suggestion.
    suggestion: Suggestion,
    /// Called with [`Suggestion::text`] when the card is pressed.
    #[prop(optional, into)]
    on_pick: Option<Callback<String>>,
) -> impl IntoView {
    let text = suggestion.text();
    view! {
        <div data-scope=SCOPE_SUGGESTION data-part="root">
            <button
                type="button"
                data-scope=SCOPE_SUGGESTION
                data-part="trigger"
                on:click=move |_| {
                    if let Some(cb) = on_pick {
                        cb.run(text.clone());
                    }
                }
            >
                <span data-scope=SCOPE_SUGGESTION data-part="title">{suggestion.title}</span>
                <span data-scope=SCOPE_SUGGESTION data-part="description">{suggestion.description}</span>
            </button>
        </div>
    }
}

/// A grid of [`AssistantSuggestion`] cards.
#[component]
pub fn AssistantSuggestions(
    /// The suggestions, in display order.
    #[prop(into)]
    items: Vec<Suggestion>,
    /// Called with [`Suggestion::text`] when a card is pressed.
    #[prop(optional, into)]
    on_pick: Option<Callback<String>>,
    /// Accessible name for the group.
    #[prop(default = "Suggestions".into(), into)]
    label: String,
) -> impl IntoView {
    view! {
        <div data-scope=SCOPE_SUGGESTION data-part="grid" role="group" aria-label=label>
            {items
                .into_iter()
                .map(|s| {
                    let pick = Callback::new(move |t: String| {
                        if let Some(cb) = on_pick {
                            cb.run(t);
                        }
                    });
                    view! { <AssistantSuggestion suggestion=s on_pick=pick /> }
                })
                .collect_view()}
        </div>
    }
}
