//! Browser-only values held by a reactive owner.
//!
//! Leptos's `new_local` storage wraps values in a `SendWrapper`, which panics when it is
//! dropped on another thread. A multi-threaded server can dispose an owner on any worker, so
//! these hold [`SendOption`] instead (only a `Some` is thread-bound) and never store a value
//! outside the browser.

use leptos::prelude::*;
pub(crate) use leptos::reactive::send_wrapper_ext::SendOption;

/// `Some(value)` in the browser, `None` on the server, so it can be dropped on any thread.
pub(crate) fn browser_only<T>(value: T) -> SendOption<T> {
    SendOption::new_local(in_browser(value))
}

fn in_browser<T>(value: T) -> Option<T> {
    cfg!(all(target_arch = "wasm32", target_os = "unknown")).then_some(value)
}

/// A slot for a browser-only value, owned by the current reactive owner. On the server it
/// stays empty: `set` drops its value and reads return `None`.
pub(crate) struct Local<T: 'static>(StoredValue<SendOption<T>>);

impl<T: 'static> Clone for Local<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for Local<T> {}

impl<T: 'static> Local<T> {
    /// An empty slot.
    pub(crate) fn empty() -> Self {
        Self(StoredValue::new(SendOption::new_local(None)))
    }

    /// A slot holding `value` in the browser.
    pub(crate) fn new(value: T) -> Self {
        Self(StoredValue::new(browser_only(value)))
    }

    /// Store `value`; false once the owner is gone.
    pub(crate) fn set(self, value: T) -> bool {
        let value = in_browser(value);
        self.0.try_update_value(|s| s.update(|o| *o = value)).is_some()
    }

    /// Read the value, if there is one.
    pub(crate) fn with<R>(self, f: impl FnOnce(&T) -> R) -> Option<R> {
        self.0.try_with_value(|s| s.as_ref().map(f)).flatten()
    }

    /// Take the value out, leaving the slot empty.
    pub(crate) fn take(self) -> Option<T> {
        let mut out = None;
        self.0.try_update_value(|s| s.update(|o| out = o.take()));
        out
    }

    /// Whether the slot is empty (or the owner gone).
    pub(crate) fn is_none(self) -> bool {
        self.0.try_with_value(|s| s.is_none()).unwrap_or(true)
    }
}
