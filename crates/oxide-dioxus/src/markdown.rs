//! `Markdown`: renders a markdown string as styled elements. `CommonMark` plus GFM
//! tables, task lists and strikethrough. Raw HTML shows as text and unsafe link URLs
//! are dropped, so untrusted input is fine.

use dioxus::prelude::*;
use oxide_headless::markdown::{self as h, Node};

use crate::code_block::CodeBlock;

/// Rendered markdown.
#[component]
pub fn Markdown(
    /// The markdown text.
    source: String,
) -> Element {
    let nodes = h::parse(&source);
    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", {all(&nodes)} }
    }
}

fn all(nodes: &[Node]) -> Element {
    rsx! {
        for node in nodes {
            {render(node)}
        }
    }
}

fn render(node: &Node) -> Element {
    match node {
        Node::Text(t) => rsx! { "{t}" },
        Node::Code(t) => rsx! { code { "{t}" } },
        Node::HardBreak => rsx! { br {} },
        Node::Rule => rsx! { hr {} },
        Node::Paragraph(c) => rsx! { p { {all(c)} } },
        Node::Heading { level, id, children } => {
            let c = all(children);
            match level {
                1 => rsx! { h1 { id: "{id}", {c} } },
                2 => rsx! { h2 { id: "{id}", {c} } },
                3 => rsx! { h3 { id: "{id}", {c} } },
                4 => rsx! { h4 { id: "{id}", {c} } },
                5 => rsx! { h5 { id: "{id}", {c} } },
                _ => rsx! { h6 { id: "{id}", {c} } },
            }
        }
        Node::Quote(c) => rsx! { blockquote { {all(c)} } },
        Node::CodeBlock { lang, code } => rsx! { CodeBlock { code: code.clone(), lang: lang.clone() } },
        Node::List { start: Some(n), children } => rsx! { ol { start: "{n}", {all(children)} } },
        Node::List { start: None, children } => rsx! { ul { {all(children)} } },
        Node::Item { checked: Some(done), children } => rsx! {
            li { "data-task": "",
                input {
                    r#type: "checkbox",
                    disabled: true,
                    checked: *done,
                    "aria-label": if *done { "Done" } else { "To do" },
                }
                {all(children)}
            }
        },
        Node::Item { checked: None, children } => rsx! { li { {all(children)} } },
        Node::Table(rows) => {
            let split = rows.iter().take_while(|r| matches!(r, Node::Row { head: true, .. })).count();
            let (head, body) = rows.split_at(split);
            rsx! {
                div { "data-part": "table",
                    table {
                        thead { {all(head)} }
                        tbody { {all(body)} }
                    }
                }
            }
        }
        Node::Row { children, .. } => rsx! { tr { {all(children)} } },
        Node::Cell { head, align, children } => {
            let style = align.as_css().map(|a| format!("text-align: {a}"));
            if *head {
                rsx! { th { style, {all(children)} } }
            } else {
                rsx! { td { style, {all(children)} } }
            }
        }
        Node::Emphasis(c) => rsx! { em { {all(c)} } },
        Node::Strong(c) => rsx! { strong { {all(c)} } },
        Node::Strike(c) => rsx! { del { {all(c)} } },
        Node::Link { href: Some(href), title, children } => rsx! {
            a { href: "{href}", title: title.clone(), {all(children)} }
        },
        Node::Link { href: None, children, .. } => all(children),
        Node::Image { src: Some(src), alt, title } => rsx! {
            img { src: "{src}", alt: "{alt}", title: title.clone(), loading: "lazy" }
        },
        Node::Image { src: None, alt, .. } => rsx! { "{alt}" },
    }
}
