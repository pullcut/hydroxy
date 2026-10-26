//! `SearchField`: a search input with a leading icon and a clear button.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};
pub use oxide_headless::input::Size;

const SCOPE: &str = "search-field";

/// Search input. The clear button and Esc empty it and keep focus in the field.
#[component]
pub fn SearchField(
    /// Accessible name. Not shown; the icon and placeholder carry the meaning visually.
    #[props(default = "Search".to_string())]
    label: String,
    /// Controlled value. Pair with `on_input`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: String,
    /// Called with the new text on every keystroke, and with `""` on clear.
    on_input: Option<EventHandler<String>>,
    /// Placeholder text.
    #[props(default = "Search".to_string())]
    placeholder: String,
    /// Form field name.
    name: Option<String>,
    /// Block editing.
    #[props(default)]
    disabled: bool,
    /// `Md` for toolbars, `Lg` for page-level search.
    #[props(default)]
    size: Size,
) -> Element {
    let mut local = use_signal(|| default_value);
    let mut control = use_signal(|| None::<Rc<MountedData>>);
    let text = value().unwrap_or_else(|| local.cloned());
    let mut set = move |v: String| {
        local.set(v.clone());
        if let Some(handler) = on_input {
            handler.call(v);
        }
    };
    let mut clear = move || {
        set(String::new());
        if let Some(el) = control.peek().clone() {
            spawn(async move {
                let _ = el.set_focus(true).await;
            });
        }
    };
    let icon_size = if size == Size::Lg { 18 } else { 14 };
    let empty = text.is_empty();

    rsx! {
        div {
            "data-scope": SCOPE,
            "data-part": "root",
            "data-size": size.as_str(),
            "data-disabled": disabled.then_some(""),
            span { "data-scope": SCOPE, "data-part": "icon",
                Icon { name: IconName::Search, size: icon_size }
            }
            input {
                "data-scope": SCOPE,
                "data-part": "control",
                r#type: "search",
                "aria-label": label,
                placeholder,
                name,
                autocomplete: "off",
                disabled,
                value: text,
                onmounted: move |e| control.set(Some(e.data())),
                oninput: move |e| set(e.value()),
                onkeydown: move |e| {
                    // With text: Esc clears. Empty: let Esc reach a parent (a dialog, say).
                    if e.key().to_string() == "Escape" && !empty {
                        e.prevent_default();
                        e.stop_propagation();
                        clear();
                    }
                },
            }
            button {
                r#type: "button",
                "data-scope": SCOPE,
                "data-part": "clear",
                "aria-label": "Clear search",
                hidden: empty || disabled,
                // Keep focus in the field when clicked.
                onpointerdown: |e| e.prevent_default(),
                onclick: move |_| clear(),
                Icon { name: IconName::X, size: 13 }
            }
        }
    }
}
