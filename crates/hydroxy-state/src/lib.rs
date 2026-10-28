//! Undo and redo for any value.
//!
//! [`History`] keeps the values a document had before each change, as shared [`Arc`]
//! snapshots, up to a limit. It never holds the current value: the owner does, and hands it
//! over on [`History::undo`] and [`History::redo`], which swap it with the snapshot.
//!
//! ```
//! use hydroxy_state::History;
//!
//! let mut doc = vec!["a"];
//! let mut history = History::new(100);
//! history.record(doc.clone());
//! doc.push("b");
//! assert!(history.undo(&mut doc));
//! assert_eq!(doc, ["a"]);
//! assert!(history.redo(&mut doc));
//! assert_eq!(doc, ["a", "b"]);
//! ```
//!
//! With the `leptos` or `dioxus` feature, `use_history` wraps a reactive value with its
//! history so components can `set`, `undo` and `redo` it.

use std::collections::VecDeque;
use std::sync::Arc;

#[cfg(feature = "dioxus")]
pub mod dioxus;
#[cfg(feature = "leptos")]
pub mod leptos;

/// How many undo steps [`History::default`] keeps.
pub const DEFAULT_LIMIT: usize = 100;

/// Past and future snapshots of a value. See the [crate docs](crate).
#[derive(Debug, Clone, PartialEq)]
pub struct History<T> {
    past: VecDeque<Arc<T>>,
    future: Vec<Arc<T>>,
    limit: usize,
}

impl<T> Default for History<T> {
    fn default() -> Self {
        Self::new(DEFAULT_LIMIT)
    }
}

impl<T> History<T> {
    /// An empty history that keeps at most `limit` undo steps (at least one).
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self { past: VecDeque::new(), future: Vec::new(), limit: limit.max(1) }
    }

    /// Remember `before` as the value to return to, ahead of a change. Clears redo, and
    /// drops the oldest step past the limit.
    pub fn record(&mut self, before: impl Into<Arc<T>>) {
        self.past.push_back(before.into());
        if self.past.len() > self.limit {
            self.past.pop_front();
        }
        self.future.clear();
    }

    /// Like [`record`](Self::record), but only when `before` differs from `current`.
    /// Returns whether a step was recorded.
    pub fn record_change(&mut self, before: impl Into<Arc<T>>, current: &T) -> bool
    where
        T: PartialEq,
    {
        let before = before.into();
        if *before == *current {
            return false;
        }
        self.record(before);
        true
    }

    /// Swap `current` with the last recorded value. Returns whether there was one.
    pub fn undo(&mut self, current: &mut T) -> bool
    where
        T: Clone,
    {
        let Some(prev) = self.past.pop_back() else { return false };
        let now = std::mem::replace(current, Arc::unwrap_or_clone(prev));
        self.future.push(Arc::new(now));
        true
    }

    /// Swap `current` with the last undone value. Returns whether there was one.
    pub fn redo(&mut self, current: &mut T) -> bool
    where
        T: Clone,
    {
        let Some(next) = self.future.pop() else { return false };
        let now = std::mem::replace(current, Arc::unwrap_or_clone(next));
        self.past.push_back(Arc::new(now));
        true
    }

    /// Whether [`undo`](Self::undo) would do anything.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    /// Whether [`redo`](Self::redo) would do anything.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    /// How many undo steps are held.
    #[must_use]
    pub fn undo_len(&self) -> usize {
        self.past.len()
    }

    /// How many redo steps are held.
    #[must_use]
    pub fn redo_len(&self) -> usize {
        self.future.len()
    }

    /// Forget everything.
    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_and_redo_swap_values() {
        let mut v = 1;
        let mut h = History::default();
        h.record(v);
        v = 2;
        h.record(v);
        v = 3;
        assert!(h.undo(&mut v));
        assert_eq!(v, 2);
        assert!(h.undo(&mut v));
        assert_eq!(v, 1);
        assert!(!h.undo(&mut v));
        assert!(h.redo(&mut v));
        assert!(h.redo(&mut v));
        assert_eq!(v, 3);
        assert!(!h.redo(&mut v));
    }

    #[test]
    fn a_new_change_clears_redo() {
        let mut v = 1;
        let mut h = History::default();
        h.record(v);
        v = 2;
        assert!(h.undo(&mut v));
        assert!(h.can_redo());
        h.record(v);
        assert!(!h.can_redo());
    }

    #[test]
    fn the_limit_drops_the_oldest() {
        let mut h = History::new(2);
        for i in 0..5 {
            h.record(i);
        }
        assert_eq!(h.undo_len(), 2);
        let mut v = 9;
        assert!(h.undo(&mut v));
        assert_eq!(v, 4);
        assert!(h.undo(&mut v));
        assert_eq!(v, 3);
        assert!(!h.can_undo());
    }

    #[test]
    fn record_change_skips_equal_values() {
        let mut h = History::default();
        assert!(!h.record_change(1, &1));
        assert!(h.record_change(1, &2));
        assert_eq!(h.undo_len(), 1);
    }
}
