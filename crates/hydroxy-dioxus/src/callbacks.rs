//! Helpers for the optional `on_*` callback props.

use dioxus::prelude::*;
use hydroxy_charts::event::Datum;
use std::cell::RefCell;
use std::rc::Rc;

/// Wrap an optional callback so it can always be handed on.
pub(crate) fn fwd<T: 'static>(cb: Option<Callback<T>>) -> Callback<T> {
    Callback::new(move |v| run(cb, v))
}

/// Run the callback with `value` if there is one.
pub(crate) fn run<T: 'static>(cb: Option<Callback<T>>, value: T) {
    if let Some(cb) = cb {
        cb.call(value);
    }
}

/// Tell `on_hover` whenever the chart's active mark `datum` changes. Fires only on change
/// (never for the starting value), the same for pointer and keyboard. A hook: call it
/// unconditionally. Does nothing when either argument is `None`.
pub(crate) fn use_hover(datum: Option<ReadSignal<Option<Datum>>>, on_hover: Option<EventHandler<Option<Datum>>>) {
    let current = use_memo(move || datum.and_then(|d| d()));
    // Lives in a hook: `use_effect` swaps its closure every render, so local state would reset.
    let last = use_hook(|| Rc::new(RefCell::new(None::<Option<Datum>>)));
    use_effect(move || {
        let now = current();
        let prev = last.replace(Some(now.clone()));
        if let (Some(prev), Some(on_hover)) = (prev, on_hover)
            && prev != now
        {
            on_hover.call(now);
        }
    });
}

/// Tell `on_select` about the active mark, if there is one. Call on click, tap or Enter.
pub(crate) fn select_active(datum: Option<ReadSignal<Option<Datum>>>, on_select: Option<EventHandler<Datum>>) {
    if let (Some(datum), Some(on_select)) = (datum, on_select)
        && let Some(d) = datum.peek().clone()
    {
        on_select.call(d);
    }
}
