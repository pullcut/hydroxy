//! `<Markdown>`: renders a markdown string as styled elements. `CommonMark` plus GFM
//! tables, task lists and strikethrough. Raw HTML shows as text and unsafe link URLs
//! are dropped, so untrusted input is fine.

use hydroxy_headless::markdown::{self as h, Node};
use leptos::prelude::*;

use crate::code_block::CodeBlock;

/// Rendered markdown. Re-renders when `source` changes.
#[component]
pub fn Markdown(
    /// The markdown text.
    #[prop(into)]
    source: Signal<String>,
) -> impl IntoView {
    view! {
        <div data-scope=h::SCOPE data-part="root">
            {move || source.with(|s| h::parse(s)).iter().map(render).collect_view()}
        </div>
    }
}

fn all(nodes: &[Node]) -> AnyView {
    nodes.iter().map(render).collect_view().into_any()
}

fn render(node: &Node) -> AnyView {
    match node {
        Node::Text(t) => t.clone().into_any(),
        Node::Code(t) => view! { <code>{t.clone()}</code> }.into_any(),
        Node::HardBreak => view! { <br /> }.into_any(),
        Node::Rule => view! { <hr /> }.into_any(),
        Node::Paragraph(c) => view! { <p>{all(c)}</p> }.into_any(),
        Node::Heading { level, id, children } => {
            let (id, c) = (id.clone(), all(children));
            match level {
                1 => view! { <h1 id=id>{c}</h1> }.into_any(),
                2 => view! { <h2 id=id>{c}</h2> }.into_any(),
                3 => view! { <h3 id=id>{c}</h3> }.into_any(),
                4 => view! { <h4 id=id>{c}</h4> }.into_any(),
                5 => view! { <h5 id=id>{c}</h5> }.into_any(),
                _ => view! { <h6 id=id>{c}</h6> }.into_any(),
            }
        }
        Node::Quote(c) => view! { <blockquote>{all(c)}</blockquote> }.into_any(),
        Node::CodeBlock { lang, code } => {
            let lang = lang.clone().unwrap_or_else(|| "text".into());
            view! { <CodeBlock code=code.clone() lang=lang /> }.into_any()
        }
        Node::List { start: Some(n), children } => view! { <ol start=*n>{all(children)}</ol> }.into_any(),
        Node::List { start: None, children } => view! { <ul>{all(children)}</ul> }.into_any(),
        Node::Item { checked: Some(done), children } => view! {
            <li data-task="">
                <input type="checkbox" disabled=true checked=*done aria-label=if *done { "Done" } else { "To do" } />
                {all(children)}
            </li>
        }
        .into_any(),
        Node::Item { checked: None, children } => view! { <li>{all(children)}</li> }.into_any(),
        Node::Table(rows) => {
            let split = rows.iter().take_while(|r| matches!(r, Node::Row { head: true, .. })).count();
            let (head, body) = rows.split_at(split);
            view! {
                <div data-part="table">
                    <table>
                        <thead>{all(head)}</thead>
                        <tbody>{all(body)}</tbody>
                    </table>
                </div>
            }
            .into_any()
        }
        Node::Row { children, .. } => view! { <tr>{all(children)}</tr> }.into_any(),
        Node::Cell { head, align, children } => {
            let style = align.as_css().map(|a| format!("text-align: {a}"));
            if *head {
                view! { <th style=style>{all(children)}</th> }.into_any()
            } else {
                view! { <td style=style>{all(children)}</td> }.into_any()
            }
        }
        Node::Emphasis(c) => view! { <em>{all(c)}</em> }.into_any(),
        Node::Strong(c) => view! { <strong>{all(c)}</strong> }.into_any(),
        Node::Strike(c) => view! { <del>{all(c)}</del> }.into_any(),
        Node::Link { href: Some(href), title, children } => {
            view! { <a href=href.clone() title=title.clone()>{all(children)}</a> }.into_any()
        }
        Node::Link { href: None, children, .. } => all(children),
        Node::Image { src: Some(src), alt, title } => {
            view! { <img src=src.clone() alt=alt.clone() title=title.clone() loading="lazy" /> }.into_any()
        }
        Node::Image { src: None, alt, .. } => alt.clone().into_any(),
    }
}
