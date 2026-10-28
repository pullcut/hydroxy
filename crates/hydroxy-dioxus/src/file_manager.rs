//! `FileManager`: a folder tree, breadcrumbs and a list or grid of the open
//! folder's items. Click selects (Ctrl or ⌘ toggles, Shift extends), double
//! click or Enter opens, F2 renames, Delete deletes and a right click opens a
//! context menu. The items belong to you: renames and deletes arrive as
//! callbacks and you pass the new list back in.

use std::rc::Rc;

use dioxus::prelude::*;
use hydroxy_headless::file_manager::{
    self as h, FileManagerCommand, FileManagerEvent, FileManagerInput, FileManagerMachine, FileManagerState,
};
use hydroxy_statechart::Machine;

use crate::context_menu::{ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger};
use crate::focus::FocusMap;
use crate::tree_view::TreeView;

pub use hydroxy_headless::file_manager::{FileItem, ItemKind, ROOT, ViewMode, apply_delete, apply_rename};

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<FileManagerState>,
    input: Memo<FileManagerInput>,
    rows: FocusMap,
    rename_id: Signal<String>,
    on_folder_change: Option<EventHandler<String>>,
    on_selection_change: Option<EventHandler<Vec<String>>>,
    on_open: Option<EventHandler<String>>,
    on_rename: Option<EventHandler<(String, String)>>,
    on_delete: Option<EventHandler<Vec<String>>>,
}

impl Ctx {
    fn send(mut self, event: FileManagerEvent) {
        let step = FileManagerMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                FileManagerCommand::FolderChange(f) => call(self.on_folder_change, f),
                FileManagerCommand::SelectionChange(ids) => call(self.on_selection_change, ids),
                FileManagerCommand::OpenFile(id) => call(self.on_open, id),
                FileManagerCommand::Rename { id, name } => call(self.on_rename, (id, name)),
                FileManagerCommand::Delete(ids) => call(self.on_delete, ids),
                FileManagerCommand::FocusItem(item) => self.rows.focus(&item),
                FileManagerCommand::FocusRename => {
                    // The field is created by this same update, so wait a tick for it.
                    let id = self.rename_id.peek().clone();
                    let _ = document::eval(&format!(
                        "setTimeout(() => {{ const e = document.getElementById({id:?}); if (e) {{ e.focus(); e.select(); }} }}, 0);"
                    ));
                }
            }
        }
    }
}

fn call<T: 'static>(handler: Option<EventHandler<T>>, value: T) {
    if let Some(handler) = handler {
        handler.call(value);
    }
}

/// A file browser. See the module docs for the keys.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn FileManager(
    /// Every file and folder, in any order. Items point at their folder through `parent`.
    items: ReadSignal<Vec<FileItem>>,
    /// Controlled open folder id. [`ROOT`] (the empty string) is the top. Pair with `on_folder_change`.
    #[props(default)]
    folder: ReadSignal<Option<String>>,
    /// The folder to start in when uncontrolled.
    #[props(default)]
    default_folder: String,
    /// Start as a list (default) or a grid.
    #[props(default)]
    default_view: ViewMode,
    /// Name of the top-level folder in the tree and breadcrumbs.
    #[props(default = "All files".to_string())]
    root_label: String,
    /// Called with the folder id when another folder opens.
    on_folder_change: Option<EventHandler<String>>,
    /// Called with the selected ids whenever the selection changes.
    on_selection_change: Option<EventHandler<Vec<String>>>,
    /// Called with a file's id when it's opened (double click or Enter).
    on_open: Option<EventHandler<String>>,
    /// Called with `(id, new name)` when a rename is confirmed. Apply it with [`apply_rename`].
    on_rename: Option<EventHandler<(String, String)>>,
    /// Called with the ids to delete. Apply it with [`apply_delete`].
    on_delete: Option<EventHandler<Vec<String>>>,
) -> Element {
    let input = use_memo(move || FileManagerInput {
        items: items(),
        folder: folder(),
        default_folder: default_folder.clone(),
        default_view,
    });
    let mut state = use_signal(|| FileManagerMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = FileManagerMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let root = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let ctx = use_hook(|| Ctx {
        state,
        input,
        rows: FocusMap::new(),
        rename_id: Signal::new(h::rename_id(&root)),
        on_folder_change,
        on_selection_change,
        on_open,
        on_rename,
        on_delete,
    });
    let mut list = use_signal(|| None::<Rc<MountedData>>);

    // The tree stays open down to the current folder, whichever way it got there.
    let mut expanded = use_signal(|| h::tree_open_ids(&input.peek().items, &state.peek().folder));
    use_effect(move || {
        let wanted = h::tree_open_ids(&input.read().items, &state.read().folder);
        let missing: Vec<String> = wanted.into_iter().filter(|n| !expanded.peek().contains(n)).collect();
        if !missing.is_empty() {
            expanded.write().extend(missing);
        }
    });

    // Say where we are after a folder change.
    let mut live = use_signal(String::new);
    let mut previous = use_signal(|| state.peek().folder.clone());
    use_effect(move || {
        let current = state.read().folder.clone();
        if *previous.peek() != current {
            live.set(h::folder_announcement(&input.peek().items, &current));
            previous.set(current);
        }
    });

    let s = state.read().clone();
    let inp = input.read().clone();
    let shown = h::visible(&inp.items, &s.folder);
    let crumbs = h::path(&inp.items, &s.folder);
    let nodes = h::tree_nodes(&inp.items, &root_label);
    let entries = h::menu_entries(&s);
    let count = h::count_text(&s, shown.len());

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", "data-view": s.view.as_str(),
            aside { "data-scope": h::SCOPE, "data-part": "sidebar",
                TreeView {
                    label: "Folders",
                    nodes,
                    expanded: Some(expanded()),
                    on_expanded_change: move |ids: Vec<String>| expanded.set(ids),
                    selected: Some(vec![h::tree_id(&s.folder)]),
                    on_selected_change: move |ids: Vec<String>| {
                        if let Some(folder) = ids.first().and_then(|n| h::folder_of_tree_id(n)) {
                            ctx.send(FileManagerEvent::Open(folder.to_string()));
                        }
                    },
                }
            }
            div { "data-scope": h::SCOPE, "data-part": "main",
                div { "data-scope": h::SCOPE, "data-part": "toolbar",
                    nav { "data-scope": h::SCOPE, "data-part": "breadcrumbs", "aria-label": "Breadcrumb",
                        ol {
                            li {
                                button {
                                    r#type: "button",
                                    "data-scope": h::SCOPE,
                                    "data-part": "crumb",
                                    "aria-current": (s.folder == ROOT).then_some("page"),
                                    onclick: move |_| ctx.send(FileManagerEvent::Open(ROOT.to_string())),
                                    "{root_label}"
                                }
                            }
                            for f in crumbs.iter() {
                                {
                                    let go = f.id.clone();
                                    rsx! {
                                        li { key: "{f.id}",
                                            button {
                                                r#type: "button",
                                                "data-scope": h::SCOPE,
                                                "data-part": "crumb",
                                                "aria-current": (s.folder == f.id).then_some("page"),
                                                onclick: move |_| ctx.send(FileManagerEvent::Open(go.clone())),
                                                "{f.name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    span { "data-scope": h::SCOPE, "data-part": "count", "{count}" }
                    div { role: "group", "aria-label": "View", "data-scope": h::SCOPE, "data-part": "view-toggle",
                        for mode in [ViewMode::List, ViewMode::Grid] {
                            button {
                                key: "{mode.as_str()}",
                                r#type: "button",
                                "aria-pressed": (s.view == mode).to_string(),
                                onclick: move |_| ctx.send(FileManagerEvent::SetView(mode)),
                                if mode == ViewMode::List { "List" } else { "Grid" }
                            }
                        }
                    }
                }
                ContextMenu {
                    items: entries,
                    on_select: move |value: String| {
                        let event = h::menu_event(&value, &ctx.state.peek());
                        if let Some(event) = event {
                            ctx.send(event);
                        }
                    },
                    ContextMenuTrigger {
                        ul {
                            role: "listbox",
                            "aria-label": "Files",
                            "aria-multiselectable": "true",
                            "data-scope": h::SCOPE,
                            "data-part": "items",
                            onmounted: move |e| list.set(Some(e.data())),
                            for item in shown.iter() {
                                {
                                    let a = h::item_attrs(&root, &s, &shown, item);
                                    let key = item.id.clone();
                                    let (mounted, clicked, double, context, focused, menu_key, keyed) =
                                        (key.clone(), key.clone(), key.clone(), key.clone(), key.clone(), key.clone(), key);
                                    let rename = s.renaming.clone().filter(|r| r.id == item.id);
                                    let error_id = h::rename_error_id(&root);
                                    let message = rename.as_ref().and_then(|r| r.error).map(h::NameError::message);
                                    rsx! {
                                        li {
                                            key: "{item.id}",
                                            role: "option",
                                            id: a.id,
                                            "aria-selected": a.selected.to_string(),
                                            tabindex: if a.tab_stop { "0" } else { "-1" },
                                            "data-scope": h::SCOPE,
                                            "data-part": "item",
                                            "data-kind": a.kind,
                                            "data-selected": a.selected.then_some(""),
                                            "data-renaming": a.renaming.then_some(""),
                                            onmounted: move |e| ctx.rows.register(mounted.clone(), e.data()),
                                            onclick: move |e| {
                                                if ctx.state.peek().renaming.is_none() {
                                                    e.stop_propagation();
                                                    let m = e.modifiers();
                                                    let extend = h::click_extend(m.ctrl() || m.meta(), m.shift());
                                                    ctx.send(FileManagerEvent::Click { id: clicked.clone(), extend });
                                                }
                                            },
                                            ondoubleclick: move |_| ctx.send(FileManagerEvent::Activate(double.clone())),
                                            oncontextmenu: move |_| ctx.send(FileManagerEvent::Context(context.clone())),
                                            onfocus: move |_| ctx.send(FileManagerEvent::Focus(focused.clone())),
                                            onkeydown: move |e| {
                                                let m = e.modifiers();
                                                let key = e.key().to_string();
                                                // The menu key opens the menu without a contextmenu event, so pick the row here.
                                                if key == "ContextMenu" || (m.shift() && key == "F10") {
                                                    ctx.send(FileManagerEvent::Context(menu_key.clone()));
                                                }
                                                let view = ctx.state.peek().view;
                                                let args = (m.ctrl() || m.meta(), m.shift(), m.alt());
                                                let map = {
                                                    let keyed = keyed.clone();
                                                    let key = key.clone();
                                                    move |columns| h::item_keydown(&key, args.0, args.1, args.2, columns, &keyed, view)
                                                };
                                                // Up and Down in a grid skip a row, so they need the container's width first.
                                                if view == ViewMode::Grid && (key == "ArrowUp" || key == "ArrowDown") && !args.2 {
                                                    e.prevent_default();
                                                    spawn(async move {
                                                        let width = match list() {
                                                            Some(el) => el.get_client_rect().await.map_or(0.0, |r| r.width()),
                                                            None => 0.0,
                                                        };
                                                        if let Some(event) = map(h::grid_columns(width)) {
                                                            ctx.send(event);
                                                        }
                                                    });
                                                } else if let Some(event) = map(1) {
                                                    e.prevent_default();
                                                    ctx.send(event);
                                                }
                                            },
                                            if item.is_folder() {
                                                span { "data-scope": h::SCOPE, "data-part": "icon", "aria-hidden": "true",
                                                    svg {
                                                        view_box: "0 0 24 24",
                                                        width: "20",
                                                        height: "20",
                                                        fill: "none",
                                                        stroke: "currentColor",
                                                        stroke_width: "1.75",
                                                        stroke_linecap: "round",
                                                        stroke_linejoin: "round",
                                                        dangerous_inner_html: h::FOLDER_ICON,
                                                    }
                                                }
                                            } else {
                                                span { "data-scope": h::SCOPE, "data-part": "badge", "aria-hidden": "true", {h::badge_text(&item.name)} }
                                            }
                                            if let Some(r) = rename {
                                                span { "data-scope": h::SCOPE, "data-part": "name", "data-editing": "",
                                                    input {
                                                        id: "{ctx.rename_id}",
                                                        r#type: "text",
                                                        "aria-label": "Name",
                                                        "aria-invalid": message.map(|_| "true"),
                                                        "aria-describedby": message.map(|_| error_id.clone()),
                                                        "data-scope": h::SCOPE,
                                                        "data-part": "rename",
                                                        value: "{r.draft}",
                                                        oninput: move |e| ctx.send(FileManagerEvent::RenameInput(e.value())),
                                                        onkeydown: move |e| {
                                                            // Keys here are for the field, not the row around it.
                                                            e.stop_propagation();
                                                            if let Some(event) = h::rename_keydown(&e.key().to_string()) {
                                                                e.prevent_default();
                                                                ctx.send(event);
                                                            }
                                                        },
                                                        onclick: |e| e.stop_propagation(),
                                                        ondoubleclick: |e| e.stop_propagation(),
                                                        onblur: move |_| ctx.send(FileManagerEvent::CommitRename),
                                                    }
                                                    if let Some(m) = message {
                                                        span { id: "{error_id}", "data-scope": h::SCOPE, "data-part": "rename-error", role: "alert", "{m}" }
                                                    }
                                                }
                                            } else {
                                                span { "data-scope": h::SCOPE, "data-part": "name", "{item.name}" }
                                            }
                                            span { "data-scope": h::SCOPE, "data-part": "modified", "{item.modified}" }
                                            span { "data-scope": h::SCOPE, "data-part": "size", {h::size_text(item)} }
                                        }
                                    }
                                }
                            }
                        }
                        if shown.is_empty() {
                            p { "data-scope": h::SCOPE, "data-part": "empty", "This folder is empty" }
                        }
                    }
                    ContextMenuContent {
                        ContextMenuItem { value: h::OPEN, hint: "↵", "Open" }
                        ContextMenuItem { value: h::RENAME, hint: "F2", "Rename" }
                        ContextMenuItem { value: h::DELETE, hint: "Del", "Delete" }
                    }
                }
            }
            div {
                "data-scope": h::SCOPE,
                "data-part": "live",
                id: h::live_id(&root),
                role: "status",
                "aria-live": "polite",
                "aria-atomic": "true",
                "{live}"
            }
        }
    }
}
