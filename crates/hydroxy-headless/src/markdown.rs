//! Markdown to a plain node tree that adapters render as real elements.
//! `CommonMark` plus GFM tables, task lists and strikethrough.
//!
//! Safe by construction: raw HTML in the source comes back as text (adapters escape
//! it), and link and image URLs keep only `http`, `https`, `mailto` and relative ones.

use std::collections::HashMap;

use pulldown_cmark::{Alignment, CodeBlockKind, Event, LinkType, Options, Parser, Tag};

/// The `data-scope` value on the markdown root.
pub const SCOPE: &str = "markdown";

/// Column alignment in a table.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Align {
    /// No alignment given.
    #[default]
    None,
    /// `:--`
    Left,
    /// `:-:`
    Center,
    /// `--:`
    Right,
}

impl Align {
    /// The CSS `text-align` value, if any.
    #[must_use]
    pub const fn as_css(self) -> Option<&'static str> {
        match self {
            Align::None => None,
            Align::Left => Some("left"),
            Align::Center => Some("center"),
            Align::Right => Some("right"),
        }
    }
}

/// One node of rendered markdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// Plain text. Raw HTML from the source also lands here, to be shown escaped.
    Text(String),
    /// Inline code.
    Code(String),
    /// A hard line break (`<br>`).
    HardBreak,
    /// A thematic break (`<hr>`).
    Rule,
    /// A paragraph.
    Paragraph(Vec<Node>),
    /// A heading, level 1 to 6, with a slug id unique within the document.
    Heading {
        /// 1 to 6.
        level: u8,
        /// Slug of the heading text, such as `getting-started`.
        id: String,
        /// Inline content.
        children: Vec<Node>,
    },
    /// A block quote.
    Quote(Vec<Node>),
    /// A fenced or indented code block.
    CodeBlock {
        /// First word of the fence info string, if any.
        lang: Option<String>,
        /// The code, without the final newline.
        code: String,
    },
    /// A list of [`Node::Item`]s. Ordered when `start` is set.
    List {
        /// First number of an ordered list.
        start: Option<u64>,
        /// The items.
        children: Vec<Node>,
    },
    /// A list item. `checked` is set for task list items.
    Item {
        /// `Some` for `- [ ]` / `- [x]` items.
        checked: Option<bool>,
        /// Item content.
        children: Vec<Node>,
    },
    /// A table of [`Node::Row`]s; the first is the header row.
    Table(Vec<Node>),
    /// A table row of [`Node::Cell`]s.
    Row {
        /// Whether this is the header row.
        head: bool,
        /// The cells.
        children: Vec<Node>,
    },
    /// A table cell.
    Cell {
        /// Header cells render as `<th>`.
        head: bool,
        /// Column alignment.
        align: Align,
        /// Inline content.
        children: Vec<Node>,
    },
    /// Emphasis (`<em>`).
    Emphasis(Vec<Node>),
    /// Strong emphasis (`<strong>`).
    Strong(Vec<Node>),
    /// Strikethrough (`<del>`).
    Strike(Vec<Node>),
    /// A link. `href` is `None` when the URL was unsafe; render the text only.
    Link {
        /// Safe URL, or `None`.
        href: Option<String>,
        /// Title attribute, if given.
        title: Option<String>,
        /// Link text.
        children: Vec<Node>,
    },
    /// An image. `src` is `None` when the URL was unsafe; render the alt text only.
    Image {
        /// Safe URL, or `None`.
        src: Option<String>,
        /// Alt text.
        alt: String,
        /// Title attribute, if given.
        title: Option<String>,
    },
}

/// `url` if it is safe to put in an `href` or `src`: `http`, `https`, `mailto`, or
/// relative (no scheme). Anything with control characters is refused, since
/// browsers strip them and `java\tscript:` would otherwise slip through.
#[must_use]
pub fn safe_url(url: &str) -> Option<String> {
    let url = url.trim();
    if url.chars().any(char::is_control) {
        return None;
    }
    // A scheme is letters/digits/+.- before the first ':' with no '/', '?' or '#' ahead of it.
    let scheme_end = url.find([':', '/', '?', '#']).filter(|&i| url.as_bytes()[i] == b':');
    if let Some(end) = scheme_end {
        let scheme = &url[..end];
        let is_scheme = scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
        if is_scheme && !["http", "https", "mailto"].iter().any(|s| s.eq_ignore_ascii_case(scheme)) {
            return None;
        }
    }
    Some(url.to_string())
}

/// GitHub-style heading slug: lowercase, letters, digits, `-` and `_` kept,
/// spaces become `-`, everything else dropped.
#[must_use]
pub fn slug(text: &str) -> String {
    text.trim()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            c if c.is_alphanumeric() || c == '-' || c == '_' => Some(c),
            _ => None,
        })
        .flat_map(char::to_lowercase)
        .collect()
}

/// The text of `nodes` with all markup removed.
#[must_use]
pub fn plain_text(nodes: &[Node]) -> String {
    let mut out = String::new();
    collect_text(nodes, &mut out);
    out
}

fn collect_text(nodes: &[Node], out: &mut String) {
    for node in nodes {
        match node {
            Node::Text(t) | Node::Code(t) | Node::CodeBlock { code: t, .. } => out.push_str(t),
            Node::Image { alt, .. } => out.push_str(alt),
            Node::HardBreak => out.push('\n'),
            Node::Rule => {}
            Node::Paragraph(c)
            | Node::Quote(c)
            | Node::Table(c)
            | Node::Emphasis(c)
            | Node::Strong(c)
            | Node::Strike(c)
            | Node::Heading { children: c, .. }
            | Node::List { children: c, .. }
            | Node::Item { children: c, .. }
            | Node::Row { children: c, .. }
            | Node::Cell { children: c, .. }
            | Node::Link { children: c, .. } => collect_text(c, out),
        }
    }
}

/// An open container while parsing.
enum Open {
    Root,
    Paragraph,
    Heading(u8),
    Quote,
    CodeBlock(Option<String>),
    List(Option<u64>),
    Item(Option<bool>),
    Table,
    Row(bool),
    Cell,
    Emphasis,
    Strong,
    Strike,
    Link(Option<String>, Option<String>),
    Image(Option<String>, Option<String>),
    /// Anything else: its children join the parent.
    Transparent,
}

/// Parse `source` into block nodes.
#[must_use]
pub fn parse(source: &str) -> Vec<Node> {
    let options = Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;
    let mut stack: Vec<(Open, Vec<Node>)> = vec![(Open::Root, Vec::new())];
    let mut slugs: HashMap<String, usize> = HashMap::new();
    let mut aligns: Vec<Align> = Vec::new();
    let mut cell = 0;

    for event in Parser::new_ext(source, options) {
        let node = match event {
            Event::Start(tag) => {
                stack.push((open(tag, &mut aligns, &mut cell), Vec::new()));
                continue;
            }
            Event::End(_) => {
                let Some((open, children)) = stack.pop() else { break };
                match open {
                    Open::Root => {
                        stack.push((Open::Root, children));
                        continue;
                    }
                    Open::Transparent => {
                        if let Some((_, parent)) = stack.last_mut() {
                            parent.extend(children);
                        }
                        continue;
                    }
                    Open::Heading(level) => {
                        let base = slug(&plain_text(&children));
                        let seen = slugs.entry(base.clone()).or_insert(0);
                        let id = if *seen == 0 { base } else { format!("{base}-{seen}") };
                        *seen += 1;
                        Node::Heading { level, id, children }
                    }
                    Open::Cell => {
                        let head = matches!(stack.last(), Some((Open::Row(true), _)));
                        let align = aligns.get(cell).copied().unwrap_or_default();
                        cell += 1;
                        Node::Cell { head, align, children }
                    }
                    open => close(open, children),
                }
            }
            // Raw HTML is kept as text, so it shows escaped and never runs.
            Event::Text(t)
            | Event::Html(t)
            | Event::InlineHtml(t)
            | Event::FootnoteReference(t)
            | Event::InlineMath(t)
            | Event::DisplayMath(t) => Node::Text(t.into_string()),
            Event::Code(t) => Node::Code(t.into_string()),
            Event::SoftBreak => Node::Text("\n".into()),
            Event::HardBreak => Node::HardBreak,
            Event::Rule => Node::Rule,
            Event::TaskListMarker(done) => {
                if let Some((Open::Item(checked), _)) =
                    stack.iter_mut().rev().find(|(open, _)| matches!(open, Open::Item(_)))
                {
                    *checked = Some(done);
                }
                continue;
            }
        };
        if let Some((_, parent)) = stack.last_mut() {
            parent.push(node);
        }
    }
    stack.into_iter().next().map(|(_, nodes)| nodes).unwrap_or_default()
}

/// The container a start tag opens.
fn open(tag: Tag, aligns: &mut Vec<Align>, cell: &mut usize) -> Open {
    match tag {
        Tag::Paragraph | Tag::HtmlBlock => Open::Paragraph,
        Tag::Heading { level, .. } => Open::Heading(level as u8),
        Tag::BlockQuote(_) => Open::Quote,
        Tag::CodeBlock(kind) => Open::CodeBlock(match kind {
            CodeBlockKind::Fenced(info) => info.split_whitespace().next().map(str::to_string),
            CodeBlockKind::Indented => None,
        }),
        Tag::List(start) => Open::List(start),
        Tag::Item => Open::Item(None),
        Tag::Table(a) => {
            *aligns = a
                .into_iter()
                .map(|a| match a {
                    Alignment::None => Align::None,
                    Alignment::Left => Align::Left,
                    Alignment::Center => Align::Center,
                    Alignment::Right => Align::Right,
                })
                .collect();
            Open::Table
        }
        Tag::TableHead | Tag::TableRow => {
            *cell = 0;
            Open::Row(matches!(tag, Tag::TableHead))
        }
        Tag::TableCell => Open::Cell,
        Tag::Emphasis => Open::Emphasis,
        Tag::Strong => Open::Strong,
        Tag::Strikethrough => Open::Strike,
        Tag::Link { link_type, dest_url, title, .. } => {
            let dest = if link_type == LinkType::Email && !dest_url.starts_with("mailto:") {
                format!("mailto:{dest_url}")
            } else {
                dest_url.to_string()
            };
            Open::Link(safe_url(&dest), non_empty(&title))
        }
        Tag::Image { dest_url, title, .. } => Open::Image(safe_url(&dest_url), non_empty(&title)),
        _ => Open::Transparent,
    }
}

/// The node for a closed container that needs no document state.
fn close(open: Open, children: Vec<Node>) -> Node {
    match open {
        Open::CodeBlock(lang) => {
            let mut code = plain_text(&children);
            if code.ends_with('\n') {
                code.pop();
            }
            Node::CodeBlock { lang, code }
        }
        Open::Quote => Node::Quote(children),
        Open::List(start) => Node::List { start, children },
        Open::Item(checked) => Node::Item { checked, children },
        Open::Table => Node::Table(children),
        Open::Row(head) => Node::Row { head, children },
        Open::Emphasis => Node::Emphasis(children),
        Open::Strong => Node::Strong(children),
        Open::Strike => Node::Strike(children),
        Open::Link(href, title) => Node::Link { href, title, children },
        Open::Image(src, title) => Node::Image { src, alt: plain_text(&children), title },
        // Handled by the caller; a paragraph is the safe fallback.
        Open::Paragraph | Open::Root | Open::Transparent | Open::Heading(_) | Open::Cell => Node::Paragraph(children),
    }
}

fn non_empty(s: &str) -> Option<String> {
    (!s.is_empty()).then(|| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Node {
        Node::Text(s.into())
    }

    #[test]
    fn headings_get_unique_slugs() {
        let nodes = parse("# Hello, World!\n\n## Hello world\n\n## Hello world");
        let ids: Vec<_> = nodes
            .iter()
            .map(|n| match n {
                Node::Heading { id, .. } => id.as_str(),
                _ => "",
            })
            .collect();
        assert_eq!(ids, ["hello-world", "hello-world-1", "hello-world-2"]);
    }

    #[test]
    fn inline_marks() {
        assert_eq!(
            parse("**a** _b_ ~~c~~ `d`"),
            vec![Node::Paragraph(vec![
                Node::Strong(vec![text("a")]),
                text(" "),
                Node::Emphasis(vec![text("b")]),
                text(" "),
                Node::Strike(vec![text("c")]),
                text(" "),
                Node::Code("d".into()),
            ])]
        );
    }

    #[test]
    fn raw_html_stays_text() {
        let nodes = parse("<script>alert(1)</script>\n\nhi <b>there</b>");
        assert!(matches!(&nodes[0], Node::Paragraph(c) if plain_text(c).contains("<script>")));
        assert_eq!(plain_text(&nodes[1..]), "hi <b>there</b>");
    }

    #[test]
    fn unsafe_links_lose_their_href() {
        let nodes = parse("[x](javascript:alert(1)) [y](JaVaScRiPt:alert(1)) [z](/docs) <me@x.com>");
        let Node::Paragraph(c) = &nodes[0] else { panic!("expected Paragraph, got {:?}", nodes[0]) };
        let hrefs: Vec<_> = c
            .iter()
            .filter_map(|n| match n {
                Node::Link { href, .. } => Some(href.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(hrefs, [None, None, Some("/docs".into()), Some("mailto:me@x.com".into())]);
    }

    #[test]
    fn url_rules() {
        assert!(safe_url("https://a.b/c").is_some());
        assert!(safe_url("mailto:a@b.c").is_some());
        assert!(safe_url("#top").is_some());
        assert!(safe_url("page?a=b:c").is_some());
        assert!(safe_url("data:text/html,x").is_none());
        assert!(safe_url("java\tscript:alert(1)").is_none());
        assert!(safe_url("  vbscript:x").is_none());
    }

    #[test]
    fn code_fences_and_tasks() {
        let nodes = parse("```rust extra\nfn main() {}\n```\n\n- [x] done\n- [ ] todo\n- plain");
        assert_eq!(nodes[0], Node::CodeBlock { lang: Some("rust".into()), code: "fn main() {}".into() });
        let Node::List { start: None, children } = &nodes[1] else {
            panic!("expected unordered List, got {:?}", nodes[1])
        };
        let checks: Vec<_> = children
            .iter()
            .map(|n| match n {
                Node::Item { checked, .. } => *checked,
                other => panic!("expected Item, got {other:?}"),
            })
            .collect();
        assert_eq!(checks, [Some(true), Some(false), None]);
    }

    #[test]
    fn tables_carry_alignment() {
        let nodes = parse("| a | b |\n|:--|--:|\n| 1 | 2 |");
        let Node::Table(rows) = &nodes[0] else { panic!("expected Table, got {:?}", nodes[0]) };
        assert!(matches!(&rows[0], Node::Row { head: true, .. }));
        let Node::Row { head: false, children } = &rows[1] else { panic!("expected body Row, got {:?}", rows[1]) };
        assert!(matches!(&children[1], Node::Cell { head: false, align: Align::Right, .. }));
    }
}
