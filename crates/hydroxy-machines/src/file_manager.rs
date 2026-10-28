//! File manager owns which folder is open, the selection, the focused item,
//! the rename in progress and the list/grid choice. The items themselves
//! belong to the owner: renames and deletes are reported as commands, and the
//! owner applies them (see [`apply_rename`] and [`apply_delete`]) and passes
//! the new list back in.

use hydroxy_statechart::{Machine, Transition};

/// The id of the top-level folder. It has no item of its own.
pub const ROOT: &str = "";

/// A folder or a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    /// Holds other items.
    Folder,
    /// A leaf.
    File,
}

/// One entry. Items form a tree through `parent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileItem {
    /// Unique key. Never [`ROOT`].
    pub id: String,
    /// Shown name.
    pub name: String,
    /// Folder or file.
    pub kind: ItemKind,
    /// The folder it sits in. [`ROOT`] for the top level.
    pub parent: String,
    /// Size in bytes, for files.
    pub size: Option<u64>,
    /// MIME type, for files. May be empty.
    pub mime: String,
    /// Last-changed text such as `"Oct 12"`. Shown as is.
    pub modified: String,
}

impl FileItem {
    /// A folder inside `parent`.
    #[must_use]
    pub fn folder(id: impl Into<String>, name: impl Into<String>, parent: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind: ItemKind::Folder,
            parent: parent.into(),
            size: None,
            mime: String::new(),
            modified: String::new(),
        }
    }

    /// A file of `size` bytes inside `parent`.
    #[must_use]
    pub fn file(id: impl Into<String>, name: impl Into<String>, parent: impl Into<String>, size: u64) -> Self {
        Self { kind: ItemKind::File, size: Some(size), ..Self::folder(id, name, parent) }
    }

    /// Set the last-changed text.
    #[must_use]
    pub fn modified(mut self, text: impl Into<String>) -> Self {
        self.modified = text.into();
        self
    }

    /// Whether this is a folder.
    #[must_use]
    pub fn is_folder(&self) -> bool {
        self.kind == ItemKind::Folder
    }
}

/// The items inside `folder`: folders first, then by name, ignoring case.
#[must_use]
pub fn visible(items: &[FileItem], folder: &str) -> Vec<FileItem> {
    let mut list: Vec<FileItem> = items.iter().filter(|i| i.parent == folder).cloned().collect();
    list.sort_by_key(|i| (!i.is_folder(), i.name.to_lowercase()));
    list
}

/// The folders from the top down to `folder`, not counting the root. Empty at the root.
#[must_use]
pub fn path(items: &[FileItem], folder: &str) -> Vec<FileItem> {
    let mut out = Vec::new();
    let mut at = folder.to_string();
    // The length bound stops a cycle in bad data.
    while at != ROOT && out.len() <= items.len() {
        let Some(item) = items.iter().find(|i| i.id == at && i.is_folder()) else { break };
        out.push(item.clone());
        at.clone_from(&item.parent);
    }
    out.reverse();
    out
}

/// Whether `id` is a folder in `items`. The root counts.
#[must_use]
pub fn is_folder(items: &[FileItem], id: &str) -> bool {
    id == ROOT || items.iter().any(|i| i.id == id && i.is_folder())
}

/// Rename `id` in `items`.
pub fn apply_rename(items: &mut [FileItem], id: &str, name: &str) {
    if let Some(item) = items.iter_mut().find(|i| i.id == id) {
        name.clone_into(&mut item.name);
    }
}

/// Remove `ids` from `items`, and everything inside any folder among them.
pub fn apply_delete(items: &mut Vec<FileItem>, ids: &[String]) {
    let mut gone: Vec<String> = ids.to_vec();
    loop {
        let before = gone.len();
        for item in items.iter() {
            if gone.contains(&item.parent) && !gone.contains(&item.id) {
                gone.push(item.id.clone());
            }
        }
        if gone.len() == before {
            break;
        }
    }
    items.retain(|i| !gone.contains(&i.id));
}

/// List rows or a grid of tiles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ViewMode {
    /// One row per item, with details.
    #[default]
    List,
    /// Tiles.
    Grid,
}

impl ViewMode {
    /// The `data-view` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ViewMode::List => "list",
            ViewMode::Grid => "grid",
        }
    }
}

/// Props for [`FileManagerMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileManagerInput {
    /// Everything in the tree, in any order.
    pub items: Vec<FileItem>,
    /// Controlled open folder. `Some` at start fixes controlled mode. [`ROOT`] is the top.
    pub folder: Option<String>,
    /// Uncontrolled initial folder.
    pub default_folder: String,
    /// Initial list or grid.
    pub default_view: ViewMode,
}

/// Why a rename was turned down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameError {
    /// Nothing but spaces.
    Empty,
    /// Another item in the folder has the name.
    Taken,
}

impl NameError {
    /// A short sentence for the user.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            NameError::Empty => "Name can't be empty",
            NameError::Taken => "That name is already used here",
        }
    }
}

/// A rename in progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rename {
    /// The item.
    pub id: String,
    /// The text in the field.
    pub draft: String,
    /// Why the last commit failed.
    pub error: Option<NameError>,
}

/// What [`FileManagerMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileManagerState {
    /// The open folder.
    pub folder: String,
    /// Selected ids, in the order they were picked.
    pub selection: Vec<String>,
    /// Where a shift-click range starts.
    pub anchor: Option<String>,
    /// The item that has (or last had) focus. The roving tab stop.
    pub focused: Option<String>,
    /// The rename in progress.
    pub renaming: Option<Rename>,
    /// List or grid.
    pub view: ViewMode,
    /// Fixed at start: the owner holds the folder.
    pub controlled_folder: bool,
}

impl FileManagerState {
    /// The item to give `tabindex="0"`: the focused one, else the first.
    #[must_use]
    pub fn tab_stop<'a>(&self, shown: &'a [FileItem]) -> Option<&'a FileItem> {
        self.focused.as_deref().and_then(|id| shown.iter().find(|i| i.id == id)).or_else(|| shown.first())
    }
}

/// How a click combines with the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extend {
    /// Replace the selection.
    Replace,
    /// Add or remove just this item (Ctrl or ⌘).
    Toggle,
    /// Select everything between the anchor and this item (Shift).
    Range,
}

/// Where a focus move goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// By this many items, forward or back. A grid's up and down use the row length.
    By(isize),
    /// The first item.
    First,
    /// The last item.
    Last,
}

/// Inputs to [`FileManagerMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileManagerEvent {
    /// Open this folder ([`ROOT`] for the top).
    Open(String),
    /// Open the parent folder.
    Up,
    /// A click on an item.
    Click {
        /// The item.
        id: String,
        /// How it combines with the selection.
        extend: Extend,
    },
    /// Double click or Enter: open a folder, or ask the owner to open a file.
    Activate(String),
    /// Right click on an item: select it unless it already is.
    Context(String),
    /// Focus landed on an item.
    Focus(String),
    /// Arrow, Home or End key.
    Move {
        /// Where to.
        step: Step,
        /// Shift held: extend the selection.
        extend: bool,
    },
    /// Select everything in the folder.
    SelectAll,
    /// Select nothing.
    ClearSelection,
    /// Start renaming this item.
    StartRename(String),
    /// The rename field changed.
    RenameInput(String),
    /// Enter or blur in the rename field.
    CommitRename,
    /// Escape in the rename field.
    CancelRename,
    /// Ask the owner to delete the selection.
    DeleteSelected,
    /// Switch between list and grid.
    SetView(ViewMode),
}

/// Side effects from [`FileManagerMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileManagerCommand {
    /// A different folder opened (or should, when controlled).
    FolderChange(String),
    /// The selection changed.
    SelectionChange(Vec<String>),
    /// Open this file.
    OpenFile(String),
    /// Rename an item. The owner applies it.
    Rename {
        /// The item.
        id: String,
        /// Its new name.
        name: String,
    },
    /// Delete these items. The owner applies it.
    Delete(Vec<String>),
    /// Move DOM focus to this item.
    FocusItem(String),
    /// Move DOM focus to the rename field, and select its text.
    FocusRename,
}

/// Folder, selection and rename state for a file browser.
#[derive(Debug, Clone, Copy)]
pub struct FileManagerMachine;

type Out = Transition<FileManagerState, FileManagerCommand>;

fn index_of(shown: &[FileItem], id: &str) -> Option<usize> {
    shown.iter().position(|i| i.id == id)
}

/// Replace the selection, reporting a change only when it differs.
fn set_selection(next: &mut FileManagerState, ids: Vec<String>, commands: &mut Vec<FileManagerCommand>) {
    if next.selection != ids {
        next.selection.clone_from(&ids);
        commands.push(FileManagerCommand::SelectionChange(ids));
    }
}

/// Go to `folder`: clears everything tied to the old one.
fn go(state: &FileManagerState, folder: &str, commands: &mut Vec<FileManagerCommand>) -> FileManagerState {
    let mut next = state.clone();
    if !state.controlled_folder {
        next.folder = folder.to_string();
    }
    next.anchor = None;
    next.focused = None;
    next.renaming = None;
    commands.push(FileManagerCommand::FolderChange(folder.to_string()));
    if !state.selection.is_empty() {
        next.selection.clear();
        commands.push(FileManagerCommand::SelectionChange(Vec::new()));
    }
    next
}

fn validate(items: &[FileItem], rename: &Rename) -> Result<String, NameError> {
    let name = rename.draft.trim();
    if name.is_empty() {
        return Err(NameError::Empty);
    }
    let Some(item) = items.iter().find(|i| i.id == rename.id) else { return Err(NameError::Empty) };
    let taken =
        items.iter().any(|i| i.parent == item.parent && i.id != item.id && i.name.trim().eq_ignore_ascii_case(name));
    if taken { Err(NameError::Taken) } else { Ok(name.to_string()) }
}

impl Machine for FileManagerMachine {
    type Input = FileManagerInput;
    type State = FileManagerState;
    type Event = FileManagerEvent;
    type Command = FileManagerCommand;

    fn initial(input: &FileManagerInput) -> FileManagerState {
        let wanted = input.folder.clone().unwrap_or_else(|| input.default_folder.clone());
        FileManagerState {
            folder: if is_folder(&input.items, &wanted) { wanted } else { ROOT.to_string() },
            view: input.default_view,
            controlled_folder: input.folder.is_some(),
            ..FileManagerState::default()
        }
    }

    #[allow(clippy::too_many_lines, reason = "one match over the events; splitting it only adds plumbing")]
    fn transition(state: &FileManagerState, event: FileManagerEvent, input: &FileManagerInput) -> Out {
        let stay = || Transition::stay();
        let shown = visible(&input.items, &state.folder);
        let mut next = state.clone();
        let mut commands = Vec::new();
        match event {
            FileManagerEvent::Open(id) => {
                if id == state.folder || !is_folder(&input.items, &id) {
                    return stay();
                }
                next = go(state, &id, &mut commands);
            }
            FileManagerEvent::Up => {
                let Some(parent) = input.items.iter().find(|i| i.id == state.folder).map(|i| i.parent.clone()) else {
                    return stay();
                };
                next = go(state, &parent, &mut commands);
            }
            FileManagerEvent::Click { id, extend } => {
                let Some(at) = index_of(&shown, &id) else { return stay() };
                next.focused = Some(id.clone());
                let ids = match extend {
                    Extend::Replace => vec![id.clone()],
                    Extend::Toggle => {
                        let mut ids = state.selection.clone();
                        if let Some(p) = ids.iter().position(|s| *s == id) {
                            ids.remove(p);
                        } else {
                            ids.push(id.clone());
                        }
                        ids
                    }
                    Extend::Range => {
                        let from = state.anchor.as_deref().and_then(|a| index_of(&shown, a)).unwrap_or(at);
                        shown[from.min(at)..=from.max(at)].iter().map(|i| i.id.clone()).collect()
                    }
                };
                if extend != Extend::Range {
                    next.anchor = Some(id);
                }
                set_selection(&mut next, ids, &mut commands);
            }
            FileManagerEvent::Activate(id) => {
                let Some(item) = shown.iter().find(|i| i.id == id) else { return stay() };
                if item.is_folder() {
                    next = go(state, &id, &mut commands);
                } else {
                    commands.push(FileManagerCommand::OpenFile(id));
                }
            }
            FileManagerEvent::Context(id) => {
                if index_of(&shown, &id).is_none() {
                    return stay();
                }
                next.focused = Some(id.clone());
                if !state.selection.contains(&id) {
                    next.anchor = Some(id.clone());
                    set_selection(&mut next, vec![id], &mut commands);
                }
            }
            FileManagerEvent::Focus(id) => {
                if index_of(&shown, &id).is_none() || state.focused.as_deref() == Some(id.as_str()) {
                    return stay();
                }
                next.focused = Some(id);
            }
            FileManagerEvent::Move { step, extend } => {
                if shown.is_empty() {
                    return stay();
                }
                let current = state.focused.as_deref().and_then(|id| index_of(&shown, id));
                let last = shown.len() - 1;
                let target = match (step, current) {
                    (Step::First, _) => 0,
                    (Step::Last, _) => last,
                    (Step::By(n), None) => {
                        if n < 0 {
                            last
                        } else {
                            0
                        }
                    }
                    (Step::By(n), Some(c)) => c.saturating_add_signed(n).min(last),
                };
                let id = shown[target].id.clone();
                next.focused = Some(id.clone());
                if extend {
                    let from = state.anchor.as_deref().and_then(|a| index_of(&shown, a)).or(current).unwrap_or(target);
                    if state.anchor.is_none() {
                        next.anchor = Some(shown[from].id.clone());
                    }
                    let ids = shown[from.min(target)..=from.max(target)].iter().map(|i| i.id.clone()).collect();
                    set_selection(&mut next, ids, &mut commands);
                } else {
                    next.anchor = Some(id.clone());
                    set_selection(&mut next, vec![id.clone()], &mut commands);
                }
                if next == *state {
                    return stay();
                }
                commands.push(FileManagerCommand::FocusItem(id));
            }
            FileManagerEvent::SelectAll => {
                let ids: Vec<String> = shown.iter().map(|i| i.id.clone()).collect();
                set_selection(&mut next, ids, &mut commands);
            }
            FileManagerEvent::ClearSelection => set_selection(&mut next, Vec::new(), &mut commands),
            FileManagerEvent::StartRename(id) => {
                let Some(item) = shown.iter().find(|i| i.id == id) else { return stay() };
                next.renaming = Some(Rename { id: id.clone(), draft: item.name.clone(), error: None });
                next.focused = Some(id);
                commands.push(FileManagerCommand::FocusRename);
            }
            FileManagerEvent::RenameInput(text) => {
                let Some(r) = next.renaming.as_mut() else { return stay() };
                r.draft = text;
                r.error = None;
            }
            FileManagerEvent::CommitRename => {
                let Some(r) = state.renaming.clone() else { return stay() };
                let unchanged = input.items.iter().any(|i| i.id == r.id && i.name == r.draft.trim());
                if unchanged {
                    next.renaming = None;
                    commands.push(FileManagerCommand::FocusItem(r.id));
                } else {
                    match validate(&input.items, &r) {
                        Err(error) => {
                            next.renaming = Some(Rename { error: Some(error), ..r });
                            commands.push(FileManagerCommand::FocusRename);
                        }
                        Ok(name) => {
                            next.renaming = None;
                            commands.push(FileManagerCommand::Rename { id: r.id.clone(), name });
                            commands.push(FileManagerCommand::FocusItem(r.id));
                        }
                    }
                }
            }
            FileManagerEvent::CancelRename => {
                let Some(r) = state.renaming.as_ref() else { return stay() };
                commands.push(FileManagerCommand::FocusItem(r.id.clone()));
                next.renaming = None;
            }
            FileManagerEvent::DeleteSelected => {
                if state.selection.is_empty() {
                    return stay();
                }
                commands.push(FileManagerCommand::Delete(state.selection.clone()));
            }
            FileManagerEvent::SetView(view) => {
                if view == state.view {
                    return stay();
                }
                next.view = view;
            }
        }
        Transition::with(next, commands)
    }

    fn sync(state: &FileManagerState, input: &FileManagerInput) -> Option<FileManagerState> {
        let mut next = state.clone();
        if state.controlled_folder {
            next.folder = input.folder.clone().unwrap_or_default();
        }
        if !is_folder(&input.items, &next.folder) {
            next.folder = ROOT.to_string();
        }
        let shown = visible(&input.items, &next.folder);
        let here = |id: &str| index_of(&shown, id).is_some();
        next.selection.retain(|id| here(id));
        next.anchor = next.anchor.filter(|id| here(id));
        next.focused = next.focused.filter(|id| here(id));
        next.renaming = next.renaming.filter(|r| here(&r.id));
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step as advance;

    fn items() -> Vec<FileItem> {
        vec![
            FileItem::folder("docs", "Docs", ROOT),
            FileItem::folder("img", "images", ROOT),
            FileItem::file("a", "b-notes.md", ROOT, 10),
            FileItem::file("b", "A-readme.md", ROOT, 20),
            FileItem::folder("old", "Old", "docs"),
            FileItem::file("c", "spec.pdf", "docs", 30),
        ]
    }

    fn input() -> FileManagerInput {
        FileManagerInput { items: items(), ..FileManagerInput::default() }
    }

    fn click(id: &str, extend: Extend) -> FileManagerEvent {
        FileManagerEvent::Click { id: id.into(), extend }
    }

    fn ids(v: &[FileItem]) -> Vec<&str> {
        v.iter().map(|i| i.id.as_str()).collect()
    }

    #[test]
    fn folders_come_first_then_names_ignoring_case() {
        assert_eq!(ids(&visible(&items(), ROOT)), ["docs", "img", "b", "a"]);
        assert_eq!(ids(&visible(&items(), "docs")), ["old", "c"]);
        assert_eq!(ids(&path(&items(), "old")), ["docs", "old"]);
        assert!(path(&items(), ROOT).is_empty());
    }

    #[test]
    fn open_up_and_activate_move_between_folders() {
        let i = input();
        let s = FileManagerMachine::initial(&i);
        let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Activate("docs".into()), &i);
        assert_eq!(step.state.folder, "docs");
        assert_eq!(step.commands, [FileManagerCommand::FolderChange("docs".into())]);
        let step = advance::<FileManagerMachine>(&step.state, FileManagerEvent::Up, &i);
        assert_eq!(step.state.folder, ROOT);
        assert!(advance::<FileManagerMachine>(&step.state, FileManagerEvent::Up, &i).commands.is_empty());
        assert!(advance::<FileManagerMachine>(&s, FileManagerEvent::Open("a".into()), &i).commands.is_empty());
        let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Activate("a".into()), &i);
        assert_eq!(step.commands, [FileManagerCommand::OpenFile("a".into())]);
    }

    #[test]
    fn leaving_a_folder_clears_the_selection() {
        let i = input();
        let s = advance::<FileManagerMachine>(&FileManagerMachine::initial(&i), click("a", Extend::Replace), &i).state;
        let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Open("docs".into()), &i);
        assert!(step.state.selection.is_empty());
        assert_eq!(step.commands[1], FileManagerCommand::SelectionChange(Vec::new()));
    }

    #[test]
    fn click_replaces_toggles_and_ranges() {
        let i = input();
        let s = FileManagerMachine::initial(&i);
        let s = advance::<FileManagerMachine>(&s, click("img", Extend::Replace), &i).state;
        let s = advance::<FileManagerMachine>(&s, click("a", Extend::Range), &i).state;
        assert_eq!(s.selection, ["img", "b", "a"]);
        let s = advance::<FileManagerMachine>(&s, click("b", Extend::Toggle), &i).state;
        assert_eq!(s.selection, ["img", "a"]);
        let step = advance::<FileManagerMachine>(&s, click("docs", Extend::Replace), &i);
        assert_eq!(step.state.selection, ["docs"]);
        assert_eq!(step.commands, [FileManagerCommand::SelectionChange(vec!["docs".into()])]);
        // Clicking what is already the whole selection reports nothing.
        assert!(advance::<FileManagerMachine>(&step.state, click("docs", Extend::Replace), &i).commands.is_empty());
    }

    #[test]
    fn arrows_move_focus_and_selection() {
        let i = input();
        let s = FileManagerMachine::initial(&i);
        let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Move { step: Step::By(1), extend: false }, &i);
        assert_eq!(step.state.focused.as_deref(), Some("docs"));
        assert_eq!(step.state.selection, ["docs"]);
        let s =
            advance::<FileManagerMachine>(&step.state, FileManagerEvent::Move { step: Step::By(2), extend: true }, &i)
                .state;
        assert_eq!(s.focused.as_deref(), Some("b"));
        assert_eq!(s.selection, ["docs", "img", "b"]);
        let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Move { step: Step::Last, extend: false }, &i);
        assert_eq!(step.state.focused.as_deref(), Some("a"));
        assert_eq!(step.commands.last(), Some(&FileManagerCommand::FocusItem("a".into())));
        // At the end already: nothing happens.
        assert!(
            advance::<FileManagerMachine>(&step.state, FileManagerEvent::Move { step: Step::By(5), extend: false }, &i)
                .commands
                .is_empty()
        );
    }

    #[test]
    fn context_keeps_a_selection_it_is_inside() {
        let i = input();
        let s =
            advance::<FileManagerMachine>(&FileManagerMachine::initial(&i), click("docs", Extend::Replace), &i).state;
        let s = advance::<FileManagerMachine>(&s, click("a", Extend::Range), &i).state;
        assert!(advance::<FileManagerMachine>(&s, FileManagerEvent::Context("img".into()), &i).commands.is_empty());
        let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Context("img".into()), &i);
        assert_eq!(step.state.selection, ["docs", "img", "b", "a"]);
        let step =
            advance::<FileManagerMachine>(&FileManagerMachine::initial(&i), FileManagerEvent::Context("a".into()), &i);
        assert_eq!(step.state.selection, ["a"]);
    }

    #[test]
    fn rename_validates_and_reports_a_change() {
        let i = input();
        let s = FileManagerMachine::initial(&i);
        let s = advance::<FileManagerMachine>(&s, FileManagerEvent::StartRename("a".into()), &i).state;
        assert_eq!(s.renaming.as_ref().map(|r| r.draft.as_str()), Some("b-notes.md"));
        let blank = advance::<FileManagerMachine>(
            &advance::<FileManagerMachine>(&s, FileManagerEvent::RenameInput("  ".into()), &i).state,
            FileManagerEvent::CommitRename,
            &i,
        );
        assert_eq!(blank.state.renaming.as_ref().and_then(|r| r.error), Some(NameError::Empty));
        let taken = advance::<FileManagerMachine>(
            &advance::<FileManagerMachine>(&s, FileManagerEvent::RenameInput("A-README.MD".into()), &i).state,
            FileManagerEvent::CommitRename,
            &i,
        );
        assert_eq!(taken.state.renaming.as_ref().and_then(|r| r.error), Some(NameError::Taken));
        // Typing clears the error.
        assert_eq!(
            advance::<FileManagerMachine>(&taken.state, FileManagerEvent::RenameInput("x".into()), &i)
                .state
                .renaming
                .and_then(|r| r.error),
            None
        );
        let ok = advance::<FileManagerMachine>(
            &advance::<FileManagerMachine>(&s, FileManagerEvent::RenameInput(" notes.md ".into()), &i).state,
            FileManagerEvent::CommitRename,
            &i,
        );
        assert!(ok.state.renaming.is_none());
        assert_eq!(ok.commands[0], FileManagerCommand::Rename { id: "a".into(), name: "notes.md".into() });
        // The same name just closes.
        let same = advance::<FileManagerMachine>(&s, FileManagerEvent::CommitRename, &i);
        assert!(!same.commands.iter().any(|c| matches!(c, FileManagerCommand::Rename { .. })));
        let cancelled = advance::<FileManagerMachine>(&s, FileManagerEvent::CancelRename, &i);
        assert!(cancelled.state.renaming.is_none());
    }

    #[test]
    fn delete_asks_the_owner_and_sync_prunes() {
        let mut i = input();
        let s =
            advance::<FileManagerMachine>(&FileManagerMachine::initial(&i), click("docs", Extend::Replace), &i).state;
        let s = advance::<FileManagerMachine>(&s, click("a", Extend::Toggle), &i).state;
        let step = advance::<FileManagerMachine>(&s, FileManagerEvent::DeleteSelected, &i);
        assert_eq!(step.commands, [FileManagerCommand::Delete(vec!["docs".into(), "a".into()])]);
        assert_eq!(step.state.selection.len(), 2);
        apply_delete(&mut i.items, &["docs".to_string(), "a".to_string()]);
        assert_eq!(ids(&visible(&i.items, ROOT)), ["img", "b"]);
        assert!(i.items.iter().all(|x| x.id != "c" && x.id != "old"));
        assert!(hydroxy_statechart::sync_or_keep::<FileManagerMachine>(&step.state, &i).selection.is_empty());
    }

    #[test]
    fn controlled_folder_waits_for_the_owner() {
        let mut i = FileManagerInput { folder: Some(ROOT.into()), ..input() };
        let s = FileManagerMachine::initial(&i);
        let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Open("docs".into()), &i);
        assert_eq!(step.state.folder, ROOT);
        assert_eq!(step.commands, [FileManagerCommand::FolderChange("docs".into())]);
        i.folder = Some("docs".into());
        assert_eq!(hydroxy_statechart::sync_or_keep::<FileManagerMachine>(&step.state, &i).folder, "docs");
        // A folder that no longer exists falls back to the top.
        i.folder = Some("gone".into());
        assert_eq!(hydroxy_statechart::sync_or_keep::<FileManagerMachine>(&step.state, &i).folder, ROOT);
    }

    #[test]
    fn apply_rename_and_view() {
        let mut list = items();
        apply_rename(&mut list, "a", "z.md");
        assert_eq!(list[2].name, "z.md");
        let i = input();
        let s = FileManagerMachine::initial(&i);
        assert_eq!(
            advance::<FileManagerMachine>(&s, FileManagerEvent::SetView(ViewMode::Grid), &i).state.view,
            ViewMode::Grid
        );
        assert!(advance::<FileManagerMachine>(&s, FileManagerEvent::SetView(ViewMode::List), &i).commands.is_empty());
    }
}
