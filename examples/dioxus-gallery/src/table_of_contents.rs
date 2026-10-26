use dioxus::prelude::*;
use oxide_dioxus::table_of_contents::{Item, TableOfContents};

#[component]
pub fn TableOfContentsDemo() -> Element {
    let items = vec![
        Item::new("toc-demo-intro", "Introduction", 1),
        Item::new("toc-demo-install", "Installation", 1),
        Item::new("toc-demo-cargo", "With Cargo", 2),
        Item::new("toc-demo-usage", "Usage", 1),
        Item::new("toc-demo-config", "Configuration", 1),
    ];

    rsx! {
        div { style: "display: grid; grid-template-columns: minmax(0, 1fr) 200px; gap: var(--space-7); align-items: start;",
            div {
                for item in items.clone() {
                    h3 {
                        key: "{item.id}",
                        id: "{item.id}",
                        style: "margin: 0 0 var(--space-3); font: var(--text-h4); scroll-margin-top: var(--ox-toc-heading-offset, 72px);",
                        "{item.label}"
                    }
                    p { style: "margin: 0 0 var(--space-8); min-height: 140px; color: var(--text-2);",
                        "Scroll the page: the outline follows the heading you are reading, and a link jumps to its section."
                    }
                }
            }
            div { style: "position: sticky; top: 72px;", TableOfContents { items, spy: true } }
        }
    }
}
