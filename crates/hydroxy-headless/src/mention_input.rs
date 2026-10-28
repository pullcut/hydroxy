//! Mention input connector. Parts, all under `data-scope="mention-input"`: root, label,
//! field, mirror, mention, anchor, input, popover, listbox, option, option-mark,
//! option-label, option-description, empty, status.
//!
//! The input is a native `<textarea>` (`role="combobox"`), so editing, selection and IME
//! stay native. Behind it sits the `mirror`: the same text, `aria-hidden`, with each
//! mention wrapped in a `mention` span that draws its pill, and an empty `anchor` span at
//! the open trigger that the `popover` is placed under (through `--mention-x` and
//! `--mention-y`). It holds a `listbox` whose options
//! are announced with `aria-activedescendant`; `status` is a polite live region with the
//! result count.

pub use crate::assistant_composer::{char_to_utf16, utf16_to_char};
pub use hydroxy_machines::mention_input::*;
pub use hydroxy_machines::shortcut_recorder::KeyPress;

/// `data-scope` for every part.
pub const SCOPE: &str = "mention-input";

/// Element ids for one mention input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The visible label.
    pub label: String,
    /// The textarea.
    pub input: String,
    /// The mirror behind the textarea.
    pub mirror: String,
    /// The anchor span at the open trigger.
    pub anchor: String,
    /// The list.
    pub listbox: String,
    /// The live region.
    pub status: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids {
        label: format!("{root}-label"),
        input: format!("{root}-input"),
        mirror: format!("{root}-mirror"),
        anchor: format!("{root}-anchor"),
        listbox: format!("{root}-listbox"),
        status: format!("{root}-status"),
    }
}

/// `id` of the option at `index`, for `aria-activedescendant`.
#[must_use]
pub fn option_id(root: &str, index: usize) -> String {
    format!("{root}-option-{index}")
}

/// One trigger and the items it lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MentionSource {
    /// The trigger character and where it opens.
    pub trigger: TriggerSpec,
    /// Accessible name of the list, such as "People" or "Commands".
    pub name: String,
    /// What the list offers.
    pub items: Vec<MentionItem>,
}

impl MentionSource {
    /// A source for `trigger`, opening at the start of any word.
    pub fn new(trigger: char, name: impl Into<String>, items: Vec<MentionItem>) -> Self {
        Self { trigger: TriggerSpec::new(trigger), name: name.into(), items }
    }

    /// Open only at the very start of the text.
    #[must_use]
    pub const fn start_only(mut self) -> Self {
        self.trigger.start_only = true;
        self
    }
}

/// The machine input for these sources.
#[must_use]
pub fn machine_input(sources: &[MentionSource]) -> MentionInputInput {
    MentionInputInput { triggers: sources.iter().map(|s| s.trigger).collect() }
}

/// The source the open trigger belongs to.
#[must_use]
pub fn open_source<'a>(state: &MentionInputState, sources: &'a [MentionSource]) -> Option<&'a MentionSource> {
    let t = state.trigger.as_ref()?;
    sources.iter().find(|s| s.trigger.char == t.char)
}

/// Rows the open list shows, filtered by the typed query. Empty when closed.
#[must_use]
pub fn rows(state: &MentionInputState, sources: &[MentionSource]) -> Vec<MentionItem> {
    match (state.trigger.as_ref(), open_source(state, sources)) {
        (Some(t), Some(source)) => filter(&source.items, &t.query).into_iter().cloned().collect(),
        _ => Vec::new(),
    }
}

/// Map a key press in the textarea. `Some` means: prevent the default and send it.
///
/// - Up and Down move through the open list.
/// - Enter or Tab inserts the highlighted row (`accept` is its id and label).
/// - Escape closes the list.
///
/// Everything else, Enter for a new line included, is left to the browser.
#[must_use]
pub fn input_key(
    press: &KeyPress,
    composing: bool,
    state: &MentionInputState,
    rows: usize,
    accept: Option<&MentionItem>,
) -> Option<MentionInputEvent> {
    use MentionInputEvent as E;
    if composing || !state.is_open() {
        return None;
    }
    match press.key.as_str() {
        "ArrowDown" if rows > 0 => Some(E::MoveActive { delta: 1, count: rows }),
        "ArrowUp" if rows > 0 => Some(E::MoveActive { delta: -1, count: rows }),
        "Enter" | "Tab" if !press.shift => {
            accept.map(|item| E::Accept { id: item.id.clone(), label: item.label.clone() })
        }
        "Escape" => Some(E::Dismiss),
        _ => None,
    }
}

/// A run of the mirror.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// Plain text.
    Text(String),
    /// A mention's text, drawn as a pill.
    Mention(String),
    /// The empty span the list is placed under.
    Anchor,
}

/// The mirror's runs: text, mentions, and the anchor at the open trigger. A trailing
/// zero-width space keeps a final empty line as tall as the textarea's.
#[must_use]
pub fn segments(state: &MentionInputState) -> Vec<Segment> {
    let chars: Vec<char> = state.text.chars().collect();
    let anchor = state.trigger.as_ref().map(|t| t.start);
    let mut out = Vec::with_capacity(state.mentions.len() * 2 + 3);
    let mut at = 0;
    let text = |out: &mut Vec<Segment>, from: usize, to: usize| {
        if let Some(a) = anchor.filter(|a| (from..to).contains(a)) {
            if from < a {
                out.push(Segment::Text(chars[from..a].iter().collect()));
            }
            out.push(Segment::Anchor);
            out.push(Segment::Text(chars[a..to].iter().collect()));
        } else if from < to {
            out.push(Segment::Text(chars[from..to].iter().collect()));
        }
    };
    for m in &state.mentions {
        let (start, end) = (m.start.min(chars.len()), m.end.min(chars.len()));
        text(&mut out, at, start);
        out.push(Segment::Mention(chars[start..end].iter().collect()));
        at = end;
    }
    text(&mut out, at, chars.len());
    out.push(Segment::Text("\u{200b}".into()));
    out
}

/// What the live region says: the result count while the list is open, else nothing.
#[must_use]
pub fn status_text(state: &MentionInputState, rows: usize) -> String {
    if !state.is_open() {
        return String::new();
    }
    match rows {
        0 => EMPTY_TEXT.into(),
        1 => "1 result. Press Enter to insert it.".into(),
        n => format!("{n} results. Use Up and Down to choose, Enter to insert."),
    }
}

/// Text shown when the open list has no rows.
pub const EMPTY_TEXT: &str = "No matches";

/// Watcher for adapters without direct DOM access, run with parameters `root` (the root
/// id) and `send`, returning a cleanup. It reports edits and caret moves in the textarea
/// as `kind|start|end|value` (see [`parse_report`]), keeps the mirror scrolled with the
/// textarea, and sets `--mention-x` / `--mention-y` on the field from the anchor span so
/// the list sits under the trigger.
pub const WATCH_JS: &str = r#"
const input = document.getElementById(root + "-input");
const mirror = document.getElementById(root + "-mirror");
if (!input || !mirror) return () => {};
const field = mirror.parentElement;
const place = () => {
  mirror.scrollTop = input.scrollTop;
  const anchor = document.getElementById(root + "-anchor");
  if (!anchor) return;
  field.style.setProperty("--mention-x", anchor.offsetLeft + "px");
  field.style.setProperty("--mention-y", (anchor.offsetTop + anchor.offsetHeight - mirror.scrollTop) + "px");
};
const report = (kind) =>
  send(kind + "|" + (input.selectionStart ?? 0) + "|" + (input.selectionEnd ?? 0) + "|" + input.value);
const onInput = () => report("input");
const onMove = (e) => {
  if (e.type !== "keyup" || /^(Arrow|Home|End|Page)/.test(e.key)) report("select");
};
const moves = ["keyup", "pointerup", "select"];
input.addEventListener("input", onInput);
moves.forEach((t) => input.addEventListener(t, onMove));
input.addEventListener("scroll", place);
const watcher = new MutationObserver(place);
watcher.observe(mirror, { childList: true, subtree: true, characterData: true });
return () => {
  input.removeEventListener("input", onInput);
  moves.forEach((t) => input.removeEventListener(t, onMove));
  input.removeEventListener("scroll", place);
  watcher.disconnect();
};
"#;

/// What a [`WATCH_JS`] report says happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Report {
    /// The text was edited: send [`MentionInputEvent::SetText`].
    Input {
        /// The textarea's text.
        text: String,
        /// Caret as a char index.
        caret: usize,
    },
    /// The caret or selection moved over this text: send [`MentionInputEvent::Select`]
    /// when it still matches the machine's text.
    Select {
        /// The textarea's text.
        text: String,
        /// Selection start as a char index.
        start: usize,
        /// Selection end as a char index.
        end: usize,
    },
}

/// Parse one [`WATCH_JS`] report, converting its UTF-16 offsets to char indices.
#[must_use]
pub fn parse_report(report: &str) -> Option<Report> {
    let mut parts = report.splitn(4, '|');
    let (kind, start, end, text) = (parts.next()?, parts.next()?, parts.next()?, parts.next()?);
    let at = |n: &str| n.parse().ok().map(|n| utf16_to_char(text, n));
    let (start, end) = (at(start)?, at(end)?);
    match kind {
        "input" => Some(Report::Input { text: text.to_string(), caret: end }),
        "select" => Some(Report::Select { text: text.to_string(), start, end }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use MentionInputEvent as E;

    fn sources() -> Vec<MentionSource> {
        vec![
            MentionSource::new(
                '@',
                "People",
                vec![MentionItem::new("u1", "Ana Lima"), MentionItem::new("u2", "Bo Chen")],
            ),
            MentionSource::new('/', "Commands", vec![MentionItem::new("c1", "assign")]).start_only(),
        ]
    }

    fn typed(text: &str, sources: &[MentionSource]) -> MentionInputState {
        let s = MentionInputState::default();
        <MentionInputMachine as hydroxy_statechart::Machine>::transition(
            &s,
            E::SetText { text: text.into(), caret: text.chars().count() },
            &machine_input(sources),
        )
        .resolve(&s)
        .state
    }

    fn press(key: &str) -> KeyPress {
        KeyPress { key: key.into(), ..KeyPress::default() }
    }

    #[test]
    fn rows_follow_the_open_trigger() {
        let src = sources();
        assert_eq!(rows(&typed("hi @bo", &src), &src).len(), 1);
        assert_eq!(rows(&typed("/as", &src), &src)[0].id, "c1");
        assert!(rows(&typed("hi /as", &src), &src).is_empty(), "/ is start only here");
        assert!(rows(&typed("hi", &src), &src).is_empty());
    }

    #[test]
    fn keys_drive_the_open_list_only() {
        let src = sources();
        let open = typed("@", &src);
        let item = MentionItem::new("u1", "Ana Lima");
        assert_eq!(input_key(&press("ArrowDown"), false, &open, 2, None), Some(E::MoveActive { delta: 1, count: 2 }));
        assert_eq!(
            input_key(&press("Tab"), false, &open, 2, Some(&item)),
            Some(E::Accept { id: "u1".into(), label: "Ana Lima".into() })
        );
        assert_eq!(input_key(&press("Escape"), false, &open, 0, None), Some(E::Dismiss));
        let closed = typed("hi", &src);
        assert_eq!(input_key(&press("Enter"), false, &closed, 0, None), None);
        assert_eq!(input_key(&press("Enter"), true, &open, 2, Some(&item)), None);
    }

    #[test]
    fn segments_wrap_mentions_and_place_the_anchor() {
        let mut s = MentionInputState::from_value(MentionValue {
            text: "hi @Ana x".into(),
            mentions: vec![Mention { id: "u1".into(), label: "Ana".into(), trigger: '@', start: 3, end: 7 }],
        });
        assert_eq!(
            segments(&s),
            vec![
                Segment::Text("hi ".into()),
                Segment::Mention("@Ana".into()),
                Segment::Text(" x".into()),
                Segment::Text("\u{200b}".into()),
            ]
        );
        s.trigger = Some(Trigger { char: '@', start: 8, query: "x".into() });
        assert_eq!(segments(&s)[2..4], [Segment::Text(" ".into()), Segment::Anchor]);
    }

    #[test]
    fn reports_parse_with_pipes_and_utf16() {
        assert_eq!(parse_report("input|3|3|a😀|b"), Some(Report::Input { text: "a😀|b".into(), caret: 2 }));
        assert_eq!(parse_report("select|0|1|hey"), Some(Report::Select { text: "hey".into(), start: 0, end: 1 }));
        assert_eq!(parse_report("other|0|0|x"), None);
        assert_eq!(parse_report("input|x"), None);
    }

    #[test]
    fn status_counts_results() {
        let src = sources();
        assert_eq!(status_text(&typed("@", &src), 2), "2 results. Use Up and Down to choose, Enter to insert.");
        assert_eq!(status_text(&typed("@zz", &src), 0), "No matches");
        assert_eq!(status_text(&typed("hi", &src), 0), "");
    }
}
