//! Modifier keys from Dioxus events.

use hydroxy_headless::Modifiers;

/// The modifiers from an event's `modifiers()`.
#[allow(dead_code, reason = "used by feature-gated components")]
pub(crate) fn mods(m: dioxus::prelude::Modifiers) -> Modifiers {
    Modifiers { shift: m.shift(), alt: m.alt(), ctrl: m.ctrl(), meta: m.meta() }
}
