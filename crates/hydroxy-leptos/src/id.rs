//! Instance ids that match between server rendering and hydration.
//!
//! Components need ids to link parts (`aria-controls`, `aria-labelledby`) and to move
//! focus. A server that renders many requests can't use one process-wide counter: the
//! browser would hydrate with different ids. Call [`provide_id_scope`] at the top of the
//! root component and every render counts from zero in the same order on both sides.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use leptos::prelude::*;

/// A per-render id counter. See [`provide_id_scope`].
#[derive(Clone, Default)]
pub struct IdScope(Arc<AtomicUsize>);

/// Start a fresh id counter for this render. Call once in your root component, before
/// any hydroxy component. Required for server rendering; harmless in client-only apps.
pub fn provide_id_scope() {
    provide_context(IdScope::default());
}

/// The next id for a component of `scope`, from the render's [`IdScope`] if there is one.
pub(crate) fn next_id(scope: &str) -> String {
    match use_context::<IdScope>() {
        Some(ids) => format!("ox-{scope}-{}", ids.0.fetch_add(1, Ordering::Relaxed)),
        None => hydroxy_headless::next_id(scope),
    }
}
