//! Terminal connector. Anatomy: root, viewport (the scrolling area, click focuses the
//! prompt), output (the `role="log"` list of lines), line, prompt-row (prompt label and
//! input). Every part carries `data-scope="terminal"` and its `data-part`.
//!
//! The 16 palette colours are `--ox-term-0` … `--ox-term-15` in the stylesheet, so themes
//! can retune them; 256-colour and truecolour arrive as exact inline colours.

use std::fmt::Write as _;

pub use crate::log_stream::at_end;
pub use oxide_machines::terminal::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "terminal";

/// Element ids for one terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The scrolling viewport.
    pub viewport: String,
    /// The prompt input.
    pub input: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { viewport: format!("{root}-viewport"), input: format!("{root}-input") }
}

/// Map a key in the prompt input to an event. `Some` means: prevent default.
#[must_use]
pub fn input_keydown(key: &str, ctrl: bool) -> Option<TerminalEvent> {
    match key {
        "Enter" => Some(TerminalEvent::Submit),
        "ArrowUp" => Some(TerminalEvent::HistoryPrev),
        "ArrowDown" => Some(TerminalEvent::HistoryNext),
        "l" | "L" if ctrl => Some(TerminalEvent::Clear),
        _ => None,
    }
}

fn color_css(color: Option<Color>, default: &str) -> String {
    match color {
        None => format!("var(--ox-term-{default})"),
        Some(Color::Ansi(n)) => format!("var(--ox-term-{n})"),
        Some(Color::Rgb(r, g, b)) => format!("rgb({r} {g} {b})"),
    }
}

/// Inline style for a span, or `None` for the default style.
#[must_use]
pub fn span_css(style: &Style) -> Option<String> {
    if *style == Style::default() {
        return None;
    }
    let (mut fg, mut bg) = (style.fg, style.bg);
    let mut css = String::new();
    if style.inverse {
        std::mem::swap(&mut fg, &mut bg);
        let _ = write!(css, "color:{};background:{};", color_css(fg, "bg"), color_css(bg, "fg"));
    } else {
        if fg.is_some() {
            let _ = write!(css, "color:{};", color_css(fg, "fg"));
        }
        if bg.is_some() {
            let _ = write!(css, "background:{};", color_css(bg, "bg"));
        }
    }
    if style.bold {
        css.push_str("font-weight:700;");
    }
    if style.dim {
        css.push_str("opacity:0.6;");
    }
    if style.italic {
        css.push_str("font-style:italic;");
    }
    if style.underline {
        css.push_str("text-decoration:underline;");
    }
    Some(css)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_map_to_events() {
        assert_eq!(input_keydown("Enter", false), Some(TerminalEvent::Submit));
        assert_eq!(input_keydown("ArrowUp", false), Some(TerminalEvent::HistoryPrev));
        assert_eq!(input_keydown("l", true), Some(TerminalEvent::Clear));
        assert_eq!(input_keydown("l", false), None);
    }

    #[test]
    fn span_css_maps_palette_rgb_and_inverse() {
        assert_eq!(span_css(&Style::default()), None);
        let s = Style { fg: Some(Color::Ansi(9)), bold: true, ..Style::default() };
        assert_eq!(span_css(&s).as_deref(), Some("color:var(--ox-term-9);font-weight:700;"));
        let s = Style { bg: Some(Color::Rgb(1, 2, 3)), ..Style::default() };
        assert_eq!(span_css(&s).as_deref(), Some("background:rgb(1 2 3);"));
        let s = Style { inverse: true, ..Style::default() };
        assert_eq!(span_css(&s).as_deref(), Some("color:var(--ox-term-bg);background:var(--ox-term-fg);"));
    }
}
