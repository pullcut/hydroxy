//! Log stream connector. Anatomy: root, toolbar (level toggles, search, pause, follow,
//! wrap, clear), viewport (the scrolling `role="log"`), line (time, level, text) and an
//! empty state. Every part carries `data-scope="log-stream"` and its `data-part`.
//!
//! Level toggles are native buttons with `aria-pressed`. The viewport is focusable so
//! the keyboard can scroll it.

pub use oxide_machines::log_stream::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "log-stream";

/// Attributes for the viewport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportAttrs {
    /// Always `"log"`.
    pub role: &'static str,
    /// `"off"` while paused so held lines don't interrupt; `"polite"` otherwise.
    pub aria_live: &'static str,
    /// Always `"0"`.
    pub tabindex: &'static str,
    /// `"following"`, `"scrolled"` or `"paused"`.
    pub data_state: &'static str,
}

/// `data-state` for every part.
#[must_use]
pub fn data_state(state: &LogStreamState) -> &'static str {
    if state.paused {
        "paused"
    } else if state.following {
        "following"
    } else {
        "scrolled"
    }
}

/// Attributes for the viewport.
#[must_use]
pub fn viewport_attrs(state: &LogStreamState) -> ViewportAttrs {
    ViewportAttrs {
        role: "log",
        aria_live: if state.paused { "off" } else { "polite" },
        tabindex: "0",
        data_state: data_state(state),
    }
}

/// Label of the pause button: a count of held lines while paused.
#[must_use]
pub fn pause_label(state: &LogStreamState) -> String {
    match (state.paused, state.buffered.len()) {
        (false, _) => "Pause".to_string(),
        (true, 0) => "Resume".to_string(),
        (true, n) => format!("Resume ({n} new)"),
    }
}

/// One line as plain text, for copying: `time LEVEL text`.
#[must_use]
pub fn line_text(line: &LogLine) -> String {
    format!("{} {} {}", line.time, line.level.tag(), line.text)
}

/// Whether the scroll position counts as the end, from geometry.
#[must_use]
pub fn at_end(scroll_height: f64, scroll_top: f64, client_height: f64) -> bool {
    scroll_height - scroll_top - client_height <= 8.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_label_counts_held_lines() {
        let mut s = LogStreamState {
            lines: std::collections::VecDeque::new(),
            buffered: std::collections::VecDeque::new(),
            paused: false,
            controlled: false,
            following: true,
            levels: [true; 4],
            query: String::new(),
            wrap: false,
            next_id: 0,
        };
        assert_eq!(pause_label(&s), "Pause");
        s.paused = true;
        assert_eq!(pause_label(&s), "Resume");
        s.buffered.push_back(LogLine { id: 0, level: LogLevel::Info, text: "x".into(), time: "t".into() });
        assert_eq!(pause_label(&s), "Resume (1 new)");
        assert_eq!(viewport_attrs(&s).aria_live, "off");
        assert_eq!(data_state(&s), "paused");
    }

    #[test]
    fn line_text_and_end() {
        let l = LogLine { id: 0, level: LogLevel::Warn, text: "slow".into(), time: "09:00:01".into() };
        assert_eq!(line_text(&l), "09:00:01 WARN slow");
        assert!(at_end(1000.0, 596.0, 400.0));
        assert!(!at_end(1000.0, 500.0, 400.0));
    }
}
