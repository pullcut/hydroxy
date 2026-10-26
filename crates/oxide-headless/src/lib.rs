//! Connectors: turn machine state into ARIA/data attributes and map keys to
//! events. Plain data, no DOM, so every framework adapter shares it.

pub mod accordion;
pub mod button;
pub mod checkbox;
pub mod collapsible;
pub mod combobox;
pub mod command_palette;
pub mod dialog;
pub mod icons;
pub mod input;
pub mod jewel;
pub mod menu;
pub mod overlay;
pub mod popover;
pub mod progress_bar;
pub mod radio_group;
pub mod select;
pub mod switch;
pub mod tabs;
pub mod tags_input;
pub mod toast;
pub mod tooltip;

use std::sync::atomic::{AtomicUsize, Ordering};

/// A unique id prefix per component instance.
// process-wide counter, fine for CSR; SSR+hydration needs ids from the framework.
#[must_use]
pub fn next_id(scope: &str) -> String {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    format!("ox-{scope}-{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

/// Layout axis. Decides which arrow keys move focus.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Orientation {
    /// Left and Right arrows.
    #[default]
    Horizontal,
    /// Up and Down arrows.
    Vertical,
}

impl Orientation {
    /// The `aria-orientation` / `data-orientation` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Orientation::Horizontal => "horizontal",
            Orientation::Vertical => "vertical",
        }
    }
}
