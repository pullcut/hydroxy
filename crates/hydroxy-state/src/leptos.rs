//! Leptos: a value with undo and redo.
//!
//! ```no_run
//! use leptos::prelude::*;
//! use hydroxy_state::leptos::use_history;
//!
//! #[component]
//! fn Counter() -> impl IntoView {
//!     let count = use_history(0);
//!     view! {
//!         <button on:click=move |_| count.update(|n| *n += 1)>{move || count.get()}</button>
//!         <button disabled=move || !count.can_undo() on:click=move |_| count.undo()>"Undo"</button>
//!     }
//! }
//! ```

use leptos::prelude::*;

use crate::History;

/// A reactive value plus its [`History`]. Copy it into closures freely.
pub struct HistoryHandle<T: Send + Sync + 'static> {
    value: RwSignal<T>,
    history: RwSignal<History<T>>,
}

impl<T: Send + Sync + 'static> std::fmt::Debug for HistoryHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HistoryHandle").finish_non_exhaustive()
    }
}

impl<T: Send + Sync + 'static> Clone for HistoryHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Send + Sync + 'static> Copy for HistoryHandle<T> {}

/// A value that remembers what it was, so it can be undone and redone.
pub fn use_history<T: Clone + Send + Sync + 'static>(initial: T) -> HistoryHandle<T> {
    HistoryHandle { value: RwSignal::new(initial), history: RwSignal::new(History::default()) }
}

impl<T: Clone + Send + Sync + 'static> HistoryHandle<T> {
    /// The current value (tracked).
    #[must_use]
    pub fn get(&self) -> T {
        self.value.get()
    }

    /// Read the current value without cloning it (tracked).
    pub fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.value.with(f)
    }

    /// The value as a read-only signal.
    #[must_use]
    pub fn signal(&self) -> ReadSignal<T> {
        self.value.read_only()
    }

    /// Replace the value, recording the old one for undo.
    pub fn set(&self, next: T) {
        let before = self.value.get_untracked();
        self.history.update(|h| h.record(before));
        self.value.set(next);
    }

    /// Change the value in place, recording the old one for undo.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let before = self.value.get_untracked();
        self.history.update(|h| h.record(before));
        self.value.update(f);
    }

    /// Go back one step, if there is one.
    pub fn undo(&self) {
        self.step(true);
    }

    /// Go forward one undone step, if there is one.
    pub fn redo(&self) {
        self.step(false);
    }

    fn step(&self, back: bool) {
        let mut current = self.value.get_untracked();
        let moved = self.history.try_update(|h| if back { h.undo(&mut current) } else { h.redo(&mut current) });
        if moved == Some(true) {
            self.value.set(current);
        }
    }

    /// Whether [`undo`](Self::undo) would do anything (tracked).
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.history.with(History::can_undo)
    }

    /// Whether [`redo`](Self::redo) would do anything (tracked).
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.history.with(History::can_redo)
    }
}
