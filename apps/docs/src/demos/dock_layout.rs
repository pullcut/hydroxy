use leptos::prelude::*;
use oxide_leptos::dock_layout::{DockLayout, DockPanel, Node, SplitAxis};

fn text(body: &'static str) -> impl IntoView {
    view! { <p style="margin: 0; font: var(--text-sm); color: var(--text-2); white-space: pre-line;">{body}</p> }
}

pub fn demo() -> AnyView {
    let (closed, set_closed) = signal(String::from("Nothing closed yet."));

    view! {
        <div style="display: grid; gap: var(--space-3); grid-template-columns: minmax(0, 1fr);">
            <div style="height: 380px;">
                <DockLayout
                    panels=vec![
                        DockPanel::new("explorer", "Explorer", || text("src/\n  main.rs\n  lib.rs\nCargo.toml")),
                        DockPanel::new("search", "Search", || text("No results.")),
                        DockPanel::new("main", "main.rs", || text("fn main() {\n    println!(\"hello\");\n}")),
                        DockPanel::new("lib", "lib.rs", || text("pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}")),
                        DockPanel::new("cargo", "Cargo.toml", || text("[package]\nname = \"demo\"")),
                        DockPanel::new("terminal", "Terminal", || text("$ cargo test\n   Compiling demo v0.1.0\n    Finished test")),
                        DockPanel::new("output", "Output", || text("Build finished in 1.2s.")),
                    ]
                    locked=vec!["explorer".to_string()]
                    default_tree=Node::split(
                        "root",
                        SplitAxis::Horizontal,
                        vec![
                            Node::tabs("side", &["explorer", "search"]),
                            Node::split(
                                "work",
                                SplitAxis::Vertical,
                                vec![
                                    Node::tabs("editor", &["main", "lib", "cargo"]),
                                    Node::tabs("bottom", &["terminal", "output"]),
                                ],
                            ),
                        ],
                    )
                    on_close=move |id: String| set_closed.set(format!("Closed {id}."))
                />
            </div>
            <p style="margin: 0; font: var(--text-label); color: var(--text-3);">{closed}</p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "dock-layout",
    name: "Dock layout",
    group: "Layout",
    summary: "An IDE-style workspace: resizable splits whose panes are tab groups. Drag tabs to reorder them or move them between groups, or use the keyboard.",
    demo,
    leptos: include_str!("dock_layout.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/dock_layout.rs"),
    parts: &[
        &["DockLayout", "root", "Fills its parent: give the parent a height"],
        &["", "split", "Wraps a ResizablePanels row or column of children"],
        &["", "group", "One pane: a tab strip over its panels. data-empty when it has none"],
        &["", "tablist", "role=tablist. data-drop-end when a drag would land after the last tab"],
        &["", "tab", "role=tab, draggable. data-state=active|inactive, data-dragging, data-drop-before"],
        &["", "close", "Button inside a tab. Absent for locked panels"],
        &["", "panel", "role=tabpanel. All of a group's panels stay mounted; only the shown one is visible"],
        &["", "instructions", "Visually hidden keyboard help, linked from every tab by aria-describedby"],
        &["", "live", "role=status region that announces keyboard moves"],
    ],
    keys: &[
        &["ArrowLeft / ArrowRight", "Show the previous or next tab in the group"],
        &["Home / End", "Show the first or last tab"],
        &["Alt + ArrowLeft / ArrowRight", "Move the tab earlier or later in its group"],
        &["Alt + ArrowUp / ArrowDown", "Send the tab to the previous or next group"],
        &["Delete / Backspace", "Close the tab, unless its panel is locked"],
    ],
    props: &[
        &["panels", "Vec<DockPanel>", "Required. DockPanel::new(id, title, content). Every id the tree names"],
        &["default_tree", "Node", "Required. Node::split(id, axis, children) and Node::tabs(id, &[panel ids])"],
        &["tree", "Option<Node>", "Controlled tree. Pair with on_layout_change"],
        &["locked", "Vec<String>", "Panels that cannot be closed"],
        &["on_layout_change", "Callback<Node>", "Called with the whole tree when tabs move, close or change"],
        &["on_close", "Callback<String>", "Called with a panel's id when it closes"],
    ],
};
