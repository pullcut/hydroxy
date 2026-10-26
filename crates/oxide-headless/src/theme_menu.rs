//! Theme menu connector. A menu of radio items (light, dark, system, then the
//! extra themes in `oxide-tokens`)
//! built from the [`crate::menu`] parts. The trigger carries
//! `data-scope="theme-menu"`; content and items keep `data-scope="menu"` so
//! they share the menu's look, with `role="menuitemradio"` and `aria-checked`.
//!
//! Applying a choice means setting `data-theme` on `<html>` and saving it under
//! [`STORAGE_KEY`]. System follows [`DARK_QUERY`], and is saved as no value, so
//! a page's boot script falls back to the system setting too.

pub use crate::menu::{
    ContentAttrs, ItemAttrs, MenuEntry, MenuEvent, content_attrs, content_id, content_keydown, item_attrs, item_id,
    trigger_id, trigger_keydown,
};
pub use crate::overlay::{Align, Placement, Side};

/// The `data-scope` value on the trigger.
pub const SCOPE: &str = "theme-menu";

/// `localStorage` key holding the saved choice.
pub const STORAGE_KEY: &str = "ox-theme";

/// Media query that is true when the system prefers dark.
pub const DARK_QUERY: &str = "(prefers-color-scheme: dark)";

/// Below the trigger, lined up with its end edge.
pub const DEFAULT_PLACEMENT: Placement = Placement::new(Side::Bottom, Align::End);

/// A theme choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Always light.
    Light,
    /// Always dark.
    Dark,
    /// Follow the system setting.
    System,
    /// Yellow ground, ink keylines, hard offset shadows.
    Neobrutalist,
    /// Neobrutalist on a near-black ground.
    NeobrutalistDark,
    /// Gunmetal and hazard orange, square corners.
    Mecha,
    /// Catppuccin Mocha pastels.
    Catppuccin,
    /// Solarized on base3.
    SolarizedLight,
    /// Solarized on base03.
    SolarizedDark,
    /// Black and white, yellow focus.
    HighContrast,
}

impl Theme {
    /// Every choice, in menu order.
    pub const ALL: [Theme; 10] = [
        Theme::Light,
        Theme::Dark,
        Theme::System,
        Theme::Neobrutalist,
        Theme::NeobrutalistDark,
        Theme::Mecha,
        Theme::Catppuccin,
        Theme::SolarizedLight,
        Theme::SolarizedDark,
        Theme::HighContrast,
    ];

    /// The menu item value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
            Theme::Neobrutalist => "neobrutalist",
            Theme::NeobrutalistDark => "neobrutalist-dark",
            Theme::Mecha => "mecha",
            Theme::Catppuccin => "catppuccin",
            Theme::SolarizedLight => "solarized-light",
            Theme::SolarizedDark => "solarized-dark",
            Theme::HighContrast => "high-contrast",
        }
    }

    /// The label shown in the menu.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
            Theme::System => "System",
            Theme::Neobrutalist => "Neobrutalist",
            Theme::NeobrutalistDark => "Neobrutalist dark",
            Theme::Mecha => "Mecha",
            Theme::Catppuccin => "Catppuccin",
            Theme::SolarizedLight => "Solarized light",
            Theme::SolarizedDark => "Solarized dark",
            Theme::HighContrast => "High contrast",
        }
    }

    /// Parse a menu item value or a saved value. Anything else is [`Theme::System`].
    #[must_use]
    pub fn parse(value: &str) -> Theme {
        Theme::ALL.into_iter().find(|t| t.as_str() == value && *t != Theme::System).unwrap_or(Theme::System)
    }

    /// The `data-theme` value to put on `<html>`, given whether the system prefers dark.
    #[must_use]
    pub const fn data_theme(self, system_dark: bool) -> &'static str {
        match self {
            Theme::System if system_dark => "dark",
            Theme::System => "light",
            other => other.as_str(),
        }
    }

    /// Whether this choice renders dark, given whether the system prefers dark.
    #[must_use]
    pub const fn is_dark(self, system_dark: bool) -> bool {
        match self {
            Theme::Light | Theme::Neobrutalist | Theme::SolarizedLight => false,
            Theme::System => system_dark,
            _ => true,
        }
    }

    /// What to save: the value, or `None` for system (remove the key).
    #[must_use]
    pub const fn stored(self) -> Option<&'static str> {
        match self {
            Theme::System => None,
            other => Some(other.as_str()),
        }
    }
}

/// The menu items, in order.
#[must_use]
pub fn entries() -> Vec<MenuEntry> {
    Theme::ALL.iter().map(|t| MenuEntry::new(t.as_str())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_round_trips_and_defaults_to_system() {
        for theme in Theme::ALL {
            assert_eq!(Theme::parse(theme.as_str()), theme);
        }
        assert_eq!(Theme::parse("nonsense"), Theme::System);
    }

    #[test]
    fn system_resolves_through_the_media_query() {
        assert_eq!(Theme::System.data_theme(true), "dark");
        assert_eq!(Theme::System.data_theme(false), "light");
        assert_eq!(Theme::Light.data_theme(true), "light");
        assert_eq!(Theme::System.stored(), None);
        assert_eq!(Theme::Dark.stored(), Some("dark"));
        assert_eq!(entries().len(), 10);
        assert_eq!(Theme::HighContrast.data_theme(false), "high-contrast");
        assert!(Theme::Mecha.is_dark(false) && !Theme::SolarizedLight.is_dark(true));
    }
}
