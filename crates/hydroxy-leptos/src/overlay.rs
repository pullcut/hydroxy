//! Open and close native overlays (`popover`, `<dialog>`) for machine commands.
//! Every call is best-effort: a missing element, or one already in the asked-for
//! state, is left alone.

use leptos::prelude::document;
use leptos::wasm_bindgen::JsCast;
use web_sys::{HtmlDialogElement, HtmlElement};

fn element<T: JsCast>(id: &str) -> Option<T> {
    document().get_element_by_id(id).and_then(|el| el.dyn_into::<T>().ok())
}

/// `showPopover()` on the element with this id.
pub(crate) fn show_popover(id: &str) {
    if let Some(el) = element::<HtmlElement>(id) {
        let _ = el.show_popover();
    }
}

/// `hidePopover()` on the element with this id.
pub(crate) fn hide_popover(id: &str) {
    if let Some(el) = element::<HtmlElement>(id) {
        let _ = el.hide_popover();
    }
}

/// `showModal()` on the `<dialog>` with this id: top layer, rest of the page inert.
pub(crate) fn show_modal(id: &str) {
    if let Some(el) = element::<HtmlDialogElement>(id)
        && !el.open()
    {
        let _ = el.show_modal();
    }
}

/// `close()` on the `<dialog>` with this id.
pub(crate) fn close_dialog(id: &str) {
    if let Some(el) = element::<HtmlDialogElement>(id) {
        el.close();
    }
}
