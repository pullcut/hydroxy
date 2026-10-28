//! Run the small JS snippets that headless connectors share with the Dioxus
//! adapter (`MEASURE_JS` and friends), so both adapters measure and position
//! the same way.

use leptos::prelude::{SetValue, StoredValue, UpdateValue};
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::{Array, Function};

/// Call a function body with named `params` (comma-separated) and `args`.
/// Returns `undefined` if it throws.
pub(crate) fn call(params: &str, body: &str, args: &[JsValue]) -> JsValue {
    let list = Array::new();
    for a in args {
        list.push(a);
    }
    Function::new_with_args(params, body).apply(&JsValue::NULL, &list).unwrap_or(JsValue::UNDEFINED)
}

/// A running watcher: the snippet's cleanup function and the Rust callback it calls.
pub(crate) struct Watch {
    stop: JsValue,
    _send: Closure<dyn FnMut(String)>,
}

impl Drop for Watch {
    fn drop(&mut self) {
        if let Some(f) = self.stop.dyn_ref::<Function>() {
            let _ = f.call0(&JsValue::NULL);
        }
    }
}

/// Start a watcher snippet (parameters `root`, `send`; returns a cleanup function).
pub(crate) fn watch(body: &str, root: &str, on: impl FnMut(String) + 'static) -> Watch {
    let send = Closure::<dyn FnMut(String)>::new(on);
    let stop = call("root, send", body, &[root.into(), send.as_ref().clone()]);
    Watch { stop, _send: send }
}

/// Keep a watcher alive in `slot` until the owner is cleaned up.
pub(crate) fn keep(slot: StoredValue<Option<Watch>, leptos::prelude::LocalStorage>, watch: Watch) {
    slot.set_value(Some(watch));
    leptos::prelude::on_cleanup(move || {
        drop(slot.try_update_value(Option::take));
    });
}

/// Write `text` to the clipboard. Best-effort: without clipboard access nothing is copied.
pub(crate) fn copy(text: &str) {
    let _ = leptos::prelude::window().navigator().clipboard().write_text(text);
}

/// Whether the visitor asked the system for less motion.
pub(crate) fn prefers_reduced_motion() -> bool {
    leptos::prelude::window()
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|m| m.matches())
}

/// Save `body` as a file called `name` with media type `mime`, through a temporary link.
pub(crate) fn download(name: &str, mime: &str, body: &str) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
    let Ok(el) = doc.create_element("a") else { return };
    let Ok(a) = el.dyn_into::<web_sys::HtmlAnchorElement>() else { return };
    a.set_href(&hydroxy_headless::whiteboard::data_url(mime, body));
    a.set_download(name);
    a.click();
}
