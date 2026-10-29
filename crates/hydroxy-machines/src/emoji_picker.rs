//! Emoji picker: a searchable grid of emoji in category tabs, with a skin-tone
//! choice and a recently-used tab the host persists.
//!
//! The grid follows the WAI-ARIA grid pattern: one tab stop, arrow keys move by
//! cell and row, Home/End go to the row ends (with Ctrl, the grid ends), Page
//! keys jump several rows, and Up from the first row goes back to the search.

use hydroxy_statechart::{Machine, Transition};

mod data;

/// The emoji categories, in tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmojiGroup {
    /// Faces, hearts, emotion.
    Smileys,
    /// Hands, people, roles.
    People,
    /// Animals and plants.
    Nature,
    /// Food and drink.
    Food,
    /// Places, transport, weather.
    Travel,
    /// Events, sport, games.
    Activities,
    /// Things.
    Objects,
    /// Signs, arrows, shapes.
    Symbols,
    /// Country and other flags.
    Flags,
}

impl EmojiGroup {
    /// Every group, in tab order.
    pub const ALL: [EmojiGroup; 9] = [
        EmojiGroup::Smileys,
        EmojiGroup::People,
        EmojiGroup::Nature,
        EmojiGroup::Food,
        EmojiGroup::Travel,
        EmojiGroup::Activities,
        EmojiGroup::Objects,
        EmojiGroup::Symbols,
        EmojiGroup::Flags,
    ];

    /// A slug for ids and `data-` attributes.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            EmojiGroup::Smileys => "smileys",
            EmojiGroup::People => "people",
            EmojiGroup::Nature => "nature",
            EmojiGroup::Food => "food",
            EmojiGroup::Travel => "travel",
            EmojiGroup::Activities => "activities",
            EmojiGroup::Objects => "objects",
            EmojiGroup::Symbols => "symbols",
            EmojiGroup::Flags => "flags",
        }
    }

    /// The tab's name.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            EmojiGroup::Smileys => "Smileys and emotion",
            EmojiGroup::People => "People and body",
            EmojiGroup::Nature => "Animals and nature",
            EmojiGroup::Food => "Food and drink",
            EmojiGroup::Travel => "Travel and places",
            EmojiGroup::Activities => "Activities",
            EmojiGroup::Objects => "Objects",
            EmojiGroup::Symbols => "Symbols",
            EmojiGroup::Flags => "Flags",
        }
    }

    /// The emoji that stands for the group on its tab.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            EmojiGroup::Smileys => "😀",
            EmojiGroup::People => "👋",
            EmojiGroup::Nature => "🐻",
            EmojiGroup::Food => "🍔",
            EmojiGroup::Travel => "🚗",
            EmojiGroup::Activities => "⚽",
            EmojiGroup::Objects => "💡",
            EmojiGroup::Symbols => "💕",
            EmojiGroup::Flags => "🏁",
        }
    }
}

/// One emoji in the data set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Emoji {
    /// The emoji itself, without a skin tone.
    pub emoji: &'static str,
    /// CLDR short name, such as "thumbs up".
    pub name: &'static str,
    /// Extra search words, space separated.
    pub keywords: &'static str,
    /// Its category.
    pub group: EmojiGroup,
    /// Takes a skin tone.
    pub tone: bool,
}

/// Every emoji the picker offers, in CLDR order.
#[must_use]
pub fn all() -> &'static [Emoji] {
    &data::EMOJI
}

/// A skin tone for emoji that take one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SkinTone {
    /// The yellow default.
    #[default]
    Default,
    /// Fitzpatrick 1–2.
    Light,
    /// Fitzpatrick 3.
    MediumLight,
    /// Fitzpatrick 4.
    Medium,
    /// Fitzpatrick 5.
    MediumDark,
    /// Fitzpatrick 6.
    Dark,
}

impl SkinTone {
    /// Every tone, lightest after the default.
    pub const ALL: [SkinTone; 6] = [
        SkinTone::Default,
        SkinTone::Light,
        SkinTone::MediumLight,
        SkinTone::Medium,
        SkinTone::MediumDark,
        SkinTone::Dark,
    ];

    /// The modifier character, `None` for the default.
    #[must_use]
    pub const fn modifier(self) -> Option<char> {
        match self {
            SkinTone::Default => None,
            SkinTone::Light => Some('\u{1F3FB}'),
            SkinTone::MediumLight => Some('\u{1F3FC}'),
            SkinTone::Medium => Some('\u{1F3FD}'),
            SkinTone::MediumDark => Some('\u{1F3FE}'),
            SkinTone::Dark => Some('\u{1F3FF}'),
        }
    }

    /// A slug for ids.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            SkinTone::Default => "default",
            SkinTone::Light => "light",
            SkinTone::MediumLight => "medium-light",
            SkinTone::Medium => "medium",
            SkinTone::MediumDark => "medium-dark",
            SkinTone::Dark => "dark",
        }
    }

    /// The spoken name.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            SkinTone::Default => "Default skin tone",
            SkinTone::Light => "Light skin tone",
            SkinTone::MediumLight => "Medium-light skin tone",
            SkinTone::Medium => "Medium skin tone",
            SkinTone::MediumDark => "Medium-dark skin tone",
            SkinTone::Dark => "Dark skin tone",
        }
    }
}

const fn is_tone(c: char) -> bool {
    matches!(c, '\u{1F3FB}'..='\u{1F3FF}')
}

/// `emoji` in `tone`. Emoji that take no tone come back unchanged. The modifier
/// goes after the first character, in place of any presentation selector.
#[must_use]
pub fn with_tone(emoji: &Emoji, tone: SkinTone) -> String {
    let (true, Some(m)) = (emoji.tone, tone.modifier()) else { return emoji.emoji.to_string() };
    let mut chars = emoji.emoji.chars();
    let mut out = String::with_capacity(emoji.emoji.len() + 4);
    out.extend(chars.next());
    out.push(m);
    out.extend(chars.skip_while(|c| *c == '\u{FE0F}'));
    out
}

/// `text` without skin-tone modifiers or presentation selectors, for lookups.
#[must_use]
pub fn base(text: &str) -> String {
    text.chars().filter(|c| !is_tone(*c) && *c != '\u{FE0F}').collect()
}

/// The data entry for an emoji, ignoring skin tone.
#[must_use]
pub fn find(text: &str) -> Option<&'static Emoji> {
    let key = base(text);
    all().iter().find(|e| base(e.emoji) == key)
}

/// How well `e` matches one lower-case query word. Lower is better.
fn word_score(e: &Emoji, word: &str) -> Option<u8> {
    if e.name == word {
        Some(0)
    } else if e.name.starts_with(word) {
        Some(1)
    } else if e.name.split(|c: char| !c.is_alphanumeric()).any(|w| w.starts_with(word)) {
        Some(2)
    } else if e.keywords.split(' ').any(|w| w.starts_with(word)) {
        Some(3)
    } else if e.name.contains(word) {
        Some(4)
    } else {
        None
    }
}

/// Emoji matching every word of `query` by name or keyword, best first. A pasted
/// emoji finds itself. An empty query matches nothing.
#[must_use]
pub fn search(query: &str) -> Vec<&'static Emoji> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    if let Some(e) = find(&query) {
        return vec![e];
    }
    let words: Vec<&str> = query.split_whitespace().collect();
    let mut hits: Vec<(u32, usize, &'static Emoji)> = all()
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            let score = words.iter().try_fold(0_u32, |sum, w| word_score(e, w).map(|s| sum + u32::from(s)))?;
            Some((score, i, e))
        })
        .collect();
    hits.sort_unstable_by_key(|&(score, i, _)| (score, i));
    hits.into_iter().map(|(_, _, e)| e).collect()
}

/// The recent list after picking `emoji`: it moves to the front, duplicates (in
/// any tone) go, and the list keeps at most `max`.
#[must_use]
pub fn push_recent(recent: &[String], emoji: &str, max: usize) -> Vec<String> {
    let key = base(emoji);
    std::iter::once(emoji.to_string()).chain(recent.iter().filter(|r| base(r) != key).cloned()).take(max).collect()
}

/// Which list the grid shows while there is no search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// The host's recently used emoji.
    Recent,
    /// One category.
    Group(EmojiGroup),
}

impl Section {
    /// A slug for ids.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Section::Recent => "recent",
            Section::Group(g) => g.as_str(),
        }
    }

    /// The tab's name.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Section::Recent => "Recently used",
            Section::Group(g) => g.label(),
        }
    }

    /// The tab's emoji.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Section::Recent => "🕘",
            Section::Group(g) => g.icon(),
        }
    }
}

/// One cell in the grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// What to insert: the emoji in the chosen tone.
    pub emoji: String,
    /// Its name.
    pub name: &'static str,
}

/// Props for [`EmojiPickerMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmojiPickerInput {
    /// Recently used emoji, newest first. The host stores them.
    pub recent: Vec<String>,
    /// How many recent emoji to keep.
    pub max_recent: usize,
    /// Cells per row.
    pub columns: usize,
    /// Controlled skin tone. `Some` at start fixes controlled mode.
    pub skin_tone: Option<SkinTone>,
    /// Uncontrolled initial skin tone.
    pub default_skin_tone: SkinTone,
}

impl Default for EmojiPickerInput {
    fn default() -> Self {
        Self { recent: Vec::new(), max_recent: 24, columns: 9, skin_tone: None, default_skin_tone: SkinTone::Default }
    }
}

/// What [`EmojiPickerMachine`] remembers between events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmojiPickerState {
    /// The search text.
    pub query: String,
    /// The tab shown while not searching.
    pub section: Section,
    /// The cell with the grid's tab stop.
    pub active: usize,
    /// The chosen skin tone.
    pub tone: SkinTone,
    /// Fixed at start: the owner holds the tone.
    pub controlled_tone: bool,
}

/// Inputs to [`EmojiPickerMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmojiPickerEvent {
    /// The search text changed.
    Query(String),
    /// A tab was chosen.
    ShowSection(Section),
    /// Arrow key on a tab: the next or previous tab, wrapping.
    StepSection(i32),
    /// A key on the grid: see [`grid_key`].
    Grid(GridMove),
    /// Arrow down or Enter in the search field.
    SearchKey(SearchKey),
    /// A cell got focus or the pointer rests on it.
    Hover(usize),
    /// A cell was clicked or Enter/Space pressed on it.
    Pick(usize),
    /// A skin tone was chosen.
    Tone(SkinTone),
}

/// Grid navigation keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridMove {
    /// Left or right one cell (`-1` / `1`).
    Cell(i32),
    /// Up or down rows (`-1` / `1`, or more for Page keys).
    Row(i32),
    /// Start of the row.
    RowStart,
    /// End of the row.
    RowEnd,
    /// First cell.
    First,
    /// Last cell.
    Last,
}

/// Keys in the search field the picker handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchKey {
    /// Into the grid.
    Down,
    /// Pick the first match.
    Enter,
}

/// Side effects from [`EmojiPickerMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmojiPickerCommand {
    /// An emoji was picked.
    Select(Cell),
    /// Store this as the new recent list.
    RecentChange(Vec<String>),
    /// The skin tone changed (or should, when controlled).
    ToneChange(SkinTone),
    /// Move focus to this cell.
    FocusCell(usize),
    /// Move focus to the search field.
    FocusSearch,
    /// Move focus to the tab for this section.
    FocusTab(Section),
}

/// The tabs: Recent (only when there is history), then every group.
#[must_use]
pub fn sections(input: &EmojiPickerInput) -> Vec<Section> {
    let recent = (!input.recent.is_empty()).then_some(Section::Recent);
    recent.into_iter().chain(EmojiGroup::ALL.into_iter().map(Section::Group)).collect()
}

/// The cells the grid shows: search hits while searching, else the section.
#[must_use]
pub fn cells(state: &EmojiPickerState, input: &EmojiPickerInput) -> Vec<Cell> {
    let cell = |e: &Emoji| Cell { emoji: with_tone(e, state.tone), name: e.name };
    if !state.query.trim().is_empty() {
        return search(&state.query).into_iter().map(cell).collect();
    }
    match state.section {
        Section::Recent => {
            input.recent.iter().filter_map(|r| find(r).map(|e| Cell { emoji: r.clone(), name: e.name })).collect()
        }
        Section::Group(g) => all().iter().filter(|e| e.group == g).map(cell).collect(),
    }
}

/// The cell index after a grid move, among `len` cells in rows of `columns`.
#[must_use]
pub fn moved(active: usize, len: usize, columns: usize, step: GridMove) -> usize {
    if len == 0 {
        return 0;
    }
    let cols = columns.max(1);
    let last = len - 1;
    let active = active.min(last);
    // Cell counts are small, so the i64 round trip never truncates.
    let at = i64::try_from(active).unwrap_or(0);
    let clamp = |target: i64| usize::try_from(target.max(0)).unwrap_or(0).min(last);
    match step {
        GridMove::Cell(d) => clamp(at + i64::from(d)),
        GridMove::Row(d) => {
            let target = at + i64::from(d) * i64::try_from(cols).unwrap_or(1);
            let past_end = usize::try_from(target).is_ok_and(|t| t > last);
            // Down from the last row stays put; down into a short last row lands on its end.
            if past_end && active / cols == last / cols { active } else { clamp(target) }
        }
        GridMove::RowStart => active - active % cols,
        GridMove::RowEnd => (active - active % cols + cols - 1).min(last),
        GridMove::First => 0,
        GridMove::Last => last,
    }
}

/// State, search, tabs and keyboard movement for one emoji picker.
#[derive(Debug, Clone, Copy)]
pub struct EmojiPickerMachine;

type Step = Transition<EmojiPickerState, EmojiPickerCommand>;

fn pick(state: &EmojiPickerState, input: &EmojiPickerInput, index: usize) -> Step {
    let Some(cell) = cells(state, input).into_iter().nth(index) else { return Transition::stay() };
    let recent = push_recent(&input.recent, &cell.emoji, input.max_recent);
    let next = EmojiPickerState { active: index, ..state.clone() };
    Transition::update(state, next, vec![EmojiPickerCommand::RecentChange(recent), EmojiPickerCommand::Select(cell)])
}

impl Machine for EmojiPickerMachine {
    type Input = EmojiPickerInput;
    type State = EmojiPickerState;
    type Event = EmojiPickerEvent;
    type Command = EmojiPickerCommand;

    fn initial(input: &EmojiPickerInput) -> EmojiPickerState {
        let section = if input.recent.is_empty() { Section::Group(EmojiGroup::Smileys) } else { Section::Recent };
        EmojiPickerState {
            query: String::new(),
            section,
            active: 0,
            tone: input.skin_tone.unwrap_or(input.default_skin_tone),
            controlled_tone: input.skin_tone.is_some(),
        }
    }

    fn transition(state: &EmojiPickerState, event: EmojiPickerEvent, input: &EmojiPickerInput) -> Step {
        match event {
            EmojiPickerEvent::Query(query) => Transition::to(EmojiPickerState { query, active: 0, ..state.clone() }),
            EmojiPickerEvent::ShowSection(section) => {
                Transition::to(EmojiPickerState { section, active: 0, query: String::new(), ..state.clone() })
            }
            EmojiPickerEvent::StepSection(d) => {
                let tabs = sections(input);
                let at = tabs.iter().position(|s| *s == state.section).unwrap_or(0);
                let i = crate::list::step_index(at, isize::try_from(d).unwrap_or(0), tabs.len(), true);
                let section = tabs[i];
                let next = EmojiPickerState { section, active: 0, query: String::new(), ..state.clone() };
                Transition::update(state, next, vec![EmojiPickerCommand::FocusTab(section)])
            }
            EmojiPickerEvent::Grid(GridMove::Row(d)) if d < 0 && state.active < input.columns.max(1) => {
                Transition::emit(vec![EmojiPickerCommand::FocusSearch])
            }
            EmojiPickerEvent::Grid(step) => {
                let len = cells(state, input).len();
                let active = moved(state.active, len, input.columns, step);
                let next = EmojiPickerState { active, ..state.clone() };
                Transition::update(state, next, vec![EmojiPickerCommand::FocusCell(active)])
            }
            EmojiPickerEvent::SearchKey(SearchKey::Down) => {
                if cells(state, input).is_empty() {
                    return Transition::stay();
                }
                let next = EmojiPickerState { active: 0, ..state.clone() };
                Transition::update(state, next, vec![EmojiPickerCommand::FocusCell(0)])
            }
            EmojiPickerEvent::SearchKey(SearchKey::Enter) => pick(state, input, 0),
            EmojiPickerEvent::Hover(active) => Transition::to(EmojiPickerState { active, ..state.clone() }),
            EmojiPickerEvent::Pick(index) => pick(state, input, index),
            EmojiPickerEvent::Tone(tone) => {
                if tone == state.tone {
                    return Transition::stay();
                }
                let mut next = state.clone();
                if !state.controlled_tone {
                    next.tone = tone;
                }
                Transition::update(state, next, vec![EmojiPickerCommand::ToneChange(tone)])
            }
        }
    }

    fn sync(state: &EmojiPickerState, input: &EmojiPickerInput) -> Option<EmojiPickerState> {
        let mut next = state.clone();
        if let (true, Some(tone)) = (state.controlled_tone, input.skin_tone) {
            next.tone = tone;
        }
        // The recent tab can disappear when the host clears its history.
        if next.section == Section::Recent && input.recent.is_empty() {
            next.section = Section::Group(EmojiGroup::Smileys);
            next.active = 0;
        }
        hydroxy_statechart::changed(state, next)
    }
}

/// The spoken result count while searching: "12 emoji found".
#[must_use]
pub fn result_status(query: &str, count: usize) -> String {
    match (query.trim().is_empty(), count) {
        (true, _) => String::new(),
        (false, 0) => format!("No emoji match \u{201c}{}\u{201d}", query.trim()),
        (false, 1) => "1 emoji found".to_string(),
        (false, n) => format!("{n} emoji found"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step as advance;

    fn input() -> EmojiPickerInput {
        EmojiPickerInput::default()
    }

    #[test]
    fn data_is_sane() {
        assert!(all().len() > 800);
        for g in EmojiGroup::ALL {
            assert!(all().iter().any(|e| e.group == g), "{g:?} has emoji");
        }
        assert!(find("👍🏽").is_some_and(|e| e.name == "thumbs up"));
    }

    #[test]
    fn search_ranks_names_before_keywords() {
        let hits = search("thumbs");
        assert_eq!(hits[0].name, "thumbs up");
        assert!(search("+1").iter().any(|e| e.name == "thumbs up"), "shortcodes are keywords");
        assert!(search("red heart").iter().any(|e| e.emoji == "❤️"));
        assert!(search("zzzzqq").is_empty());
        assert_eq!(search("🎉")[0].name, "party popper");
    }

    #[test]
    fn tones_apply_only_where_supported() {
        let thumbs = find("👍").unwrap();
        assert_eq!(with_tone(thumbs, SkinTone::Dark), "👍🏿");
        assert_eq!(with_tone(thumbs, SkinTone::Default), "👍");
        let party = find("🎉").unwrap();
        assert_eq!(with_tone(party, SkinTone::Dark), "🎉");
        // A presentation selector after the base is replaced by the modifier.
        let raised = find("☝️").unwrap();
        assert_eq!(with_tone(raised, SkinTone::Light), "☝🏻");
    }

    #[test]
    fn recent_dedupes_across_tones_and_caps() {
        let recent = vec!["👍".to_string(), "🎉".to_string(), "👀".to_string()];
        assert_eq!(push_recent(&recent, "👍🏽", 3), vec!["👍🏽", "🎉", "👀"]);
        assert_eq!(push_recent(&recent, "🔥", 2), vec!["🔥", "👍"]);
    }

    #[test]
    fn grid_moves_by_cell_row_and_ends() {
        // 20 cells in rows of 9: rows 0-8, 9-17, 18-19.
        assert_eq!(moved(0, 20, 9, GridMove::Cell(1)), 1);
        assert_eq!(moved(0, 20, 9, GridMove::Cell(-1)), 0);
        assert_eq!(moved(4, 20, 9, GridMove::Row(1)), 13);
        assert_eq!(moved(13, 20, 9, GridMove::Row(1)), 19, "short last row: clamp to the last cell");
        assert_eq!(moved(19, 20, 9, GridMove::Row(1)), 19);
        assert_eq!(moved(13, 20, 9, GridMove::RowStart), 9);
        assert_eq!(moved(18, 20, 9, GridMove::RowEnd), 19);
        assert_eq!(moved(5, 20, 9, GridMove::Last), 19);
        assert_eq!(moved(5, 0, 9, GridMove::Last), 0);
    }

    #[test]
    fn up_from_the_first_row_returns_to_search() {
        let i = input();
        let s = EmojiPickerMachine::initial(&i);
        let t = advance::<EmojiPickerMachine>(&s, EmojiPickerEvent::Grid(GridMove::Row(-1)), &i);
        assert_eq!(t.commands, vec![EmojiPickerCommand::FocusSearch]);
    }

    #[test]
    fn picking_selects_and_updates_recent() {
        let i = EmojiPickerInput { default_skin_tone: SkinTone::Medium, ..input() };
        let s = EmojiPickerMachine::initial(&i);
        let s = advance::<EmojiPickerMachine>(&s, EmojiPickerEvent::Query("thumbs up".into()), &i).state;
        let t = advance::<EmojiPickerMachine>(&s, EmojiPickerEvent::SearchKey(SearchKey::Enter), &i);
        assert_eq!(
            t.commands,
            vec![
                EmojiPickerCommand::RecentChange(vec!["👍🏽".into()]),
                EmojiPickerCommand::Select(Cell { emoji: "👍🏽".into(), name: "thumbs up" }),
            ]
        );
    }

    #[test]
    fn recent_tab_shows_only_with_history() {
        assert_eq!(sections(&input())[0], Section::Group(EmojiGroup::Smileys));
        let i = EmojiPickerInput { recent: vec!["🔥".into()], ..input() };
        assert_eq!(sections(&i)[0], Section::Recent);
        let s = EmojiPickerMachine::initial(&i);
        assert_eq!(cells(&s, &i)[0].emoji, "🔥");
        // History cleared: fall back to the first group.
        assert_eq!(
            hydroxy_statechart::sync_or_keep::<EmojiPickerMachine>(&s, &input()).section,
            Section::Group(EmojiGroup::Smileys)
        );
    }

    #[test]
    fn tab_arrows_wrap() {
        let i = input();
        let s = EmojiPickerMachine::initial(&i);
        let t = advance::<EmojiPickerMachine>(&s, EmojiPickerEvent::StepSection(-1), &i);
        assert_eq!(t.state.section, Section::Group(EmojiGroup::Flags));
        assert_eq!(t.commands, vec![EmojiPickerCommand::FocusTab(Section::Group(EmojiGroup::Flags))]);
    }

    #[test]
    fn controlled_tone_only_requests() {
        let i = EmojiPickerInput { skin_tone: Some(SkinTone::Light), ..input() };
        let s = EmojiPickerMachine::initial(&i);
        let t = advance::<EmojiPickerMachine>(&s, EmojiPickerEvent::Tone(SkinTone::Dark), &i);
        assert_eq!(t.state.tone, SkinTone::Light);
        assert_eq!(t.commands, vec![EmojiPickerCommand::ToneChange(SkinTone::Dark)]);
        assert_eq!(result_status("cat", 2), "2 emoji found");
        assert_eq!(result_status(" ", 0), "");
    }
}
