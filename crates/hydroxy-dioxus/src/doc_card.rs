//! `DocCard`: a document in a grid: type tile, title, snippet, optional pin and footer.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};

pub use hydroxy_headless::jewel::Jewel;

/// A document card. With `on_click` the whole card opens the document (Enter or Space).
#[component]
pub fn DocCard(
    /// The document's title.
    title: String,
    /// What kind of document, such as "Summary" or "Interview".
    type_label: Option<String>,
    /// Type tile icon. Default: a document.
    #[props(default = IconName::FileText)]
    icon: IconName,
    /// Type tile colour. Default amethyst.
    #[props(default = Jewel::Amethyst)]
    jewel: Jewel,
    /// A few lines of the document, clamped to three.
    snippet: Option<String>,
    /// Pinned state. The pin button shows only with `on_toggle_pin`.
    #[props(default)]
    pinned: ReadSignal<bool>,
    /// Shows a pin button and runs when it's pressed.
    on_toggle_pin: Option<EventHandler<()>>,
    /// Opens the document.
    on_click: Option<EventHandler<()>>,
    /// Footer content, such as an avatar and a date.
    footer: Option<Element>,
) -> Element {
    let clickable = on_click.is_some();
    let open = move || {
        if let Some(handler) = on_click {
            handler.call(());
        }
    };
    let is_pinned = pinned();
    let pin_label = if is_pinned { "Unpin" } else { "Pin" };
    rsx! {
        article {
            "data-scope": "doc-card",
            "data-clickable": clickable.then_some(""),
            role: clickable.then_some("button"),
            tabindex: clickable.then_some("0"),
            onclick: move |_| open(),
            onkeydown: move |e| {
                let key = e.key().to_string();
                if clickable && (key == "Enter" || key == " ") {
                    e.prevent_default();
                    open();
                }
            },
            div { "data-part": "top",
                span { "data-part": "icon", "data-jewel": jewel.as_str(),
                    Icon { name: icon, size: 16 }
                }
                if let Some(t) = type_label {
                    span { "data-part": "type", "{t}" }
                }
                if let Some(handler) = on_toggle_pin {
                    button {
                        r#type: "button",
                        "data-part": "pin",
                        "aria-pressed": is_pinned.to_string(),
                        "aria-label": pin_label,
                        title: pin_label,
                        onclick: move |e| {
                            e.stop_propagation();
                            handler.call(());
                        },
                        onkeydown: move |e| e.stop_propagation(),
                        Icon { name: IconName::Star, size: 15 }
                    }
                }
            }
            h3 { "data-part": "title", "{title}" }
            if let Some(s) = snippet {
                p { "data-part": "snippet", "{s}" }
            }
            if let Some(f) = footer {
                div { "data-part": "footer", {f} }
            }
        }
    }
}
