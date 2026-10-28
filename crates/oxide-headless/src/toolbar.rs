//! Toolbar connector. Anatomy: root (`role="toolbar"`), group (`role="group"`),
//! separator (`role="separator"`), item (a `<button>`; toggles carry
//! `aria-pressed`), and more (the "More" menu button holding items that don't
//! fit). Every part carries `data-scope="toolbar"`.
//!
//! One tab stop: the active item has `tabindex="0"`, the rest `-1`.

pub use oxide_machines::toolbar::*;

use crate::icons::Icon;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "toolbar";

/// Layout direction, which also picks the arrow keys.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Orientation {
    /// A row; Left and Right move. Only horizontal toolbars overflow.
    #[default]
    Horizontal,
    /// A column; Up and Down move.
    Vertical,
}

impl Orientation {
    /// The `aria-orientation` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// One toolbar item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolbarItem {
    /// Unique key.
    pub value: String,
    /// Accessible name and tooltip; shown as text when there's no icon, and in the "More" menu.
    pub label: String,
    /// Icon-only button when set.
    pub icon: Option<Icon>,
    /// Items with the same group sit together between separators.
    pub group: u16,
    /// A toggle button (`aria-pressed`).
    pub toggle: bool,
    /// Not clickable, skipped by arrow keys.
    pub disabled: bool,
    /// Higher stays in the bar longer when space runs out.
    pub priority: i32,
    /// Keyboard shortcut shown in the "More" menu, such as "⌘B".
    pub shortcut: Option<String>,
}

impl ToolbarItem {
    /// An action button in group 0.
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            group: 0,
            toggle: false,
            disabled: false,
            priority: 0,
            shortcut: None,
        }
    }

    /// Make it a toggle button.
    #[must_use]
    pub fn toggle(mut self) -> Self {
        self.toggle = true;
        self
    }

    /// Show this icon instead of the label.
    #[must_use]
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Put it in group `group`.
    #[must_use]
    pub fn group(mut self, group: u16) -> Self {
        self.group = group;
        self
    }

    /// Overflow priority: higher stays longer.
    #[must_use]
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Show a shortcut hint in the "More" menu.
    #[must_use]
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Disable it.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// The machine's view of the items.
#[must_use]
pub fn entries(items: &[ToolbarItem]) -> Vec<ToolbarEntry> {
    items
        .iter()
        .map(|i| ToolbarEntry { value: i.value.clone(), toggle: i.toggle, disabled: i.disabled, priority: i.priority })
        .collect()
}

/// Items split into runs of the same group, keeping order. Each run is `(group, indices)`.
#[must_use]
pub fn groups(items: &[ToolbarItem]) -> Vec<(u16, Vec<usize>)> {
    let mut out: Vec<(u16, Vec<usize>)> = Vec::new();
    for (i, item) in items.iter().enumerate() {
        match out.last_mut() {
            Some((g, run)) if *g == item.group => run.push(i),
            _ => out.push((item.group, vec![i])),
        }
    }
    out
}

/// Map a key to an event. `Some` means: prevent default.
#[must_use]
pub fn keydown(key: &str, orientation: Orientation) -> Option<ToolbarEvent> {
    match (key, orientation) {
        ("ArrowRight", Orientation::Horizontal) | ("ArrowDown", Orientation::Vertical) => {
            Some(ToolbarEvent::Move(Direction::Next))
        }
        ("ArrowLeft", Orientation::Horizontal) | ("ArrowUp", Orientation::Vertical) => {
            Some(ToolbarEvent::Move(Direction::Prev))
        }
        ("Home", _) => Some(ToolbarEvent::Home),
        ("End", _) => Some(ToolbarEvent::End),
        _ => None,
    }
}

/// The DOM id of an item (or of the "More" button for [`MORE`]).
#[must_use]
pub fn item_id(root: &str, value: &str) -> String {
    format!("{root}-item-{value}")
}

/// `tabindex` for an item: `0` for the tab stop, `-1` otherwise.
#[must_use]
pub fn tabindex(state: &ToolbarState, input: &ToolbarInput, value: &str) -> &'static str {
    if tab_stop(state, input).as_deref() == Some(value) { "0" } else { "-1" }
}

/// Watches the toolbar and reports widths. A JS function body with parameters
/// `root` (the toolbar's id) and `send` (called with a string); it returns a
/// cleanup function. Each report is `"available,more,w0,w1,…"` in item order.
/// Hidden items keep the width they had when last shown; separators always
/// count, so hiding a whole group never makes room that would bring it back.
pub const MEASURE_JS: &str = r#"
const bar = document.getElementById(root);
if (!bar) return () => {};
const cache = new Map();
let sep = 0, more = 0;
const measure = () => {
  const cs = getComputedStyle(bar);
  const gap = parseFloat(cs.columnGap) || 0;
  const inner = bar.clientWidth - (parseFloat(cs.paddingLeft) || 0) - (parseFloat(cs.paddingRight) || 0);
  const items = [...bar.querySelectorAll('[data-part="item"][data-index]')];
  items.sort((a, b) => a.dataset.index - b.dataset.index);
  const widths = items.map((el) => {
    if (el.offsetParent !== null) cache.set(el.dataset.value, el.getBoundingClientRect().width + gap);
    return cache.get(el.dataset.value) ?? 32 + gap;
  });
  const s = bar.querySelector('[data-part="separator"]');
  if (s && s.offsetParent !== null) sep = s.getBoundingClientRect().width + gap;
  const m = bar.querySelector('[data-part="more"]');
  if (m && m.offsetParent !== null) more = m.getBoundingClientRect().width + gap;
  const seps = Number(bar.dataset.groups || 1) - 1;
  const available = inner + gap - seps * sep;
  send([available, more || 32 + gap, ...widths].map((n) => n.toFixed(1)).join(","));
};
const ro = new ResizeObserver(measure);
ro.observe(bar);
document.fonts?.ready.then(measure);
measure();
return () => ro.disconnect();
"#;

/// Parse one [`MEASURE_JS`] report into a [`ToolbarEvent::Measured`].
#[must_use]
pub fn parse_measure(report: &str) -> Option<ToolbarEvent> {
    let mut nums = report.split(',').map(|n| n.trim().parse::<f64>().ok());
    let available = nums.next()??;
    let more = nums.next()??;
    let widths = nums.collect::<Option<Vec<f64>>>()?;
    Some(ToolbarEvent::Measured { widths, available, more })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_follow_runs() {
        let items = vec![
            ToolbarItem::new("a", "A"),
            ToolbarItem::new("b", "B"),
            ToolbarItem::new("c", "C").group(1),
            ToolbarItem::new("d", "D").group(2),
        ];
        assert_eq!(groups(&items), vec![(0, vec![0, 1]), (1, vec![2]), (2, vec![3])]);
    }

    #[test]
    fn arrows_follow_orientation() {
        assert_eq!(keydown("ArrowRight", Orientation::Horizontal), Some(ToolbarEvent::Move(Direction::Next)));
        assert_eq!(keydown("ArrowDown", Orientation::Horizontal), None);
        assert_eq!(keydown("ArrowUp", Orientation::Vertical), Some(ToolbarEvent::Move(Direction::Prev)));
        assert_eq!(keydown("End", Orientation::Vertical), Some(ToolbarEvent::End));
    }

    #[test]
    fn parses_reports() {
        assert_eq!(
            parse_measure("200,36,40.5,40"),
            Some(ToolbarEvent::Measured { widths: vec![40.5, 40.0], available: 200.0, more: 36.0 })
        );
        assert_eq!(parse_measure(""), None);
    }

    #[test]
    fn one_tab_stop() {
        let items = vec![ToolbarItem::new("a", "A"), ToolbarItem::new("b", "B")];
        let input = ToolbarInput { items: entries(&items), ..Default::default() };
        let state = ToolbarState::default();
        assert_eq!(tabindex(&state, &input, "a"), "0");
        assert_eq!(tabindex(&state, &input, "b"), "-1");
    }
}
