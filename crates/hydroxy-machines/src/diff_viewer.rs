//! Diff viewer: a line-level Myers diff with word-level marks inside changed lines,
//! unchanged runs folded into gaps that can be expanded, and unified or side-by-side
//! row layouts. The machine itself only remembers the mode and which gaps are open;
//! the diff is a pure function of the two texts so adapters can memoise it.

use std::ops::Range;

use hydroxy_statechart::{Machine, Transition};

/// One step of an edit script between `a` and `b`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edit {
    /// `a[i] == b[j]`.
    Equal(usize, usize),
    /// `a[i]` was removed.
    Delete(usize),
    /// `b[j]` was added.
    Insert(usize),
}

// past this many edits the middle falls back to "all removed, all added";
// keeps the trace under ~4 MB. Raise it (or switch to linear-space Myers) if huge,
// heavily edited files need finer diffs.
const MAX_D: usize = 1000;

/// The shortest edit script from `a` to `b` (Myers' O(ND) algorithm), in order.
/// Common prefix and suffix are trimmed first, so small edits to big inputs are cheap.
#[must_use]
pub fn myers<T: PartialEq>(a: &[T], b: &[T]) -> Vec<Edit> {
    let pre = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let suf = a[pre..].iter().rev().zip(b[pre..].iter().rev()).take_while(|(x, y)| x == y).count();
    let mut out: Vec<Edit> = (0..pre).map(|i| Edit::Equal(i, i)).collect();
    middle(&a[pre..a.len() - suf], &b[pre..b.len() - suf], pre, &mut out);
    out.extend((0..suf).map(|k| Edit::Equal(a.len() - suf + k, b.len() - suf + k)));
    out
}

#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    reason = "indices stay far below isize::MAX; names follow Myers' paper"
)]
fn middle<T: PartialEq>(a: &[T], b: &[T], off: usize, out: &mut Vec<Edit>) {
    let (n, m) = (a.len() as isize, b.len() as isize);
    let max = a.len() + b.len();
    let base = max as isize + 1;
    let mut v = vec![0isize; 2 * max + 3];
    // trace[d] holds v[-d..=d] as it was before step d.
    let mut trace: Vec<Vec<isize>> = Vec::new();
    for d in 0..=max.min(MAX_D) as isize {
        trace.push(v[(base - d) as usize..=(base + d) as usize].to_vec());
        let mut k = -d;
        while k <= d {
            let i = (base + k) as usize;
            let mut x = if k == -d || (k != d && v[i - 1] < v[i + 1]) { v[i + 1] } else { v[i - 1] + 1 };
            let mut y = x - k;
            while x < n && y < m && a[x as usize] == b[y as usize] {
                x += 1;
                y += 1;
            }
            v[i] = x;
            if x >= n && y >= m {
                let start = out.len();
                backtrack(&trace, n, m, off, out);
                out[start..].reverse();
                return;
            }
            k += 2;
        }
    }
    out.extend((0..a.len()).map(|i| Edit::Delete(off + i)));
    out.extend((0..b.len()).map(|j| Edit::Insert(off + j)));
}

/// Walk the trace back from (n, m), pushing edits in reverse order.
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    reason = "indices stay far below isize::MAX; names follow Myers' paper"
)]
fn backtrack(trace: &[Vec<isize>], n: isize, m: isize, off: usize, out: &mut Vec<Edit>) {
    let (mut x, mut y) = (n, m);
    for d in (1..trace.len() as isize).rev() {
        let w = &trace[d as usize];
        let at = |k: isize| w[(k + d) as usize];
        let k = x - y;
        let prev_k = if k == -d || (k != d && at(k - 1) < at(k + 1)) { k + 1 } else { k - 1 };
        let prev_x = at(prev_k);
        let prev_y = prev_x - prev_k;
        while x > prev_x && y > prev_y {
            x -= 1;
            y -= 1;
            out.push(Edit::Equal(off + x as usize, off + y as usize));
        }
        if x == prev_x {
            out.push(Edit::Insert(off + prev_y as usize));
        } else {
            out.push(Edit::Delete(off + prev_x as usize));
        }
        x = prev_x;
        y = prev_y;
    }
    while x > 0 && y > 0 {
        x -= 1;
        y -= 1;
        out.push(Edit::Equal(off + x as usize, off + y as usize));
    }
}

/// What happened to a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineKind {
    /// In both texts.
    Context,
    /// Only in the new text.
    Added,
    /// Only in the old text.
    Removed,
}

impl LineKind {
    /// The `data-kind` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            LineKind::Context => "context",
            LineKind::Added => "added",
            LineKind::Removed => "removed",
        }
    }
}

/// One line of the diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    /// Context, added or removed.
    pub kind: LineKind,
    /// 1-based line number in the old text.
    pub old_no: Option<usize>,
    /// 1-based line number in the new text.
    pub new_no: Option<usize>,
    /// The line, without its newline.
    pub text: String,
    /// Byte ranges of `text` that changed against the paired line.
    pub marks: Vec<Range<usize>>,
}

/// Line counts for the header.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiffStats {
    /// Lines added.
    pub added: usize,
    /// Lines removed.
    pub removed: usize,
}

/// Diff two texts by line. Within each changed block removed lines come before added
/// ones, and the i-th removed line is paired with the i-th added one for word marks.
#[must_use]
pub fn diff_lines(old: &str, new: &str) -> Vec<DiffLine> {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let line =
        |kind, old_no, new_no, text: &str| DiffLine { kind, old_no, new_no, text: text.to_string(), marks: Vec::new() };
    let mut out = Vec::with_capacity(a.len().max(b.len()));
    let (mut del, mut ins) = (Vec::new(), Vec::new());
    let flush = |out: &mut Vec<DiffLine>, del: &mut Vec<usize>, ins: &mut Vec<usize>| {
        let start = out.len();
        out.extend(del.iter().map(|&i| line(LineKind::Removed, Some(i + 1), None, a[i])));
        out.extend(ins.iter().map(|&j| line(LineKind::Added, None, Some(j + 1), b[j])));
        for p in 0..del.len().min(ins.len()) {
            let (o, n) = word_marks(a[del[p]], b[ins[p]]);
            out[start + p].marks = o;
            out[start + del.len() + p].marks = n;
        }
        del.clear();
        ins.clear();
    };
    for edit in myers(&a, &b) {
        match edit {
            Edit::Equal(i, j) => {
                flush(&mut out, &mut del, &mut ins);
                out.push(line(LineKind::Context, Some(i + 1), Some(j + 1), a[i]));
            }
            Edit::Delete(i) => del.push(i),
            Edit::Insert(j) => ins.push(j),
        }
    }
    flush(&mut out, &mut del, &mut ins);
    out
}

/// Added and removed line counts.
#[must_use]
pub fn stats(lines: &[DiffLine]) -> DiffStats {
    lines.iter().fold(DiffStats::default(), |mut s, l| {
        match l.kind {
            LineKind::Added => s.added += 1,
            LineKind::Removed => s.removed += 1,
            LineKind::Context => {}
        }
        s
    })
}

/// Split a line into words, whitespace runs and single punctuation characters.
fn tokens(s: &str) -> Vec<(usize, &str)> {
    let class = |c: char| {
        if c.is_alphanumeric() || c == '_' {
            0
        } else if c.is_whitespace() {
            1
        } else {
            2
        }
    };
    let mut out: Vec<(usize, &str)> = Vec::new();
    let mut start = 0;
    let mut prev = None;
    for (i, c) in s.char_indices() {
        let k = class(c);
        if i > start && (prev != Some(k) || k == 2) {
            out.push((start, &s[start..i]));
            start = i;
        }
        prev = Some(k);
    }
    if start < s.len() {
        out.push((start, &s[start..]));
    }
    out
}

/// Changed byte ranges in `old` and `new` from a word-level diff. Lines with nothing
/// in common but whitespace get no marks: the whole line is the change.
#[must_use]
pub fn word_marks(old: &str, new: &str) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
    let (a, b) = (tokens(old), tokens(new));
    let ta: Vec<&str> = a.iter().map(|t| t.1).collect();
    let tb: Vec<&str> = b.iter().map(|t| t.1).collect();
    let edits = myers(&ta, &tb);
    let shared = edits.iter().any(|e| matches!(e, Edit::Equal(i, _) if !ta[*i].trim().is_empty()));
    if !shared {
        return (Vec::new(), Vec::new());
    }
    let (mut mo, mut mn) = (Vec::new(), Vec::new());
    let push = |marks: &mut Vec<Range<usize>>, (at, t): (usize, &str)| match marks.last_mut() {
        Some(last) if last.end == at => last.end = at + t.len(),
        _ => marks.push(at..at + t.len()),
    };
    for e in edits {
        match e {
            Edit::Delete(i) => push(&mut mo, a[i]),
            Edit::Insert(j) => push(&mut mn, b[j]),
            Edit::Equal(..) => {}
        }
    }
    (mo, mn)
}

/// A row in the unified layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Row {
    /// `lines[i]`.
    Line(usize),
    /// Folded unchanged lines `start..start + len`.
    Gap {
        /// First hidden line; also the gap's id for [`DiffViewerEvent::Expand`].
        start: usize,
        /// Number of hidden lines.
        len: usize,
    },
}

/// Unified rows: every changed line with `context` unchanged lines around it; longer
/// unchanged runs fold into a gap unless its start is in `expanded`. A gap would hide
/// at least two lines; a single line is just shown.
#[must_use]
pub fn rows(lines: &[DiffLine], context: usize, expanded: &[usize]) -> Vec<Row> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].kind != LineKind::Context {
            out.push(Row::Line(i));
            i += 1;
            continue;
        }
        let s = i;
        while i < lines.len() && lines[i].kind == LineKind::Context {
            i += 1;
        }
        let head = if s == 0 { 0 } else { context };
        let tail = if i == lines.len() { 0 } else { context };
        let hidden = (i - s).saturating_sub(head + tail);
        let start = s + head;
        if hidden > 1 && !expanded.contains(&start) {
            out.extend((s..start).map(Row::Line));
            out.push(Row::Gap { start, len: hidden });
            out.extend((start + hidden..i).map(Row::Line));
        } else {
            out.extend((s..i).map(Row::Line));
        }
    }
    out
}

/// A row in the side-by-side layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplitRow {
    /// Old line on the left, new line on the right; either may be blank.
    Pair(Option<usize>, Option<usize>),
    /// Folded unchanged lines, as in [`Row::Gap`].
    Gap {
        /// First hidden line.
        start: usize,
        /// Number of hidden lines.
        len: usize,
    },
}

/// Side-by-side rows from unified ones: context lines sit on both sides, and each
/// changed block puts its removed lines beside its added lines.
#[must_use]
pub fn split_rows(lines: &[DiffLine], rows: &[Row]) -> Vec<SplitRow> {
    let mut out = Vec::new();
    let (mut del, mut ins) = (Vec::new(), Vec::new());
    let flush = |out: &mut Vec<SplitRow>, del: &mut Vec<usize>, ins: &mut Vec<usize>| {
        for p in 0..del.len().max(ins.len()) {
            out.push(SplitRow::Pair(del.get(p).copied(), ins.get(p).copied()));
        }
        del.clear();
        ins.clear();
    };
    for &row in rows {
        match row {
            Row::Line(i) if lines[i].kind == LineKind::Removed => del.push(i),
            Row::Line(i) if lines[i].kind == LineKind::Added => ins.push(i),
            Row::Line(i) => {
                flush(&mut out, &mut del, &mut ins);
                out.push(SplitRow::Pair(Some(i), Some(i)));
            }
            Row::Gap { start, len } => {
                flush(&mut out, &mut del, &mut ins);
                out.push(SplitRow::Gap { start, len });
            }
        }
    }
    flush(&mut out, &mut del, &mut ins);
    out
}

/// Unified (one column) or split (old beside new).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DiffMode {
    /// One column, removed lines above added ones.
    #[default]
    Unified,
    /// Old text on the left, new on the right.
    Split,
}

impl DiffMode {
    /// The `data-mode` value, also the segmented control's value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            DiffMode::Unified => "unified",
            DiffMode::Split => "split",
        }
    }

    /// Parse [`DiffMode::as_str`]; anything else is unified.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        if s == "split" { DiffMode::Split } else { DiffMode::Unified }
    }
}

/// Props for [`DiffViewerMachine`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiffViewerInput {
    /// The layout to start in.
    pub default_mode: DiffMode,
}

/// What [`DiffViewerMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiffViewerState {
    /// Current layout.
    pub mode: DiffMode,
    /// Starts of gaps the reader opened.
    pub expanded: Vec<usize>,
}

/// Inputs to [`DiffViewerMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffViewerEvent {
    /// Switch layout.
    SetMode(DiffMode),
    /// Open the gap starting at this line.
    Expand(usize),
    /// The texts changed: fold everything again.
    Reset,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffViewerCommand {
    /// Focus the row showing this line (the first one a gap revealed), since the
    /// expand button that had focus is gone.
    FocusLine(usize),
}

/// Layout and folding state for a diff.
#[derive(Debug, Clone, Copy)]
pub struct DiffViewerMachine;

impl Machine for DiffViewerMachine {
    type Input = DiffViewerInput;
    type State = DiffViewerState;
    type Event = DiffViewerEvent;
    type Command = DiffViewerCommand;

    fn initial(input: &DiffViewerInput) -> DiffViewerState {
        DiffViewerState { mode: input.default_mode, expanded: Vec::new() }
    }

    fn transition(
        state: &DiffViewerState,
        event: DiffViewerEvent,
        _input: &DiffViewerInput,
    ) -> Transition<DiffViewerState, DiffViewerCommand> {
        let mut next = state.clone();
        match event {
            DiffViewerEvent::SetMode(mode) => next.mode = mode,
            DiffViewerEvent::Expand(start) => {
                if !next.expanded.contains(&start) {
                    next.expanded.push(start);
                }
                return Transition::with(next, vec![DiffViewerCommand::FocusLine(start)]);
            }
            DiffViewerEvent::Reset => next.expanded.clear(),
        }
        Transition::to(next)
    }
}

#[cfg(test)]
#[allow(clippy::many_single_char_names, reason = "a and b are the two sequences, as in the paper")]
mod tests {
    use super::*;

    /// Apply an edit script to check it really turns `a` into `b`.
    fn apply<'a>(a: &[&'a str], b: &[&'a str], edits: &[Edit]) -> (Vec<&'a str>, Vec<&'a str>) {
        let mut from_a = Vec::new();
        let mut into_b = Vec::new();
        for e in edits {
            match *e {
                Edit::Equal(i, j) => {
                    assert_eq!(a[i], b[j]);
                    from_a.push(a[i]);
                    into_b.push(b[j]);
                }
                Edit::Delete(i) => from_a.push(a[i]),
                Edit::Insert(j) => into_b.push(b[j]),
            }
        }
        (from_a, into_b)
    }

    fn edit_count(edits: &[Edit]) -> usize {
        edits.iter().filter(|e| !matches!(e, Edit::Equal(..))).count()
    }

    #[test]
    fn myers_finds_the_shortest_script() {
        // The classic example from Myers' paper: D = 5.
        let a: Vec<&str> = "ABCABBA".split("").filter(|s| !s.is_empty()).collect();
        let b: Vec<&str> = "CBABAC".split("").filter(|s| !s.is_empty()).collect();
        let edits = myers(&a, &b);
        assert_eq!(edit_count(&edits), 5);
        assert_eq!(apply(&a, &b, &edits), (a.clone(), b.clone()));
    }

    #[test]
    fn myers_handles_empty_and_equal_inputs() {
        let a = ["x", "y"];
        assert_eq!(myers::<&str>(&[], &[]), vec![]);
        assert_eq!(myers(&a, &[]), vec![Edit::Delete(0), Edit::Delete(1)]);
        assert_eq!(myers(&[], &a), vec![Edit::Insert(0), Edit::Insert(1)]);
        assert_eq!(myers(&a, &a), vec![Edit::Equal(0, 0), Edit::Equal(1, 1)]);
    }

    #[test]
    fn myers_scripts_are_valid_for_many_pairs() {
        let words = ["a", "b", "c"];
        // Every pair of short sequences over a tiny alphabet.
        let seqs: Vec<Vec<&str>> = (0..81)
            .map(|mut n: usize| {
                let len = n % 5;
                n /= 5;
                (0..len)
                    .map(|_| {
                        let w = words[n % 3];
                        n /= 3;
                        w
                    })
                    .collect()
            })
            .collect();
        for a in &seqs {
            for b in &seqs {
                let edits = myers(a, b);
                assert_eq!(&apply(a, b, &edits), &(a.clone(), b.clone()));
                assert!(edit_count(&edits) <= a.len() + b.len());
            }
        }
    }

    #[test]
    fn lines_pair_up_and_number() {
        let lines = diff_lines("one\ntwo\nthree\n", "one\n2\nthree\nfour\n");
        let kinds: Vec<_> = lines.iter().map(|l| (l.kind, l.old_no, l.new_no)).collect();
        assert_eq!(
            kinds,
            vec![
                (LineKind::Context, Some(1), Some(1)),
                (LineKind::Removed, Some(2), None),
                (LineKind::Added, None, Some(2)),
                (LineKind::Context, Some(3), Some(3)),
                (LineKind::Added, None, Some(4)),
            ]
        );
        assert_eq!(stats(&lines), DiffStats { added: 2, removed: 1 });
    }

    #[test]
    fn word_marks_cover_only_the_changed_words() {
        let (o, n) = word_marks("let total = price * qty;", "let total = price * quantity + tax;");
        assert_eq!(o.iter().map(|r| &"let total = price * qty;"[r.clone()]).collect::<Vec<_>>(), vec!["qty"]);
        let new = "let total = price * quantity + tax;";
        assert_eq!(n.iter().map(|r| &new[r.clone()]).collect::<Vec<_>>(), vec!["quantity + tax"]);
        // Nothing shared: no marks, the whole line is the change.
        assert_eq!(word_marks("alpha", "beta"), (vec![], vec![]));
    }

    #[test]
    fn long_unchanged_runs_fold_into_gaps() {
        let old = (1..=20).map(|i| format!("line {i}")).collect::<Vec<_>>().join("\n");
        let new = old.replace("line 10\n", "line ten\n");
        let lines = diff_lines(&old, &new);
        let r = rows(&lines, 3, &[]);
        // 1..=6 hidden, 7..9 context, change, 11..13 context, 14..=20 hidden.
        assert_eq!(r[0], Row::Gap { start: 0, len: 6 });
        assert_eq!(r.len(), 1 + 3 + 2 + 3 + 1);
        assert_eq!(r.last(), Some(&Row::Gap { start: 14, len: 7 }));
        let open = rows(&lines, 3, &[0]);
        assert_eq!(open.len(), r.len() - 1 + 6);
        // A single hidden line isn't worth a button.
        assert!(rows(&lines, 9, &[]).iter().all(|r| matches!(r, Row::Line(_))));
    }

    #[test]
    fn split_rows_put_removed_beside_added() {
        let lines = diff_lines("a\nb\nc\nd\n", "a\nB\nd\ne\n");
        let r = rows(&lines, 3, &[]);
        let s = split_rows(&lines, &r);
        assert_eq!(s[0], SplitRow::Pair(Some(0), Some(0)));
        // b, c removed; B added: one pair plus one left-only row.
        assert_eq!(s[1], SplitRow::Pair(Some(1), Some(3)));
        assert_eq!(s[2], SplitRow::Pair(Some(2), None));
        assert_eq!(s.last(), Some(&SplitRow::Pair(None, Some(5))));
    }

    #[test]
    fn expand_opens_once_and_moves_focus() {
        let input = DiffViewerInput::default();
        let s = DiffViewerMachine::initial(&input);
        let t = DiffViewerMachine::transition(&s, DiffViewerEvent::Expand(4), &input).resolve(&s);
        assert_eq!(t.commands, vec![DiffViewerCommand::FocusLine(4)]);
        let t = DiffViewerMachine::transition(&t.state, DiffViewerEvent::Expand(4), &input).resolve(&t.state);
        assert_eq!(t.state.expanded, vec![4]);
        let s = DiffViewerMachine::transition(&t.state, DiffViewerEvent::SetMode(DiffMode::Split), &input)
            .resolve(&t.state)
            .state;
        assert_eq!(s.mode, DiffMode::Split);
        assert!(
            DiffViewerMachine::transition(&s, DiffViewerEvent::Reset, &input).resolve(&s).state.expanded.is_empty()
        );
    }
}
