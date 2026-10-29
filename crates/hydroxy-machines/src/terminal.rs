//! `Terminal` keeps styled output lines, a prompt draft and command history. Output is
//! parsed for ANSI SGR escapes (16, 256 and truecolour, bold, dim, italic, underline,
//! inverse); other escape sequences are dropped. The host runs commands; the machine
//! only reports them.

use std::collections::VecDeque;

use hydroxy_statechart::{Machine, Transition};

/// A colour from an SGR sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// One of the 16 palette colours: 0-7 normal, 8-15 bright. Themed by the stylesheet.
    Ansi(u8),
    /// An exact colour, from 256-colour (indexes 16-255) or truecolour sequences.
    Rgb(u8, u8, u8),
}

impl Color {
    /// A 256-colour index: 0-15 stay palette colours, the rest become exact colours.
    #[must_use]
    pub fn from_256(n: u8) -> Self {
        const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
        match n {
            0..=15 => Color::Ansi(n),
            16..=231 => {
                let i = n - 16;
                Color::Rgb(LEVELS[usize::from(i / 36)], LEVELS[usize::from(i / 6 % 6)], LEVELS[usize::from(i % 6)])
            }
            _ => {
                let v = 8 + (n - 232) * 10;
                Color::Rgb(v, v, v)
            }
        }
    }
}

/// The pen: how the next characters are drawn.
#[allow(clippy::struct_excessive_bools, reason = "independent SGR attributes")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Style {
    /// Foreground. `None` is the terminal's default.
    pub fg: Option<Color>,
    /// Background. `None` is the terminal's default.
    pub bg: Option<Color>,
    /// SGR 1.
    pub bold: bool,
    /// SGR 2.
    pub dim: bool,
    /// SGR 3.
    pub italic: bool,
    /// SGR 4.
    pub underline: bool,
    /// SGR 7: swap foreground and background.
    pub inverse: bool,
}

impl Style {
    /// Apply one SGR parameter list (the numbers between `ESC [` and `m`).
    pub fn apply_sgr(&mut self, params: &[u16]) {
        if params.is_empty() {
            *self = Style::default();
            return;
        }
        let mut i = 0;
        while i < params.len() {
            let p = params[i];
            match p {
                0 => *self = Style::default(),
                1 => self.bold = true,
                2 => self.dim = true,
                3 => self.italic = true,
                4 => self.underline = true,
                7 => self.inverse = true,
                22 => (self.bold, self.dim) = (false, false),
                23 => self.italic = false,
                24 => self.underline = false,
                27 => self.inverse = false,
                30..=37 => self.fg = Some(Color::Ansi(small(p - 30))),
                90..=97 => self.fg = Some(Color::Ansi(small(p - 90 + 8))),
                40..=47 => self.bg = Some(Color::Ansi(small(p - 40))),
                100..=107 => self.bg = Some(Color::Ansi(small(p - 100 + 8))),
                39 => self.fg = None,
                49 => self.bg = None,
                38 | 48 => {
                    let (color, used) = extended(&params[i + 1..]);
                    if p == 38 {
                        self.fg = color.or(self.fg);
                    } else {
                        self.bg = color.or(self.bg);
                    }
                    i += used;
                }
                _ => {}
            }
            i += 1;
        }
    }
}

fn small(n: u16) -> u8 {
    u8::try_from(n).unwrap_or(0)
}

/// `5;n` or `2;r;g;b` after a 38/48. Returns the colour and how many params it used.
fn extended(rest: &[u16]) -> (Option<Color>, usize) {
    let byte = |i: usize| rest.get(i).map(|&v| small(v.min(255)));
    match rest.first() {
        Some(5) => (byte(1).map(Color::from_256), 2.min(rest.len())),
        Some(2) => match (byte(1), byte(2), byte(3)) {
            (Some(r), Some(g), Some(b)) => (Some(Color::Rgb(r, g, b)), 4),
            _ => (None, rest.len()),
        },
        _ => (None, rest.len().min(1)),
    }
}

/// A run of text in one style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// The characters, without escapes.
    pub text: String,
    /// How they're drawn.
    pub style: Style,
}

/// Parse one line of text (no `\n`) into styled spans, updating `pen` as SGR
/// sequences go by so style carries into the next line. Other CSI sequences, OSC
/// strings and stray control characters are dropped. Adjacent same-style runs merge.
#[must_use]
pub fn parse_line(line: &str, pen: &mut Style) -> Vec<Span> {
    let mut spans: Vec<Span> = Vec::new();
    let mut put = |c: char, style: Style| match spans.last_mut() {
        Some(last) if last.style == style => last.text.push(c),
        _ => spans.push(Span { text: c.to_string(), style }),
    };
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\x1b' => match chars.next() {
                Some('[') => {
                    let mut body = String::new();
                    for c in chars.by_ref() {
                        if ('@'..='~').contains(&c) {
                            if c == 'm' {
                                let params: Vec<u16> = if body.is_empty() {
                                    Vec::new()
                                } else {
                                    body.split(';').map(|p| p.parse().unwrap_or(0)).collect()
                                };
                                pen.apply_sgr(&params);
                            }
                            break;
                        }
                        body.push(c);
                    }
                }
                Some(']') => {
                    // OSC: runs to BEL or ESC \.
                    while let Some(c) = chars.next() {
                        if c == '\x07' || (c == '\x1b' && chars.next_if_eq(&'\\').is_some()) {
                            break;
                        }
                    }
                }
                _ => {}
            },
            '\t' => put(c, *pen),
            c if c.is_control() => {}
            c => put(c, *pen),
        }
    }
    spans
}

/// One retained line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermLine {
    /// Increases with every line ever added; a stable key.
    pub id: u64,
    /// Styled runs. Empty for a blank line.
    pub spans: Vec<Span>,
    /// An echoed command (`prompt text`), not program output.
    pub echo: bool,
}

impl TermLine {
    /// The line without styling, for copying.
    #[must_use]
    pub fn text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }
}

/// Props for [`TerminalMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalInput {
    /// Most lines kept; the oldest go first. Default 2000.
    pub max_lines: usize,
    /// Prompt shown before the input and echoed with each command. Default `"$"`.
    pub prompt: String,
}

impl Default for TerminalInput {
    fn default() -> Self {
        Self { max_lines: 2000, prompt: "$".into() }
    }
}

/// Most history entries kept.
pub const MAX_HISTORY: usize = 200;

/// What [`TerminalMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TerminalState {
    /// Lines on screen, oldest first.
    pub lines: VecDeque<TermLine>,
    /// The pen at the end of the last output, so colour carries across writes.
    pub pen: Style,
    /// Text in the prompt.
    pub draft: String,
    /// Submitted commands, oldest first, no consecutive repeats.
    pub history: Vec<String>,
    /// Position while walking history with Up/Down. `None` is the live draft.
    pub history_index: Option<usize>,
    /// The live draft, kept while walking history.
    pub saved_draft: String,
    /// The view sticks to the newest line.
    pub following: bool,
    /// Id for the next line.
    pub next_id: u64,
}

/// Inputs to [`TerminalMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalEvent {
    /// Program output. Split on `\n`; a trailing newline adds no blank line.
    /// `ESC [2J` clears the screen first.
    Write(String),
    /// The prompt text changed.
    Input(String),
    /// Enter: echo and run the draft.
    Submit,
    /// Up: an older command.
    HistoryPrev,
    /// Down: a newer command, then back to the draft.
    HistoryNext,
    /// Ctrl+L: drop every line.
    Clear,
    /// The reader scrolled away from the end (`false`) or back to it (`true`).
    SetFollowing(bool),
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalCommand {
    /// Scroll to the newest line.
    ScrollToEnd,
    /// The host should run this command line.
    Run(String),
}

/// Styled output, a prompt and history.
#[derive(Debug, Clone, Copy)]
pub struct TerminalMachine;

fn push(state: &mut TerminalState, spans: Vec<Span>, echo: bool, max: usize) {
    state.lines.push_back(TermLine { id: state.next_id, spans, echo });
    state.next_id += 1;
    while state.lines.len() > max.max(1) {
        state.lines.pop_front();
    }
}

impl Machine for TerminalMachine {
    type Input = TerminalInput;
    type State = TerminalState;
    type Event = TerminalEvent;
    type Command = TerminalCommand;

    fn initial(_: &TerminalInput) -> TerminalState {
        TerminalState { following: true, ..TerminalState::default() }
    }

    fn transition(
        state: &TerminalState,
        event: TerminalEvent,
        input: &TerminalInput,
    ) -> Transition<TerminalState, TerminalCommand> {
        let mut next = state.clone();
        let mut commands = Vec::new();
        match event {
            TerminalEvent::Write(text) => {
                let text = match text.rfind("\x1b[2J") {
                    Some(at) => {
                        next.lines.clear();
                        &text[at + 4..]
                    }
                    None => text.as_str(),
                };
                let text = text.strip_suffix('\n').unwrap_or(text);
                if text.is_empty() {
                    return Transition::update(state, next, Vec::new());
                }
                for raw in text.split('\n') {
                    // `\r` keeps only the last overwrite of a line (progress bars);
                    // styles set in the overwritten part are lost. Full cursor emulation if needed.
                    let raw = raw.rsplit('\r').find(|s| !s.is_empty()).unwrap_or("");
                    let spans = parse_line(raw, &mut next.pen);
                    push(&mut next, spans, false, input.max_lines);
                }
                if state.following {
                    commands.push(TerminalCommand::ScrollToEnd);
                }
            }
            TerminalEvent::Input(text) => {
                next.draft = text;
                next.history_index = None;
            }
            TerminalEvent::Submit => {
                let line = std::mem::take(&mut next.draft);
                let echo = vec![Span { text: format!("{} {line}", input.prompt), style: Style::default() }];
                push(&mut next, echo, true, input.max_lines);
                next.history_index = None;
                next.saved_draft.clear();
                next.following = true;
                commands.push(TerminalCommand::ScrollToEnd);
                if !line.trim().is_empty() {
                    if next.history.last() != Some(&line) {
                        next.history.push(line.clone());
                        if next.history.len() > MAX_HISTORY {
                            next.history.remove(0);
                        }
                    }
                    commands.push(TerminalCommand::Run(line));
                }
            }
            TerminalEvent::HistoryPrev => {
                let at = match state.history_index {
                    _ if state.history.is_empty() => return Transition::update(state, next, Vec::new()),
                    None => {
                        next.saved_draft.clone_from(&state.draft);
                        state.history.len() - 1
                    }
                    Some(i) => i.saturating_sub(1),
                };
                next.history_index = Some(at);
                next.draft.clone_from(&state.history[at]);
            }
            TerminalEvent::HistoryNext => match state.history_index {
                None => return Transition::update(state, next, Vec::new()),
                Some(i) if i + 1 < state.history.len() => {
                    next.history_index = Some(i + 1);
                    next.draft.clone_from(&state.history[i + 1]);
                }
                Some(_) => {
                    next.history_index = None;
                    next.draft = std::mem::take(&mut next.saved_draft);
                }
            },
            TerminalEvent::Clear => next.lines.clear(),
            TerminalEvent::SetFollowing(f) => {
                next.following = f;
                if f && !state.following {
                    commands.push(TerminalCommand::ScrollToEnd);
                }
            }
        }
        Transition::update(state, next, commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    fn run(s: &TerminalState, e: TerminalEvent) -> hydroxy_statechart::Step<TerminalState, TerminalCommand> {
        hydroxy_statechart::step::<TerminalMachine>(s, e, &TerminalInput::default())
    }

    fn start() -> TerminalState {
        TerminalMachine::initial(&TerminalInput::default())
    }

    #[test]
    fn sgr_basic_bright_and_reset() {
        let mut pen = Style::default();
        let spans = parse_line("a\x1b[1;31mb\x1b[92mc\x1b[0md\x1b[mE", &mut pen);
        let texts: Vec<_> = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(texts, ["a", "b", "c", "dE"]);
        assert_eq!(spans[1].style.fg, Some(Color::Ansi(1)));
        assert!(spans[1].style.bold);
        assert_eq!(spans[2].style.fg, Some(Color::Ansi(10)));
        assert_eq!(spans[3].style, Style::default());
    }

    #[test]
    fn sgr_256_and_truecolour() {
        let mut pen = Style::default();
        let spans = parse_line("\x1b[38;5;196mx\x1b[48;2;10;20;30;4my\x1b[38;5;244mz\x1b[38;5;3mw", &mut pen);
        assert_eq!(spans[0].style.fg, Some(Color::Rgb(255, 0, 0)));
        assert_eq!(spans[1].style.bg, Some(Color::Rgb(10, 20, 30)));
        assert!(spans[1].style.underline);
        assert_eq!(spans[2].style.fg, Some(Color::Rgb(128, 128, 128)));
        assert_eq!(spans[3].style.fg, Some(Color::Ansi(3)));
    }

    #[test]
    fn attribute_resets_and_carry_over() {
        let mut pen = Style::default();
        let _ = parse_line("\x1b[1;2;3;7;45m", &mut pen);
        assert!(pen.bold && pen.dim && pen.italic && pen.inverse);
        assert_eq!(pen.bg, Some(Color::Ansi(5)));
        let _ = parse_line("\x1b[22;23;27;49m", &mut pen);
        assert_eq!(pen, Style::default());
    }

    #[test]
    fn other_sequences_are_dropped() {
        let mut pen = Style::default();
        let spans = parse_line("\x1b]0;title\x07a\x1b[2Kb\x1b]8;;http://x\x1b\\c\x08d\x1b", &mut pen);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "abcd");
    }

    #[test]
    fn write_splits_lines_caps_and_carries_pen() {
        let input = TerminalInput { max_lines: 2, ..TerminalInput::default() };
        let s = TerminalMachine::initial(&input);
        let t = step::<TerminalMachine>(&s, TerminalEvent::Write("\x1b[32mone\ntwo\nthree\n".into()), &input);
        assert_eq!(t.state.lines.len(), 2);
        assert_eq!(t.state.lines[1].text(), "three");
        assert_eq!(t.state.lines[1].spans[0].style.fg, Some(Color::Ansi(2)));
        assert_eq!(t.commands, vec![TerminalCommand::ScrollToEnd]);
        let s = run(&t.state, TerminalEvent::Write("50%\r100%".into())).state;
        assert_eq!(s.lines.back().map(TermLine::text).as_deref(), Some("100%"));
        let s = run(&s, TerminalEvent::Write("\x1b[2Jfresh".into())).state;
        assert_eq!(s.lines.len(), 1);
    }

    #[test]
    fn submit_echoes_runs_and_records_history() {
        let s = run(&start(), TerminalEvent::Input("ls".into())).state;
        let t = run(&s, TerminalEvent::Submit);
        assert_eq!(t.commands, vec![TerminalCommand::ScrollToEnd, TerminalCommand::Run("ls".into())]);
        assert_eq!(t.state.lines[0].text(), "$ ls");
        assert!(t.state.lines[0].echo);
        assert_eq!(t.state.draft, "");
        let t = run(&t.state, TerminalEvent::Submit);
        assert_eq!(t.commands, vec![TerminalCommand::ScrollToEnd]);
        assert_eq!(t.state.history, vec!["ls".to_string()]);
    }

    #[test]
    fn history_walks_and_restores_draft() {
        let mut s = start();
        for cmd in ["a", "b"] {
            s = run(&s, TerminalEvent::Input(cmd.into())).state;
            s = run(&s, TerminalEvent::Submit).state;
        }
        s = run(&s, TerminalEvent::Input("half".into())).state;
        s = run(&s, TerminalEvent::HistoryPrev).state;
        assert_eq!(s.draft, "b");
        s = run(&s, TerminalEvent::HistoryPrev).state;
        s = run(&s, TerminalEvent::HistoryPrev).state;
        assert_eq!(s.draft, "a");
        s = run(&s, TerminalEvent::HistoryNext).state;
        assert_eq!(s.draft, "b");
        s = run(&s, TerminalEvent::HistoryNext).state;
        assert_eq!(s.draft, "half");
        assert_eq!(s.history_index, None);
    }

    #[test]
    fn follow_pauses_and_clear_empties() {
        let s = run(&start(), TerminalEvent::SetFollowing(false)).state;
        let t = run(&s, TerminalEvent::Write("x".into()));
        assert!(t.commands.is_empty());
        let s = run(&t.state, TerminalEvent::Clear).state;
        assert!(s.lines.is_empty());
    }
}
