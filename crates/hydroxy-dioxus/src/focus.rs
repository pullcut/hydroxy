//! Focus for machine commands that move focus. Uses `MountedData`, so it
//! works on web, desktop and mobile.

use std::collections::HashMap;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::js;

/// Mounted elements by item key, so a command can focus one.
#[derive(Clone, Copy)]
pub(crate) struct FocusMap(Signal<HashMap<String, Rc<MountedData>>>);

impl FocusMap {
    pub(crate) fn new() -> Self {
        Self(Signal::new(HashMap::new()))
    }

    /// Call from `onmounted`.
    pub(crate) fn register(mut self, key: String, el: Rc<MountedData>) {
        self.0.write().insert(key, el);
    }

    /// Best-effort: an unmounted element just stays unfocused.
    pub(crate) fn focus(self, key: &str) {
        if let Some(el) = self.0.peek().get(key).cloned() {
            spawn(async move {
                let _ = el.set_focus(true).await;
            });
        }
    }
}

/// Focus the element with this id. Best-effort: a missing element stays unfocused.
pub(crate) fn focus_id(id: &str) {
    js::run(&[("id", id)], "document.getElementById(id)?.focus();");
}

/// Focus the element with this id on the next frame, once the view has rendered it.
pub(crate) fn focus_soon(id: &str) {
    js::run(&[("id", id)], "requestAnimationFrame(() => document.getElementById(id)?.focus());");
}

/// Focus the input or textarea with this id and select its text.
pub(crate) fn focus_and_select(id: &str) {
    js::run(&[("id", id)], "const el = document.getElementById(id); if (el) { el.focus(); el.select?.(); }");
}

/// Focus the first element matching `selector` inside the element with this id, or that
/// element itself when nothing matches.
pub(crate) fn focus_first(container_id: &str, selector: &str) {
    js::run(
        &[("id", container_id), ("selector", selector)],
        "const c = document.getElementById(id); (c?.querySelector(selector) ?? c)?.focus();",
    );
}
