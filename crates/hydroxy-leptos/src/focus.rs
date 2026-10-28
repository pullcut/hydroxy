//! DOM focus for machine commands that move focus.

use leptos::prelude::document;
use leptos::wasm_bindgen::JsCast;

/// Focus the element with this id. Best-effort: a missing element stays unfocused.
pub(crate) fn focus_id(id: &str) {
    if let Some(el) = document().get_element_by_id(id) {
        let _ = el.unchecked_into::<web_sys::HtmlElement>().focus();
    }
}
