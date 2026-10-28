//! Diff viewer connector. Anatomy: root, header (file, stats, mode toggle), body (the
//! scrolling region), table, line (one `<tr>` per row; `data-kind` added, removed or
//! context), num (line numbers), code (marker, screen-reader label and the coloured
//! text, with `data-word` on changed words), gap (a row holding the expand button).
//! Every part carries `data-scope="diff-viewer"`.
//!
//! The diff is a native `<table>` with a visually hidden caption and column headers.
//! Added and removed lines start with a visually hidden "Added" / "Removed" so the change
//! is not carried by colour alone. Line numbers and markers are `aria-hidden`.

use std::ops::Range;

pub use oxide_machines::diff_viewer::*;

use crate::code::highlight;

/// `data-scope` for every part.
pub const SCOPE: &str = "diff-viewer";

/// The id of the row showing `lines[index]`, for moving focus after an expand.
#[must_use]
pub fn line_id(root: &str, index: usize) -> String {
    format!("{root}-line-{index}")
}

/// The visually hidden word read before a line, or `None` for unchanged lines.
#[must_use]
pub const fn kind_label(kind: LineKind) -> Option<&'static str> {
    match kind {
        LineKind::Added => Some("Added: "),
        LineKind::Removed => Some("Removed: "),
        LineKind::Context => None,
    }
}

/// The visible gutter marker.
#[must_use]
pub const fn marker(kind: LineKind) -> &'static str {
    match kind {
        LineKind::Added => "+",
        LineKind::Removed => "−",
        LineKind::Context => " ",
    }
}

/// The expand button's text: "Expand 12 unchanged lines".
#[must_use]
pub fn expand_label(len: usize) -> String {
    if len == 1 { "Expand 1 unchanged line".into() } else { format!("Expand {len} unchanged lines") }
}

/// The caption: "Changes to src/app.rs: 5 additions, 3 deletions".
#[must_use]
pub fn summary(file: Option<&str>, stats: DiffStats) -> String {
    let plural = |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
    let counts =
        format!("{}, {}", plural(stats.added, "addition", "additions"), plural(stats.removed, "deletion", "deletions"));
    match file {
        Some(f) => format!("Changes to {f}: {counts}"),
        None => format!("Changes: {counts}"),
    }
}

/// A run of one line's text with one colour and one changed/unchanged state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    /// The `data-hl` token, if coloured.
    pub token: Option<&'static str>,
    /// Inside a word-level change.
    pub changed: bool,
    /// The text.
    pub text: String,
}

/// Cut `text` at both syntax-colour and word-mark boundaries. Concatenating the
/// segments gives back `text`.
// highlighting runs per line, so block comments and multi-line strings only
// colour their first line. Feed whole-file tokens in if that matters.
#[must_use]
pub fn segments(text: &str, marks: &[Range<usize>], lang: &str) -> Vec<Segment> {
    let mut out = Vec::new();
    let mut at = 0;
    for (token, run) in highlight(text, lang) {
        let end = at + run.len();
        let mut cut = at;
        while cut < end {
            let changed = marks.iter().any(|m| m.contains(&cut));
            // Next boundary: the end of the run or the next mark edge inside it.
            let next = marks
                .iter()
                .flat_map(|m| [m.start, m.end])
                .filter(|&b| b > cut && b < end)
                .min()
                .unwrap_or(end);
            out.push(Segment { token: token.map(crate::code::Token::as_str), changed, text: text[cut..next].to_string() });
            cut = next;
        }
        at = end;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_split_at_marks_and_tokens() {
        let text = "let qty = 42;";
        let segs = segments(text, &[std::ops::Range { start: 4, end: 7 }], "rust");
        assert_eq!(segs.iter().map(|s| s.text.as_str()).collect::<String>(), text);
        let qty = segs.iter().find(|s| s.text == "qty").expect("qty is its own segment");
        assert!(qty.changed);
        assert!(segs.iter().any(|s| s.text == "let" && s.token == Some("k") && !s.changed));
        assert!(segs.iter().any(|s| s.text == "42" && s.token == Some("n")));
    }

    #[test]
    fn labels_read_well() {
        assert_eq!(expand_label(1), "Expand 1 unchanged line");
        assert_eq!(expand_label(12), "Expand 12 unchanged lines");
        let s = DiffStats { added: 1, removed: 3 };
        assert_eq!(summary(Some("app.rs"), s), "Changes to app.rs: 1 addition, 3 deletions");
        assert_eq!(kind_label(LineKind::Context), None);
        assert_eq!(line_id("ox-diff-viewer-0", 4), "ox-diff-viewer-0-line-4");
    }
}
