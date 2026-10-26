//! Command palette connector. Anatomy: content (the native `<dialog>`), panel,
//! search (row), input, list, group, group-label, item, item-icon, item-hint,
//! empty. Every part carries `data-scope="command-palette"`.
//!
//! Commands are described with [`CommandItem`]; [`grouped`] puts them in display
//! order (groups in first-appearance order), which is also the arrow-key order.

pub use oxide_machines::command_palette::*;

use crate::icons::Icon;
use oxide_machines::listbox;

/// The `data-scope` value on every palette part.
pub const SCOPE: &str = "command-palette";

/// One command as the owner describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandItem {
    /// Unique key. Reported by `on_select`.
    pub id: String,
    /// What the row says. Say what happens: "Open the project".
    pub label: String,
    /// Heading the command is listed under.
    pub group: String,
    /// Extra words the search matches, not shown.
    pub keywords: String,
    /// Right-aligned shortcut, such as "⌘N".
    pub hint: Option<String>,
    /// Leading icon.
    pub icon: Option<Icon>,
}

impl CommandItem {
    /// A command with a label and a group.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, group: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            group: group.into(),
            keywords: String::new(),
            hint: None,
            icon: None,
        }
    }

    /// Extra words the search matches.
    #[must_use]
    pub fn keywords(mut self, keywords: impl Into<String>) -> Self {
        self.keywords = keywords.into();
        self
    }

    /// A right-aligned shortcut.
    #[must_use]
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// A leading icon.
    #[must_use]
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    fn search_text(&self) -> String {
        format!("{} {} {}", self.label, self.group, self.keywords)
    }
}

/// Commands that match `query`, grouped, groups in first-appearance order.
#[must_use]
pub fn grouped<'a>(items: &'a [CommandItem], query: &str) -> Vec<(&'a str, Vec<&'a CommandItem>)> {
    let mut groups: Vec<(&str, Vec<&CommandItem>)> = Vec::new();
    for item in items.iter().filter(|i| listbox::matches(&i.search_text(), query)) {
        match groups.iter_mut().find(|(g, _)| *g == item.group) {
            Some((_, list)) => list.push(item),
            None => groups.push((item.group.as_str(), vec![item])),
        }
    }
    groups
}

/// The machine's view of `items`, in display (grouped) order.
#[must_use]
pub fn entries(items: &[CommandItem]) -> Vec<CommandEntry> {
    grouped(items, "")
        .into_iter()
        .flat_map(|(_, list)| list)
        .map(|i| CommandEntry { id: i.id.clone(), text: i.search_text() })
        .collect()
}

/// The `<dialog>`'s id.
#[must_use]
pub fn content_id(root: &str) -> String {
    format!("{root}-content")
}

/// The search input's id.
#[must_use]
pub fn input_id(root: &str) -> String {
    format!("{root}-input")
}

/// The list's id.
#[must_use]
pub fn list_id(root: &str) -> String {
    format!("{root}-list")
}

/// The id of command `id`.
#[must_use]
pub fn item_id(root: &str, id: &str) -> String {
    format!("{root}-item-{id}")
}

/// The id of a group's heading.
#[must_use]
pub fn group_label_id(root: &str, index: usize) -> String {
    format!("{root}-group-{index}")
}

/// `aria-activedescendant` for the search input.
#[must_use]
pub fn activedescendant(root: &str, state: &CommandPaletteState) -> Option<String> {
    state.highlighted.as_deref().map(|id| item_id(root, id))
}

/// `data-state` for open state.
#[must_use]
pub const fn data_state(open: bool) -> &'static str {
    if open { "open" } else { "closed" }
}

/// Keys on the search input. Escape is handled on the dialog. `Some` means: prevent the default.
#[must_use]
pub fn keydown(key: &str) -> Option<CommandPaletteEvent> {
    match key {
        "ArrowDown" => Some(CommandPaletteEvent::Move(Direction::Next)),
        "ArrowUp" => Some(CommandPaletteEvent::Move(Direction::Prev)),
        "Enter" => Some(CommandPaletteEvent::Enter),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_keep_first_appearance_order_and_entries_follow_it() {
        let items = vec![
            CommandItem::new("new", "New project", "Projects"),
            CommandItem::new("theme", "Switch theme", "Settings"),
            CommandItem::new("open", "Open project", "Projects"),
        ];
        let groups = grouped(&items, "");
        assert_eq!(
            groups.iter().map(|(g, l)| (*g, l.len())).collect::<Vec<_>>(),
            vec![("Projects", 2), ("Settings", 1)]
        );
        assert_eq!(entries(&items).iter().map(|e| e.id.as_str()).collect::<Vec<_>>(), vec!["new", "open", "theme"]);
        assert_eq!(grouped(&items, "settings").len(), 1);
    }
}
