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

/// Chart families, in sidebar order. A page in one of these groups lives under `/charts`.
pub(crate) const CHART_GROUPS: &[&str] = &[
    "Bars",
    "Lines and areas",
    "Points",
    "Distributions",
    "Time",
    "Finance",
    "Parts of a whole",
    "Radial",
    "Relationships",
    "Maps",
    "Grids and tables",
    "KPIs and frames",
];

/// The two halves of the catalogue, each with its own index and sidebar.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Section {
    Components,
    Charts,
}

impl Section {
    /// The index page.
    pub fn href(self) -> &'static str {
        match self {
            Self::Components => "/components",
            Self::Charts => "/charts",
        }
    }

    /// The section a path belongs to; guides count as components.
    pub fn of_path(path: &str) -> Self {
        if path.starts_with("/charts") { Self::Charts } else { Self::Components }
    }
}

impl Doc {
    /// Which half of the catalogue the page is in.
    pub fn section(&self) -> Section {
        if CHART_GROUPS.contains(&self.group) { Section::Charts } else { Section::Components }
    }

    /// The page's URL.
    pub fn href(&self) -> String {
        format!("{}/{}", self.section().href(), self.slug)
    }

    /// The demo code without the page data below the marker.
    pub fn leptos_code(&self) -> &'static str {
        self.leptos.split("\n// @docs").next().unwrap_or(self.leptos).trim_end()
    }
}
