//! [`RenderFn`]: the type behind render props such as `KanbanBoard`'s `card`.

use std::sync::Arc;

use leptos::prelude::*;

/// Builds a view from a value. Any `Fn(T) -> impl IntoView` closure converts into it,
/// so callers never write `.into_any()`; the view is erased here instead.
pub struct RenderFn<T>(Arc<dyn Fn(T) -> AnyView + Send + Sync>);

impl<T> Clone for RenderFn<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T, F, V> From<F> for RenderFn<T>
where
    F: Fn(T) -> V + Send + Sync + 'static,
    V: IntoView + 'static,
{
    fn from(f: F) -> Self {
        Self(Arc::new(move |value| f(value).into_any()))
    }
}

impl<T> RenderFn<T> {
    /// Render `value`.
    #[must_use]
    pub fn run(&self, value: T) -> AnyView {
        (self.0)(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_view_closure_converts_without_into_any() {
        use std::sync::atomic::{AtomicU32, Ordering};
        static SEEN: AtomicU32 = AtomicU32::new(0);
        let render: RenderFn<u32> = (|n: u32| {
            SEEN.store(n, Ordering::Relaxed);
            view! { <b>{n}</b> }
        })
        .into();
        // A clone shares the closure; rendering to HTML needs `ssr`, so check the call.
        let _view = render.run(7);
        assert_eq!(SEEN.load(Ordering::Relaxed), 7);
    }
}
