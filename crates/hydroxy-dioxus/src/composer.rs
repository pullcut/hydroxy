//! `Composer`: a chat input with a send button and optional suggestions.

use dioxus::prelude::*;

use crate::button::ButtonVariant;
use crate::icon::IconName;
use crate::icon_button::IconButton;
use crate::tag::Tag;

/// A message box. Enter sends, Shift+Enter adds a line; blank messages are ignored.
#[component]
pub fn Composer(
    /// Controlled text. Pair with `on_input`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Called with the text as it changes.
    on_input: Option<EventHandler<String>>,
    /// Called with the trimmed message on send. The field then clears.
    on_send: Option<EventHandler<String>>,
    /// Placeholder text.
    #[props(default = "Ask anything".to_string())]
    placeholder: String,
    /// Accessible name for the text field.
    #[props(default = "Message".to_string())]
    label: String,
    /// Suggestion tags shown above the field.
    #[props(default)]
    suggestions: Vec<String>,
    /// Called when a suggestion is picked. Without it the suggestion fills the field.
    on_suggestion: Option<EventHandler<String>>,
) -> Element {
    let mut draft = use_signal(|| value.peek().clone().unwrap_or_default());
    let text = value().unwrap_or_else(|| draft.cloned());
    let mut set = move |t: String| {
        draft.set(t.clone());
        if let Some(handler) = on_input {
            handler.call(t);
        }
    };
    let mut send = move || {
        let message = value().unwrap_or_else(|| draft.cloned()).trim().to_string();
        if message.is_empty() {
            return;
        }
        if let Some(handler) = on_send {
            handler.call(message);
        }
        set(String::new());
    };

    rsx! {
        div { "data-scope": "composer",
            if !suggestions.is_empty() {
                div { "data-part": "suggestions",
                    for s in suggestions {
                        Tag {
                            on_click: {
                                let pick = s.clone();
                                move |()| match on_suggestion {
                                    Some(handler) => handler.call(pick.clone()),
                                    None => set(pick.clone()),
                                }
                            },
                            "{s}"
                        }
                    }
                }
            }
            div { "data-part": "field",
                textarea {
                    rows: "1",
                    "aria-label": "{label}",
                    placeholder: "{placeholder}",
                    value: "{text}",
                    oninput: move |e| set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter && !e.modifiers().shift() && !e.is_composing() {
                            e.prevent_default();
                            send();
                        }
                    },
                }
                IconButton { icon: IconName::ArrowUp, label: "Send", variant: ButtonVariant::Accent, on_click: move |_| send() }
            }
        }
    }
}
