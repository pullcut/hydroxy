//! `Badge`: a small status label, such as "Ready" or "2 flags".

use dioxus::prelude::*;

/// Badge colour. Each tone pairs a soft background with a strong text colour.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tone {
    /// Grey. For counts and plain labels.
    #[default]
    Neutral,
    /// Green. Done, passed, ready.
    Success,
    /// Yellow. Needs attention soon.
    Warning,
    /// Red. Failed or blocked.
    Danger,
    /// Blue. Informational.
    Info,
    /// Violet. New or highlighted.
    Accent,
}

impl Tone {
    /// The `data-tone` value the styles key off.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Tone::Neutral => "neutral",
            Tone::Success => "success",
            Tone::Warning => "warning",
            Tone::Danger => "danger",
            Tone::Info => "info",
            Tone::Accent => "accent",
        }
    }
}

/// A status label.
#[component]
pub fn Badge(
    /// Colour. Default neutral.
    #[props(default)]
    tone: Tone,
    /// Show a dot before the text.
    #[props(default)]
    dot: bool,
    /// The label.
    children: Element,
) -> Element {
    rsx! {
        span { "data-scope": "badge", "data-tone": tone.as_str(),
            if dot {
                span { "data-part": "dot", "aria-hidden": "true" }
            }
            {children}
        }
    }
}
