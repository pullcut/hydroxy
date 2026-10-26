//! Focus for machine commands that move focus. Uses `MountedData`, so it
//! works on web, desktop and mobile.

use std::collections::HashMap;
use std::rc::Rc;

use dioxus::prelude::*;

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
