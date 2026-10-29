//! Open and close native overlays (`popover`, `<dialog>`) for machine commands.
//!
//! Dioxus has no typed handle for these DOM methods, so they run through
//! `document::eval`. That works on web and in the desktop/mobile webview; the
//! native (Blitz) renderer has no popover or dialog support yet.
//! Every call is best-effort: a missing element, or one already in the asked-for
//! state, is left alone.

use dioxus::prelude::document;

fn call(id: &str, method: &str) {
    let _ = document::eval(&format!(
        "try {{ document.getElementById({id})?.{method}(); }} catch (_) {{}}",
        id = crate::js::js_string(id)
    ));
}

/// `showPopover()` on the element with this id.
pub(crate) fn show_popover(id: &str) {
    call(id, "showPopover");
}

/// `hidePopover()` on the element with this id.
pub(crate) fn hide_popover(id: &str) {
    call(id, "hidePopover");
}

/// `showModal()` on the `<dialog>` with this id: top layer, rest of the page inert.
pub(crate) fn show_modal(id: &str) {
    let _ = document::eval(&format!(
        "try {{ const d = document.getElementById({id}); if (d && !d.open) d.showModal(); }} catch (_) {{}}",
        id = crate::js::js_string(id)
    ));
}

/// `close()` on the `<dialog>` with this id.
pub(crate) fn close_dialog(id: &str) {
    call(id, "close");
}
