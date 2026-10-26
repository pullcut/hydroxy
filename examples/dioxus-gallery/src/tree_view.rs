use dioxus::prelude::*;
use oxide_dioxus::tree_view::{SelectionMode, TreeNode, TreeView};

fn files() -> Vec<TreeNode> {
    vec![
        TreeNode::new("research", "Research").children(vec![
            TreeNode::new("interviews", "Interviews").children(vec![
                TreeNode::new("q3", "interviews-q3.pdf"),
                TreeNode::new("q4", "interviews-q4.pdf"),
            ]),
            TreeNode::new("tickets", "tickets-sept.csv"),
        ]),
        TreeNode::new("summaries", "Summaries").children(vec![
            TreeNode::new("weekly", "weekly-digest.md"),
            TreeNode::new("board", "board-update.md"),
        ]),
        TreeNode::new("archive", "Archive").children(vec![TreeNode::new("old", "2025-notes.md")]).disabled(),
        TreeNode::new("readme", "README.md"),
    ]
}

#[component]
pub fn TreeViewDemo() -> Element {
    let mut selected = use_signal(|| vec!["q3".to_string()]);

    rsx! {
        div { style: "display: grid; gap: var(--space-8); max-width: 360px;",
            // Uncontrolled expansion, controlled single selection.
            div {
                TreeView {
                    label: "Project files",
                    nodes: files(),
                    default_expanded: vec!["research".to_string(), "interviews".to_string()],
                    selected: Some(selected()),
                    on_selected_change: move |v| selected.set(v),
                }
                p { "Selected: {selected().join(\", \")}" }
            }

            // Multiple selection: Enter, Space or click toggles each item.
            TreeView {
                label: "Files to share",
                nodes: files(),
                selection_mode: SelectionMode::Multiple,
                default_expanded: vec!["summaries".to_string()],
                default_selected: vec!["weekly".to_string(), "board".to_string()],
            }
        }
    }
}
