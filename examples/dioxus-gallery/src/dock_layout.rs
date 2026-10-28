use dioxus::prelude::*;
use hydroxy_dioxus::dock_layout::{DockLayout, DockPanel, Node, SplitAxis};

fn text(body: &'static str) -> Element {
    rsx! {
        p { style: "margin: 0; font: var(--text-sm); color: var(--text-2); white-space: pre-line;", {body} }
    }
}

#[component]
pub fn DockLayoutDemo() -> Element {
    let mut closed = use_signal(|| String::from("Nothing closed yet."));

    rsx! {
        div { style: "display: grid; gap: var(--space-3);",
            div { style: "height: 380px;",
                DockLayout {
                    panels: vec![
                        DockPanel::new("explorer", "Explorer", text("src/\n  main.rs\n  lib.rs\nCargo.toml")),
                        DockPanel::new("search", "Search", text("No results.")),
                        DockPanel::new("main", "main.rs", text("fn main() {\n    println!(\"hello\");\n}")),
                        DockPanel::new("lib", "lib.rs", text("pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}")),
                        DockPanel::new("cargo", "Cargo.toml", text("[package]\nname = \"demo\"")),
                        DockPanel::new("terminal", "Terminal", text("$ cargo test\n   Compiling demo v0.1.0\n    Finished test")),
                        DockPanel::new("output", "Output", text("Build finished in 1.2s.")),
                    ],
                    locked: vec!["explorer".to_string()],
                    default_tree: Node::split(
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
                    ),
                    on_close: move |id: String| closed.set(format!("Closed {id}.")),
                }
            }
            p { style: "margin: 0; font: var(--text-label); color: var(--text-3);", "{closed}" }
        }
    }
}
