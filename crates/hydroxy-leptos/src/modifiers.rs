//! Modifier keys from DOM events.

use hydroxy_headless::Modifiers;

/// The modifiers held during a key event.
#[allow(dead_code, reason = "used by feature-gated components")]
pub(crate) fn key_mods(ev: &web_sys::KeyboardEvent) -> Modifiers {
    Modifiers { shift: ev.shift_key(), alt: ev.alt_key(), ctrl: ev.ctrl_key(), meta: ev.meta_key() }
}

/// The modifiers held during a mouse or pointer event.
#[allow(dead_code, reason = "used by feature-gated components")]
pub(crate) fn mouse_mods(ev: &web_sys::MouseEvent) -> Modifiers {
    Modifiers { shift: ev.shift_key(), alt: ev.alt_key(), ctrl: ev.ctrl_key(), meta: ev.meta_key() }
}
