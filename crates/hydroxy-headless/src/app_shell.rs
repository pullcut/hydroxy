//! App shell connector. Anatomy: root, skip-link, header, menu, body,
//! navigation, main, aside, footer. Every part carries `data-scope="app-shell"`
//! and its `data-part`. On narrow screens the navigation is a native popover.

pub use hydroxy_machines::app_shell::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "app-shell";

/// Defaults for the labels.
pub const NAVIGATION_LABEL: &str = "Navigation";

/// The menu button's accessible name.
pub const TRIGGER_LABEL: &str = "Open navigation";

/// The close button's accessible name.
pub const CLOSE_LABEL: &str = "Close navigation";

/// The navigation element id, the menu button's `aria-controls` and `popovertarget`.
#[must_use]
pub fn navigation_id(root: &str) -> String {
    format!("{root}-navigation")
}

/// The id the skip link points at, when the host gave none.
#[must_use]
pub fn default_main_id(root: &str) -> String {
    format!("{root}-main")
}

/// A hamburger icon for a 24x24 line-icon `<svg>`.
pub const MENU: &str = r#"<line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="18" y2="18"/>"#;

/// A close icon.
pub const CLOSE: &str = r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#;

/// The inline style that gives `main` scroll mode its height.
#[must_use]
pub fn host_style(mode: ScrollMode, block_size: Option<f64>) -> Option<String> {
    match (mode, block_size) {
        (ScrollMode::Main, Some(h)) if h.is_finite() && h > 0.0 => Some(format!("--ox-app-shell-block-size: {h}px")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_and_style() {
        assert_eq!(navigation_id("a"), "a-navigation");
        assert_eq!(host_style(ScrollMode::Main, Some(600.0)).as_deref(), Some("--ox-app-shell-block-size: 600px"));
        assert_eq!(host_style(ScrollMode::Document, Some(600.0)), None);
        assert_eq!(host_style(ScrollMode::Main, None), None);
    }
}
