//! `Card`: the surface container. Jewel tones are for feature and highlight cards.

use dioxus::prelude::*;

pub use oxide_headless::jewel::Jewel;

/// A card's surface.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CardTone {
    /// White (ink-800 in dark) with a border and a light shadow.
    #[default]
    Default,
    /// Recessed page-tone surface.
    Sunken,
    /// Ink on light pages, paper on dark.
    Inverse,
    /// Full jewel colour, for feature and highlight cards.
    Jewel(Jewel),
}

impl CardTone {
    /// The `data-tone` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            CardTone::Default => "default",
            CardTone::Sunken => "sunken",
            CardTone::Inverse => "inverse",
            CardTone::Jewel(jewel) => jewel.as_str(),
        }
    }
}

/// Surface container. With `on_click` it becomes a keyboard-reachable button (Enter or Space).
#[component]
pub fn Card(
    /// See [`CardTone`].
    #[props(default)]
    tone: CardTone,
    /// Inner padding in px.
    #[props(default = 24)]
    padding: u16,
    /// Lift on hover. Implied by `on_click`.
    #[props(default)]
    interactive: bool,
    /// Makes the whole card a button.
    on_click: Option<EventHandler<()>>,
    /// Card content.
    children: Element,
) -> Element {
    let clickable = on_click.is_some();
    let activate = move || {
        if let Some(handler) = on_click {
            handler.call(());
        }
    };
    rsx! {
        div {
            "data-scope": "card",
            "data-part": "root",
            "data-tone": tone.as_str(),
            "data-interactive": (interactive || clickable).then_some(""),
            role: clickable.then_some("button"),
            tabindex: clickable.then_some("0"),
            style: "padding: {padding}px;",
            onclick: move |_| activate(),
            onkeydown: move |e| {
                let key = e.key().to_string();
                if clickable && (key == "Enter" || key == " ") {
                    e.prevent_default();
                    activate();
                }
            },
            {children}
        }
    }
}
