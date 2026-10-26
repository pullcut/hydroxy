//! One docs page.

use leptos::prelude::AnyView;

/// Everything a component page shows.
pub(crate) struct Doc {
    pub slug: &'static str,
    pub name: &'static str,
    pub group: &'static str,
    pub summary: &'static str,
    pub demo: fn() -> AnyView,
    /// The demo file itself (`include_str!`). Cut at the `// @docs` marker.
    pub leptos: &'static str,
    /// The matching module in `examples/dioxus-gallery`.
    pub dioxus: &'static str,
    /// Rows: component, data-part, what it renders.
    pub parts: &'static [&'static [&'static str]],
    /// Rows: key, what it does.
    pub keys: &'static [&'static [&'static str]],
    /// Rows: prop, type, notes.
    pub props: &'static [&'static [&'static str]],
}

impl Doc {
    /// The demo code without the page data below the marker.
    pub fn leptos_code(&self) -> &'static str {
        self.leptos.split("\n// @docs").next().unwrap_or(self.leptos).trim_end()
    }
}
