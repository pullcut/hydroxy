//! `PriorityMark`: P0 to P3 in mono. P0 is solid garnet.

use dioxus::prelude::*;

/// Names for P0 to P3, used as the tooltip and accessible name.
pub const PRIORITY_NAMES: [&str; 4] = ["Urgent", "High", "Medium", "Low"];

/// A compact priority label.
#[component]
pub fn PriorityMark(
    /// 0 (urgent) to 3 (low). Larger values show as 3.
    #[props(default = 2)]
    level: u8,
    /// Overrides the name, such as "Blocks release".
    label: Option<String>,
) -> Element {
    let level = level.min(3);
    let name = label.unwrap_or_else(|| PRIORITY_NAMES[usize::from(level)].to_string());
    rsx! {
        span {
            "data-scope": "priority-mark",
            "data-level": "{level}",
            title: "{name}",
            "aria-label": "Priority {level}: {name}",
            "P{level}"
        }
    }
}
