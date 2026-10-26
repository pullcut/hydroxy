//! `Callout`: a tinted box. Each kind has a fixed jewel and icon.

use dioxus::prelude::*;

use crate::icon::{Icon, IconName};

/// What the callout is for. Fixes its colour and icon.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CalloutKind {
    /// Background information. Sapphire, info icon.
    #[default]
    Note,
    /// A helpful suggestion. Emerald, light bulb.
    Tip,
    /// Something to check before going on. Citrine, warning triangle.
    Warning,
    /// Something destructive or broken. Garnet, alert circle.
    Danger,
}

impl CalloutKind {
    /// The `data-kind` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            CalloutKind::Note => "note",
            CalloutKind::Tip => "tip",
            CalloutKind::Warning => "warning",
            CalloutKind::Danger => "danger",
        }
    }

    /// The icon shown for this kind.
    #[must_use]
    pub const fn icon(self) -> IconName {
        match self {
            CalloutKind::Note => IconName::Info,
            CalloutKind::Tip => IconName::Lightbulb,
            CalloutKind::Warning => IconName::TriangleAlert,
            CalloutKind::Danger => IconName::CircleAlert,
        }
    }
}

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
    let role = if live && kind == CalloutKind::Danger { "alert" } else { "note" };
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
