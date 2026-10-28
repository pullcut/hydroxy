//! `JsonInspector` owns a parsed JSON value shown as a tree: which nodes are open,
//! which row has focus, and a search that opens the way to every match. Containers
//! with more than [`CHUNK`] children show range nodes (`[0…99]`) instead, so a huge
//! array never renders more than a hundred rows per level.
//!
//! Also here: a small JSON parser ([`parse`]) with line:column errors, and a
//! serializer ([`to_json`]) for copying values back out.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::hash::BuildHasher;

use hydroxy_statechart::{Machine, Transition};

pub use crate::list::Direction;

/// Most children shown under one node before they are grouped into ranges.
pub const CHUNK: usize = 100;

/// Deepest nesting the parser accepts, so hostile input can't overflow the stack.
pub const MAX_DEPTH: usize = 512;

/// A parsed JSON value. Object keys keep their source order.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A number, kept as its source text so nothing is lost to float rounding.
    Number(String),
    /// A string, unescaped.
    String(String),
    /// An array.
    Array(Vec<JsonValue>),
    /// An object, in source order.
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// Number of children: items or entries. Zero for primitives.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            JsonValue::Array(a) => a.len(),
            JsonValue::Object(o) => o.len(),
            _ => 0,
        }
    }

    /// Whether this has no children (always true for primitives).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Where and why parsing failed. `line` and `column` count from 1, columns in chars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// What went wrong.
    pub message: String,
    /// Line of the problem.
    pub line: usize,
    /// Column of the problem.
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at line {}, column {}", self.message, self.line, self.column)
    }
}

impl std::error::Error for ParseError {}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
    depth: usize,
}

impl Parser<'_> {
    fn error(&self, message: impl Into<String>) -> ParseError {
        let before = &self.src[..self.pos.min(self.src.len())];
        let line = before.matches('\n').count() + 1;
        let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        ParseError { message: message.into(), line, column }
    }

    fn peek(&self) -> Option<u8> {
        self.src.as_bytes().get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, b: u8) -> Result<(), ParseError> {
        if self.peek() == Some(b) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.unexpected(&format!("'{}'", b as char)))
        }
    }

    fn unexpected(&self, wanted: &str) -> ParseError {
        match self.src[self.pos.min(self.src.len())..].chars().next() {
            None => self.error(format!("Unexpected end of input, expected {wanted}")),
            Some(c) => self.error(format!("Unexpected '{c}', expected {wanted}")),
        }
    }

    fn value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.nested(Self::object),
            Some(b'[') => self.nested(Self::array),
            Some(b'"') => self.string().map(JsonValue::String),
            Some(b't') => self.word("true", JsonValue::Bool(true)),
            Some(b'f') => self.word("false", JsonValue::Bool(false)),
            Some(b'n') => self.word("null", JsonValue::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.unexpected("a value")),
        }
    }

    fn nested(&mut self, f: fn(&mut Self) -> Result<JsonValue, ParseError>) -> Result<JsonValue, ParseError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(self.error("Nested too deeply"));
        }
        let v = f(self);
        self.depth -= 1;
        v
    }

    fn word(&mut self, word: &str, v: JsonValue) -> Result<JsonValue, ParseError> {
        if self.src[self.pos..].starts_with(word) {
            self.pos += word.len();
            Ok(v)
        } else {
            Err(self.unexpected("a value"))
        }
    }

    fn digits(&mut self) -> usize {
        let start = self.pos;
        while self.peek().is_some_and(|b| b.is_ascii_digit()) {
            self.pos += 1;
        }
        self.pos - start
    }

    fn number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                self.digits();
            }
            _ => return Err(self.unexpected("a digit")),
        }
        if self.peek() == Some(b'.') {
            self.pos += 1;
            if self.digits() == 0 {
                return Err(self.unexpected("a digit"));
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if self.digits() == 0 {
                return Err(self.unexpected("a digit"));
            }
        }
        Ok(JsonValue::Number(self.src[start..self.pos].to_string()))
    }

    fn hex4(&mut self) -> Result<u32, ParseError> {
        let hex = self.src.get(self.pos..self.pos + 4).filter(|h| h.bytes().all(|b| b.is_ascii_hexdigit()));
        let Some(hex) = hex else { return Err(self.error("Invalid \\u escape")) };
        self.pos += 4;
        Ok(u32::from_str_radix(hex, 16).unwrap_or(0))
    }

    fn string(&mut self) -> Result<String, ParseError> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let rest = &self.src[self.pos..];
            // Copy the run up to the next quote, backslash or control char in one go.
            let run = rest.find(|c: char| c == '"' || c == '\\' || c < ' ').unwrap_or(rest.len());
            out.push_str(&rest[..run]);
            self.pos += run;
            match self.peek() {
                None => return Err(self.error("Unterminated string")),
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.pos += 1;
                    let Some(e) = self.peek() else { return Err(self.error("Unterminated string")) };
                    self.pos += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let mut code = self.hex4()?;
                            if (0xD800..0xDC00).contains(&code) && self.src[self.pos..].starts_with("\\u") {
                                let save = self.pos;
                                self.pos += 2;
                                let low = self.hex4()?;
                                if (0xDC00..0xE000).contains(&low) {
                                    code = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                                } else {
                                    self.pos = save;
                                }
                            }
                            out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                        }
                        _ => {
                            self.pos -= 1;
                            return Err(self.error("Invalid escape"));
                        }
                    }
                }
                Some(_) => return Err(self.error("Control character in string")),
            }
        }
    }

    fn array(&mut self) -> Result<JsonValue, ParseError> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            items.push(self.value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Array(items));
                }
                _ => return Err(self.unexpected("',' or ']'")),
            }
        }
    }

    fn object(&mut self) -> Result<JsonValue, ParseError> {
        self.expect(b'{')?;
        let mut entries = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(self.unexpected("a string key"));
            }
            let key = self.string()?;
            self.skip_ws();
            self.expect(b':')?;
            entries.push((key, self.value()?));
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(entries));
                }
                _ => return Err(self.unexpected("',' or '}'")),
            }
        }
    }
}

/// Parse JSON text (RFC 8259).
///
/// # Errors
/// A [`ParseError`] naming the first problem and where it is.
pub fn parse(src: &str) -> Result<JsonValue, ParseError> {
    let mut p = Parser { src, pos: 0, depth: 0 };
    let v = p.value()?;
    p.skip_ws();
    if p.pos < src.len() {
        return Err(p.unexpected("end of input"));
    }
    Ok(v)
}

/// `s` as a quoted JSON string.
#[must_use]
pub fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Serialize `v`. `pretty` indents by two spaces per level.
#[must_use]
pub fn to_json(v: &JsonValue, pretty: bool) -> String {
    let mut out = String::new();
    write_json(&mut out, v, pretty, 0);
    out
}

fn write_json(out: &mut String, v: &JsonValue, pretty: bool, indent: usize) {
    let newline = |out: &mut String, n: usize| {
        if pretty {
            out.push('\n');
            out.push_str(&"  ".repeat(n));
        }
    };
    match v {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        JsonValue::Number(n) => out.push_str(n),
        JsonValue::String(s) => out.push_str(&quote(s)),
        JsonValue::Array(items) if items.is_empty() => out.push_str("[]"),
        JsonValue::Object(entries) if entries.is_empty() => out.push_str("{}"),
        JsonValue::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                newline(out, indent + 1);
                write_json(out, item, pretty, indent + 1);
            }
            newline(out, indent);
            out.push(']');
        }
        JsonValue::Object(entries) => {
            out.push('{');
            for (i, (k, item)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                newline(out, indent + 1);
                out.push_str(&quote(k));
                out.push_str(if pretty { ": " } else { ":" });
                write_json(out, item, pretty, indent + 1);
            }
            newline(out, indent);
            out.push('}');
        }
    }
}

/// The path of `key` under `parent`: `$.name` for identifiers, `$["a b"]` otherwise.
#[must_use]
pub fn key_path(parent: &str, key: &str) -> String {
    let mut chars = key.chars();
    let ident = chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
    if ident { format!("{parent}.{key}") } else { format!("{parent}[{}]", quote(key)) }
}

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
struct Node<'a> {
    id: String,
    /// The value's path; for a range, its container's path.
    path: String,
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
            && (self.key.as_deref().is_some_and(|k| contains(k, q)) || !matches!(self.value, JsonValue::Array(_) | JsonValue::Object(_)) && contains(&self.text(), q))
    }

    /// The value to copy: the subtree, or the slice for a range.
    fn copy_text(&self) -> String {
        match (self.range, self.value) {
            (Some((s, e)), JsonValue::Array(items)) => to_json(&JsonValue::Array(items[s..e].to_vec()), true),
            (Some((s, e)), JsonValue::Object(entries)) => to_json(&JsonValue::Object(entries[s..e].to_vec()), true),
            _ => to_json(self.value, true),
        }
    }
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 { format!("1 {word}") } else { format!("{n} {word}s") }
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

fn visible_nodes<'a>(root: &'a JsonValue, expanded: &HashSet<String>) -> Vec<Node<'a>> {
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

/// Props for [`JsonInspectorMachine`].
#[derive(Debug, Clone, PartialEq)]
pub struct JsonInspectorInput {
    /// The value to show.
    pub root: JsonValue,
    /// Levels open at start; the root is level 1. Default 2.
    pub expand_depth: usize,
}

impl Default for JsonInspectorInput {
    fn default() -> Self {
        Self { root: JsonValue::Null, expand_depth: 2 }
    }
}

/// What [`JsonInspectorMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JsonInspectorState {
    /// Ids of open nodes.
    pub expanded: HashSet<String>,
    /// The row with the roving tab stop.
    pub focused: Option<String>,
    /// Search text.
    pub query: String,
    /// Ids matching `query`, in document order.
    pub matches: Vec<String>,
    /// Index into `matches` of the last match jumped to.
    pub active_match: Option<usize>,
}

impl JsonInspectorState {
    /// The row that takes Tab: the focused one if visible, else the first.
    #[must_use]
    pub fn tab_stop<'r>(&self, rows: &'r [JsonRow]) -> Option<&'r str> {
        let focused = self.focused.as_deref().and_then(|f| rows.iter().find(|r| r.id == f));
        focused.or(rows.first()).map(|r| r.id.as_str())
    }
}

/// What the copy command carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyWhat {
    /// The value as pretty JSON.
    Value,
    /// The `JSONPath`.
    Path,
}

/// Inputs to [`JsonInspectorMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonInspectorEvent {
    /// Open or close a node.
    Toggle(String),
    /// A row took focus.
    Focus(String),
    /// Arrow up or down.
    Move(Direction),
    /// First row.
    Home,
    /// Last row.
    End,
    /// Arrow right: open, or step into the first child.
    ExpandOrEnter,
    /// Arrow left: close, or step out to the parent.
    CollapseOrExit,
    /// `*`: open every sibling of the focused row.
    ExpandSiblings,
    /// Open everything down to this many levels; 0 closes everything.
    ExpandToDepth(usize),
    /// Set the search text.
    SetQuery(String),
    /// Focus the next match (`true`) or the previous one.
    StepMatch(bool),
    /// Copy a row's value or path.
    Copy(String, CopyWhat),
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonInspectorCommand {
    /// Move DOM focus to this row.
    FocusRow(String),
    /// Put this text on the clipboard and say so.
    Copy(String, CopyWhat),
}

/// Expansion, focus and search over a JSON tree.
#[derive(Debug, Clone, Copy)]
pub struct JsonInspectorMachine;

type Step = Transition<JsonInspectorState, JsonInspectorCommand>;

fn focus(mut next: JsonInspectorState, id: String) -> Step {
    next.focused = Some(id.clone());
    Transition::with(next, vec![JsonInspectorCommand::FocusRow(id)])
}

impl Machine for JsonInspectorMachine {
    type Input = JsonInspectorInput;
    type State = JsonInspectorState;
    type Event = JsonInspectorEvent;
    type Command = JsonInspectorCommand;

    fn initial(input: &JsonInspectorInput) -> JsonInspectorState {
        JsonInspectorState { expanded: expand_to(&input.root, input.expand_depth), ..Default::default() }
    }

    #[allow(clippy::too_many_lines, reason = "one match arm per event")]
    fn transition(state: &JsonInspectorState, event: JsonInspectorEvent, input: &JsonInspectorInput) -> Step {
        let mut next = state.clone();
        let rows = || visible_rows(&input.root, &state.expanded, "");
        let current = |rows: &[JsonRow]| {
            let stop = state.tab_stop(rows)?;
            rows.iter().position(|r| r.id == stop)
        };
        match event {
            JsonInspectorEvent::Toggle(id) => {
                if !next.expanded.remove(&id) {
                    next.expanded.insert(id.clone());
                }
                next.focused = Some(id);
                Transition::to(next)
            }
            JsonInspectorEvent::Focus(id) => {
                next.focused = Some(id);
                Transition::to(next)
            }
            JsonInspectorEvent::Move(dir) => {
                let rows = rows();
                let Some(at) = current(&rows) else { return Transition::to(next) };
                let to = match dir {
                    Direction::Next => (at + 1).min(rows.len() - 1),
                    Direction::Prev => at.saturating_sub(1),
                };
                focus(next, rows[to].id.clone())
            }
            JsonInspectorEvent::Home | JsonInspectorEvent::End => {
                let rows = rows();
                let row = if event == JsonInspectorEvent::Home { rows.first() } else { rows.last() };
                match row {
                    Some(r) => focus(next, r.id.clone()),
                    None => Transition::to(next),
                }
            }
            JsonInspectorEvent::ExpandOrEnter => {
                let rows = rows();
                let Some(at) = current(&rows) else { return Transition::to(next) };
                let row = &rows[at];
                if !row.expandable {
                    Transition::to(next)
                } else if row.expanded {
                    focus(next, rows[at + 1].id.clone())
                } else {
                    next.expanded.insert(row.id.clone());
                    focus(next, row.id.clone())
                }
            }
            JsonInspectorEvent::CollapseOrExit => {
                let rows = rows();
                let Some(at) = current(&rows) else { return Transition::to(next) };
                let row = &rows[at];
                if row.expanded {
                    next.expanded.remove(&row.id);
                    focus(next, row.id.clone())
                } else if let Some(parent) = row.parent.clone() {
                    focus(next, parent)
                } else {
                    Transition::to(next)
                }
            }
            JsonInspectorEvent::ExpandSiblings => {
                let rows = rows();
                let Some(at) = current(&rows) else { return Transition::to(next) };
                let parent = rows[at].parent.clone();
                next.expanded.extend(rows.iter().filter(|r| r.parent == parent && r.expandable).map(|r| r.id.clone()));
                Transition::to(next)
            }
            JsonInspectorEvent::ExpandToDepth(depth) => {
                next.expanded = expand_to(&input.root, depth);
                Transition::to(next)
            }
            JsonInspectorEvent::SetQuery(query) => {
                let (hits, open) = find_matches(&input.root, &query);
                next.expanded.extend(open);
                next.matches = hits;
                next.active_match = None;
                next.query = query;
                Transition::to(next)
            }
            JsonInspectorEvent::StepMatch(forward) => {
                let n = state.matches.len();
                if n == 0 {
                    return Transition::to(next);
                }
                let i = match (state.active_match, forward) {
                    (None, true) => 0,
                    (None, false) => n - 1,
                    (Some(i), true) => (i + 1) % n,
                    (Some(i), false) => (i + n - 1) % n,
                };
                next.active_match = Some(i);
                let id = state.matches[i].clone();
                focus(next, id)
            }
            JsonInspectorEvent::Copy(id, what) => {
                let found = visible_nodes(&input.root, &state.expanded).into_iter().find(|n| n.id == id);
                let Some(node) = found else { return Transition::to(next) };
                let text = match what {
                    CopyWhat::Value => node.copy_text(),
                    CopyWhat::Path => node.path,
                };
                Transition::with(next, vec![JsonInspectorCommand::Copy(text, what)])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(src: &str, depth: usize) -> JsonInspectorInput {
        JsonInspectorInput { root: parse(src).unwrap(), expand_depth: depth }
    }

    fn step(s: &JsonInspectorState, e: JsonInspectorEvent, i: &JsonInspectorInput) -> hydroxy_statechart::Step<JsonInspectorState, JsonInspectorCommand> {
        JsonInspectorMachine::transition(s, e, i).resolve(s)
    }

    #[test]
    fn parses_and_round_trips() {
        let src = r#"{"a":[1,-2.5e3,true,null],"b c":"x\"é😀","d":{}}"#;
        let v = parse(src).unwrap();
        assert_eq!(to_json(&v, false), r#"{"a":[1,-2.5e3,true,null],"b c":"x\"é😀","d":{}}"#);
        assert_eq!(to_json(&parse("[1,{\"k\":[]}]").unwrap(), true), "[\n  1,\n  {\n    \"k\": []\n  }\n]");
    }

    #[test]
    fn errors_name_line_and_column() {
        let e = parse("{\n  \"a\": 1,\n  \"b\" 2\n}").unwrap_err();
        assert_eq!((e.line, e.column), (3, 7));
        assert!(e.message.contains("':'"));
        assert!(parse("[1,]").is_err());
        assert!(parse("01").is_err());
        assert!(parse("\"tab\there\"").is_err());
        assert!(parse("[1] x").unwrap_err().message.contains("end of input"));
        assert_eq!(parse(&"[".repeat(MAX_DEPTH + 1)).unwrap_err().message, "Nested too deeply");
    }

    #[test]
    fn paths_quote_non_identifiers() {
        assert_eq!(key_path("$", "name"), "$.name");
        assert_eq!(key_path("$", "first name"), "$[\"first name\"]");
        assert_eq!(key_path("$", "2x"), "$[\"2x\"]");
        let i = input(r#"{"a":[0,0,{"b":1}]}"#, 9);
        let rows = visible_rows(&i.root, &JsonInspectorMachine::initial(&i).expanded, "");
        assert!(rows.iter().any(|r| r.path == "$.a[2].b"));
    }

    #[test]
    fn expands_to_depth() {
        let i = input(r#"{"a":{"b":{"c":1}}}"#, 2);
        let s = JsonInspectorMachine::initial(&i);
        let rows = visible_rows(&i.root, &s.expanded, "");
        assert_eq!(rows.iter().map(|r| r.path.as_str()).collect::<Vec<_>>(), ["$", "$.a", "$.a.b"]);
        assert!(!rows[2].expanded);
        let s = step(&s, JsonInspectorEvent::ExpandToDepth(0), &i).state;
        assert_eq!(visible_rows(&i.root, &s.expanded, "").len(), 1);
    }

    #[test]
    fn large_arrays_chunk_into_ranges() {
        let src = format!("[{}]", (0..250).map(|n| n.to_string()).collect::<Vec<_>>().join(","));
        let i = input(&src, 1);
        let s = JsonInspectorMachine::initial(&i);
        let rows = visible_rows(&i.root, &s.expanded, "");
        let keys: Vec<_> = rows[1..].iter().map(|r| r.key.clone().unwrap()).collect();
        assert_eq!(keys, ["[0…99]", "[100…199]", "[200…249]"]);
        let s = step(&s, JsonInspectorEvent::Toggle(rows[3].id.clone()), &i).state;
        let rows = visible_rows(&i.root, &s.expanded, "");
        assert_eq!(rows.len(), 4 + 50);
        assert_eq!(rows[4].path, "$[200]");
        let t = step(&s, JsonInspectorEvent::Copy(rows[3].id.clone(), CopyWhat::Value), &i);
        let JsonInspectorCommand::Copy(text, _) = &t.commands[0] else { panic!() };
        assert!(text.starts_with("[\n  200,"));
    }

    #[test]
    fn search_opens_the_way_and_steps_through_matches() {
        let i = input(r#"{"users":[{"name":"Ada"},{"name":"Grace"}],"admin":"grace"}"#, 1);
        let s = JsonInspectorMachine::initial(&i);
        let s = step(&s, JsonInspectorEvent::SetQuery("GRACE".into()), &i).state;
        assert_eq!(s.matches, ["$.users[1].name", "$.admin"]);
        let rows = visible_rows(&i.root, &s.expanded, &s.query);
        assert!(rows.iter().any(|r| r.path == "$.users[1].name" && r.matched));
        let t = step(&s, JsonInspectorEvent::StepMatch(false), &i);
        assert_eq!(t.commands, vec![JsonInspectorCommand::FocusRow("$.admin".into())]);
        assert_eq!(split_matches("\"Grace\"", "grace"), vec![(false, "\""), (true, "Grace"), (false, "\"")]);
    }

    #[test]
    fn keyboard_walks_the_tree() {
        let i = input(r#"{"a":{"b":1},"c":2}"#, 1);
        let s = JsonInspectorMachine::initial(&i);
        let s = step(&s, JsonInspectorEvent::Move(Direction::Next), &i).state;
        assert_eq!(s.focused.as_deref(), Some("$.a"));
        let s = step(&s, JsonInspectorEvent::ExpandOrEnter, &i).state;
        assert!(s.expanded.contains("$.a"));
        let s = step(&s, JsonInspectorEvent::ExpandOrEnter, &i).state;
        assert_eq!(s.focused.as_deref(), Some("$.a.b"));
        let s = step(&s, JsonInspectorEvent::CollapseOrExit, &i).state;
        assert_eq!(s.focused.as_deref(), Some("$.a"));
        let s = step(&s, JsonInspectorEvent::End, &i).state;
        assert_eq!(s.focused.as_deref(), Some("$.c"));
        let t = step(&s, JsonInspectorEvent::Copy("$.c".into(), CopyWhat::Path), &i);
        assert_eq!(t.commands, vec![JsonInspectorCommand::Copy("$.c".into(), CopyWhat::Path)]);
    }
}
