//! Skeleton connector. Anatomy: root (the loading region), placeholder (the
//! shapes, `aria-hidden`), status (visually hidden "Loading…" text), content,
//! and the shape parts: text, line, circle, rect. Every part carries
//! `data-scope="skeleton"`.
//!
//! The root carries `aria-busy` while loading. Placeholder and content share one
//! grid cell, so the content fades in over the fading placeholder.

pub use hydroxy_machines::skeleton::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "skeleton";

/// A ready-made placeholder shaped like an hydroxy component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// A paragraph of `lines` ragged lines.
    Text(usize),
    /// A card: media block, heading and a short paragraph.
    Card,
    /// Data table rows: a header row and `rows` × `cols` cells.
    TableRows(usize, usize),
    /// A list row: avatar, two lines and a trailing action.
    ListItem,
    /// A chart: title, then bars of varied height over an axis.
    Chart,
    /// A stat card: label, big figure, change line.
    StatCard,
}

impl Default for Preset {
    fn default() -> Self {
        Self::Text(3)
    }
}

impl Preset {
    /// The `data-preset` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text(_) => "text",
            Self::Card => "card",
            Self::TableRows(..) => "table",
            Self::ListItem => "list-item",
            Self::Chart => "chart",
            Self::StatCard => "stat-card",
        }
    }
}

/// Bar heights (percent of the plot) for the chart preset, from `seed`.
#[must_use]
pub fn chart_bars(count: usize, seed: u32) -> Vec<u8> {
    // Ragged widths double as bar heights: they already avoid both extremes.
    cell_widths(1, count, seed).into_iter().next().unwrap_or_default()
}

/// Attributes for the root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootAttrs {
    /// `aria-busy`.
    pub aria_busy: bool,
    /// `data-state`: `"loading"` or `"ready"`.
    pub data_state: &'static str,
}

/// Attributes for the root.
#[must_use]
pub const fn root_attrs(loading: bool) -> RootAttrs {
    RootAttrs { aria_busy: loading, data_state: if loading { "loading" } else { "ready" } }
}

/// Inline style for one placeholder text line.
#[must_use]
pub fn line_style(width: u8) -> String {
    format!("inline-size: {width}%;")
}

/// Inline style for a circle `size` px across.
#[must_use]
pub fn circle_style(size: u16) -> String {
    format!("inline-size: {size}px; block-size: {size}px;")
}

/// Inline style for a rectangle; any CSS lengths.
#[must_use]
pub fn rect_style(width: &str, height: &str) -> String {
    format!("inline-size: {width}; block-size: {height};")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_mirrors_loading() {
        assert!(root_attrs(true).aria_busy);
        assert_eq!(root_attrs(false).data_state, "ready");
    }

    #[test]
    fn styles_and_presets() {
        assert_eq!(line_style(72), "inline-size: 72%;");
        assert_eq!(circle_style(40), "inline-size: 40px; block-size: 40px;");
        assert_eq!(Preset::TableRows(3, 4).as_str(), "table");
        assert_eq!(chart_bars(7, 1).len(), 7);
    }
}
