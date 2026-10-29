//! Callout vocabulary: the kinds, with their `data-kind`, icon and role. Shared by the
//! adapters' `Callout`.

use crate::icons::Icon;

/// What the callout is for. Fixes its colour and icon.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
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
    pub const fn icon(self) -> Icon {
        match self {
            CalloutKind::Note => Icon::Info,
            CalloutKind::Tip => Icon::Lightbulb,
            CalloutKind::Warning => Icon::TriangleAlert,
            CalloutKind::Danger => Icon::CircleAlert,
        }
    }

    /// The ARIA role: `"alert"` for a `live` danger callout, `"note"` otherwise.
    #[must_use]
    pub const fn role(self, live: bool) -> &'static str {
        if live && matches!(self, CalloutKind::Danger) { "alert" } else { "note" }
    }
}
