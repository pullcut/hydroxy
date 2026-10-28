//! `<TreeView>`: a hierarchy of items with expandable branches, roving focus
//! and single or multiple selection.

use hydroxy_headless::tree_view::{
    self as h, TreeViewCommand, TreeViewEvent, TreeViewInput, TreeViewMachine, TreeViewState, visible_rows,
};
use hydroxy_statechart::Machine;
use leptos::attr::custom::custom_attribute;
use leptos::prelude::*;

use crate::focus::focus_id;
use crate::icon::{Icon, IconName};

pub use hydroxy_headless::tree_view::{SelectionMode, TreeNode};

/// One rendered row, owned so the view can look it up by id.
#[derive(Clone, PartialEq)]
struct Row {
    id: String,
    label: String,
    branch: bool,
    attrs: h::ItemAttrs,
}

#[derive(Clone, Copy)]
struct Ctx {
    id: StoredValue<String>,
    state: RwSignal<TreeViewState>,
    input: Memo<TreeViewInput>,
    on_expanded_change: Option<Callback<Vec<String>>>,
    on_selected_change: Option<Callback<Vec<String>>>,
}

impl Ctx {
    fn send(self, event: TreeViewEvent) {
        // Events can still arrive after unmount.
        let Some(step) = self
            .input
            .try_with_untracked(|i| self.state.try_with_untracked(|s| TreeViewMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                TreeViewCommand::FocusItem(item) => focus_id(&h::item_id(&self.id.get_value(), &item)),
                TreeViewCommand::ExpandedChange(v) => {
                    if let Some(cb) = self.on_expanded_change {
                        cb.run(v);
                    }
                }
                TreeViewCommand::SelectedChange(v) => {
                    if let Some(cb) = self.on_selected_change {
                        cb.run(v);
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
    #[prop(into)]
    nodes: Signal<Vec<TreeNode>>,
    /// Accessible name for the tree.
    #[prop(into)]
    label: String,
    /// Single (default) or multiple selection.
    #[prop(optional)]
    selection_mode: SelectionMode,
    /// Controlled expanded branch ids. Pair with `on_expanded_change`.
    #[prop(optional, into)]
    expanded: MaybeProp<Vec<String>>,
    /// Initially expanded branch ids when uncontrolled.
    #[prop(optional, into)]
    default_expanded: Vec<String>,
    /// Controlled selected ids. Pair with `on_selected_change`.
    #[prop(optional, into)]
    selected: MaybeProp<Vec<String>>,
    /// Initially selected ids when uncontrolled.
    #[prop(optional, into)]
    default_selected: Vec<String>,
    /// Called with the expanded ids whenever a branch opens or closes.
    #[prop(optional, into)]
    on_expanded_change: Option<Callback<Vec<String>>>,
    /// Called with the selected ids whenever the selection changes.
    #[prop(optional, into)]
    on_selected_change: Option<Callback<Vec<String>>>,
) -> impl IntoView {
    let id = StoredValue::new(crate::id::next_id(h::SCOPE));
    let input = Memo::new(move |_| TreeViewInput {
        nodes: nodes.get(),
        selection_mode,
        expanded: expanded.get(),
        default_expanded: default_expanded.clone(),
        selected: selected.get(),
        default_selected: default_selected.clone(),
    });
    let state = RwSignal::new(input.with_untracked(TreeViewMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| TreeViewMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });

    let ctx = Ctx { id, state, input, on_expanded_change, on_selected_change };
    let rows = Memo::new(move |_| {
        input.with(|i| {
            state.with(|s: &TreeViewState| {
                let rows = visible_rows(&i.nodes, &s.expanded);
                let stop = s.tab_stop(&rows);
                rows.iter()
                    .map(|r| Row {
                        id: r.node.id.clone(),
                        label: r.node.label.clone(),
                        branch: r.node.is_branch(),
                        attrs: h::item_attrs(&id.get_value(), s, r, stop),
                    })
                    .collect::<Vec<_>>()
            })
        })
    });
    let keys = Memo::new(move |_| rows.with(|rs| rs.iter().map(|r| r.id.clone()).collect::<Vec<_>>()));

    view! {
        <ul
            role="tree"
            aria-label=label
            aria-multiselectable=(selection_mode == SelectionMode::Multiple).then_some("true")
            data-scope=h::SCOPE
            data-part="tree"
        >
            <For each=move || keys.get() key=String::clone children=move |key| row_view(ctx, rows, key) />
        </ul>
    }
}

fn row_view(ctx: Ctx, rows: Memo<Vec<Row>>, key: String) -> impl IntoView {
    // linear lookup per row, O(n²) per update; index by id if trees reach thousands of rows.
    let row = {
        let key = key.clone();
        Memo::new(move |_| rows.with(|rs| rs.iter().find(|r| r.id == key).cloned()))
    };
    let a = move || row.get().map(|r| r.attrs);
    let (clicked, focused, keyed, toggled) = (key.clone(), key.clone(), key.clone(), key);
    let (label, branch) = row.with_untracked(|r| r.as_ref().map(|r| (r.label.clone(), r.branch))).unwrap_or_default();
    view! {
        <li
            role="treeitem"
            id=move || a().map(|a| a.id)
            aria-setsize=move || a().map(|a| a.aria_setsize)
            aria-posinset=move || a().map(|a| a.aria_posinset)
            aria-expanded=move || a().and_then(|a| a.aria_expanded).map(|e| e.to_string())
            aria-selected=move || a().is_some_and(|a| a.selected).to_string()
            aria-disabled=move || a().is_some_and(|a| a.disabled).then_some("true")
            tabindex=move || a().map_or(-1, |a| a.tabindex)
            style=move || a().map(|a| a.style)
            data-scope=h::SCOPE
            data-part="item"
            data-state=move || a().and_then(|a| a.data_state)
            data-selected=move || a().is_some_and(|a| a.selected).then_some("")
            data-disabled=move || a().is_some_and(|a| a.disabled).then_some("")
            on:click=move |_| ctx.send(TreeViewEvent::Select(clicked.clone()))
            on:focus=move |_| ctx.send(TreeViewEvent::Focus(focused.clone()))
            on:keydown=move |ev| {
                if ev.ctrl_key() || ev.meta_key() || ev.alt_key() {
                    return;
                }
                if let Some(event) = h::keydown(&ev.key(), &keyed) {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            <span
                data-scope=h::SCOPE
                data-part="toggle"
                aria-hidden="true"
                on:click=move |ev| {
                    if branch {
                        ev.stop_propagation();
                        ctx.send(TreeViewEvent::Toggle(toggled.clone()));
                    }
                }
            >
                {branch.then(|| view! { <Icon name=IconName::ChevronRight size=14 /> })}
            </span>
            <span data-scope=h::SCOPE data-part="label">{label}</span>
        </li>
    }
    // tachys has no typed `aria-level`.
    .add_any_attr(custom_attribute("aria-level", move || a().map(|a| a.aria_level.to_string())))
}
