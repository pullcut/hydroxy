//! Dock layout: an IDE-style workspace. A tree of splits whose leaves are tab
//! groups. Tabs activate, close, reorder and move between groups, by pointer
//! drag or the keyboard. Split sizes belong to the resizable machine.
//!
//! The tree maths ([`DockNode::moved`], [`DockNode::closed`]) is pure and
//! prunes what it empties: an empty group disappears and a split left with one
//! child becomes that child.

use hydroxy_statechart::{Machine, Transition};

/// Which way a split lays its children out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SplitAxis {
    /// Side by side.
    #[default]
    Horizontal,
    /// Stacked.
    Vertical,
}

/// A group of tabs and which one shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockGroup {
    /// Unique key.
    pub id: String,
    /// Panel ids, in tab order.
    pub panels: Vec<String>,
    /// The panel showing. Falls back to the first.
    pub active: Option<String>,
}

impl DockGroup {
    /// A group showing its first panel.
    #[must_use]
    pub fn new(id: impl Into<String>, panels: &[&str]) -> Self {
        Self { id: id.into(), panels: panels.iter().map(|p| (*p).to_string()).collect(), active: None }
    }

    /// The panel that shows: `active` if it is in the group, else the first.
    #[must_use]
    pub fn shown(&self) -> Option<&str> {
        self.active.as_deref().filter(|a| self.panels.iter().any(|p| p == a)).or_else(|| self.panels.first().map(String::as_str))
    }

    fn drop_panel(&mut self, panel: &str) {
        let Some(at) = self.panels.iter().position(|p| p == panel) else { return };
        let was_shown = self.shown() == Some(panel);
        self.panels.remove(at);
        if was_shown {
            self.active = self.panels.get(at).or_else(|| self.panels.last()).cloned();
        }
    }
}

/// A node of the dock tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockNode {
    /// Children laid out along an axis, each in a resizable panel.
    Split {
        /// Unique key.
        id: String,
        /// Layout direction.
        axis: SplitAxis,
        /// Two or more children.
        children: Vec<DockNode>,
    },
    /// A leaf: one tab group.
    Tabs(DockGroup),
}

impl DockNode {
    /// A split of `children`.
    #[must_use]
    pub fn split(id: impl Into<String>, axis: SplitAxis, children: Vec<DockNode>) -> Self {
        Self::Split { id: id.into(), axis, children }
    }

    /// A leaf with the given panels.
    #[must_use]
    pub fn tabs(id: impl Into<String>, panels: &[&str]) -> Self {
        Self::Tabs(DockGroup::new(id, panels))
    }

    /// Every tab group, in reading order.
    #[must_use]
    pub fn groups(&self) -> Vec<&DockGroup> {
        match self {
            Self::Tabs(g) => vec![g],
            Self::Split { children, .. } => children.iter().flat_map(Self::groups).collect(),
        }
    }

    /// The group holding `panel`.
    #[must_use]
    pub fn group_of(&self, panel: &str) -> Option<&DockGroup> {
        self.groups().into_iter().find(|g| g.panels.iter().any(|p| p == panel))
    }

    /// The tree's shape alone: splits and group ids, with every group's tabs cleared.
    /// Adapters key their structure on this, so moving a tab does not rebuild the workspace.
    #[must_use]
    pub fn skeleton(&self) -> Self {
        match self {
            Self::Tabs(g) => Self::Tabs(DockGroup { id: g.id.clone(), panels: Vec::new(), active: None }),
            Self::Split { id, axis, children } => {
                Self::Split { id: id.clone(), axis: *axis, children: children.iter().map(Self::skeleton).collect() }
            }
        }
    }

    fn group_mut(&mut self, id: &str) -> Option<&mut DockGroup> {
        match self {
            Self::Tabs(g) => (g.id == id).then_some(g),
            Self::Split { children, .. } => children.iter_mut().find_map(|c| c.group_mut(id)),
        }
    }

    /// The tree without empty groups or one-child splits. `None` if nothing is left.
    fn pruned(self) -> Option<Self> {
        match self {
            Self::Tabs(g) => (!g.panels.is_empty()).then_some(Self::Tabs(g)),
            Self::Split { id, axis, children } => {
                let mut kept: Vec<Self> = children.into_iter().filter_map(Self::pruned).collect();
                match kept.len() {
                    0 => None,
                    1 => kept.pop(),
                    _ => Some(Self::Split { id, axis, children: kept }),
                }
            }
        }
    }

    /// Prune, keeping an empty first group when the whole tree empties.
    fn settled(self) -> Self {
        let first = self.groups().first().map(|g| g.id.clone()).unwrap_or_default();
        self.pruned().unwrap_or_else(|| Self::Tabs(DockGroup::new(first, &[])))
    }

    /// Make `panel` the shown tab of its group. `None` if it already is, or is unknown.
    #[must_use]
    pub fn activated(&self, panel: &str) -> Option<Self> {
        let group = self.group_of(panel)?;
        if group.shown() == Some(panel) {
            return None;
        }
        let mut next = self.clone();
        next.group_mut(&group.id)?.active = Some(panel.to_string());
        Some(next)
    }

    /// Move `panel` into group `to`, before panel `before` or at the end, and
    /// show it there. `None` if that changes nothing.
    #[must_use]
    pub fn moved(&self, panel: &str, to: &str, before: Option<&str>) -> Option<Self> {
        let from = self.group_of(panel)?.id.clone();
        if before == Some(panel) || !self.groups().iter().any(|g| g.id == to) {
            return None;
        }
        let mut next = self.clone();
        next.group_mut(&from)?.drop_panel(panel);
        let dest = next.group_mut(to)?;
        let at = before.and_then(|b| dest.panels.iter().position(|p| p == b)).unwrap_or(dest.panels.len());
        dest.panels.insert(at, panel.to_string());
        dest.active = Some(panel.to_string());
        let next = next.settled();
        // Landing in its own slot only changes which tab shows: not a move.
        (next.layout() != self.layout()).then_some(next)
    }

    fn layout(&self) -> Vec<(&str, &[String])> {
        self.groups().into_iter().map(|g| (g.id.as_str(), g.panels.as_slice())).collect()
    }

    /// Remove `panel`, pruning what it empties. `None` if it is unknown.
    #[must_use]
    pub fn closed(&self, panel: &str) -> Option<Self> {
        let id = self.group_of(panel)?.id.clone();
        let mut next = self.clone();
        next.group_mut(&id)?.drop_panel(panel);
        Some(next.settled())
    }
}

/// A tab in flight and where a drop would land.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drag {
    /// The panel being dragged.
    pub panel: String,
    /// The group a drop would land in.
    pub group: String,
    /// Land before this panel, or at the end when `None`.
    pub before: Option<String>,
}

/// Props for [`DockLayoutMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockInput {
    /// Controlled tree. `Some` at start fixes controlled mode.
    pub tree: Option<DockNode>,
    /// Uncontrolled initial tree.
    pub default_tree: DockNode,
    /// Panels that cannot be closed.
    pub locked: Vec<String>,
}

impl Default for DockInput {
    fn default() -> Self {
        Self { tree: None, default_tree: DockNode::tabs("main", &[]), locked: Vec::new() }
    }
}

/// What [`DockLayoutMachine`] remembers between events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockState {
    /// The committed tree.
    pub tree: DockNode,
    /// The tab being dragged.
    pub drag: Option<Drag>,
    /// Fixed at start: the owner holds the tree.
    pub controlled: bool,
}

/// Inputs to [`DockLayoutMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockEvent {
    /// Click, or an arrow key, on a tab: show it.
    Activate(String),
    /// Delete on a tab, or its close button.
    Close(String),
    /// Alt+Left or Alt+Right: move the tab this many places in its group.
    Reorder {
        /// The tab.
        panel: String,
        /// Places to move: negative is earlier.
        delta: i32,
    },
    /// Alt+Up or Alt+Down: send the tab to the previous or next group, at the end.
    SendToGroup {
        /// The tab.
        panel: String,
        /// Groups to skip: negative is earlier in reading order.
        delta: i32,
    },
    /// A pointer drag started on this tab.
    DragStart(String),
    /// The drag is over `group`, before tab `before` or at the end.
    DragOver {
        /// The group under the pointer.
        group: String,
        /// The tab under the pointer. `None` means after the last tab.
        before: Option<String>,
    },
    /// A pointer drop.
    Drop,
    /// The drag ended without a drop.
    Cancel,
}

/// What a screen reader should hear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Announcement {
    /// The tab sits at `position` of `count` in group `group` of `groups`.
    Moved {
        /// The panel.
        panel: String,
        /// 1-based group number in reading order.
        group: usize,
        /// Groups in the workspace.
        groups: usize,
        /// 1-based position in the group.
        position: usize,
        /// Tabs in the group.
        count: usize,
    },
}

/// Side effects from [`DockLayoutMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockCommand {
    /// The tree changed (or should, when controlled).
    LayoutChange(DockNode),
    /// A panel was closed.
    Closed(String),
    /// Move DOM focus to this panel's tab.
    FocusTab(String),
    /// Say this in the live region.
    Announce(Announcement),
}

/// Tree and drag state for a dock workspace.
#[derive(Debug, Clone, Copy)]
pub struct DockLayoutMachine;

type Step = Transition<DockState, DockCommand>;

fn announce(tree: &DockNode, panel: &str) -> Option<DockCommand> {
    let groups = tree.groups();
    let at = groups.iter().position(|g| g.panels.iter().any(|p| p == panel))?;
    let g = groups[at];
    Some(DockCommand::Announce(Announcement::Moved {
        panel: panel.to_string(),
        group: at + 1,
        groups: groups.len(),
        position: g.panels.iter().position(|p| p == panel)? + 1,
        count: g.panels.len(),
    }))
}

/// Commit `tree` as the new layout.
fn commit(state: &DockState, tree: DockNode, mut commands: Vec<DockCommand>) -> Step {
    let mut next = state.clone();
    next.drag = None;
    if !state.controlled {
        next.tree = tree.clone();
    }
    commands.insert(0, DockCommand::LayoutChange(tree));
    Transition::with(next, commands)
}

fn offset(len: usize, from: usize, delta: i32) -> usize {
    let to = i64::try_from(from).unwrap_or(0) + i64::from(delta);
    usize::try_from(to.max(0)).unwrap_or(0).min(len.saturating_sub(1))
}

impl Machine for DockLayoutMachine {
    type Input = DockInput;
    type State = DockState;
    type Event = DockEvent;
    type Command = DockCommand;

    fn initial(input: &DockInput) -> DockState {
        DockState {
            tree: input.tree.clone().unwrap_or_else(|| input.default_tree.clone()),
            drag: None,
            controlled: input.tree.is_some(),
        }
    }

    fn transition(state: &DockState, event: DockEvent, input: &DockInput) -> Step {
        let stay = || Transition::stay();
        let tree = &state.tree;
        match event {
            DockEvent::Activate(panel) => {
                let focus = DockCommand::FocusTab(panel.clone());
                match tree.activated(&panel) {
                    Some(next) => commit(state, next, vec![focus]),
                    None if tree.group_of(&panel).is_some() => Transition::emit(vec![focus]),
                    None => stay(),
                }
            }
            DockEvent::Close(panel) => {
                if input.locked.contains(&panel) {
                    return stay();
                }
                let Some(next) = tree.closed(&panel) else { return stay() };
                // The tab that takes over keeps keyboard focus where it was.
                let heir = tree.group_of(&panel).and_then(|g| {
                    let at = g.panels.iter().position(|p| *p == panel)?;
                    g.panels.get(at + 1).or_else(|| at.checked_sub(1).and_then(|i| g.panels.get(i))).cloned()
                });
                let mut commands = vec![DockCommand::Closed(panel)];
                commands.extend(heir.map(DockCommand::FocusTab));
                commit(state, next, commands)
            }
            DockEvent::Reorder { panel, delta } => {
                let Some(g) = tree.group_of(&panel) else { return stay() };
                let Some(from) = g.panels.iter().position(|p| *p == panel) else { return stay() };
                let to = offset(g.panels.len(), from, delta);
                if to == from {
                    return stay();
                }
                // `before` is the tab currently at the destination slot, seen with the panel removed.
                let rest: Vec<&String> = g.panels.iter().filter(|p| **p != panel).collect();
                let before = rest.get(to).map(|p| (*p).clone());
                match tree.moved(&panel, &g.id.clone(), before.as_deref()) {
                    Some(next) => {
                        let mut commands = vec![DockCommand::FocusTab(panel.clone())];
                        commands.extend(announce(&next, &panel));
                        commit(state, next, commands)
                    }
                    None => stay(),
                }
            }
            DockEvent::SendToGroup { panel, delta } => {
                let groups = tree.groups();
                let Some(from) = groups.iter().position(|g| g.panels.contains(&panel)) else { return stay() };
                let to = offset(groups.len(), from, delta);
                if to == from {
                    return stay();
                }
                let dest = groups[to].id.clone();
                match tree.moved(&panel, &dest, None) {
                    Some(next) => {
                        let mut commands = vec![DockCommand::FocusTab(panel.clone())];
                        commands.extend(announce(&next, &panel));
                        commit(state, next, commands)
                    }
                    None => stay(),
                }
            }
            DockEvent::DragStart(panel) => {
                let Some(g) = tree.group_of(&panel) else { return stay() };
                let mut next = state.clone();
                next.drag = Some(Drag { panel, group: g.id.clone(), before: None });
                Transition::to(next)
            }
            DockEvent::DragOver { group, before } => match &state.drag {
                Some(d) if tree.groups().iter().any(|g| g.id == group) => {
                    let over = Drag { panel: d.panel.clone(), group, before };
                    if over == *d {
                        return stay();
                    }
                    let mut next = state.clone();
                    next.drag = Some(over);
                    Transition::to(next)
                }
                _ => stay(),
            },
            DockEvent::Drop => {
                let Some(d) = &state.drag else { return stay() };
                if let Some(next) = tree.moved(&d.panel, &d.group, d.before.as_deref()) {
                    let mut commands = vec![DockCommand::FocusTab(d.panel.clone())];
                    commands.extend(announce(&next, &d.panel));
                    commit(state, next, commands)
                } else {
                    let mut next = state.clone();
                    next.drag = None;
                    Transition::to(next)
                }
            }
            DockEvent::Cancel => {
                let mut next = state.clone();
                next.drag = None;
                Transition::to(next)
            }
        }
    }

    fn sync(state: &DockState, input: &DockInput) -> Option<DockState> {
        let mut next = state.clone();
        if state.controlled {
            next.tree = input.tree.clone().unwrap_or_else(|| input.default_tree.clone());
        }
        if next.drag.as_ref().is_some_and(|d| next.tree.group_of(&d.panel).is_none()) {
            next.drag = None;
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> DockNode {
        DockNode::split(
            "root",
            SplitAxis::Horizontal,
            vec![
                DockNode::tabs("left", &["files", "search"]),
                DockNode::split(
                    "right",
                    SplitAxis::Vertical,
                    vec![DockNode::tabs("editor", &["a.rs", "b.rs", "c.rs"]), DockNode::tabs("bottom", &["terminal"])],
                ),
            ],
        )
    }

    fn input() -> DockInput {
        DockInput { default_tree: tree(), ..DockInput::default() }
    }

    fn panels(t: &DockNode) -> Vec<(String, Vec<String>)> {
        t.groups().into_iter().map(|g| (g.id.clone(), g.panels.clone())).collect()
    }

    fn run(state: &DockState, event: DockEvent) -> hydroxy_statechart::Step<DockState, DockCommand> {
        DockLayoutMachine::transition(state, event, &input()).resolve(state)
    }

    #[test]
    fn closing_the_last_tab_prunes_its_group_and_collapses_the_split() {
        let t = tree().closed("terminal").unwrap();
        assert_eq!(panels(&t).len(), 2);
        let DockNode::Split { children, .. } = &t else { panic!("root should stay a split") };
        assert!(matches!(&children[1], DockNode::Tabs(g) if g.id == "editor"));
    }

    #[test]
    fn closing_the_shown_tab_shows_its_neighbour() {
        let t = tree().activated("b.rs").unwrap().closed("b.rs").unwrap();
        assert_eq!(t.group_of("c.rs").unwrap().shown(), Some("c.rs"));
        let t = t.activated("c.rs");
        assert!(t.is_none(), "already shown");
    }

    #[test]
    fn closing_everything_keeps_an_empty_group() {
        let t = DockNode::tabs("only", &["x"]).closed("x").unwrap();
        assert_eq!(panels(&t), [("only".to_string(), vec![])]);
    }

    #[test]
    fn move_reorders_and_crosses_groups() {
        let t = tree().moved("c.rs", "editor", Some("a.rs")).unwrap();
        assert_eq!(t.group_of("a.rs").unwrap().panels, ["c.rs", "a.rs", "b.rs"]);
        let t = tree().moved("search", "bottom", None).unwrap();
        assert_eq!(t.group_of("terminal").unwrap().panels, ["terminal", "search"]);
        assert_eq!(t.group_of("search").unwrap().shown(), Some("search"));
        assert!(tree().moved("a.rs", "editor", Some("a.rs")).is_none());
        assert!(tree().moved("b.rs", "editor", Some("c.rs")).is_none(), "landing in its own slot is a no-op");
        assert!(tree().moved("a.rs", "nope", None).is_none());
    }

    #[test]
    fn moving_the_last_tab_out_removes_the_group() {
        let t = tree().moved("terminal", "left", None).unwrap();
        assert_eq!(panels(&t).len(), 2);
    }

    #[test]
    fn keyboard_reorder_and_send() {
        let s0 = DockLayoutMachine::initial(&input());
        let step = run(&s0, DockEvent::Reorder { panel: "a.rs".into(), delta: 1 });
        assert_eq!(step.state.tree.group_of("a.rs").unwrap().panels, ["b.rs", "a.rs", "c.rs"]);
        assert!(step.commands.contains(&DockCommand::FocusTab("a.rs".into())));
        assert!(run(&s0, DockEvent::Reorder { panel: "a.rs".into(), delta: -1 }).commands.is_empty());
        let sent = run(&s0, DockEvent::SendToGroup { panel: "files".into(), delta: 1 });
        assert_eq!(sent.state.tree.group_of("files").unwrap().id, "editor");
    }

    #[test]
    fn pointer_drag_lands_where_the_pointer_was() {
        let s0 = DockLayoutMachine::initial(&input());
        let s = run(&s0, DockEvent::DragStart("terminal".into())).state;
        let s = run(&s, DockEvent::DragOver { group: "editor".into(), before: Some("b.rs".into()) }).state;
        let done = run(&s, DockEvent::Drop);
        assert_eq!(done.state.tree.group_of("terminal").unwrap().panels, ["a.rs", "terminal", "b.rs", "c.rs"]);
        assert!(done.state.drag.is_none());
        let cancelled = run(&run(&s0, DockEvent::DragStart("a.rs".into())).state, DockEvent::Cancel);
        assert!(cancelled.state.drag.is_none() && cancelled.state.tree == tree());
    }

    #[test]
    fn locked_panels_cannot_close_and_controlled_waits_for_the_owner() {
        let inp = DockInput { tree: Some(tree()), locked: vec!["files".into()], ..DockInput::default() };
        let s0 = DockLayoutMachine::initial(&inp);
        assert!(DockLayoutMachine::transition(&s0, DockEvent::Close("files".into()), &inp).resolve(&s0).commands.is_empty());
        let step = DockLayoutMachine::transition(&s0, DockEvent::Close("a.rs".into()), &inp).resolve(&s0);
        assert_eq!(step.state.tree, tree());
        assert!(matches!(&step.commands[0], DockCommand::LayoutChange(_)));
    }

    #[test]
    fn skeleton_ignores_tab_changes() {
        let moved = tree().moved("c.rs", "bottom", None).unwrap();
        assert_eq!(moved.skeleton(), tree().skeleton());
        assert_ne!(tree().closed("terminal").unwrap().skeleton(), tree().skeleton());
    }

    #[test]
    fn activate_changes_the_shown_tab() {
        let s0 = DockLayoutMachine::initial(&input());
        let step = run(&s0, DockEvent::Activate("search".into()));
        assert_eq!(step.state.tree.group_of("search").unwrap().shown(), Some("search"));
    }
}
