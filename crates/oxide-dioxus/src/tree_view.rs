//! `TreeView`: a hierarchy of items with expandable branches, roving focus
//! and single or multiple selection.

use dioxus::prelude::*;
use oxide_headless::tree_view::{
    self as h, TreeViewCommand, TreeViewEvent, TreeViewInput, TreeViewMachine, TreeViewState, visible_rows,
};
use oxide_statechart::Machine;

use crate::focus::FocusMap;
use crate::icon::{Icon, IconName};

pub use oxide_headless::tree_view::{SelectionMode, TreeNode};

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<TreeViewState>,
    input: Memo<TreeViewInput>,
    items: FocusMap,
    on_expanded_change: Option<EventHandler<Vec<String>>>,
    on_selected_change: Option<EventHandler<Vec<String>>>,
}

impl Ctx {
    fn send(mut self, event: TreeViewEvent) {
        let step = TreeViewMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                TreeViewCommand::FocusItem(id) => self.items.focus(&id),
                TreeViewCommand::ExpandedChange(v) => {
                    if let Some(handler) = self.on_expanded_change {
                        handler.call(v);
                    }
                }
                TreeViewCommand::SelectedChange(v) => {
                    if let Some(handler) = self.on_selected_change {
                        handler.call(v);
                    }
                }
            }
        }
    }
}

/// A tree. Rows render flat in display order with `aria-level` for depth.
#[component]
pub fn TreeView(
    /// The top-level nodes.
    nodes: ReadSignal<Vec<TreeNode>>,
    /// Accessible name for the tree.
    label: String,
    /// Single (default) or multiple selection.
    #[props(default)]
    selection_mode: SelectionMode,
    /// Controlled expanded branch ids. Pair with `on_expanded_change`.
    #[props(default)]
    expanded: ReadSignal<Option<Vec<String>>>,
    /// Initially expanded branch ids when uncontrolled.
    #[props(default)]
    default_expanded: Vec<String>,
    /// Controlled selected ids. Pair with `on_selected_change`.
    #[props(default)]
    selected: ReadSignal<Option<Vec<String>>>,
    /// Initially selected ids when uncontrolled.
    #[props(default)]
    default_selected: Vec<String>,
    /// Called with the expanded ids whenever a branch opens or closes.
    on_expanded_change: Option<EventHandler<Vec<String>>>,
    /// Called with the selected ids whenever the selection changes.
    on_selected_change: Option<EventHandler<Vec<String>>>,
) -> Element {
    let input = use_memo(move || TreeViewInput {
        nodes: nodes(),
        selection_mode,
        expanded: expanded(),
        default_expanded: default_expanded.clone(),
        selected: selected(),
        default_selected: default_selected.clone(),
    });
    let mut state = use_signal(|| TreeViewMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = TreeViewMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let id = use_hook(|| oxide_headless::next_id(h::SCOPE));
    let ctx = use_hook(|| Ctx { state, input, items: FocusMap::new(), on_expanded_change, on_selected_change });

    let input = input.read();
    let state = state.read();
    let rows = visible_rows(&input.nodes, &state.expanded);
    let stop = state.tab_stop(&rows);

    rsx! {
        ul {
            role: "tree",
            "aria-label": label,
            "aria-multiselectable": (selection_mode == SelectionMode::Multiple).then_some("true"),
            "data-scope": h::SCOPE,
            "data-part": "tree",
            for row in rows.iter() {
                {
                    let a = h::item_attrs(&id, &state, row, stop);
                    let key = row.node.id.clone();
                    let branch = row.node.is_branch();
                    let (mounted, clicked, focused, keyed, toggled) =
                        (key.clone(), key.clone(), key.clone(), key.clone(), key.clone());
                    rsx! {
                        li {
                            key: "{key}",
                            role: "treeitem",
                            id: a.id,
                            "aria-level": a.aria_level,
                            "aria-setsize": a.aria_setsize,
                            "aria-posinset": a.aria_posinset,
                            "aria-expanded": a.aria_expanded.map(|e| e.to_string()),
                            "aria-selected": a.selected.to_string(),
                            "aria-disabled": a.disabled.then_some("true"),
                            tabindex: a.tabindex,
                            style: a.style,
                            "data-scope": h::SCOPE,
                            "data-part": "item",
                            "data-state": a.data_state,
                            "data-selected": a.selected.then_some(""),
                            "data-disabled": a.disabled.then_some(""),
                            onmounted: move |e| ctx.items.register(mounted.clone(), e.data()),
                            onclick: move |_| ctx.send(TreeViewEvent::Select(clicked.clone())),
                            onfocus: move |_| ctx.send(TreeViewEvent::Focus(focused.clone())),
                            onkeydown: move |e| {
                                let m = e.modifiers();
                                if m.ctrl() || m.meta() || m.alt() {
                                    return;
                                }
                                if let Some(event) = h::keydown(&e.key().to_string(), &keyed) {
                                    e.prevent_default();
                                    ctx.send(event);
                                }
                            },
                            span {
                                "data-scope": h::SCOPE,
                                "data-part": "toggle",
                                "aria-hidden": "true",
                                onclick: move |e| {
                                    if branch {
                                        e.stop_propagation();
                                        ctx.send(TreeViewEvent::Toggle(toggled.clone()));
                                    }
                                },
                                if branch {
                                    Icon { name: IconName::ChevronRight, size: 14 }
                                }
                            }
                            span { "data-scope": h::SCOPE, "data-part": "label", "{row.node.label}" }
                        }
                    }
                }
            }
        }
    }
}
