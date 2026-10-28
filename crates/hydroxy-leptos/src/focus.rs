//! DOM focus for machine commands that move focus.

use std::time::Duration;

use leptos::prelude::{document, set_timeout};
use leptos::wasm_bindgen::JsCast;

/// Focus the element with this id. Best-effort: a missing element stays unfocused.
pub(crate) fn focus_id(id: &str) {
    if let Some(el) = document().get_element_by_id(id) {
        let _ = el.unchecked_into::<web_sys::HtmlElement>().focus();
    }
}

/// Focus the element with this id on the next tick, once the view has rendered it.
pub(crate) fn focus_soon(id: &str) {
    let id = id.to_string();
    set_timeout(move || focus_id(&id), Duration::ZERO);
}

/// Focus the input or textarea with this id and select its text.
pub(crate) fn focus_and_select(id: &str) {
    if let Some(el) = document().get_element_by_id(id) {
        // `select()` exists on both inputs and textareas.
        let el = el.unchecked_into::<web_sys::HtmlInputElement>();
        let _ = web_sys::HtmlElement::focus(&el);
        el.select();
    }
}

/// Focus the first element matching `selector` inside the element with this id, or that
/// element itself when nothing matches.
pub(crate) fn focus_first(container_id: &str, selector: &str) {
    let Some(container) = document().get_element_by_id(container_id) else { return };
    let target = container.query_selector(selector).ok().flatten().unwrap_or(container);
    let _ = target.unchecked_into::<web_sys::HtmlElement>().focus();
}
