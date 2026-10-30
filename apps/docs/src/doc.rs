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
    /// Rows: event prop, type, when it fires. Empty for components and charts without marks.
    pub events: &'static [&'static [&'static str]],
}

/// A reference table: anchor, heading, column names, rows.
pub(crate) type Table = (&'static str, &'static str, &'static [&'static str], &'static [&'static [&'static str]]);

/// Rows every chart shares: what the user is on, and what they picked.
pub(crate) const CHART_EVENTS: &[&[&str]] = &[
    &[
        "on_hover",
        "Callback<Option<Datum>>",
        "The mark under the pointer or keyboard focus changes, or is left. Fires only on a change",
    ],
    &["on_select", "Callback<Datum>", "Click, tap or Enter on a mark"],
];

/// Added for charts with a zoomable plot.
const VIEW_EVENTS: &[&[&str]] = &[
    &["view", "Signal<View>", "Controlled zoom window; share one signal to link charts. Uncontrolled when left off"],
    &[
        "on_view_change",
        "Callback<View>",
        "Called once a zoom, pan or reset settles; not for changes that came from `view`",
    ],
];

pub(crate) const CHART_EVENTS_WITH_VIEW: &[&[&str]] =
    &[CHART_EVENTS[0], CHART_EVENTS[1], VIEW_EVENTS[0], VIEW_EVENTS[1]];

/// Hierarchy charts: the focused node.
pub(crate) const NODE_EVENTS: &[&[&str]] = &[
    CHART_EVENTS[0],
    CHART_EVENTS[1],
    &["view", "Signal<View>", "Controlled focused node. Uncontrolled when left off"],
    &[
        "on_view_change",
        "Callback<View>",
        "Called when the focused node changes; not for changes that came from `view`",
    ],
];

/// Parallel coordinates: the brushed ranges.
pub(crate) const FILTER_EVENTS: &[&[&str]] = &[
    CHART_EVENTS[0],
    CHART_EVENTS[1],
    &["on_filter", "Callback<Vec<Filter>>", "Called when a brush on an axis is drawn, moved or cleared"],
];

/// Bar race: the playing frame.
pub(crate) const FRAME_EVENTS: &[&[&str]] = &[
    CHART_EVENTS[0],
    CHART_EVENTS[1],
    &["frame", "Signal<usize>", "Controlled frame index. Uncontrolled when left off"],
    &["on_frame_change", "Callback<usize>", "Called when the shown frame changes"],
];

/// Chart frames toggle series from their legend.
pub(crate) const FRAME_TOGGLE_EVENTS: &[&[&str]] = &[&[
    "on_series_toggle",
    "Callback<(String, bool)>",
    "A legend entry is switched off or on; the bool is whether the series is now shown",
]];

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
    pub(crate) fn href(self) -> &'static str {
        match self {
            Self::Components => "/components",
            Self::Charts => "/charts",
        }
    }

    /// The section a path belongs to; guides count as components.
    pub(crate) fn of_path(path: &str) -> Self {
        if path.starts_with("/charts") { Self::Charts } else { Self::Components }
    }
}

impl Doc {
    /// Which half of the catalogue the page is in.
    pub(crate) fn section(&self) -> Section {
        if CHART_GROUPS.contains(&self.group) { Section::Charts } else { Section::Components }
    }

    /// The page's URL.
    pub(crate) fn href(&self) -> String {
        format!("{}/{}", self.section().href(), self.slug)
    }

    /// The reference tables in page order: anchor, heading, column names, rows.
    pub(crate) fn tables(&self) -> [Table; 4] {
        [
            ("anatomy", "Anatomy", &["Component", "data-part", "Renders"], self.parts),
            ("keyboard", "Keyboard", &["Key", "Does"], self.keys),
            ("props", "Props", &["Prop", "Type", "Notes"], self.props),
            ("events", "Events", &["Prop", "Type", "Notes"], self.events),
        ]
    }

    /// The demo code without the page data below the marker.
    pub(crate) fn leptos_code(&self) -> &'static str {
        self.leptos.split("\n// @docs").next().unwrap_or(self.leptos).trim_end()
    }
}
