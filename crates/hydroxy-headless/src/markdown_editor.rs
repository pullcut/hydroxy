//! Markdown editor toolbar logic: wrap or prefix the selection, as a pure function
//! over the text and the selection.
//!
//! Offsets are UTF-16 code units, the same as a textarea's `selectionStart` and
//! `selectionEnd`, so adapters pass them straight through.

use hydroxy_machines::Modifiers;

/// The `data-scope` value on every markdown editor part.
pub const SCOPE: &str = "markdown-editor";

/// A toolbar action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// `**bold**`
    Bold,
    /// `_italic_`
    Italic,
    /// `## heading`
    Heading,
    /// `[text](url)`
    Link,
    /// `` `code` ``, or a fence around several lines.
    Code,
    /// `- item`
    BulletList,
    /// `1. item`
    NumberedList,
    /// `> quote`
    Quote,
}

impl Format {
    /// Every action, in toolbar order.
    pub const ALL: [Format; 8] = [
        Format::Bold,
        Format::Italic,
        Format::Heading,
        Format::Link,
        Format::Code,
        Format::BulletList,
        Format::NumberedList,
        Format::Quote,
    ];

    /// Accessible name for the toolbar button, with the shortcut if there is one.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Format::Bold => "Bold (Ctrl+B)",
            Format::Italic => "Italic (Ctrl+I)",
            Format::Heading => "Heading",
            Format::Link => "Link (Ctrl+K)",
            Format::Code => "Code",
            Format::BulletList => "Bulleted list",
            Format::NumberedList => "Numbered list",
            Format::Quote => "Quote",
        }
    }

    /// Short glyph shown on the button.
    #[must_use]
    pub const fn glyph(self) -> &'static str {
        match self {
            Format::Bold => "B",
            Format::Italic => "I",
            Format::Heading => "H",
            Format::Link => "Link",
            Format::Code => "</>",
            Format::BulletList => "•",
            Format::NumberedList => "1.",
            Format::Quote => "\u{201c}",
        }
    }

    /// The `data-format` value on the button.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Format::Bold => "bold",
            Format::Italic => "italic",
            Format::Heading => "heading",
            Format::Link => "link",
            Format::Code => "code",
            Format::BulletList => "bulleted-list",
            Format::NumberedList => "numbered-list",
            Format::Quote => "quote",
        }
    }
}

/// The action for a keydown, if any: Cmd/Ctrl with B, I or K.
#[must_use]
pub fn shortcut(key: &str, mods: Modifiers) -> Option<Format> {
    if !mods.command() {
        return None;
    }
    match key.to_ascii_lowercase().as_str() {
        "b" => Some(Format::Bold),
        "i" => Some(Format::Italic),
        "k" => Some(Format::Link),
        _ => None,
    }
}

/// Text and selection after an edit. Offsets in UTF-16 code units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    /// The new text.
    pub text: String,
    /// Selection start.
    pub start: usize,
    /// Selection end.
    pub end: usize,
}

/// Apply `format` to the selection `start..end` (UTF-16 offsets) of `text`.
/// Wrapping formats toggle off when the selection is already wrapped; line formats
/// toggle off when every selected line already has the prefix.
#[must_use]
pub fn apply(format: Format, text: &str, start: usize, end: usize) -> Edit {
    let (a, b) = (to_byte(text, start.min(end)), to_byte(text, start.max(end)));
    let (text, a, b) = match format {
        Format::Bold => wrap(text, a, b, "**", "bold text"),
        Format::Italic => wrap(text, a, b, "_", "italic text"),
        Format::Code if text[a..b].contains('\n') => fence(text, a, b),
        Format::Code => wrap(text, a, b, "`", "code"),
        Format::Link => link(text, a, b),
        Format::Heading | Format::BulletList | Format::NumberedList | Format::Quote => lines(format, text, a, b),
    };
    Edit { start: to_utf16(&text, a), end: to_utf16(&text, b), text }
}

/// Byte offset of UTF-16 offset `n`, clamped to the text.
fn to_byte(text: &str, n: usize) -> usize {
    let mut units = 0;
    for (i, c) in text.char_indices() {
        if units >= n {
            return i;
        }
        units += c.len_utf16();
    }
    text.len()
}

fn to_utf16(text: &str, byte: usize) -> usize {
    text[..byte].encode_utf16().count()
}

fn splice(text: &str, a: usize, b: usize, with: &str) -> String {
    [&text[..a], with, &text[b..]].concat()
}

fn wrap(text: &str, a: usize, b: usize, mark: &str, placeholder: &str) -> (String, usize, usize) {
    let m = mark.len();
    let sel = &text[a..b];
    // Already wrapped outside the selection: unwrap.
    if text[..a].ends_with(mark) && text[b..].starts_with(mark) {
        let out = [&text[..a - m], sel, &text[b + m..]].concat();
        return (out, a - m, b - m);
    }
    // The selection includes the marks: unwrap.
    if sel.len() >= 2 * m && sel.starts_with(mark) && sel.ends_with(mark) {
        return (splice(text, a, b, &sel[m..sel.len() - m]), a, b - 2 * m);
    }
    let inner = if sel.is_empty() { placeholder } else { sel };
    (splice(text, a, b, &format!("{mark}{inner}{mark}")), a + m, a + m + inner.len())
}

fn fence(text: &str, a: usize, b: usize) -> (String, usize, usize) {
    let sel = &text[a..b];
    let lead = if a == 0 || text[..a].ends_with('\n') { "" } else { "\n" };
    let open = format!("{lead}```\n");
    (splice(text, a, b, &format!("{open}{sel}\n```")), a + open.len(), a + open.len() + sel.len())
}

fn link(text: &str, a: usize, b: usize) -> (String, usize, usize) {
    let sel = &text[a..b];
    if sel.is_empty() {
        // Select the text placeholder; the url comes after.
        (splice(text, a, b, "[text](url)"), a + 1, a + 5)
    } else {
        let url = a + sel.len() + 3;
        (splice(text, a, b, &format!("[{sel}](url)")), url, url + 3)
    }
}

/// Length of `line`'s existing prefix for `format`, or `None`.
fn prefix_len(format: Format, line: &str) -> Option<usize> {
    let fixed = match format {
        Format::Heading => "## ",
        Format::BulletList => "- ",
        Format::Quote => "> ",
        _ => {
            let digits = line.bytes().take_while(u8::is_ascii_digit).count();
            return (digits > 0 && line[digits..].starts_with(". ")).then_some(digits + 2);
        }
    };
    line.starts_with(fixed).then_some(fixed.len())
}

fn lines(format: Format, text: &str, a: usize, b: usize) -> (String, usize, usize) {
    let line_start = text[..a].rfind('\n').map_or(0, |i| i + 1);
    let line_end = text[b..].find('\n').map_or(text.len(), |i| b + i);
    let block: Vec<&str> = text[line_start..line_end].split('\n').collect();
    let remove =
        block.iter().all(|l| l.is_empty() || prefix_len(format, l).is_some()) && block.iter().any(|l| !l.is_empty());
    let mut n = 0;
    let out: Vec<String> = block
        .iter()
        .map(|line| {
            if remove {
                line[prefix_len(format, line).unwrap_or(0)..].to_string()
            } else if line.is_empty() && block.len() > 1 {
                String::new()
            } else {
                n += 1;
                let prefix = match format {
                    Format::Heading => "## ".to_string(),
                    Format::BulletList => "- ".to_string(),
                    Format::Quote => "> ".to_string(),
                    _ => format!("{n}. "),
                };
                format!("{prefix}{line}")
            }
        })
        .collect();
    let joined = out.join("\n");
    let new_text = splice(text, line_start, line_end, &joined);
    if a == b {
        // Caret: keep it in place relative to the line's text.
        let shift = out[0].len().cast_signed() - block[0].len().cast_signed();
        let caret = a.cast_signed() + shift;
        let caret = caret.clamp(line_start.cast_signed(), (line_start + out[0].len()).cast_signed()).cast_unsigned();
        (new_text, caret, caret)
    } else {
        (new_text, line_start, line_start + joined.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(format: Format, text: &str, start: usize, end: usize) -> (String, usize, usize) {
        let e = apply(format, text, start, end);
        (e.text, e.start, e.end)
    }

    #[test]
    fn bold_wraps_and_toggles() {
        assert_eq!(run(Format::Bold, "say hi now", 4, 6), ("say **hi** now".into(), 6, 8));
        assert_eq!(run(Format::Bold, "say **hi** now", 6, 8), ("say hi now".into(), 4, 6));
        assert_eq!(run(Format::Bold, "say **hi** now", 4, 10), ("say hi now".into(), 4, 6));
    }

    #[test]
    fn empty_selection_inserts_a_selected_placeholder() {
        assert_eq!(run(Format::Italic, "", 0, 0), ("_italic text_".into(), 1, 12));
        assert_eq!(run(Format::Link, "go ", 3, 3), ("go [text](url)".into(), 4, 8));
    }

    #[test]
    fn link_selects_the_url() {
        assert_eq!(run(Format::Link, "see docs", 4, 8), ("see [docs](url)".into(), 11, 14));
    }

    #[test]
    fn code_inline_or_fenced() {
        assert_eq!(run(Format::Code, "a b", 2, 3), ("a `b`".into(), 3, 4));
        assert_eq!(run(Format::Code, "x\ny", 0, 3), ("```\nx\ny\n```".into(), 4, 7));
    }

    #[test]
    fn line_prefixes_cover_whole_lines_and_toggle() {
        assert_eq!(run(Format::BulletList, "one\ntwo", 1, 5), ("- one\n- two".into(), 0, 11));
        assert_eq!(run(Format::BulletList, "- one\n- two", 0, 11), ("one\ntwo".into(), 0, 7));
        assert_eq!(run(Format::NumberedList, "a\nb", 0, 3), ("1. a\n2. b".into(), 0, 9));
        assert_eq!(run(Format::NumberedList, "1. a\n2. b", 0, 9), ("a\nb".into(), 0, 3));
        assert_eq!(run(Format::Quote, "hi", 1, 1), ("> hi".into(), 3, 3));
        assert_eq!(run(Format::Heading, "## Title", 8, 8), ("Title".into(), 5, 5));
    }

    #[test]
    fn offsets_are_utf16() {
        // "é" is one UTF-16 unit but two bytes; "😀" is two units, four bytes.
        assert_eq!(run(Format::Bold, "é😀x", 1, 3), ("é**😀**x".into(), 3, 5));
    }

    #[test]
    fn shortcut_needs_modifier_and_maps_format() {
        assert_eq!(shortcut("b", Modifiers::CTRL), Some(Format::Bold));
        assert_eq!(shortcut("K", Modifiers::CTRL), Some(Format::Link));
        assert_eq!(shortcut("b", Modifiers::NONE), None);
    }
}
