//! Tool trace connector. Anatomy: root, header (summary and an expand-all
//! button), steps (`<ol>`), step (`<li>` with `data-kind` and `data-status`,
//! `aria-busy` while running), step-header (a disclosure `<button
//! aria-expanded>`), icon, title, status, duration, progress (an indeterminate
//! bar while running), body (the step's details), section (labelled arguments or
//! result, each a code block with copy), text, children (a nested `<ol>`),
//! live (a polite live region). Every part carries `data-scope="tool-trace"`
//! and its `data-part`.

pub use hydroxy_machines::tool_trace::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "tool-trace";

/// A step's header id.
#[must_use]
pub fn header_id(root: &str, step: &str) -> String {
    format!("{root}-step-{step}")
}

/// A step's body id, referenced by its header's `aria-controls`.
#[must_use]
pub fn body_id(root: &str, step: &str) -> String {
    format!("{root}-body-{step}")
}

/// Every step id, depth first, for expand and collapse all.
#[must_use]
pub fn all_ids(steps: &[TraceStep]) -> Vec<String> {
    steps.iter().flat_map(TraceStep::walk).map(|s| s.id.clone()).collect()
}

/// Whether every step is open, so the toggle-all button offers to collapse.
#[must_use]
pub fn all_open(state: &ToolTraceState, steps: &[TraceStep]) -> bool {
    steps.iter().flat_map(TraceStep::walk).all(|s| is_open(state, s))
}

/// The header's accessible description: "Running", "Done in 1.2 s", "Failed after 300 ms".
#[must_use]
pub fn status_text(step: &TraceStep) -> String {
    match (step.status, step.duration_ms) {
        (StepStatus::Done, Some(ms)) => format!("Done in {}", format_duration(ms)),
        (StepStatus::Failed, Some(ms)) => format!("Failed after {}", format_duration(ms)),
        (status, _) => status.label().to_string(),
    }
}

/// Pretty-print JSON-ish text for display, two-space indented. Text that isn't a
/// JSON object or array comes back unchanged. Strings and escapes are respected.
#[must_use]
pub fn pretty(text: &str) -> String {
    let t = text.trim();
    if !(t.starts_with('{') || t.starts_with('[')) || t.contains('\n') {
        return text.to_string();
    }
    let mut out = String::with_capacity(t.len() * 2);
    let mut depth = 0_usize;
    let (mut in_str, mut escaped) = (false, false);
    let newline = |out: &mut String, depth: usize| {
        out.push('\n');
        out.extend(std::iter::repeat_n("  ", depth));
    };
    let chars: Vec<char> = t.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if in_str {
            out.push(c);
            match (escaped, c) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => in_str = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => {
                in_str = true;
                out.push(c);
            }
            '{' | '[' => {
                out.push(c);
                let empty = chars[i + 1..].iter().find(|c| !c.is_whitespace()).is_some_and(|n| matches!(n, '}' | ']'));
                depth += 1;
                if !empty {
                    newline(&mut out, depth);
                }
            }
            '}' | ']' => {
                depth = depth.saturating_sub(1);
                if !matches!(out.chars().last(), Some('{' | '[')) {
                    newline(&mut out, depth);
                }
                out.push(c);
            }
            ',' => {
                out.push(c);
                newline(&mut out, depth);
            }
            ':' => out.push_str(": "),
            c if c.is_whitespace() => {}
            c => out.push(c),
        }
    }
    out
}

/// Inner SVG markup (24 × 24, stroked) for a step's kind.
#[must_use]
pub const fn kind_icon(kind: &StepKind) -> &'static str {
    match kind {
        // A lightbulb.
        StepKind::Thinking { .. } => {
            r#"<path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5"/><path d="M9 18h6"/><path d="M10 22h4"/>"#
        }
        // A wrench.
        StepKind::ToolCall { .. } => {
            r#"<path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.8-3.8a6 6 0 0 1-7.9 7.9l-6.9 6.9a2.1 2.1 0 0 1-3-3l6.9-6.9a6 6 0 0 1 7.9-7.9l-3.8 3.8Z"/>"#
        }
        // A speech bubble.
        StepKind::Message { .. } => r#"<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2Z"/>"#,
        // A warning triangle.
        StepKind::Error { .. } => {
            r#"<path d="m21.7 18-8-14a2 2 0 0 0-3.4 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.7-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/>"#
        }
        // Nested squares: work handed to another agent.
        StepKind::Agent { .. } => {
            r#"<rect x="3" y="3" width="13" height="13" rx="2"/><path d="M8 21h11a2 2 0 0 0 2-2V8"/>"#
        }
    }
}

/// Whether text looks like JSON, to pick the code block's colouring.
#[must_use]
pub fn is_json(text: &str) -> bool {
    let t = text.trim();
    (t.starts_with('{') && t.ends_with('}')) || (t.starts_with('[') && t.ends_with(']'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pretty_prints_json_and_leaves_text_alone() {
        assert_eq!(
            pretty(r#"{"q":"a, b","n":[1,2],"e":{}}"#),
            "{\n  \"q\": \"a, b\",\n  \"n\": [\n    1,\n    2\n  ],\n  \"e\": {}\n}"
        );
        assert_eq!(pretty("plain text"), "plain text");
        assert_eq!(pretty(r#"{"s":"say \"hi\": ok"}"#), "{\n  \"s\": \"say \\\"hi\\\": ok\"\n}");
    }

    #[test]
    fn status_text_includes_timing() {
        let s = TraceStep::new("1", StepKind::Message { text: "x".into() });
        assert_eq!(status_text(&s), "Running");
        assert_eq!(status_text(&s.clone().done(1_200)), "Done in 1.2 s");
        assert_eq!(status_text(&s.failed(300)), "Failed after 300 ms");
    }

    #[test]
    fn all_open_follows_defaults() {
        let steps = vec![TraceStep::new("1", StepKind::Message { text: "x".into() }).done(1)];
        assert!(!all_open(&ToolTraceState::default(), &steps));
        assert_eq!(all_ids(&steps), vec!["1".to_string()]);
        assert!(is_json(" [1] "));
    }
}
