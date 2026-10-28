//! Helpers for the optional `on_*` callback props.

use hydroxy_charts::event::Datum;
use leptos::prelude::{Callable, Callback, Effect, Get, GetUntracked, Memo, Signal, untrack};

/// Run the callback with `value` if there is one.
pub(crate) fn run<T: 'static>(cb: Option<Callback<T>>, value: T) {
    if let Some(cb) = cb {
        cb.run(value);
    }
}

/// Tell `on_hover` whenever the chart's active mark `datum` changes. Fires only on change
/// (never for the starting value), the same for pointer and keyboard. Does nothing when
/// either argument is `None`.
pub(crate) fn watch_hover(datum: Option<Signal<Option<Datum>>>, on_hover: Option<Callback<Option<Datum>>>) {
    let (Some(datum), Some(on_hover)) = (datum, on_hover) else { return };
    let current = Memo::new(move |_| datum.get());
    Effect::new(move |prev: Option<()>| {
        let now = current.get();
        if prev.is_some() {
            untrack(|| on_hover.run(now));
        }
    });
}

/// Tell `on_select` about the active mark, if there is one. Call on click, tap or Enter.
pub(crate) fn select_active(datum: Option<Signal<Option<Datum>>>, on_select: Option<Callback<Datum>>) {
    if let (Some(datum), Some(on_select)) = (datum, on_select)
        && let Some(d) = datum.get_untracked()
    {
        on_select.run(d);
    }
}
