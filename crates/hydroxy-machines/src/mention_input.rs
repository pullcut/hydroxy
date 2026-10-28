//! `MentionInput` is a plain text field where a trigger character (`@` for people, `/` for
//! commands, or any others the owner configures) opens a filtered list. A picked row
//! becomes a mention: an atomic token with an id that is deleted as a unit and that the
//! caret never lands inside. The value is the text plus the mentions and their ranges.
//!
//! Positions are char indices throughout. The trigger type is shared with
//! [`crate::assistant_composer`], whose popover this generalises.

use hydroxy_statechart::{Machine, Transition, changed};

pub use crate::assistant_composer::Trigger;

/// Most rows the list shows, so huge directories stay fast to filter and render.
pub const MAX_ROWS: usize = 50;

/// A character that opens a list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriggerSpec {
    /// The character, such as `@`.
    pub char: char,
    /// Only open at the very start of the text, like a chat `/command`. Otherwise it
    /// opens at the start of any word.
    pub start_only: bool,
}

impl TriggerSpec {
    /// A trigger that opens at the start of any word.
    #[must_use]
    pub const fn new(char: char) -> Self {
        Self { char, start_only: false }
    }

    /// A trigger that opens only at the start of the text.
    #[must_use]
    pub const fn start_only(char: char) -> Self {
        Self { char, start_only: true }
    }
}

/// A row the list can offer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MentionItem {
    /// Stable id carried by the mention, such as a user id.
    pub id: String,
    /// Shown in the list and inserted after the trigger character.
    pub label: String,
    /// Secondary text in the list, such as a role or a command's summary. Searched too.
    pub description: String,
}

impl MentionItem {
    /// An item with no description.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), description: String::new() }
    }

    /// Set the secondary text.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

/// An inserted mention. Its text in the field is `trigger` followed by `label`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mention {
    /// The picked item's id.
    pub id: String,
    /// The picked item's label.
    pub label: String,
    /// The character that opened the list.
    pub trigger: char,
    /// Char index of the trigger character.
    pub start: usize,
    /// Char index just past the label.
    pub end: usize,
}

impl Mention {
    /// The text the mention occupies in the field.
    #[must_use]
    pub fn text(&self) -> String {
        format!("{}{}", self.trigger, self.label)
    }

    /// `index` is strictly inside the mention, so a caret there would split it.
    #[must_use]
    pub const fn contains(&self, index: usize) -> bool {
        self.start < index && index < self.end
    }
}

/// The field's value: its text and the mentions in it, in order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MentionValue {
    /// The full text, mentions included.
    pub text: String,
    /// Mentions in text order. Ranges never overlap.
    pub mentions: Vec<Mention>,
}

impl MentionValue {
    /// Plain text with no mentions.
    pub fn text(text: impl Into<String>) -> Self {
        Self { text: text.into(), mentions: Vec::new() }
    }
}

/// Props for [`MentionInputMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MentionInputInput {
    /// Characters that open a list.
    pub triggers: Vec<TriggerSpec>,
}

impl Default for MentionInputInput {
    /// `@` and `/`, both at the start of any word.
    fn default() -> Self {
        Self { triggers: vec![TriggerSpec::new('@'), TriggerSpec::new('/')] }
    }
}

/// What [`MentionInputMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MentionInputState {
    /// The text.
    pub text: String,
    /// Caret, or the focus end of the selection, as a char index.
    pub caret: usize,
    /// Mentions in text order.
    pub mentions: Vec<Mention>,
    /// The open list's trigger.
    pub trigger: Option<Trigger>,
    /// Highlighted row.
    pub active: usize,
    /// Start of a trigger the person closed with Escape, kept closed until they leave it.
    pub dismissed: Option<usize>,
}

impl MentionInputState {
    /// Start from a value. Mentions that are out of range, overlap an earlier one, or do
    /// not match the text they claim are dropped (their text stays). The caret goes to
    /// the end.
    #[must_use]
    pub fn from_value(value: MentionValue) -> Self {
        let chars: Vec<char> = value.text.chars().collect();
        let mut mentions = value.mentions;
        mentions.sort_by_key(|m| m.start);
        let mut kept: Vec<Mention> = Vec::with_capacity(mentions.len());
        for m in mentions {
            let fits = m.start < m.end
                && m.end <= chars.len()
                && kept.last().is_none_or(|k| k.end <= m.start)
                && chars[m.start..m.end].iter().copied().eq(m.text().chars());
            if fits {
                kept.push(m);
            }
        }
        Self { caret: chars.len(), text: value.text, mentions: kept, ..Self::default() }
    }

    /// The current value.
    #[must_use]
    pub fn value(&self) -> MentionValue {
        MentionValue { text: self.text.clone(), mentions: self.mentions.clone() }
    }

    /// The list is open.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.trigger.is_some()
    }
}

/// Inputs to [`MentionInputMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MentionInputEvent {
    /// The browser edited the text. `caret` is a char index.
    SetText {
        /// The field's new text.
        text: String,
        /// Caret after the edit.
        caret: usize,
    },
    /// The caret or selection moved without an edit. Char indices, in any order.
    Select {
        /// Anchor end.
        start: usize,
        /// Focus end.
        end: usize,
    },
    /// Move the highlight down (`1`) or up (`-1`), wrapping within `count` rows.
    MoveActive {
        /// `1` or `-1`.
        delta: i32,
        /// Rows currently shown.
        count: usize,
    },
    /// Replace the trigger text with a mention of this item.
    Accept {
        /// Item id.
        id: String,
        /// Item label.
        label: String,
    },
    /// Close the list.
    Dismiss,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MentionInputCommand {
    /// The value changed. Report it to the owner.
    Changed(MentionValue),
    /// The field must show the machine's text with this selection (char indices), because
    /// the machine changed what the browser did: a whole mention was removed, a mention
    /// was inserted, or the caret was moved out of a mention.
    PlaceCaret {
        /// Selection start.
        start: usize,
        /// Selection end, where the caret is.
        end: usize,
    },
}

/// The text, mentions and list of a mention input.
#[derive(Debug, Clone, Copy)]
pub struct MentionInputMachine;

/// Find the trigger the caret is in, if any. A trigger opens at the start of a word (or of
/// the text, for `start_only`) and runs to the caret with no whitespace. Never inside or
/// straight after a mention.
#[must_use]
pub fn detect(text: &str, caret: usize, triggers: &[TriggerSpec], mentions: &[Mention]) -> Option<Trigger> {
    let chars: Vec<char> = text.chars().collect();
    let caret = caret.min(chars.len());
    let mut i = caret;
    while i > 0 {
        let c = chars[i - 1];
        if c.is_whitespace() || mentions.iter().any(|m| m.start < i && i <= m.end) {
            return None;
        }
        if let Some(spec) = triggers.iter().find(|t| t.char == c) {
            let ok = if spec.start_only { i == 1 } else { i == 1 || chars[i - 2].is_whitespace() };
            return ok.then(|| Trigger { char: c, start: i - 1, query: chars[i..caret].iter().collect() });
        }
        i -= 1;
    }
    None
}

/// Items matching `query`, ignoring case, best first and at most [`MAX_ROWS`]: labels that
/// start with it, then labels with a word that starts with it, then any other label or
/// description containing it. A blank query keeps the first rows in order.
#[must_use]
pub fn filter<'a>(items: &'a [MentionItem], query: &str) -> Vec<&'a MentionItem> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return items.iter().take(MAX_ROWS).collect();
    }
    let mut ranked: Vec<(u8, usize, &MentionItem)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| {
            let label = item.label.to_lowercase();
            let rank = if label.starts_with(&q) {
                0
            } else if label.split_whitespace().any(|w| w.starts_with(&q)) {
                1
            } else if label.contains(&q) || item.description.to_lowercase().contains(&q) {
                2
            } else {
                return None;
            };
            Some((rank, i, item))
        })
        .collect();
    ranked.sort_unstable_by_key(|&(rank, i, _)| (rank, i));
    ranked.into_iter().take(MAX_ROWS).map(|(_, _, item)| item).collect()
}

/// The span one edit changed: `(start, old_end, new_end)`. `old[start..old_end]` became
/// `new[start..new_end]`. The caret (the end of what was typed) settles repeated
/// characters, so deleting one `l` of `ll` is placed where it happened.
fn edit_span(old: &[char], new: &[char], caret: usize) -> (usize, usize, usize) {
    let shortest = old.len().min(new.len());
    let prefix = old.iter().zip(new).take(shortest.min(caret)).take_while(|(a, b)| a == b).count();
    let suffix = old.iter().rev().zip(new.iter().rev()).take(shortest - prefix).take_while(|(a, b)| a == b).count();
    (prefix, old.len() - suffix, new.len() - suffix)
}

fn retrigger(next: &mut MentionInputState, input: &MentionInputInput, collapsed: bool) {
    let found = if collapsed { detect(&next.text, next.caret, &input.triggers, &next.mentions) } else { None };
    // a dismissed trigger is keyed by its start; an edit before it can shift it and reopen the list.
    if found.as_ref().map(|t| t.start) != next.dismissed {
        next.dismissed = None;
    }
    let found = found.filter(|t| next.dismissed != Some(t.start));
    if found.as_ref().map(|t| t.start) != next.trigger.as_ref().map(|t| t.start) {
        next.active = 0;
    }
    next.trigger = found;
}

fn set_text(state: &MentionInputState, text: String, caret: usize) -> (MentionInputState, Option<(usize, usize)>) {
    let old: Vec<char> = state.text.chars().collect();
    let new: Vec<char> = text.chars().collect();
    let caret = caret.min(new.len());
    let mut next = state.clone();
    if old == new {
        next.caret = caret;
        return (next, None);
    }
    let (a0, b0, ne) = edit_span(&old, &new, caret);
    // Widen the edit to cover every mention it touches, so a mention goes as a whole.
    let (mut a, mut b) = (a0, b0);
    for m in &state.mentions {
        let touches = if a0 == b0 { m.contains(a0) } else { m.start < b0 && a0 < m.end };
        if touches {
            a = a.min(m.start);
            b = b.max(m.end);
        }
    }
    let inserted = &new[a0..ne];
    let delta = (a + inserted.len()).cast_signed() - b.cast_signed();
    next.mentions = state
        .mentions
        .iter()
        .filter(|m| m.end <= a || m.start >= b)
        .map(|m| {
            let mut m = m.clone();
            if m.start >= b {
                m.start = m.start.saturating_add_signed(delta);
                m.end = m.end.saturating_add_signed(delta);
            }
            m
        })
        .collect();
    if (a, b) == (a0, b0) {
        next.text = text;
        next.caret = caret;
        return (next, None);
    }
    next.text = old[..a].iter().chain(inserted).chain(&old[b..]).collect();
    let caret = a + inserted.len();
    next.caret = caret;
    (next, Some((caret, caret)))
}

fn select(state: &MentionInputState, start: usize, end: usize) -> (usize, usize) {
    let len = state.text.chars().count();
    let (mut s, mut e) = (start.min(len).min(end.min(len)), start.max(end).min(len));
    if s == e {
        if let Some(m) = state.mentions.iter().find(|m| m.contains(s)) {
            // One step from the old caret is an arrow key: carry on across the mention.
            // Anything else (a click) goes to the nearer edge.
            s = match s.abs_diff(state.caret) {
                1 if s > state.caret => m.end,
                1 => m.start,
                _ if s - m.start <= m.end - s => m.start,
                _ => m.end,
            };
            e = s;
        }
    } else {
        for m in &state.mentions {
            if m.contains(s) {
                s = m.start;
            }
            if m.contains(e) {
                e = m.end;
            }
        }
    }
    (s, e)
}

/// Replace the trigger text with a mention and a space after it.
fn accept(state: &MentionInputState, t: &Trigger, id: String, label: String) -> MentionInputState {
    let chars: Vec<char> = state.text.chars().collect();
    let end = state.caret.min(chars.len());
    let mention = Mention { id, label, trigger: t.char, start: t.start, end: 0 };
    let token: Vec<char> = mention.text().chars().collect();
    let mention = Mention { end: t.start + token.len(), ..mention };
    // A space after the mention, unless one is already there.
    let space = !chars.get(end).is_some_and(|c| c.is_whitespace());
    let mut text: Vec<char> = Vec::with_capacity(chars.len() + token.len() + 1);
    text.extend_from_slice(&chars[..t.start]);
    text.extend_from_slice(&token);
    if space {
        text.push(' ');
    }
    let caret = mention.end + 1;
    text.extend_from_slice(&chars[end..]);
    let delta = text.len().cast_signed() - chars.len().cast_signed();
    let mut mentions: Vec<Mention> = state
        .mentions
        .iter()
        .map(|m| {
            let mut m = m.clone();
            if m.start >= end {
                m.start = m.start.saturating_add_signed(delta);
                m.end = m.end.saturating_add_signed(delta);
            }
            m
        })
        .collect();
    let at = mentions.partition_point(|m| m.start < mention.start);
    mentions.insert(at, mention);
    MentionInputState { text: text.into_iter().collect(), caret, mentions, trigger: None, active: 0, dismissed: None }
}

impl Machine for MentionInputMachine {
    type Input = MentionInputInput;
    type State = MentionInputState;
    type Event = MentionInputEvent;
    type Command = MentionInputCommand;

    fn initial(_: &MentionInputInput) -> MentionInputState {
        MentionInputState::default()
    }

    fn transition(
        state: &MentionInputState,
        event: MentionInputEvent,
        input: &MentionInputInput,
    ) -> Transition<MentionInputState, MentionInputCommand> {
        use MentionInputCommand as C;
        use MentionInputEvent as E;
        let mut commands = Vec::new();
        let next = match event {
            E::SetText { text, caret } => {
                let (mut next, place) = set_text(state, text, caret);
                retrigger(&mut next, input, true);
                if next.text != state.text || next.mentions != state.mentions {
                    commands.push(C::Changed(next.value()));
                }
                if let Some((start, end)) = place {
                    commands.push(C::PlaceCaret { start, end });
                }
                next
            }
            E::Select { start, end } => {
                let (s, e) = select(state, start, end);
                let mut next = state.clone();
                next.caret = e;
                retrigger(&mut next, input, s == e);
                if (s, e) != (start.min(end), start.max(end)) {
                    commands.push(C::PlaceCaret { start: s, end: e });
                }
                next
            }
            E::MoveActive { delta, count } => {
                let mut next = state.clone();
                if state.trigger.is_some() && count > 0 {
                    let count = i64::try_from(count).unwrap_or(i64::MAX);
                    let cur = i64::try_from(state.active).unwrap_or(0);
                    next.active = usize::try_from((cur + i64::from(delta)).rem_euclid(count)).unwrap_or(0);
                }
                next
            }
            E::Accept { id, label } => {
                let Some(t) = &state.trigger else { return Transition::stay() };
                let next = accept(state, t, id, label);
                commands.push(C::Changed(next.value()));
                commands.push(C::PlaceCaret { start: next.caret, end: next.caret });
                next
            }
            E::Dismiss => {
                let mut next = state.clone();
                next.dismissed = state.trigger.as_ref().map(|t| t.start);
                next.trigger = None;
                next.active = 0;
                next
            }
        };
        match changed(state, next) {
            Some(next) => Transition::with(next, commands),
            None => Transition::emit(commands),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use MentionInputCommand as C;
    use MentionInputEvent as E;

    fn run(s: &MentionInputState, e: MentionInputEvent) -> hydroxy_statechart::Step<MentionInputState, MentionInputCommand> {
        MentionInputMachine::transition(s, e, &MentionInputInput::default()).resolve(s)
    }

    fn typed(s: &MentionInputState, text: &str) -> MentionInputState {
        run(s, E::SetText { text: text.into(), caret: text.chars().count() }).state
    }

    fn pick(s: &MentionInputState, id: &str, label: &str) -> MentionInputState {
        run(s, E::Accept { id: id.into(), label: label.into() }).state
    }

    /// "Ask @Ana Lima about it" with the mention picked through the list.
    fn with_mention() -> MentionInputState {
        let s = typed(&MentionInputState::default(), "Ask @an");
        let s = pick(&s, "u1", "Ana Lima");
        typed(&s, "Ask @Ana Lima about it")
    }

    #[test]
    fn triggers_open_at_word_starts_only() {
        let t = MentionInputInput::default().triggers;
        assert_eq!(detect("hi @al", 6, &t, &[]), Some(Trigger { char: '@', start: 3, query: "al".into() }));
        assert_eq!(detect("mail a@b", 8, &t, &[]), None);
        assert_eq!(detect("hi @al x", 8, &t, &[]), None);
        assert_eq!(detect("run /dep", 8, &t, &[]).map(|t| t.char), Some('/'));
    }

    #[test]
    fn start_only_and_custom_triggers() {
        let t = [TriggerSpec::start_only('/'), TriggerSpec::new('#')];
        assert_eq!(detect("a /x", 4, &t, &[]), None);
        assert_eq!(detect("/x", 2, &t, &[]).map(|t| t.query), Some("x".into()));
        assert_eq!(detect("see #ops", 8, &t, &[]).map(|t| t.char), Some('#'));
        assert_eq!(detect("hi @al", 6, &t, &[]), None);
    }

    #[test]
    fn accept_inserts_an_atomic_mention_and_reports_it() {
        let s = typed(&MentionInputState::default(), "Ask @an");
        assert!(s.is_open());
        let step = run(&s, E::Accept { id: "u1".into(), label: "Ana Lima".into() });
        assert_eq!(step.state.text, "Ask @Ana Lima ");
        assert_eq!(step.state.caret, 14);
        assert!(!step.state.is_open());
        let m = &step.state.mentions[0];
        assert_eq!((m.id.as_str(), m.start, m.end), ("u1", 4, 13));
        assert_eq!(step.commands[0], C::Changed(step.state.value()));
        assert_eq!(step.commands[1], C::PlaceCaret { start: 14, end: 14 });
    }

    #[test]
    fn accept_before_existing_text_shifts_later_mentions_and_reuses_a_space() {
        let s = with_mention();
        // Put the caret at the start, type "@b" and pick Bo.
        let s = run(&s, E::Select { start: 0, end: 0 }).state;
        let s = run(&s, E::SetText { text: "@bAsk @Ana Lima about it".into(), caret: 2 }).state;
        assert!(s.is_open());
        let s = pick(&s, "u2", "Bo");
        assert_eq!(s.text, "@Bo Ask @Ana Lima about it");
        assert_eq!((s.mentions[0].start, s.mentions[0].end), (0, 3));
        assert_eq!((s.mentions[1].start, s.mentions[1].end), (8, 17));
        assert_eq!(s.text.chars().skip(8).take(9).collect::<String>(), s.mentions[1].text());
        let s = typed(&MentionInputState::default(), "hi @b there");
        let s = run(&s, E::Select { start: 5, end: 5 }).state;
        assert_eq!(pick(&s, "u2", "Bo").text, "hi @Bo there");
    }

    #[test]
    fn backspace_at_the_end_removes_the_whole_mention() {
        let s = with_mention();
        // Caret after "@Ana Lima" (index 13); the browser deletes the "a".
        let step = run(&s, E::SetText { text: "Ask @Ana Lim about it".into(), caret: 12 });
        assert_eq!(step.state.text, "Ask  about it");
        assert!(step.state.mentions.is_empty());
        assert_eq!(step.state.caret, 4);
        assert!(step.commands.contains(&C::PlaceCaret { start: 4, end: 4 }));
    }

    #[test]
    fn delete_at_the_start_removes_the_whole_mention() {
        let s = with_mention();
        let step = run(&s, E::SetText { text: "Ask Ana Lima about it".into(), caret: 4 });
        assert_eq!(step.state.text, "Ask  about it");
    }

    #[test]
    fn typing_inside_replaces_the_mention() {
        let s = with_mention();
        let step = run(&s, E::SetText { text: "Ask @Anxa Lima about it".into(), caret: 8 });
        assert_eq!(step.state.text, "Ask x about it");
        assert_eq!(step.state.caret, 5);
    }

    #[test]
    fn edits_elsewhere_keep_and_shift_mentions() {
        let s = with_mention();
        let s = run(&s, E::Select { start: 0, end: 0 }).state;
        let step = run(&s, E::SetText { text: "Please ask @Ana Lima about it".into(), caret: 7 });
        let m = &step.state.mentions[0];
        assert_eq!((m.start, m.end), (11, 20));
        assert!(!step.commands.iter().any(|c| matches!(c, C::PlaceCaret { .. })));
        // Typing straight after the mention keeps it.
        let s = typed(&MentionInputState::default(), "@a");
        let s = pick(&s, "u1", "Ana");
        let s = run(&s, E::Select { start: 4, end: 4 }).state;
        let s = run(&s, E::SetText { text: "@Ana!".into(), caret: 5 }).state;
        assert_eq!(s.mentions.len(), 1, "replacing the space after it keeps the mention");
    }

    #[test]
    fn repeated_characters_use_the_caret() {
        let old: Vec<char> = "aa".chars().collect();
        let new: Vec<char> = "a".chars().collect();
        assert_eq!(edit_span(&old, &new, 1), (1, 2, 1));
        assert_eq!(edit_span(&old, &new, 0), (0, 1, 0));
    }

    #[test]
    fn a_selection_over_part_of_a_mention_widens_to_all_of_it() {
        let s = with_mention();
        let step = run(&s, E::Select { start: 2, end: 6 });
        assert_eq!(step.commands, vec![C::PlaceCaret { start: 2, end: 13 }]);
        // Cutting "k @An" removes the whole mention.
        let step = run(&s, E::SetText { text: "Asa Lima about it".into(), caret: 2 });
        assert_eq!(step.state.text, "As about it");
    }

    #[test]
    fn the_caret_skips_over_mentions() {
        let s = with_mention();
        let s = run(&s, E::Select { start: 4, end: 4 }).state;
        // ArrowRight from 4 lands on 5, inside: carry on to the end.
        assert_eq!(run(&s, E::Select { start: 5, end: 5 }).commands, vec![C::PlaceCaret { start: 13, end: 13 }]);
        let s = run(&s, E::Select { start: 13, end: 13 }).state;
        // ArrowLeft from 13 lands on 12: carry on to the start.
        assert_eq!(run(&s, E::Select { start: 12, end: 12 }).commands, vec![C::PlaceCaret { start: 4, end: 4 }]);
        // A click near the end goes to the end.
        let s = run(&s, E::Select { start: 0, end: 0 }).state;
        assert_eq!(run(&s, E::Select { start: 11, end: 11 }).state.caret, 13);
    }

    #[test]
    fn no_list_right_after_a_mention() {
        let s = with_mention();
        let s = run(&s, E::Select { start: 13, end: 13 }).state;
        assert!(!s.is_open());
    }

    #[test]
    fn escape_keeps_the_list_closed_until_the_trigger_is_left() {
        let s = typed(&MentionInputState::default(), "hi @a");
        let s = run(&s, E::Dismiss).state;
        assert!(!s.is_open());
        let s = typed(&s, "hi @al");
        assert!(!s.is_open());
        let s = typed(&s, "hi @al ");
        let s = typed(&s, "hi @al @");
        assert!(s.is_open());
    }

    #[test]
    fn active_row_wraps_and_resets_for_a_new_trigger() {
        let s = typed(&MentionInputState::default(), "@");
        let s = run(&s, E::MoveActive { delta: -1, count: 3 }).state;
        assert_eq!(s.active, 2);
        let s = typed(&s, "@a");
        assert_eq!(s.active, 2, "same trigger keeps the row");
        let s = typed(&s, "@a @");
        assert_eq!(s.active, 0);
    }

    #[test]
    fn from_value_drops_bad_mentions() {
        let good = Mention { id: "u1".into(), label: "Ana".into(), trigger: '@', start: 3, end: 7 };
        let wrong = Mention { id: "u2".into(), label: "Bo".into(), trigger: '@', start: 0, end: 3 };
        let s = MentionInputState::from_value(MentionValue {
            text: "hi @Ana".into(),
            mentions: vec![good.clone(), wrong],
        });
        assert_eq!(s.mentions, vec![good]);
        assert_eq!(s.caret, 7);
    }

    #[test]
    fn filter_ranks_prefix_then_word_then_contains() {
        let items = vec![
            MentionItem::new("1", "Dana Ortiz"),
            MentionItem::new("2", "Ana Lima"),
            MentionItem::new("3", "Joan Park").description("Design"),
            MentionItem::new("4", "Bo Chen").description("Analytics"),
        ];
        let ids: Vec<&str> = filter(&items, "an").iter().map(|i| i.id.as_str()).collect();
        assert_eq!(ids, ["2", "1", "3", "4"]);
        assert_eq!(filter(&items, "").len(), 4);
        assert_eq!(filter(&items, "zz").len(), 0);
        let many: Vec<MentionItem> = (0..500).map(|i| MentionItem::new(i.to_string(), format!("Person {i}"))).collect();
        assert_eq!(filter(&many, "person").len(), MAX_ROWS);
    }

    #[test]
    fn no_op_events_report_no_state() {
        let s = MentionInputState::default();
        assert_eq!(MentionInputMachine::transition(&s, E::Dismiss, &MentionInputInput::default()).state, None);
        let t = MentionInputMachine::transition(&s, E::Accept { id: "x".into(), label: "x".into() }, &MentionInputInput::default());
        assert_eq!(t, Transition::stay());
    }
}
