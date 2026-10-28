//! `Callout`: a tinted box. Each kind has a fixed jewel and icon.

use dioxus::prelude::*;

use crate::icon::Icon;

pub use hydroxy_headless::callout::CalloutKind;

/// Tinted callout. `role="note"`; a `live` danger callout is `role="alert"` and is announced.
#[component]
pub fn Callout(
    /// See [`CalloutKind`].
    #[props(default)]
    kind: CalloutKind,
    /// Optional bold first line.
    #[props(default)]
    title: Option<String>,
    /// Announce a danger callout when it appears. Ignored for other kinds.
    #[props(default)]
    live: bool,
    /// The body.
    children: Element,
) -> Element {
    let role = kind.role(live);
    rsx! {
        div { "data-scope": "callout", "data-part": "root", "data-kind": kind.as_str(), role,
            span { "data-scope": "callout", "data-part": "icon",
                Icon { name: kind.icon(), size: 18 }
            }
            div { "data-scope": "callout", "data-part": "content",
                if let Some(t) = title {
                    p { "data-scope": "callout", "data-part": "title", "{t}" }
                }
                div { "data-scope": "callout", "data-part": "body", {children} }
            }
        }
    }
}
