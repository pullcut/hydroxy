//! Tree view owns which branches are expanded, which items are selected and
//! which item has focus. Navigation walks the visible rows: the tree flattened
//! in display order, skipping children of collapsed branches.

use oxide_statechart::{Machine, Transition};

pub use crate::list::Direction;

/// One node in the tree. Branches have children; leaves don't.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode {
    /// Unique key across the whole tree.
    pub id: String,
    /// Visible text, also used for typeahead.
    pub label: String,
    /// Child nodes, in display order.
    pub children: Vec<TreeNode>,
    /// Focusable, but can't be selected or expanded.
    pub disabled: bool,
}

impl TreeNode {
    /// A leaf.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), children: Vec::new(), disabled: false }
    }

    /// Set the children, making this a branch.
    #[must_use]
    pub fn children(mut self, children: Vec<TreeNode>) -> Self {
        self.children = children;
        self
    }

    /// Mark this node disabled.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Whether this node has children.
    #[must_use]
    pub fn is_branch(&self) -> bool {
        !self.children.is_empty()
    }
}

/// One visible row of the flattened tree.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeRow<'a> {
    /// The node this row shows.
    pub node: &'a TreeNode,
    /// The parent's id. `None` at the top level.
    pub parent: Option<&'a str>,
    /// Depth, starting at 1 (`aria-level`).
    pub level: usize,
    /// Number of siblings including this one (`aria-setsize`).
    pub set_size: usize,
    /// 1-based position among siblings (`aria-posinset`).
    pub pos_in_set: usize,
    /// Whether this branch is open. Always false for leaves.
    pub expanded: bool,
}

/// Flatten `nodes` into the rows a reader sees, in display order.
#[must_use]
pub fn visible_rows<'a>(nodes: &'a [TreeNode], expanded: &[String]) -> Vec<TreeRow<'a>> {
    fn walk<'a>(
        nodes: &'a [TreeNode],
        parent: Option<&'a str>,
        level: usize,
        expanded: &[String],
        out: &mut Vec<TreeRow<'a>>,
    ) {
        for (i, node) in nodes.iter().enumerate() {
            let open = node.is_branch() && expanded.contains(&node.id);
            out.push(TreeRow { node, parent, level, set_size: nodes.len(), pos_in_set: i + 1, expanded: open });
            if open {
                walk(&node.children, Some(&node.id), level + 1, expanded, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(nodes, None, 1, expanded, &mut out);
    out
}

/// Find a node anywhere in the tree.
#[must_use]
pub fn find<'a>(nodes: &'a [TreeNode], id: &str) -> Option<&'a TreeNode> {
    nodes.iter().find_map(|n| if n.id == id { Some(n) } else { find(&n.children, id) })
}

/// The sibling list containing `id` (the top level or a parent's children).
fn siblings<'a>(nodes: &'a [TreeNode], id: &str) -> Option<&'a [TreeNode]> {
    if nodes.iter().any(|n| n.id == id) {
        return Some(nodes);
    }
    nodes.iter().find_map(|n| siblings(&n.children, id))
}

/// Whether `id` sits somewhere below `ancestor`.
fn is_descendant(nodes: &[TreeNode], ancestor: &str, id: &str) -> bool {
    find(nodes, ancestor).is_some_and(|a| find(&a.children, id).is_some())
}

/// How many items can be selected.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SelectionMode {
    /// One at a time. Selecting replaces.
    #[default]
    Single,
    /// Any number. Selecting toggles.
    Multiple,
}

/// Props for [`TreeViewMachine`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TreeViewInput {
    /// The top-level nodes.
    pub nodes: Vec<TreeNode>,
    /// Single or multiple selection.
    pub selection_mode: SelectionMode,
    /// Controlled expanded branches. `Some` at start fixes controlled mode.
    pub expanded: Option<Vec<String>>,
    /// Uncontrolled initially expanded branches.
    pub default_expanded: Vec<String>,
    /// Controlled selection. `Some` at start fixes controlled mode.
    pub selected: Option<Vec<String>>,
    /// Uncontrolled initial selection.
    pub default_selected: Vec<String>,
}

/// What [`TreeViewMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TreeViewState {
    /// Expanded branch ids.
    pub expanded: Vec<String>,
    /// Selected item ids, in the order they were selected.
    pub selected: Vec<String>,
    /// The item that last had focus.
    pub focused: Option<String>,
    /// Fixed at start: the owner holds `expanded`.
    pub expanded_controlled: bool,
    /// Fixed at start: the owner holds `selected`.
    pub selected_controlled: bool,
}

impl TreeViewState {
    /// The row in the tab order: the focused one if visible, else the first.
    #[must_use]
    pub fn tab_stop<'a>(&self, rows: &[TreeRow<'a>]) -> Option<&'a str> {
        let focused = self.focused.as_deref();
        rows.iter().find(|r| Some(r.node.id.as_str()) == focused).or(rows.first()).map(|r| r.node.id.as_str())
    }
}

/// Inputs to [`TreeViewMachine`].
#[derive(Debug, Clone, PartialEq)]
pub enum TreeViewEvent {
    /// An item received focus.
    Focus(String),
    /// Up or Down: the previous or next visible row. Doesn't wrap.
    Move(Direction),
    /// The first visible row.
    Home,
    /// The last visible row.
    End,
    /// Right: expand a closed branch, or step into an open one.
    ExpandOrEnter,
    /// Left: collapse an open branch, or step out to the parent.
    CollapseOrExit,
    /// Open or close this branch (click on its chevron).
    Toggle(String),
    /// Select this item (click, Enter, Space). Toggles in multiple mode.
    Select(String),
    /// `*`: expand every branch beside the focused item.
    ExpandSiblings,
    /// A printable key: focus the next visible row whose label starts with it.
    Typeahead(char),
}

/// Side effects from [`TreeViewMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq)]
pub enum TreeViewCommand {
    /// Move DOM focus to this item.
    FocusItem(String),
    /// The expanded branches changed (or should, when controlled).
    ExpandedChange(Vec<String>),
    /// The selection changed (or should, when controlled).
    SelectedChange(Vec<String>),
}

/// Expansion, selection and roving focus for a tree.
#[derive(Debug, Clone, Copy)]
pub struct TreeViewMachine;

type Step = Transition<TreeViewState, TreeViewCommand>;

fn focus(state: &TreeViewState, id: &str) -> Step {
    let mut next = state.clone();
    next.focused = Some(id.to_string());
    Transition { state: next, commands: vec![TreeViewCommand::FocusItem(id.to_string())] }
}

/// Commit (when uncontrolled) and always request the new expanded set.
fn set_expanded(state: &TreeViewState, nodes: &[TreeNode], expanded: Vec<String>) -> Step {
    let mut next = state.clone();
    if !state.expanded_controlled {
        next.expanded.clone_from(&expanded);
        // Collapsing an ancestor of the focused row hands focus to the nearest visible ancestor.
        if let Some(f) = state.focused.as_deref()
            && let Some(collapsed) =
                state.expanded.iter().find(|id| !expanded.contains(id) && is_descendant(nodes, id, f))
        {
            next.focused = Some(collapsed.clone());
        }
    }
    Transition { state: next, commands: vec![TreeViewCommand::ExpandedChange(expanded)] }
}

/// SELECT: replace in single mode, toggle in multiple. Disabled items refuse.
fn select(state: &TreeViewState, input: &TreeViewInput, id: String) -> Step {
    if find(&input.nodes, &id).is_none_or(|n| n.disabled) {
        return Transition::stay(state.clone());
    }
    let selected = match input.selection_mode {
        SelectionMode::Single if state.selected == [id.clone()] => return Transition::stay(state.clone()),
        SelectionMode::Single => vec![id.clone()],
        SelectionMode::Multiple if state.selected.contains(&id) => {
            state.selected.iter().filter(|s| **s != id).cloned().collect()
        }
        SelectionMode::Multiple => state.selected.iter().cloned().chain([id.clone()]).collect(),
    };
    let mut next = state.clone();
    next.focused = Some(id);
    if !state.selected_controlled {
        next.selected.clone_from(&selected);
    }
    Transition { state: next, commands: vec![TreeViewCommand::SelectedChange(selected)] }
}

impl Machine for TreeViewMachine {
    type Input = TreeViewInput;
    type State = TreeViewState;
    type Event = TreeViewEvent;
    type Command = TreeViewCommand;

    fn initial(input: &TreeViewInput) -> TreeViewState {
        TreeViewState {
            expanded: input.expanded.clone().unwrap_or_else(|| input.default_expanded.clone()),
            selected: input.selected.clone().unwrap_or_else(|| input.default_selected.clone()),
            focused: None,
            expanded_controlled: input.expanded.is_some(),
            selected_controlled: input.selected.is_some(),
        }
    }

    fn transition(state: &TreeViewState, event: TreeViewEvent, input: &TreeViewInput) -> Step {
        let nodes = &input.nodes;
        let rows = visible_rows(nodes, &state.expanded);
        let current = state.tab_stop(&rows).and_then(|id| rows.iter().position(|r| r.node.id == id));
        let stay = || Transition::stay(state.clone());

        match event {
            TreeViewEvent::Focus(id) => {
                if !rows.iter().any(|r| r.node.id == id) {
                    return stay();
                }
                let mut next = state.clone();
                next.focused = Some(id);
                Transition::stay(next)
            }
            TreeViewEvent::Move(direction) => {
                let Some(i) = current else { return stay() };
                let target = match direction {
                    Direction::Prev => i.saturating_sub(1),
                    Direction::Next => (i + 1).min(rows.len() - 1),
                };
                focus(state, &rows[target].node.id)
            }
            TreeViewEvent::Home => rows.first().map_or_else(stay, |r| focus(state, &r.node.id)),
            TreeViewEvent::End => rows.last().map_or_else(stay, |r| focus(state, &r.node.id)),
            TreeViewEvent::ExpandOrEnter => {
                let Some(i) = current else { return stay() };
                let row = &rows[i];
                if !row.node.is_branch() || row.node.disabled {
                    stay()
                } else if row.expanded {
                    focus(state, &rows[i + 1].node.id)
                } else {
                    let mut expanded = state.expanded.clone();
                    expanded.push(row.node.id.clone());
                    let mut step = set_expanded(state, nodes, expanded);
                    // Keyboard focus stays on the branch, which the adapter may not have recorded yet.
                    step.state.focused = Some(row.node.id.clone());
                    step
                }
            }
            TreeViewEvent::CollapseOrExit => {
                let Some(i) = current else { return stay() };
                let row = &rows[i];
                if row.expanded && !row.node.disabled {
                    let expanded = state.expanded.iter().filter(|id| **id != row.node.id).cloned().collect();
                    let mut step = set_expanded(state, nodes, expanded);
                    step.state.focused = Some(row.node.id.clone());
                    step
                } else {
                    row.parent.map_or_else(stay, |p| focus(state, p))
                }
            }
            TreeViewEvent::Toggle(id) => match find(nodes, &id) {
                Some(node) if node.is_branch() && !node.disabled => {
                    let expanded = if state.expanded.contains(&id) {
                        state.expanded.iter().filter(|e| **e != id).cloned().collect()
                    } else {
                        state.expanded.iter().cloned().chain([id]).collect()
                    };
                    set_expanded(state, nodes, expanded)
                }
                _ => stay(),
            },
            TreeViewEvent::Select(id) => select(state, input, id),
            TreeViewEvent::ExpandSiblings => {
                let Some(i) = current else { return stay() };
                let Some(sibs) = siblings(nodes, &rows[i].node.id) else { return stay() };
                let mut expanded = state.expanded.clone();
                for n in sibs.iter().filter(|n| n.is_branch() && !n.disabled) {
                    if !expanded.contains(&n.id) {
                        expanded.push(n.id.clone());
                    }
                }
                if expanded.len() == state.expanded.len() {
                    return stay();
                }
                set_expanded(state, nodes, expanded)
            }
            TreeViewEvent::Typeahead(c) => {
                let start = current.map_or(0, |i| i + 1);
                let matches =
                    |r: &&TreeRow| r.node.label.chars().next().is_some_and(|f| f.to_lowercase().eq(c.to_lowercase()));
                rows.iter()
                    .cycle()
                    .skip(start)
                    .take(rows.len())
                    .find(matches)
                    .map_or_else(stay, |r| focus(state, &r.node.id))
            }
        }
    }

    fn sync(state: &TreeViewState, input: &TreeViewInput) -> TreeViewState {
        let mut next = state.clone();
        if state.expanded_controlled {
            next.expanded = input.expanded.clone().unwrap_or_default();
        }
        if state.selected_controlled {
            next.selected = input.selected.clone().unwrap_or_default();
        }
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // docs
    //   guides (branch)
    //     intro
    //     setup
    //   api (branch)
    //     client
    // readme
    // drafts (disabled branch)
    //   wip
    fn input() -> TreeViewInput {
        TreeViewInput {
            nodes: vec![
                TreeNode::new("docs", "Docs").children(vec![
                    TreeNode::new("guides", "Guides")
                        .children(vec![TreeNode::new("intro", "Intro"), TreeNode::new("setup", "Setup")]),
                    TreeNode::new("api", "API").children(vec![TreeNode::new("client", "Client")]),
                ]),
                TreeNode::new("readme", "Readme"),
                TreeNode::new("drafts", "Drafts").children(vec![TreeNode::new("wip", "WIP")]).disabled(),
            ],
            ..TreeViewInput::default()
        }
    }

    fn run(state: &TreeViewState, event: TreeViewEvent, input: &TreeViewInput) -> Step {
        TreeViewMachine::transition(state, event, input)
    }

    fn at(id: &str, expanded: &[&str]) -> TreeViewState {
        TreeViewState {
            focused: Some(id.into()),
            expanded: expanded.iter().map(ToString::to_string).collect(),
            ..TreeViewState::default()
        }
    }

    fn ids(rows: &[TreeRow]) -> Vec<String> {
        rows.iter().map(|r| r.node.id.clone()).collect()
    }

    #[test]
    fn flattens_only_expanded_branches_with_aria_positions() {
        let input = input();
        assert_eq!(ids(&visible_rows(&input.nodes, &[])), ["docs", "readme", "drafts"]);
        let rows = visible_rows(&input.nodes, &["docs".into(), "api".into()]);
        assert_eq!(ids(&rows), ["docs", "guides", "api", "client", "readme", "drafts"]);
        let client = &rows[3];
        assert_eq!((client.level, client.set_size, client.pos_in_set, client.parent), (3, 1, 1, Some("api")));
        assert_eq!((rows[2].pos_in_set, rows[2].set_size, rows[2].expanded), (2, 2, true));
        // An expanded leaf id is ignored; a hidden expanded branch stays hidden.
        assert_eq!(ids(&visible_rows(&input.nodes, &["readme".into(), "guides".into()])), ["docs", "readme", "drafts"]);
    }

    #[test]
    fn tab_stop_falls_back_to_first_row() {
        let input = input();
        let rows = visible_rows(&input.nodes, &[]);
        assert_eq!(TreeViewState::default().tab_stop(&rows), Some("docs"));
        assert_eq!(at("intro", &[]).tab_stop(&rows), Some("docs"));
        assert_eq!(at("readme", &[]).tab_stop(&rows), Some("readme"));
    }

    #[test]
    fn up_down_walk_visible_rows_without_wrapping() {
        let input = input();
        let step = run(&at("docs", &["docs"]), TreeViewEvent::Move(Direction::Next), &input);
        assert_eq!(step.commands, [TreeViewCommand::FocusItem("guides".into())]);
        let step = run(&at("drafts", &[]), TreeViewEvent::Move(Direction::Next), &input);
        assert_eq!(step.state.focused.as_deref(), Some("drafts"));
        assert_eq!(run(&at("readme", &[]), TreeViewEvent::End, &input).state.focused.as_deref(), Some("drafts"));
    }

    #[test]
    fn right_expands_then_enters() {
        let input = input();
        let step = run(&at("docs", &[]), TreeViewEvent::ExpandOrEnter, &input);
        assert_eq!(step.state.expanded, ["docs"]);
        assert_eq!(step.commands, [TreeViewCommand::ExpandedChange(vec!["docs".into()])]);
        let step = run(&step.state, TreeViewEvent::ExpandOrEnter, &input);
        assert_eq!(step.state.focused.as_deref(), Some("guides"));
        // Leaves and disabled branches do nothing.
        assert!(run(&at("readme", &[]), TreeViewEvent::ExpandOrEnter, &input).commands.is_empty());
        assert!(run(&at("drafts", &[]), TreeViewEvent::ExpandOrEnter, &input).commands.is_empty());
    }

    #[test]
    fn left_collapses_then_exits_to_parent() {
        let input = input();
        let step = run(&at("intro", &["docs", "guides"]), TreeViewEvent::CollapseOrExit, &input);
        assert_eq!(step.state.focused.as_deref(), Some("guides"));
        let step = run(&step.state, TreeViewEvent::CollapseOrExit, &input);
        assert_eq!(step.state.expanded, ["docs"]);
        assert_eq!(step.state.focused.as_deref(), Some("guides"));
        assert!(run(&at("docs", &[]), TreeViewEvent::CollapseOrExit, &input).commands.is_empty());
    }

    #[test]
    fn collapsing_an_ancestor_moves_focus_to_it() {
        let input = input();
        let step = run(&at("intro", &["docs", "guides"]), TreeViewEvent::Toggle("docs".into()), &input);
        assert_eq!(step.state.expanded, ["guides"]);
        assert_eq!(step.state.focused.as_deref(), Some("docs"));
    }

    #[test]
    fn star_expands_sibling_branches() {
        let input = input();
        let step = run(&at("guides", &["docs"]), TreeViewEvent::ExpandSiblings, &input);
        assert_eq!(step.state.expanded, ["docs", "guides", "api"]);
        // Top level: docs is the only enabled branch.
        let step = run(&at("readme", &[]), TreeViewEvent::ExpandSiblings, &input);
        assert_eq!(step.state.expanded, ["docs"]);
        assert!(run(&step.state, TreeViewEvent::ExpandSiblings, &input).commands.is_empty());
    }

    #[test]
    fn typeahead_cycles_from_after_focus() {
        let input = input();
        let rows = &["docs"];
        assert_eq!(
            run(&at("docs", rows), TreeViewEvent::Typeahead('r'), &input).state.focused.as_deref(),
            Some("readme")
        );
        assert_eq!(
            run(&at("drafts", rows), TreeViewEvent::Typeahead('D'), &input).state.focused.as_deref(),
            Some("docs")
        );
        assert_eq!(
            run(&at("docs", rows), TreeViewEvent::Typeahead('d'), &input).state.focused.as_deref(),
            Some("drafts")
        );
        assert!(run(&at("docs", rows), TreeViewEvent::Typeahead('z'), &input).commands.is_empty());
    }

    #[test]
    fn single_select_replaces_multiple_toggles() {
        let mut input = input();
        let s = run(&TreeViewState::default(), TreeViewEvent::Select("readme".into()), &input).state;
        let step = run(&s, TreeViewEvent::Select("intro".into()), &input);
        assert_eq!(step.state.selected, ["intro"]);
        assert!(run(&step.state, TreeViewEvent::Select("intro".into()), &input).commands.is_empty());
        assert!(run(&s, TreeViewEvent::Select("drafts".into()), &input).commands.is_empty());

        input.selection_mode = SelectionMode::Multiple;
        let s = run(&s, TreeViewEvent::Select("intro".into()), &input).state;
        assert_eq!(s.selected, ["readme", "intro"]);
        assert_eq!(run(&s, TreeViewEvent::Select("readme".into()), &input).state.selected, ["intro"]);
    }

    #[test]
    fn controlled_requests_but_waits_for_sync() {
        let mut input = input();
        input.expanded = Some(vec![]);
        input.selected = Some(vec![]);
        let state = TreeViewMachine::initial(&input);
        let step = run(&state, TreeViewEvent::Toggle("docs".into()), &input);
        assert!(step.state.expanded.is_empty());
        assert_eq!(step.commands, [TreeViewCommand::ExpandedChange(vec!["docs".into()])]);
        let step = run(&state, TreeViewEvent::Select("readme".into()), &input);
        assert!(step.state.selected.is_empty());
        input.expanded = Some(vec!["docs".into()]);
        input.selected = Some(vec!["readme".into()]);
        let synced = TreeViewMachine::sync(&step.state, &input);
        assert_eq!((synced.expanded, synced.selected), (vec!["docs".to_string()], vec!["readme".to_string()]));
    }
}
