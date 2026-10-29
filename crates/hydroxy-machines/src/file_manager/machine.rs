//! The file manager machine.

use super::items::{FileItem, ROOT, is_folder, visible};
use super::model::{
    Extend, FileManagerCommand, FileManagerEvent, FileManagerInput, FileManagerState, NameError, Rename, Step,
};
use hydroxy_statechart::{Controllable, Machine, Transition};

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
    next.folder.set(folder.to_string());
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
            folder: Controllable::new(
                if is_folder(&input.items, &wanted) { wanted } else { ROOT.to_string() },
                input.folder.is_some(),
            ),
            view: input.default_view,
            ..FileManagerState::default()
        }
    }

    #[allow(clippy::too_many_lines, reason = "one match over the events; splitting it only adds plumbing")]
    fn transition(state: &FileManagerState, event: FileManagerEvent, input: &FileManagerInput) -> Out {
        let stay = || Transition::stay();
        let shown = visible(&input.items, &state.folder.value);
        let mut next = state.clone();
        let mut commands = Vec::new();
        match event {
            FileManagerEvent::Open(id) => {
                if id == state.folder.value || !is_folder(&input.items, &id) {
                    return stay();
                }
                next = go(state, &id, &mut commands);
            }
            FileManagerEvent::Up => {
                let Some(parent) = input.items.iter().find(|i| i.id == state.folder.value).map(|i| i.parent.clone())
                else {
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
        next.folder.sync(Some(input.folder.clone().unwrap_or_default()));
        if !is_folder(&input.items, &next.folder.value) {
            next.folder.value = ROOT.to_string();
        }
        let shown = visible(&input.items, &next.folder.value);
        let here = |id: &str| index_of(&shown, id).is_some();
        next.selection.retain(|id| here(id));
        next.anchor = next.anchor.filter(|id| here(id));
        next.focused = next.focused.filter(|id| here(id));
        next.renaming = next.renaming.filter(|r| here(&r.id));
        hydroxy_statechart::changed(state, next)
    }
}
