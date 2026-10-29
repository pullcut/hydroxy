//! The input, state, event and command types.

use hydroxy_statechart::Controllable;

use super::items::FileItem;

/// List rows or a grid of tiles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
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
    /// Controlled open folder. `Some` at start fixes controlled mode. [`ROOT`](super::ROOT) is the top.
    pub folder: Option<String>,
    /// Uncontrolled initial folder.
    pub default_folder: String,
    /// Initial list or grid.
    pub default_view: ViewMode,
}

/// Why a rename was turned down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl std::fmt::Display for NameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for NameError {}

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
    /// The open folder. Controlled when the owner holds it.
    pub folder: Controllable<String>,
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
}

impl FileManagerState {
    /// The item to give `tabindex="0"`: the focused one, else the first.
    #[must_use]
    pub fn tab_stop<'a>(&self, shown: &'a [FileItem]) -> Option<&'a FileItem> {
        self.focused.as_deref().and_then(|id| shown.iter().find(|i| i.id == id)).or_else(|| shown.first())
    }
}

/// How a click combines with the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Extend {
    /// Replace the selection.
    Replace,
    /// Add or remove just this item (Ctrl or ⌘).
    Toggle,
    /// Select everything between the anchor and this item (Shift).
    Range,
}

/// Where a focus move goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// Open this folder ([`ROOT`](super::ROOT) for the top).
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
