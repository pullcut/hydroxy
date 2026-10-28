//! `<Composer>`: a chat input with a send button and optional suggestions.

use leptos::prelude::*;

use crate::button::ButtonVariant;
use crate::icon::IconName;
use crate::icon_button::IconButton;
use crate::tag::Tag;

/// A message box. Enter sends, Shift+Enter adds a line; blank messages are ignored.
#[component]
pub fn Composer(
    /// Controlled text. Pair with `on_input`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Called with the text as it changes.
    #[prop(optional, into)]
    on_input: Option<Callback<String>>,
    /// Called with the trimmed message on send. The field then clears.
    #[prop(optional, into)]
    on_send: Option<Callback<String>>,
    /// Placeholder text.
    #[prop(default = "Ask anything".into(), into)]
    placeholder: String,
    /// Accessible name for the text field.
    #[prop(default = "Message".into(), into)]
    label: String,
    /// Suggestion tags shown above the field.
    #[prop(optional, into)]
    suggestions: Vec<String>,
    /// Called when a suggestion is picked. Without it the suggestion fills the field.
    #[prop(optional, into)]
    on_suggestion: Option<Callback<String>>,
) -> impl IntoView {
    let draft = RwSignal::new(value.get_untracked().unwrap_or_default());
    let text = move || value.get().unwrap_or_else(|| draft.get());
    let set = move |t: String| {
        draft.set(t.clone());
        if let Some(cb) = on_input {
            cb.run(t);
        }
    };
    let send = move || {
        let message = text().trim().to_string();
        if message.is_empty() {
            return;
        }
        if let Some(cb) = on_send {
            cb.run(message);
        }
        set(String::new());
    };

    view! {
        <div data-scope="composer">
            {(!suggestions.is_empty())
                .then(|| {
                    view! {
                        <div data-part="suggestions">
                            {suggestions
                                .into_iter()
                                .map(|s| {
                                    let pick = s.clone();
                                    view! {
                                        <Tag on_click=move || match on_suggestion {
                                            Some(cb) => cb.run(pick.clone()),
                                            None => set(pick.clone()),
                                        }>{s}</Tag>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                })}
            <div data-part="field">
                <textarea
                    rows="1"
                    aria-label=label
                    placeholder=placeholder
                    prop:value=text
                    on:input=move |ev| set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" && !ev.shift_key() && !ev.is_composing() {
                            ev.prevent_default();
                            send();
                        }
                    }
                ></textarea>
                <IconButton icon=IconName::ArrowUp label="Send" variant=ButtonVariant::Accent on_click=move |_| send() />
            </div>
        </div>
    }
}
