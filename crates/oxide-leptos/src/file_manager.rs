//! `<FileManager>`: a folder tree, breadcrumbs and a list or grid of the open
//! folder's items. Click selects (Ctrl or ⌘ toggles, Shift extends), double
//! click or Enter opens, F2 renames, Delete deletes and a right click opens a
//! context menu. The items belong to you: renames and deletes arrive as
//! callbacks and you pass the new list back in.

use leptos::html::Ul;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::file_manager::{
    self as h, FileManagerCommand, FileManagerEvent, FileManagerInput, FileManagerMachine, FileManagerState,
};
use oxide_statechart::Machine;

use crate::context_menu::{ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger};
use crate::focus::focus_id;
use crate::tree_view::TreeView;

pub use oxide_headless::file_manager::{FileItem, ItemKind, ROOT, ViewMode, apply_delete, apply_rename};

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<FileManagerState>,
    input: Memo<FileManagerInput>,
    on_folder_change: Option<Callback<String>>,
    on_selection_change: Option<Callback<Vec<String>>>,
    on_open: Option<Callback<String>>,
    on_rename: Option<Callback<(String, String)>>,
    on_delete: Option<Callback<Vec<String>>>,
}

impl Ctx {
    fn send(self, event: FileManagerEvent) {
        let Some(step) = self
            .input
            .try_with_untracked(|i| self.state.try_with_untracked(|s| FileManagerMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        // Focus on a row that is about to leave with its folder would fall to the page.
        let in_list = self.id.try_with_value(|root| {
            document().active_element().is_some_and(|e| e.id().starts_with(&format!("{root}-item-")))
        }) == Some(true);
        self.state.set(step.state);
        for command in step.commands {
            match command {
                FileManagerCommand::FolderChange(f) => {
                    if in_list {
                        let Some(live) = self.id.try_with_value(|root| h::live_id(root)) else { continue };
                        set_timeout(
                            move || {
                                let first = document()
                                    .get_element_by_id(&live)
                                    .and_then(|l| l.parent_element())
                                    .and_then(|root| root.query_selector("[data-scope=file-manager][data-part=item][tabindex=\"0\"]").ok().flatten());
                                if let Some(first) = first {
                                    let _ = first.unchecked_into::<web_sys::HtmlElement>().focus();
                                }
                            },
                            std::time::Duration::ZERO,
                        );
                    }
                    run(self.on_folder_change, f);
                }
                FileManagerCommand::SelectionChange(ids) => run(self.on_selection_change, ids),
                FileManagerCommand::OpenFile(id) => run(self.on_open, id),
                FileManagerCommand::Rename { id, name } => run(self.on_rename, (id, name)),
                FileManagerCommand::Delete(ids) => run(self.on_delete, ids),
                FileManagerCommand::FocusItem(item) => {
                    // The row may re-render (it can move after a rename); focus once the DOM has caught up.
                    let target = h::item_id(&self.id.get_value(), &item);
                    set_timeout(move || focus_id(&target), std::time::Duration::ZERO);
                }
                FileManagerCommand::FocusRename => {
                    let target = h::rename_id(&self.id.get_value());
                    set_timeout(move || focus_and_select(&target), std::time::Duration::ZERO);
                }
            }
        }
    }
}

fn run<T: 'static>(callback: Option<Callback<T>>, value: T) {
    if let Some(cb) = callback {
        cb.run(value);
    }
}

fn focus_and_select(id: &str) {
    if let Some(el) = document().get_element_by_id(id) {
        let el = el.unchecked_into::<web_sys::HtmlInputElement>();
        let _ = web_sys::HtmlElement::focus(&el);
        el.select();
    }
}

/// A file browser. See the module docs for the keys.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn FileManager(
    /// Every file and folder, in any order. Items point at their folder through `parent`.
    #[prop(into)]
    items: Signal<Vec<FileItem>>,
    /// Controlled open folder id. [`ROOT`] (the empty string) is the top. Pair with `on_folder_change`.
    #[prop(optional, into)]
    folder: MaybeProp<String>,
    /// The folder to start in when uncontrolled.
    #[prop(optional, into)]
    default_folder: String,
    /// Start as a list (default) or a grid.
    #[prop(optional)]
    default_view: ViewMode,
    /// Name of the top-level folder in the tree and breadcrumbs.
    #[prop(default = "All files".into(), into)]
    root_label: String,
    /// Called with the folder id when another folder opens.
    #[prop(optional, into)]
    on_folder_change: Option<Callback<String>>,
    /// Called with the selected ids whenever the selection changes.
    #[prop(optional, into)]
    on_selection_change: Option<Callback<Vec<String>>>,
    /// Called with a file's id when it's opened (double click or Enter).
    #[prop(optional, into)]
    on_open: Option<Callback<String>>,
    /// Called with `(id, new name)` when a rename is confirmed. Apply it with [`apply_rename`].
    #[prop(optional, into)]
    on_rename: Option<Callback<(String, String)>>,
    /// Called with the ids to delete. Apply it with [`apply_delete`].
    #[prop(optional, into)]
    on_delete: Option<Callback<Vec<String>>>,
) -> impl IntoView {
    let input = Memo::new(move |_| FileManagerInput {
        items: items.get(),
        folder: folder.get(),
        default_folder: default_folder.clone(),
        default_view,
    });
    let state = RwSignal::new(input.with_untracked(FileManagerMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| FileManagerMachine::sync(s, i)));
        state.set(synced);
    });
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let ctx = Ctx { id, state, input, on_folder_change, on_selection_change, on_open, on_rename, on_delete };
    let root = id.get_value();

    let shown = Memo::new(move |_| input.with(|i| state.with(|s| h::visible(&i.items, &s.folder))));
    let shown_ids = Memo::new(move |_| shown.with(|v| v.iter().map(|i| i.id.clone()).collect::<Vec<_>>()));
    let crumbs = Memo::new(move |_| input.with(|i| state.with(|s| h::path(&i.items, &s.folder))));
    let list = NodeRef::<Ul>::new();
    let is_grid = move || state.with(|s| s.view == ViewMode::Grid);

    // Up and Down skip a row, which is as many tiles as fit across.
    let columns = move || {
        if !is_grid() {
            return 1;
        }
        list.get_untracked().map_or(1, |el| h::grid_columns(el.get_bounding_client_rect().width()))
    };

    // The tree stays open down to the current folder, whichever way it got there.
    let expanded = RwSignal::new(h::tree_open_ids(&input.with_untracked(|i| i.items.clone()), &state.with_untracked(|s| s.folder.clone())));
    Effect::new(move |_| {
        let wanted = input.with(|i| state.with(|s| h::tree_open_ids(&i.items, &s.folder)));
        expanded.update(|open| {
            for node in wanted {
                if !open.contains(&node) {
                    open.push(node);
                }
            }
        });
    });
    let root_label = StoredValue::new(root_label);
    let nodes = Signal::derive(move || items.with(|i| h::tree_nodes(i, &root_label.get_value())));

    // Say where we are after a folder change.
    let live = RwSignal::new(String::new());
    Effect::new(move |prev: Option<String>| {
        let folder = state.with(|s| s.folder.clone());
        if prev.as_ref().is_some_and(|p| *p != folder) {
            live.set(items.with_untracked(|i| h::folder_announcement(i, &folder)));
        }
        folder
    });

    let menu_items = Signal::derive(move || state.with(h::menu_entries));
    let sel_root = root.clone();
    let render_item = move |item: String| item_view(ctx, sel_root.clone(), shown, item, columns);

    view! {
        <div data-scope=h::SCOPE data-part="root" data-view=move || state.with(|s| s.view.as_str())>
            <aside data-scope=h::SCOPE data-part="sidebar">
                <TreeView
                    label="Folders"
                    nodes=nodes
                    expanded=Signal::derive(move || Some(expanded.get()))
                    on_expanded_change={move |ids: Vec<String>| expanded.set(ids)}
                    selected=Signal::derive(move || state.with(|s| Some(vec![h::tree_id(&s.folder)])))
                    on_selected_change={move |ids: Vec<String>| {
                        if let Some(folder) = ids.first().and_then(|n| h::folder_of_tree_id(n)) {
                            ctx.send(FileManagerEvent::Open(folder.to_string()));
                        }
                    }}
                />
            </aside>
            <div data-scope=h::SCOPE data-part="main">
                <div data-scope=h::SCOPE data-part="toolbar">
                    <nav data-scope=h::SCOPE data-part="breadcrumbs" aria-label="Breadcrumb">
                        <ol>
                            <li>
                                <button
                                    type="button"
                                    data-scope=h::SCOPE
                                    data-part="crumb"
                                    aria-current=move || state.with(|s| (s.folder == ROOT).then_some("page"))
                                    on:click=move |_| ctx.send(FileManagerEvent::Open(ROOT.to_string()))
                                >
                                    {move || root_label.get_value()}
                                </button>
                            </li>
                            <For
                                each=move || crumbs.get()
                                key=|f| f.id.clone()
                                children=move |f| {
                                    let (go, current) = (f.id.clone(), f.id.clone());
                                    view! {
                                        <li>
                                            <button
                                                type="button"
                                                data-scope=h::SCOPE
                                                data-part="crumb"
                                                aria-current=move || state.with(|s| (s.folder == current).then_some("page"))
                                                on:click=move |_| ctx.send(FileManagerEvent::Open(go.clone()))
                                            >
                                                {f.name}
                                            </button>
                                        </li>
                                    }
                                }
                            />
                        </ol>
                    </nav>
                    <span data-scope=h::SCOPE data-part="count">{move || state.with(|s| h::count_text(s, shown.with(Vec::len)))}</span>
                    <div role="group" aria-label="View" data-scope=h::SCOPE data-part="view-toggle">
                        {[ViewMode::List, ViewMode::Grid]
                            .into_iter()
                            .map(|mode| {
                                view! {
                                    <button
                                        type="button"
                                        aria-pressed=move || state.with(|s| (s.view == mode).to_string())
                                        on:click=move |_| ctx.send(FileManagerEvent::SetView(mode))
                                    >
                                        {if mode == ViewMode::List { "List" } else { "Grid" }}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
                <ContextMenu
                    items=menu_items
                    on_select=move |value: String| {
                        if let Some(event) = state.with_untracked(|s| h::menu_event(&value, s)) {
                            ctx.send(event);
                        }
                    }
                >
                    <ContextMenuTrigger>
                        <ul
                            node_ref=list
                            role="listbox"
                            aria-label="Files"
                            aria-multiselectable="true"
                            data-scope=h::SCOPE
                            data-part="items"
                            on:click=move |ev| {
                                // A click on the empty space clears the selection.
                                if ev.target() == ev.current_target() {
                                    ctx.send(FileManagerEvent::ClearSelection);
                                }
                            }
                        >
                            <For
                                each=move || shown_ids.get()
                                key=String::clone
                                children=render_item
                            />
                        </ul>
                        <Show when=move || shown.with(Vec::is_empty)>
                            <p data-scope=h::SCOPE data-part="empty">"This folder is empty"</p>
                        </Show>
                    </ContextMenuTrigger>
                    <ContextMenuContent>
                        <ContextMenuItem value=h::OPEN hint="↵">"Open"</ContextMenuItem>
                        <ContextMenuItem value=h::RENAME hint="F2">"Rename"</ContextMenuItem>
                        <ContextMenuItem value=h::DELETE hint="Del">"Delete"</ContextMenuItem>
                    </ContextMenuContent>
                </ContextMenu>
            </div>
            <div data-scope=h::SCOPE data-part="live" id=h::live_id(&root) role="status" aria-live="polite" aria-atomic="true">
                {move || live.get()}
            </div>
        </div>
    }
}

#[allow(clippy::too_many_lines, clippy::needless_pass_by_value, reason = "one view! tree, and `impl IntoView` must not borrow")]
fn item_view(
    ctx: Ctx,
    root: String,
    shown: Memo<Vec<FileItem>>,
    key: String,
    columns: impl Fn() -> usize + Copy + 'static,
) -> impl IntoView {
    let item = {
        let key = key.clone();
        Memo::new(move |_| shown.with(|v| v.iter().find(|i| i.id == key).cloned()))
    };
    let attrs = {
        let root = root.clone();
        Memo::new(move |_| {
            let fallback = FileItem::file(String::new(), String::new(), ROOT, 0);
            ctx.state.with(|s| shown.with(|v| item.with(|i| h::item_attrs(&root, s, v, i.as_ref().unwrap_or(&fallback)))))
        })
    };
    let name = move || item.with(|i| i.as_ref().map(|i| i.name.clone()).unwrap_or_default());
    let is_folder = move || item.with(|i| i.as_ref().is_some_and(FileItem::is_folder));
    let rename = move || ctx.state.with(|s| s.renaming.clone().filter(|r| r.id == key_of(item)));
    let (on_click, on_dbl, on_ctx, on_focus, on_key) = (key.clone(), key.clone(), key.clone(), key.clone(), key);
    let error_id = h::rename_error_id(&root);
    let rename_dom_id = h::rename_id(&root);

    let leading = move || {
        if is_folder() {
            Either::Left(view! {
                <span data-scope=h::SCOPE data-part="icon" aria-hidden="true">
                    <svg
                        viewBox="0 0 24 24"
                        width="20"
                        height="20"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.75"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        inner_html=h::FOLDER_ICON
                    ></svg>
                </span>
            })
        } else {
            Either::Right(view! { <span data-scope=h::SCOPE data-part="badge" aria-hidden="true">{move || h::badge_text(&name())}</span> })
        }
    };
    let label = {
        let (error_id, rename_dom_id) = (error_id.clone(), rename_dom_id.clone());
        move || match rename() {
            Some(r) => {
                let described = r.error.map(|_| error_id.clone());
                let message = r.error.map(h::NameError::message);
                Either::Left(view! {
                    <span data-scope=h::SCOPE data-part="name" data-editing="">
                        <input
                            id=rename_dom_id.clone()
                            type="text"
                            aria-label="Name"
                            aria-invalid=message.map(|_| "true")
                            aria-describedby=described
                            data-scope=h::SCOPE
                            data-part="rename"
                            prop:value=r.draft
                            on:input=move |ev| ctx.send(FileManagerEvent::RenameInput(event_target_value(&ev)))
                            on:keydown=move |ev| {
                                // Keys here are for the field, not the row around it.
                                ev.stop_propagation();
                                if let Some(event) = h::rename_keydown(&ev.key()) {
                                    ev.prevent_default();
                                    ctx.send(event);
                                }
                            }
                            on:click=|ev| ev.stop_propagation()
                            on:dblclick=|ev| ev.stop_propagation()
                            on:blur=move |_| ctx.send(FileManagerEvent::CommitRename)
                        />
                        {message.map(|m| view! { <span id=error_id.clone() data-scope=h::SCOPE data-part="rename-error" role="alert">{m}</span> })}
                    </span>
                })
            }
            None => Either::Right(view! { <span data-scope=h::SCOPE data-part="name">{name}</span> }),
        }
    };

    view! {
        <li
            role="option"
            id=move || attrs.get().id
            aria-selected=move || attrs.get().selected.to_string()
            tabindex=move || if attrs.get().tab_stop { "0" } else { "-1" }
            data-scope=h::SCOPE
            data-part="item"
            data-kind=move || attrs.get().kind
            data-selected=move || attrs.get().selected.then_some("")
            data-renaming=move || attrs.get().renaming.then_some("")
            on:click=move |ev| {
                if ctx.state.with_untracked(|s| s.renaming.is_none()) {
                    ev.stop_propagation();
                    let extend = h::click_extend(ev.ctrl_key() || ev.meta_key(), ev.shift_key());
                    ctx.send(FileManagerEvent::Click { id: on_click.clone(), extend });
                }
            }
            on:dblclick=move |_| ctx.send(FileManagerEvent::Activate(on_dbl.clone()))
            on:contextmenu=move |_| ctx.send(FileManagerEvent::Context(on_ctx.clone()))
            on:focus=move |_| ctx.send(FileManagerEvent::Focus(on_focus.clone()))
            on:keydown=move |ev| {
                // The menu key opens the menu without a contextmenu event, so pick the row here.
                if ev.key() == "ContextMenu" || (ev.shift_key() && ev.key() == "F10") {
                    ctx.send(FileManagerEvent::Context(on_key.clone()));
                }
                let view = ctx.state.with_untracked(|s| s.view);
                let event = h::item_keydown(
                    &ev.key(),
                    ev.ctrl_key() || ev.meta_key(),
                    ev.shift_key(),
                    ev.alt_key(),
                    columns(),
                    &on_key,
                    view,
                );
                if let Some(event) = event {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            {leading}
            {label}
            <span data-scope=h::SCOPE data-part="modified">{move || item.with(|i| i.as_ref().map(|i| i.modified.clone()))}</span>
            <span data-scope=h::SCOPE data-part="size">{move || item.with(|i| i.as_ref().map(h::size_text))}</span>
        </li>
    }
}

/// The id of the item a memo holds, or an empty string.
fn key_of(item: Memo<Option<FileItem>>) -> String {
    item.with(|i| i.as_ref().map(|i| i.id.clone()).unwrap_or_default())
}
