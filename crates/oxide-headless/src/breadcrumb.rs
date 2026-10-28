//! Breadcrumb connector. Anatomy: root (`<nav aria-label="Breadcrumb">`), list
//! (`<ol>`), item (`<li>`), link, separator (`aria-hidden`), ellipsis (the "…"
//! button for folded items), siblings (a button listing the pages beside an
//! item), menu (the links in either popover), and measure (an invisible copy
//! of the full trail used to decide what folds). Every part carries
//! `data-scope="breadcrumb"`.
//!
//! The last item is the current page: `aria-current="page"`.

pub use oxide_machines::breadcrumb::*;

use crate::icons::Icon;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "breadcrumb";

/// The nav's accessible name.
pub const LABEL: &str = "Breadcrumb";

/// A plain link, for sibling and folded-page lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// Visible text.
    pub label: String,
    /// Destination.
    pub href: String,
}

impl Link {
    /// A link from its parts.
    #[must_use]
    pub fn new(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self { label: label.into(), href: href.into() }
    }
}

/// One step in the trail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Crumb {
    /// Visible text.
    pub label: String,
    /// Destination. The current page may leave it empty.
    pub href: Option<String>,
    /// Icon before the label.
    pub icon: Option<Icon>,
    /// Pages beside this one, listed in a dropdown after it.
    pub siblings: Vec<Link>,
}

impl Crumb {
    /// A linked crumb.
    #[must_use]
    pub fn new(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self { label: label.into(), href: Some(href.into()), icon: None, siblings: Vec::new() }
    }

    /// A crumb with no link, for the current page.
    #[must_use]
    pub fn current(label: impl Into<String>) -> Self {
        Self { label: label.into(), href: None, icon: None, siblings: Vec::new() }
    }

    /// Show `icon` before the label.
    #[must_use]
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// List these pages in a dropdown after the crumb.
    #[must_use]
    pub fn siblings(mut self, siblings: Vec<Link>) -> Self {
        self.siblings = siblings;
        self
    }
}

/// The "…" button's accessible name.
#[must_use]
pub fn ellipsis_label(count: usize) -> String {
    if count == 1 { "Show 1 hidden page".to_string() } else { format!("Show {count} hidden pages") }
}

/// The siblings button's accessible name.
#[must_use]
pub fn siblings_label(label: &str) -> String {
    format!("Pages next to {label}")
}

/// Watches the trail and reports what fits. A JS function body with parameters
/// `root` (the nav's id) and `send` (called with a string); it returns a cleanup
/// function. Each report is `"available,ellipsis,w0,w1,…"` in pixels, measured
/// from the invisible full-width copy, so folded items keep their real widths.
pub const MEASURE_JS: &str = r#"
const nav = document.getElementById(root);
if (!nav) return () => {};
const measure = () => {
  const copy = nav.querySelector('[data-part="measure"]');
  if (!copy) return;
  const items = [...copy.querySelectorAll(':scope > [data-part="item"]')].map((el) => el.getBoundingClientRect().width);
  const dots = copy.querySelector(':scope > [data-part="ellipsis-item"]');
  send([nav.clientWidth, dots ? dots.getBoundingClientRect().width : 32, ...items].map((n) => n.toFixed(1)).join(","));
};
const ro = new ResizeObserver(measure);
ro.observe(nav);
const copy = nav.querySelector('[data-part="measure"]');
if (copy) ro.observe(copy);
document.fonts?.ready.then(measure);
measure();
return () => ro.disconnect();
"#;

/// Parse one [`MEASURE_JS`] report into `(available, ellipsis, widths)`.
#[must_use]
pub fn parse_measure(report: &str) -> Option<(f64, f64, Vec<f64>)> {
    let mut nums = report.split(',').map(|n| n.trim().parse::<f64>().ok());
    let available = nums.next()??;
    let ellipsis = nums.next()??;
    let widths = nums.collect::<Option<Vec<f64>>>()?;
    Some((available, ellipsis, widths))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reports() {
        let (a, e, w) = parse_measure("300.0,24.5,40,60.5").unwrap();
        assert!((a - 300.0).abs() < 1e-9 && (e - 24.5).abs() < 1e-9);
        assert_eq!(w, vec![40.0, 60.5]);
        assert!(parse_measure("x").is_none());
    }

    #[test]
    fn labels_count_pages() {
        assert_eq!(ellipsis_label(1), "Show 1 hidden page");
        assert_eq!(ellipsis_label(3), "Show 3 hidden pages");
        assert_eq!(siblings_label("Projects"), "Pages next to Projects");
    }
}
