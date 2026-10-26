//! `LogStream` keeps a capped buffer of log lines with follow-tail, a pause buffer, level
//! and text filters, and a wrap toggle. Timestamps arrive with each entry; the machine
//! never reads a clock.

use std::collections::VecDeque;

use oxide_statechart::{Machine, Transition};

/// How serious a line is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Detail for debugging.
    Debug,
    /// Normal progress.
    Info,
    /// Something to look at.
    Warn,
    /// Something failed.
    Error,
}

impl LogLevel {
    /// Every level, quietest first.
    pub const ALL: [LogLevel; 4] = [LogLevel::Debug, LogLevel::Info, LogLevel::Warn, LogLevel::Error];

    /// The `data-level` value and short label source.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }

    /// The upper-case tag shown in front of a line.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            LogLevel::Debug => "DBG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERR",
        }
    }

    const fn index(self) -> usize {
        self as usize
    }
}

/// A line to add.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// Severity.
    pub level: LogLevel,
    /// The text, without a trailing newline.
    pub text: String,
    /// Display timestamp, such as "09:42:07".
    pub time: String,
}

impl LogEntry {
    /// A new entry.
    pub fn new(level: LogLevel, time: impl Into<String>, text: impl Into<String>) -> Self {
        Self { level, text: text.into(), time: time.into() }
    }
}

/// A retained line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    /// Increases with every line ever added; a stable key.
    pub id: u64,
    /// Severity.
    pub level: LogLevel,
    /// The text.
    pub text: String,
    /// Display timestamp.
    pub time: String,
}

/// Props for [`LogStreamMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogStreamInput {
    /// Most lines kept; the oldest are dropped first. Default 1000.
    pub max_lines: usize,
    /// Controlled pause. `Some` at start fixes controlled mode.
    pub paused: Option<bool>,
    /// Uncontrolled initial pause.
    pub default_paused: bool,
}

impl Default for LogStreamInput {
    fn default() -> Self {
        Self { max_lines: 1000, paused: None, default_paused: false }
    }
}

/// What [`LogStreamMachine`] remembers.
#[allow(clippy::struct_excessive_bools, reason = "independent switches: paused, controlled, following, wrap")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogStreamState {
    /// Lines on screen.
    pub lines: VecDeque<LogLine>,
    /// Lines that arrived while paused, shown on resume.
    pub buffered: VecDeque<LogLine>,
    /// New lines are held back.
    pub paused: bool,
    /// Fixed at start: the owner holds `paused`.
    pub controlled: bool,
    /// The view follows the newest line.
    pub following: bool,
    /// Levels shown, by [`LogLevel::ALL`] order.
    pub levels: [bool; 4],
    /// Text filter, case-insensitive. Empty shows everything.
    pub query: String,
    /// Long lines wrap instead of scrolling sideways.
    pub wrap: bool,
    /// Id for the next line.
    pub next_id: u64,
}

impl LogStreamState {
    /// Whether lines of `level` are shown.
    #[must_use]
    pub fn shows(&self, level: LogLevel) -> bool {
        self.levels[level.index()]
    }

    /// Lines passing the level and text filters, oldest first.
    #[must_use]
    pub fn visible(&self) -> Vec<&LogLine> {
        let query = self.query.to_lowercase();
        self.lines
            .iter()
            .filter(|l| self.shows(l.level) && (query.is_empty() || l.text.to_lowercase().contains(&query)))
            .collect()
    }
}

/// Inputs to [`LogStreamMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogStreamEvent {
    /// New lines arrived.
    Push(Vec<LogEntry>),
    /// Hold new lines back.
    Pause,
    /// Show held lines and carry on.
    Resume,
    /// Show or hide one level.
    ToggleLevel(LogLevel),
    /// Set the text filter.
    SetQuery(String),
    /// Wrap or scroll long lines.
    ToggleWrap,
    /// The reader scrolled away from the end (`false`) or back to it (`true`).
    SetFollowing(bool),
    /// Remove every line, held ones included.
    Clear,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogStreamCommand {
    /// Scroll to the newest line.
    ScrollToEnd,
    /// Tell the owner the pause changed (or should, when controlled).
    PausedChange(bool),
}

/// A capped, filterable log buffer.
#[derive(Debug, Clone, Copy)]
pub struct LogStreamMachine;

fn push_capped(into: &mut VecDeque<LogLine>, next_id: &mut u64, entries: Vec<LogEntry>, max: usize) {
    for e in entries {
        into.push_back(LogLine { id: *next_id, level: e.level, text: e.text, time: e.time });
        *next_id += 1;
    }
    while into.len() > max.max(1) {
        into.pop_front();
    }
}

impl Machine for LogStreamMachine {
    type Input = LogStreamInput;
    type State = LogStreamState;
    type Event = LogStreamEvent;
    type Command = LogStreamCommand;

    fn initial(input: &LogStreamInput) -> LogStreamState {
        LogStreamState {
            lines: VecDeque::new(),
            buffered: VecDeque::new(),
            paused: input.paused.unwrap_or(input.default_paused),
            controlled: input.paused.is_some(),
            following: true,
            levels: [true; 4],
            query: String::new(),
            wrap: false,
            next_id: 0,
        }
    }

    fn transition(
        state: &LogStreamState,
        event: LogStreamEvent,
        input: &LogStreamInput,
    ) -> Transition<LogStreamState, LogStreamCommand> {
        let mut next = state.clone();
        let mut commands = Vec::new();
        match event {
            LogStreamEvent::Push(entries) => {
                if entries.is_empty() {
                    return Transition::stay(next);
                }
                if state.paused {
                    push_capped(&mut next.buffered, &mut next.next_id, entries, input.max_lines);
                } else {
                    push_capped(&mut next.lines, &mut next.next_id, entries, input.max_lines);
                    if state.following {
                        commands.push(LogStreamCommand::ScrollToEnd);
                    }
                }
            }
            LogStreamEvent::Pause | LogStreamEvent::Resume => {
                let paused = matches!(event, LogStreamEvent::Pause);
                if paused == state.paused {
                    return Transition::stay(next);
                }
                commands.push(LogStreamCommand::PausedChange(paused));
                if !state.controlled {
                    next.paused = paused;
                    if !paused {
                        resume(&mut next, input.max_lines, &mut commands);
                    }
                }
            }
            LogStreamEvent::ToggleLevel(level) => next.levels[level.index()] = !state.levels[level.index()],
            LogStreamEvent::SetQuery(q) => next.query = q,
            LogStreamEvent::ToggleWrap => next.wrap = !state.wrap,
            LogStreamEvent::SetFollowing(f) => {
                next.following = f;
                if f && !state.following {
                    commands.push(LogStreamCommand::ScrollToEnd);
                }
            }
            LogStreamEvent::Clear => {
                next.lines.clear();
                next.buffered.clear();
            }
        }
        Transition { state: next, commands }
    }

    fn sync(state: &LogStreamState, input: &LogStreamInput) -> LogStreamState {
        let mut next = state.clone();
        if let (true, Some(paused)) = (state.controlled, input.paused) {
            if state.paused && !paused {
                resume(&mut next, input.max_lines, &mut Vec::new());
            }
            next.paused = paused;
        }
        next
    }
}

/// Move held lines onto the screen. Following is kept, so the adapter scrolls after the
/// next render (the adapter also scrolls whenever lines change while following).
fn resume(state: &mut LogStreamState, max: usize, commands: &mut Vec<LogStreamCommand>) {
    state.lines.append(&mut state.buffered);
    while state.lines.len() > max.max(1) {
        state.lines.pop_front();
    }
    if state.following {
        commands.push(LogStreamCommand::ScrollToEnd);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(level: LogLevel, text: &str) -> LogEntry {
        LogEntry::new(level, "00:00:00", text)
    }

    fn step(
        state: &LogStreamState,
        event: LogStreamEvent,
        input: &LogStreamInput,
    ) -> Transition<LogStreamState, LogStreamCommand> {
        LogStreamMachine::transition(state, event, input)
    }

    #[test]
    fn caps_retained_lines_and_keeps_ids_rising() {
        let input = LogStreamInput { max_lines: 3, ..LogStreamInput::default() };
        let s = LogStreamMachine::initial(&input);
        let entries = (0..5).map(|i| entry(LogLevel::Info, &format!("line {i}"))).collect();
        let s = step(&s, LogStreamEvent::Push(entries), &input).state;
        assert_eq!(s.lines.len(), 3);
        assert_eq!(s.lines.front().map(|l| l.id), Some(2));
        assert_eq!(s.next_id, 5);
    }

    #[test]
    fn follows_tail_until_reader_scrolls_away() {
        let input = LogStreamInput::default();
        let s = LogStreamMachine::initial(&input);
        let t = step(&s, LogStreamEvent::Push(vec![entry(LogLevel::Info, "a")]), &input);
        assert_eq!(t.commands, vec![LogStreamCommand::ScrollToEnd]);
        let s = step(&t.state, LogStreamEvent::SetFollowing(false), &input).state;
        let t = step(&s, LogStreamEvent::Push(vec![entry(LogLevel::Info, "b")]), &input);
        assert!(t.commands.is_empty());
        let t = step(&t.state, LogStreamEvent::SetFollowing(true), &input);
        assert_eq!(t.commands, vec![LogStreamCommand::ScrollToEnd]);
    }

    #[test]
    fn pause_buffers_and_resume_flushes() {
        let input = LogStreamInput::default();
        let s = LogStreamMachine::initial(&input);
        let s = step(&s, LogStreamEvent::Pause, &input).state;
        let s = step(&s, LogStreamEvent::Push(vec![entry(LogLevel::Warn, "held")]), &input).state;
        assert!(s.lines.is_empty());
        assert_eq!(s.buffered.len(), 1);
        let t = step(&s, LogStreamEvent::Resume, &input);
        assert_eq!(t.state.lines.len(), 1);
        assert!(t.state.buffered.is_empty());
        assert!(t.commands.contains(&LogStreamCommand::ScrollToEnd));
    }

    #[test]
    fn controlled_pause_only_requests() {
        let mut input = LogStreamInput { paused: Some(false), ..LogStreamInput::default() };
        let s = LogStreamMachine::initial(&input);
        let t = step(&s, LogStreamEvent::Pause, &input);
        assert!(!t.state.paused);
        assert_eq!(t.commands, vec![LogStreamCommand::PausedChange(true)]);
        input.paused = Some(true);
        assert!(LogStreamMachine::sync(&t.state, &input).paused);
    }

    #[test]
    fn filters_by_level_and_text() {
        let input = LogStreamInput::default();
        let s = LogStreamMachine::initial(&input);
        let s = step(
            &s,
            LogStreamEvent::Push(vec![
                entry(LogLevel::Info, "Server started"),
                entry(LogLevel::Error, "Connection refused"),
                entry(LogLevel::Debug, "server tick"),
            ]),
            &input,
        )
        .state;
        let s = step(&s, LogStreamEvent::ToggleLevel(LogLevel::Debug), &input).state;
        assert_eq!(s.visible().len(), 2);
        let s = step(&s, LogStreamEvent::SetQuery("SERVER".into()), &input).state;
        assert_eq!(s.visible().len(), 1);
        assert_eq!(s.visible()[0].text, "Server started");
    }

    #[test]
    fn clear_drops_everything() {
        let input = LogStreamInput::default();
        let s = LogStreamMachine::initial(&input);
        let s = step(&s, LogStreamEvent::Push(vec![entry(LogLevel::Info, "a")]), &input).state;
        let s = step(&s, LogStreamEvent::Clear, &input).state;
        assert!(s.lines.is_empty());
    }
}
