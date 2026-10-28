//! The tree walk: row kinds, visible rows, expansion and search over a [`JsonValue`].

use std::collections::HashSet;
use std::hash::BuildHasher;

use super::value::{JsonValue, key_path, to_json};
use crate::text::plural;

/// Most children shown under one node before they are grouped into ranges.
pub const CHUNK: usize = 100;

/// What a row shows, for colouring (`data-type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonKind {
    /// `null`.
    Null,
    /// `true` / `false`.
    Bool,
    /// A number.
    Number,
    /// A string.
    String,
    /// An array.
    Array,
    /// An object.
    Object,
    /// A group of up to [`CHUNK`] children of a large container.
    Range,
}

impl JsonKind {
    /// The `data-type` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            JsonKind::Null => "null",
            JsonKind::Bool => "boolean",
            JsonKind::Number => "number",
            JsonKind::String => "string",
            JsonKind::Array => "array",
            JsonKind::Object => "object",
            JsonKind::Range => "range",
        }
    }
}

/// One node in the walk: a value, or a range of a container's children.
#[derive(Clone)]
pub(super) struct Node<'a> {
    pub(super) id: String,
    /// The value's path; for a range, its container's path.
    pub(super) path: String,
    key: Option<String>,
    /// `key` is an array index.
    index: bool,
    value: &'a JsonValue,
    /// `Some((start, end))`, end exclusive, for a range node.
    range: Option<(usize, usize)>,
}

impl<'a> Node<'a> {
    fn root(value: &'a JsonValue) -> Self {
        Node { id: "$".into(), path: "$".into(), key: None, index: false, value, range: None }
    }

    fn expandable(&self) -> bool {
        self.range.is_some() || !self.value.is_empty()
    }

    fn child(&self, i: usize) -> Node<'a> {
        match self.value {
            JsonValue::Array(items) => {
                let path = format!("{}[{i}]", self.path);
                Node { id: path.clone(), path, key: Some(i.to_string()), index: true, value: &items[i], range: None }
            }
            JsonValue::Object(entries) => {
                let (k, v) = &entries[i];
                let path = key_path(&self.path, k);
                Node { id: path.clone(), path, key: Some(k.clone()), index: false, value: v, range: None }
            }
            _ => unreachable!("only containers have children"),
        }
    }

    fn children(&self) -> Vec<Node<'a>> {
        if let Some((start, end)) = self.range {
            return (start..end).map(|i| self.child(i)).collect();
        }
        let len = self.value.len();
        if len <= CHUNK {
            return (0..len).map(|i| self.child(i)).collect();
        }
        (0..len)
            .step_by(CHUNK)
            .map(|start| {
                let end = (start + CHUNK).min(len);
                let label = format!("[{start}…{}]", end - 1);
                Node {
                    id: format!("{}{label}", self.path),
                    path: self.path.clone(),
                    key: Some(label),
                    index: false,
                    value: self.value,
                    range: Some((start, end)),
                }
            })
            .collect()
    }

    fn kind(&self) -> JsonKind {
        if self.range.is_some() {
            return JsonKind::Range;
        }
        match self.value {
            JsonValue::Null => JsonKind::Null,
            JsonValue::Bool(_) => JsonKind::Bool,
            JsonValue::Number(_) => JsonKind::Number,
            JsonValue::String(_) => JsonKind::String,
            JsonValue::Array(_) => JsonKind::Array,
            JsonValue::Object(_) => JsonKind::Object,
        }
    }

    /// The value as shown: JSON for primitives, a size summary for containers.
    fn text(&self) -> String {
        if let Some((start, end)) = self.range {
            return format!("{} items", end - start);
        }
        match self.value {
            JsonValue::Array(items) => format!("[{}]", plural(items.len(), "item")),
            JsonValue::Object(entries) => format!("{{{}}}", plural(entries.len(), "key")),
            v => to_json(v, false),
        }
    }

    /// Key or primitive value contains `q` (already lower case). Ranges never match.
    fn matches(&self, q: &str) -> bool {
        self.range.is_none()
            && (self.key.as_deref().is_some_and(|k| contains(k, q))
                || !matches!(self.value, JsonValue::Array(_) | JsonValue::Object(_)) && contains(&self.text(), q))
    }

    /// The value to copy: the subtree, or the slice for a range.
    pub(super) fn copy_text(&self) -> String {
        match (self.range, self.value) {
            (Some((s, e)), JsonValue::Array(items)) => to_json(&JsonValue::Array(items[s..e].to_vec()), true),
            (Some((s, e)), JsonValue::Object(entries)) => to_json(&JsonValue::Object(entries[s..e].to_vec()), true),
            _ => to_json(self.value, true),
        }
    }
}

/// Whether `text` contains `query`, ignoring case. `query` must already be lower case.
fn contains(text: &str, query: &str) -> bool {
    text.to_lowercase().contains(query)
}

/// One visible row, in display order.
#[allow(clippy::struct_excessive_bools, reason = "independent row flags")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JsonRow {
    /// Stable key: the path, or path plus range for range nodes.
    pub id: String,
    /// `$.a[2].b`. For a range, its container's path.
    pub path: String,
    /// Object key, array index or range label. `None` for the root.
    pub key: Option<String>,
    /// Array index rather than object key (shown without quotes).
    pub index: bool,
    /// For colouring.
    pub kind: JsonKind,
    /// The value as shown.
    pub text: String,
    /// Depth from 1 (`aria-level`).
    pub level: usize,
    /// Siblings including this one (`aria-setsize`).
    pub set_size: usize,
    /// Position among siblings from 1 (`aria-posinset`).
    pub pos_in_set: usize,
    /// Has children to show.
    pub expandable: bool,
    /// Children are showing.
    pub expanded: bool,
    /// Key or value contains the search query.
    pub matched: bool,
    /// Parent row id. `None` for the root.
    pub parent: Option<String>,
}

/// Level, set size, position in set, parent id.
type Meta = (usize, usize, usize, Option<String>);

fn walk_visible<'a, H: BuildHasher>(
    node: Node<'a>,
    meta: Meta,
    expanded: &HashSet<String, H>,
    out: &mut Vec<(Node<'a>, Meta)>,
) {
    let open = node.expandable() && expanded.contains(&node.id);
    let level = meta.0;
    let id = node.id.clone();
    let kids = if open { node.children() } else { Vec::new() };
    out.push((node, meta));
    let n = kids.len();
    for (i, kid) in kids.into_iter().enumerate() {
        walk_visible(kid, (level + 1, n, i + 1, Some(id.clone())), expanded, out);
    }
}

pub(super) fn visible_nodes<'a>(root: &'a JsonValue, expanded: &HashSet<String>) -> Vec<Node<'a>> {
    let mut out = Vec::new();
    walk_visible(Node::root(root), (1, 1, 1, None), expanded, &mut out);
    out.into_iter().map(|(n, _)| n).collect()
}

/// The rows on screen, in order, for `query` (any case; empty matches nothing).
#[must_use]
pub fn visible_rows<H: BuildHasher>(root: &JsonValue, expanded: &HashSet<String, H>, query: &str) -> Vec<JsonRow> {
    let mut out = Vec::new();
    walk_visible(Node::root(root), (1, 1, 1, None), expanded, &mut out);
    let q = query.trim().to_lowercase();
    out.into_iter()
        .map(|(node, (level, set_size, pos_in_set, parent))| {
            let expandable = node.expandable();
            let text = node.text();
            let kind = node.kind();
            let matched = !q.is_empty() && node.matches(&q);
            JsonRow {
                expanded: expandable && expanded.contains(&node.id),
                index: node.index,
                id: node.id,
                path: node.path,
                key: node.key,
                kind,
                text,
                level,
                set_size,
                pos_in_set,
                expandable,
                matched,
                parent,
            }
        })
        .collect()
}

/// Ids to open so every node down to `depth` levels (root is level 1) shows its children.
#[must_use]
pub fn expand_to(root: &JsonValue, depth: usize) -> HashSet<String> {
    fn go(node: &Node, level: usize, depth: usize, out: &mut HashSet<String>) {
        if level > depth || !node.expandable() {
            return;
        }
        out.insert(node.id.clone());
        for kid in node.children() {
            go(&kid, level + 1, depth, out);
        }
    }
    let mut out = HashSet::new();
    go(&Node::root(root), 1, depth, &mut out);
    out
}

/// Every node matching `query`, in document order, and the ids that must open to show them.
#[must_use]
pub fn find_matches(root: &JsonValue, query: &str) -> (Vec<String>, HashSet<String>) {
    fn go(node: &Node, q: &str, trail: &mut Vec<String>, hits: &mut Vec<String>, open: &mut HashSet<String>) {
        if node.matches(q) {
            hits.push(node.id.clone());
            open.extend(trail.iter().cloned());
        }
        if node.expandable() {
            trail.push(node.id.clone());
            for kid in node.children() {
                go(&kid, q, trail, hits, open);
            }
            trail.pop();
        }
    }
    let q = query.trim().to_lowercase();
    let (mut hits, mut open) = (Vec::new(), HashSet::new());
    if !q.is_empty() {
        go(&Node::root(root), &q, &mut Vec::new(), &mut hits, &mut open);
    }
    (hits, open)
}

/// Split `text` into runs that do (`true`) and don't match `query`, ignoring case.
/// Concatenating the runs gives back `text`.
#[must_use]
pub fn split_matches<'t>(text: &'t str, query: &str) -> Vec<(bool, &'t str)> {
    let q = query.trim().to_lowercase();
    let lower = text.to_lowercase();
    // byte offsets only line up when lower-casing keeps lengths (true for ASCII
    // and most scripts); otherwise the row is matched but not marked.
    if q.is_empty() || lower.len() != text.len() {
        return vec![(false, text)];
    }
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(i) = lower[at..].find(&q) {
        let (s, e) = (at + i, at + i + q.len());
        if !text.is_char_boundary(s) || !text.is_char_boundary(e) {
            break;
        }
        if s > at {
            out.push((false, &text[at..s]));
        }
        out.push((true, &text[s..e]));
        at = e;
    }
    if at < text.len() {
        out.push((false, &text[at..]));
    }
    out
}
