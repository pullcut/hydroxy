//! JSON inspector connector. Anatomy: root, toolbar (search, depth buttons, status),
//! tree (`role="tree"`), item (`role="treeitem"`: toggle, key, value, actions with
//! copy buttons), mark (search hit inside a key or value) and error (parse failure).
//! Every part carries `data-scope="json-inspector"` and its `data-part`.
//!
//! Rows render flat in display order; `aria-level`, `aria-setsize` and
//! `aria-posinset` carry the hierarchy. One row holds the tab stop.

pub use hydroxy_machines::json_inspector::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "json-inspector";

/// The DOM id of row `id`. Paths can hold any character, so they are hashed.
#[must_use]
pub fn row_dom_id(root: &str, id: &str) -> String {
    // FNV-1a: stable across server and client, no dependency.
    let hash = id.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3));
    format!("{root}-n{hash:x}")
}

/// Attributes for one row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemAttrs {
    /// `id`.
    pub id: String,
    /// `aria-level`.
    pub aria_level: usize,
    /// `aria-setsize`.
    pub aria_setsize: usize,
    /// `aria-posinset`.
    pub aria_posinset: usize,
    /// `aria-expanded`; `None` for leaves.
    pub aria_expanded: Option<bool>,
    /// `data-state`: `"open"`, `"closed"`, or `None` for leaves.
    pub data_state: Option<&'static str>,
    /// `data-type`: the value's kind, for colouring.
    pub data_type: &'static str,
    /// `style`: `--ox-json-level` for the indent.
    pub style: String,
    /// Accessible name: "name: \"Ada\"", or "users: 3 items".
    pub aria_label: String,
}

/// Attributes for `row`.
#[must_use]
pub fn item_attrs(root: &str, row: &JsonRow) -> ItemAttrs {
    ItemAttrs {
        id: row_dom_id(root, &row.id),
        aria_level: row.level,
        aria_setsize: row.set_size,
        aria_posinset: row.pos_in_set,
        aria_expanded: row.expandable.then_some(row.expanded),
        data_state: row.expandable.then_some(if row.expanded { "open" } else { "closed" }),
        data_type: row.kind.as_str(),
        style: format!("--ox-json-level: {}", row.level - 1),
        aria_label: match &row.key {
            Some(k) => format!("{k}: {}", row.text),
            None => format!("root: {}", row.text),
        },
    }
}

/// `tabindex` for a row.
#[must_use]
pub fn tabindex(row_id: &str, tab_stop: Option<&str>) -> i32 {
    if tab_stop == Some(row_id) { 0 } else { -1 }
}

/// The key as shown: quoted for object keys, bare for indexes and ranges.
#[must_use]
pub fn key_text(row: &JsonRow) -> Option<String> {
    let key = row.key.as_ref()?;
    Some(if row.index || row.kind == JsonKind::Range { key.clone() } else { quote(key) })
}

/// Map a key on row `id` to an event. `modifier` is Ctrl or ⌘, for copying.
/// `Some` means: prevent default.
#[must_use]
pub fn keydown(key: &str, id: &str, modifier: bool, shift: bool) -> Option<JsonInspectorEvent> {
    if modifier {
        return matches!(key, "c" | "C").then(|| {
            JsonInspectorEvent::Copy(id.to_string(), if shift { CopyWhat::Path } else { CopyWhat::Value })
        });
    }
    match key {
        "ArrowDown" => Some(JsonInspectorEvent::Move(Direction::Next)),
        "ArrowUp" => Some(JsonInspectorEvent::Move(Direction::Prev)),
        "ArrowRight" => Some(JsonInspectorEvent::ExpandOrEnter),
        "ArrowLeft" => Some(JsonInspectorEvent::CollapseOrExit),
        "Home" => Some(JsonInspectorEvent::Home),
        "End" => Some(JsonInspectorEvent::End),
        "Enter" | " " => Some(JsonInspectorEvent::Toggle(id.to_string())),
        "*" => Some(JsonInspectorEvent::ExpandSiblings),
        _ => None,
    }
}

/// Map a key in the search field: Enter jumps to the next match, Shift+Enter the previous.
#[must_use]
pub fn search_keydown(key: &str, shift: bool) -> Option<JsonInspectorEvent> {
    (key == "Enter").then_some(JsonInspectorEvent::StepMatch(!shift))
}

/// The status line: match count while searching.
#[must_use]
pub fn status(state: &JsonInspectorState) -> String {
    if state.query.trim().is_empty() {
        return String::new();
    }
    match (state.matches.len(), state.active_match) {
        (0, _) => "No matches".into(),
        (n, Some(i)) => format!("{} of {n}", i + 1),
        (1, None) => "1 match".into(),
        (n, None) => format!("{n} matches"),
    }
}

/// What to announce after a copy.
#[must_use]
pub fn copied_label(what: CopyWhat) -> &'static str {
    match what {
        CopyWhat::Value => "Copied value",
        CopyWhat::Path => "Copied path",
    }
}

/// The parse error as shown: "Unexpected '}' (line 3, column 7)".
#[must_use]
pub fn error_text(e: &ParseError) -> String {
    format!("{} (line {}, column {})", e.message, e.line, e.column)
}

/// The depth buttons: label and depth. "All" opens everything.
pub const DEPTHS: [(&str, usize); 4] = [("1", 1), ("2", 2), ("3", 3), ("All", usize::MAX)];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_carry_hierarchy_and_names() {
        let root = parse(r#"{"user name":{"tags":["a"]}}"#).unwrap();
        let rows = visible_rows(&root, &expand_to(&root, 9), "");
        let a = item_attrs("j", &rows[1]);
        assert_eq!((a.aria_level, a.aria_expanded, a.data_state, a.data_type), (2, Some(true), Some("open"), "object"));
        assert_eq!(key_text(&rows[1]).as_deref(), Some("\"user name\""));
        assert_eq!(key_text(&rows[3]).as_deref(), Some("0"));
        assert_eq!(item_attrs("j", &rows[3]).aria_label, "0: \"a\"");
        assert_ne!(row_dom_id("j", "$.a"), row_dom_id("j", "$.b"));
        assert!(!row_dom_id("j", "$[\"a b\"]").contains(' '));
    }

    #[test]
    fn keys_and_status() {
        assert_eq!(keydown("c", "$", true, true), Some(JsonInspectorEvent::Copy("$".into(), CopyWhat::Path)));
        assert_eq!(keydown("c", "$", false, false), None);
        assert_eq!(keydown("*", "$", false, false), Some(JsonInspectorEvent::ExpandSiblings));
        assert_eq!(search_keydown("Enter", true), Some(JsonInspectorEvent::StepMatch(false)));
        let mut s = JsonInspectorState { query: "x".into(), ..Default::default() };
        assert_eq!(status(&s), "No matches");
        s.matches = vec!["$.a".into(), "$.b".into()];
        assert_eq!(status(&s), "2 matches");
        s.active_match = Some(1);
        assert_eq!(status(&s), "2 of 2");
    }
}
