//! `JsonInspector` owns a parsed JSON value shown as a tree: which nodes are open,
//! which row has focus, and a search that opens the way to every match. Containers
//! with more than [`CHUNK`] children show range nodes (`[0…99]`) instead, so a huge
//! array never renders more than a hundred rows per level.
//!
//! Also here: a small JSON parser ([`parse`]) with line:column errors, and a
//! serializer ([`to_json`]) for copying values back out.

mod tree;
mod value;

use std::collections::HashSet;

use hydroxy_statechart::{Machine, Transition};

pub use crate::list::Direction;
use crate::list::next_index;
use tree::visible_nodes;
pub use tree::{CHUNK, JsonKind, JsonRow, expand_to, find_matches, split_matches, visible_rows};
pub use value::{JsonValue, MAX_DEPTH, ParseError, key_path, parse, quote, to_json};

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
        focused.or_else(|| rows.first()).map(|r| r.id.as_str())
    }
}

/// What the copy command carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
                Transition::update(state, next, Vec::new())
            }
            JsonInspectorEvent::Focus(id) => {
                next.focused = Some(id);
                Transition::update(state, next, Vec::new())
            }
            JsonInspectorEvent::Move(dir) => {
                let rows = rows();
                let Some(at) = current(&rows) else { return Transition::stay() };
                focus(next, rows[next_index(Some(at), rows.len(), dir, false)].id.clone())
            }
            JsonInspectorEvent::Home | JsonInspectorEvent::End => {
                let rows = rows();
                let row = if event == JsonInspectorEvent::Home { rows.first() } else { rows.last() };
                match row {
                    Some(r) => focus(next, r.id.clone()),
                    None => Transition::stay(),
                }
            }
            JsonInspectorEvent::ExpandOrEnter => {
                let rows = rows();
                let Some(at) = current(&rows) else { return Transition::stay() };
                let row = &rows[at];
                if !row.expandable {
                    Transition::stay()
                } else if row.expanded {
                    focus(next, rows[at + 1].id.clone())
                } else {
                    next.expanded.insert(row.id.clone());
                    focus(next, row.id.clone())
                }
            }
            JsonInspectorEvent::CollapseOrExit => {
                let rows = rows();
                let Some(at) = current(&rows) else { return Transition::stay() };
                let row = &rows[at];
                if row.expanded {
                    next.expanded.remove(&row.id);
                    focus(next, row.id.clone())
                } else if let Some(parent) = row.parent.clone() {
                    focus(next, parent)
                } else {
                    Transition::stay()
                }
            }
            JsonInspectorEvent::ExpandSiblings => {
                let rows = rows();
                let Some(at) = current(&rows) else { return Transition::stay() };
                let parent = rows[at].parent.clone();
                next.expanded.extend(rows.iter().filter(|r| r.parent == parent && r.expandable).map(|r| r.id.clone()));
                Transition::update(state, next, Vec::new())
            }
            JsonInspectorEvent::ExpandToDepth(depth) => {
                next.expanded = expand_to(&input.root, depth);
                Transition::update(state, next, Vec::new())
            }
            JsonInspectorEvent::SetQuery(query) => {
                let (hits, open) = find_matches(&input.root, &query);
                next.expanded.extend(open);
                next.matches = hits;
                next.active_match = None;
                next.query = query;
                Transition::update(state, next, Vec::new())
            }
            JsonInspectorEvent::StepMatch(forward) => {
                let n = state.matches.len();
                if n == 0 {
                    return Transition::stay();
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
                let Some(node) = found else { return Transition::stay() };
                let text = match what {
                    CopyWhat::Value => node.copy_text(),
                    CopyWhat::Path => node.path,
                };
                Transition::emit(vec![JsonInspectorCommand::Copy(text, what)])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step as advance;

    fn input(src: &str, depth: usize) -> JsonInspectorInput {
        JsonInspectorInput { root: parse(src).unwrap(), expand_depth: depth }
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
        assert_eq!(parse(&"[".repeat(MAX_DEPTH + 1)).unwrap_err().message, "nested too deeply");
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
        let s = advance::<JsonInspectorMachine>(&s, JsonInspectorEvent::ExpandToDepth(0), &i).state;
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
        let s = advance::<JsonInspectorMachine>(&s, JsonInspectorEvent::Toggle(rows[3].id.clone()), &i).state;
        let rows = visible_rows(&i.root, &s.expanded, "");
        assert_eq!(rows.len(), 4 + 50);
        assert_eq!(rows[4].path, "$[200]");
        let t = advance::<JsonInspectorMachine>(&s, JsonInspectorEvent::Copy(rows[3].id.clone(), CopyWhat::Value), &i);
        let JsonInspectorCommand::Copy(text, _) = &t.commands[0] else {
            panic!("expected Copy, got {:?}", t.commands[0])
        };
        assert!(text.starts_with("[\n  200,"));
    }

    #[test]
    fn search_opens_the_way_and_steps_through_matches() {
        let i = input(r#"{"users":[{"name":"Ada"},{"name":"Grace"}],"admin":"grace"}"#, 1);
        let s = JsonInspectorMachine::initial(&i);
        let s = advance::<JsonInspectorMachine>(&s, JsonInspectorEvent::SetQuery("GRACE".into()), &i).state;
        assert_eq!(s.matches, ["$.users[1].name", "$.admin"]);
        let rows = visible_rows(&i.root, &s.expanded, &s.query);
        assert!(rows.iter().any(|r| r.path == "$.users[1].name" && r.matched));
        let t = advance::<JsonInspectorMachine>(&s, JsonInspectorEvent::StepMatch(false), &i);
        assert_eq!(t.commands, vec![JsonInspectorCommand::FocusRow("$.admin".into())]);
        assert_eq!(split_matches("\"Grace\"", "grace"), vec![(false, "\""), (true, "Grace"), (false, "\"")]);
    }

    #[test]
    fn keyboard_walks_the_tree() {
        let i = input(r#"{"a":{"b":1},"c":2}"#, 1);
        let s = JsonInspectorMachine::initial(&i);
        let s = advance::<JsonInspectorMachine>(&s, JsonInspectorEvent::Move(Direction::Next), &i).state;
        assert_eq!(s.focused.as_deref(), Some("$.a"));
        let s = advance::<JsonInspectorMachine>(&s, JsonInspectorEvent::ExpandOrEnter, &i).state;
        assert!(s.expanded.contains("$.a"));
        let s = advance::<JsonInspectorMachine>(&s, JsonInspectorEvent::ExpandOrEnter, &i).state;
        assert_eq!(s.focused.as_deref(), Some("$.a.b"));
        let s = advance::<JsonInspectorMachine>(&s, JsonInspectorEvent::CollapseOrExit, &i).state;
        assert_eq!(s.focused.as_deref(), Some("$.a"));
        let s = advance::<JsonInspectorMachine>(&s, JsonInspectorEvent::End, &i).state;
        assert_eq!(s.focused.as_deref(), Some("$.c"));
        let t = advance::<JsonInspectorMachine>(&s, JsonInspectorEvent::Copy("$.c".into(), CopyWhat::Path), &i);
        assert_eq!(t.commands, vec![JsonInspectorCommand::Copy("$.c".into(), CopyWhat::Path)]);
    }
}
