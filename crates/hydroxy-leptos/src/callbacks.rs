//! Helpers for the optional `on_*` callback props.

use leptos::prelude::{Callable, Callback};

/// Wrap an optional callback so it can always be handed on.
pub(crate) fn fwd<T: 'static>(cb: Option<Callback<T>>) -> Callback<T> {
    Callback::new(move |v| run(cb, v))
}

/// Run the callback with `value` if there is one.
pub(crate) fn run<T: 'static>(cb: Option<Callback<T>>, value: T) {
    if let Some(cb) = cb {
        cb.run(value);
    }
}

/// Run the callback with `value` and return what it returned, or `None` if there is no callback.
pub(crate) fn call<T: 'static, R: 'static>(cb: Option<Callback<T, R>>, value: T) -> Option<R> {
    cb.map(|cb| cb.run(value))
}
