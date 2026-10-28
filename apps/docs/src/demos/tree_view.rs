use hydroxy_leptos::tree_view::*;
use leptos::prelude::*;

fn files() -> Vec<TreeNode> {
    vec![
        TreeNode::new("research", "Research").children(vec![
            TreeNode::new("interviews", "Interviews")
                .children(vec![TreeNode::new("q3", "interviews-q3.pdf"), TreeNode::new("q4", "interviews-q4.pdf")]),
            TreeNode::new("tickets", "tickets-sept.csv"),
        ]),
        TreeNode::new("summaries", "Summaries")
            .children(vec![TreeNode::new("weekly", "weekly-digest.md"), TreeNode::new("board", "board-update.md")]),
        TreeNode::new("archive", "Archive").children(vec![TreeNode::new("old", "2025-notes.md")]).disabled(),
        TreeNode::new("readme", "README.md"),
    ]
}

pub fn demo() -> AnyView {
    let (selected, set_selected) = signal(vec!["q3".to_string()]);

    view! {
        <div style="display: grid; gap: var(--space-8); max-width: 360px;">
            // Uncontrolled expansion, controlled single selection.
            <div>
                <TreeView
                    label="Project files"
                    nodes=files()
                    default_expanded=vec!["research".to_string(), "interviews".to_string()]
                    selected=selected
                    on_selected_change=move |v| set_selected.set(v)
                />
                <p>"Selected: " {move || selected.get().join(", ")}</p>
            </div>

            // Multiple selection: Enter, Space or click toggles each item.
            <TreeView
                label="Files to share"
                nodes=files()
                selection_mode=SelectionMode::Multiple
                default_expanded=vec!["summaries".to_string()]
                default_selected=vec!["weekly".to_string(), "board".to_string()]
            />
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "tree-view",
    name: "Tree view",
    group: "Navigation",
    summary: "A hierarchy of items with branches that open and close. One tab stop; arrow keys walk the visible rows.",
    demo,
    leptos: include_str!("tree_view.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/tree_view.rs"),
    parts: &[
        &["TreeView", "tree", "<ul role=tree>, aria-label, aria-multiselectable in multiple mode"],
        &[
            "TreeView",
            "item",
            "<li role=treeitem>: aria-level, aria-setsize, aria-posinset, aria-expanded on branches, aria-selected. data-state=open|closed, data-selected, data-disabled. --ox-tree-level drives the indent",
        ],
        &["TreeView", "toggle", "Chevron. Click opens or closes the branch without selecting"],
        &["TreeView", "label", "The node's text"],
    ],
    keys: &[
        &["ArrowDown / ArrowUp", "Next / previous visible item"],
        &["ArrowRight", "Opens a closed branch, or moves to its first child"],
        &["ArrowLeft", "Closes an open branch, or moves to the parent"],
        &["Home / End", "First / last visible item"],
        &["Enter / Space", "Selects the item. Toggles it in multiple mode"],
        &["*", "Opens every branch beside the focused item"],
        &["Letter", "Next visible item starting with that letter"],
    ],
    props: &[
        &["nodes", "Vec<TreeNode>", "Required. TreeNode::new(id, label).children(..).disabled()"],
        &["label", "String", "Required. Accessible name"],
        &["selection_mode", "SelectionMode", "Single (default) or Multiple"],
        &["expanded", "Option<Vec<String>>", "Controlled open branches. Pair with on_expanded_change"],
        &["default_expanded", "Vec<String>", "Initially open branches when uncontrolled"],
        &["selected", "Option<Vec<String>>", "Controlled selection. Pair with on_selected_change"],
        &["default_selected", "Vec<String>", "Initial selection when uncontrolled"],
        &["on_expanded_change", "Callback<Vec<String>>", "Called when a branch opens or closes"],
        &["on_selected_change", "Callback<Vec<String>>", "Called when the selection changes"],
    ],
};
