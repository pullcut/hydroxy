use leptos::prelude::*;
use hydroxy_leptos::table_of_contents::{Item, TableOfContents};

pub fn demo() -> AnyView {
    let items = vec![
        Item::new("toc-demo-intro", "Introduction", 1),
        Item::new("toc-demo-install", "Installation", 1),
        Item::new("toc-demo-cargo", "With Cargo", 2),
        Item::new("toc-demo-usage", "Usage", 1),
        Item::new("toc-demo-config", "Configuration", 1),
    ];
    let heading = |id: &'static str, text: &'static str| {
        view! {
            <h3 id=id style="margin: 0 0 var(--space-3); font: var(--text-h4); scroll-margin-top: var(--ox-toc-heading-offset, 72px);">
                {text}
            </h3>
            <p style="margin: 0 0 var(--space-8); min-height: 140px; color: var(--text-2);">
                "Scroll the page: the outline follows the heading you are reading, and a link jumps to its section."
            </p>
        }
    };

    view! {
        <div style="display: grid; grid-template-columns: minmax(0, 1fr) 200px; gap: var(--space-7); align-items: start;">
            <div>
                {heading("toc-demo-intro", "Introduction")} {heading("toc-demo-install", "Installation")}
                {heading("toc-demo-cargo", "With Cargo")} {heading("toc-demo-usage", "Usage")}
                {heading("toc-demo-config", "Configuration")}
            </div>
            <div style="position: sticky; top: 72px;">
                <TableOfContents items=items spy=true />
            </div>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "table-of-contents",
    name: "Table of contents",
    group: "Navigation",
    summary: "An in-page outline of real hash links. The host passes the headings and, optionally, the current one; turn on the spy to follow the page's scroll instead. The last heading wins at the bottom of the page.",
    demo,
    leptos: include_str!("table_of_contents.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/table_of_contents.rs"),
    parts: &[
        &["TableOfContents", "root", "A nav landmark named On this page"],
        &["(inside)", "list", "An ordered list of items"],
        &["(inside)", "item", "data-level is the outline depth, which indents the link"],
        &["(inside)", "link", "A plain #hash anchor. aria-current=true on the current heading"],
    ],
    keys: &[
        &["Enter", "On a link: jump to the heading"],
        &["Tab", "Moves through the links"],
    ],
    props: &[
        &["items", "Vec<Item>", "id, label and level for each heading. Blank and duplicate ids are dropped"],
        &["active_id", "Option<String>", "The current heading, owned by the host. Wins over the spy"],
        &["label", "Option<String>", "Accessible name. Default On this page"],
        &["heading_offset", "f64", "Sticky-header clearance in pixels: the spy's reading line. Default 72"],
        &["spy", "bool", "Follow the page's scroll position"],
        &["on_navigate", "Callback<String>", "A link was followed; the value is the heading id"],
    ],
};
