//! Dioxus: a value with undo and redo.
//!
//! ```no_run
//! use dioxus::prelude::*;
//! use hydroxy_state::dioxus::use_history;
//!
//! #[component]
//! fn Counter() -> Element {
//!     let mut count = use_history(|| 0);
//!     rsx! {
//!         button { onclick: move |_| count.update(|n| *n += 1), "{count.get()}" }
//!         button { disabled: !count.can_undo(), onclick: move |_| count.undo(), "Undo" }
//!     }
//! }
//! ```

use dioxus::prelude::*;

use crate::History;

/// A reactive value plus its [`History`]. Copy it into closures freely.
pub struct HistoryHandle<T: 'static> {
    value: Signal<T>,
    history: Signal<History<T>>,
}

impl<T: 'static> Clone for HistoryHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for HistoryHandle<T> {}

/// A value that remembers what it was, so it can be undone and redone.
pub fn use_history<T: Clone + 'static>(initial: impl FnOnce() -> T) -> HistoryHandle<T> {
    let value = use_signal(initial);
    let history = use_signal(History::default);
    HistoryHandle { value, history }
}

impl<T: Clone + 'static> HistoryHandle<T> {
    /// The current value (subscribes the caller).
    #[must_use]
    pub fn get(&self) -> T {
        self.value.read().clone()
    }

    /// The value's signal, for reading without cloning.
    #[must_use]
    pub fn signal(&self) -> ReadSignal<T> {
        self.value.into()
    }

    /// Replace the value, recording the old one for undo.
    pub fn set(&mut self, next: T) {
        let before = self.value.peek().clone();
        self.history.write().record(before);
        self.value.set(next);
    }

    /// Change the value in place, recording the old one for undo.
    pub fn update(&mut self, f: impl FnOnce(&mut T)) {
        let before = self.value.peek().clone();
        self.history.write().record(before);
        f(&mut self.value.write());
    }

    /// Go back one step, if there is one.
    pub fn undo(&mut self) {
        self.step(true);
    }

    /// Go forward one undone step, if there is one.
    pub fn redo(&mut self) {
        self.step(false);
    }

    fn step(&mut self, back: bool) {
        let mut current = self.value.peek().clone();
        let moved = {
            let mut h = self.history.write();
            if back { h.undo(&mut current) } else { h.redo(&mut current) }
        };
        if moved {
            self.value.set(current);
        }
    }

    /// Whether [`undo`](Self::undo) would do anything (subscribes the caller).
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.history.read().can_undo()
    }

    /// Whether [`redo`](Self::redo) would do anything (subscribes the caller).
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.history.read().can_redo()
    }
}
