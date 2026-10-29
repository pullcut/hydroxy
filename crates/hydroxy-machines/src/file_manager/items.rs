//! The item tree and the helpers that read or edit it.

/// The id of the top-level folder. It has no item of its own.
pub const ROOT: &str = "";

/// A folder or a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
