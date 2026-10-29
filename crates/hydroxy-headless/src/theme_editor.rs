//! Theme editor connector. Anatomy: root, controls, section, row (with
//! `data-modified`), swatch, value, warning, reset, preview, contrast, export,
//! live. Every part carries `data-scope="theme-editor"` and its `data-part`.
//!
//! Colour rows open the colour picker in a popover; radii and spacing are
//! sliders; fonts are selects of the families hydroxy ships. The preview is a
//! container with the preset's `data-theme` and [`ThemeEditorState::style`]
//! inline, so edits apply to it alone. Presets are read from the stylesheet
//! passed to [`input`] (normally `hydroxy_tokens::CSS`), so they match it exactly.

pub use hydroxy_machines::color::Rgba;
pub use hydroxy_machines::theme_editor::*;

use crate::theme_menu::Theme;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "theme-editor";

/// Font families hydroxy's stylesheets load (fonts.css and theme-fonts.css), then system stacks.
pub const FONT_OPTIONS: [(&str, &str); 10] = [
    ("Unbounded", "'Unbounded', 'Hanken Grotesk', system-ui, sans-serif"),
    ("Hanken Grotesk", "'Hanken Grotesk', system-ui, -apple-system, sans-serif"),
    ("Space Grotesk", "'Space Grotesk', 'Hanken Grotesk', system-ui, sans-serif"),
    ("IBM Plex Sans", "'IBM Plex Sans', 'Hanken Grotesk', system-ui, sans-serif"),
    ("Chakra Petch", "'Chakra Petch', 'Unbounded', system-ui, sans-serif"),
    ("JetBrains Mono", "'JetBrains Mono', ui-monospace, 'SFMono-Regular', monospace"),
    ("IBM Plex Mono", "'IBM Plex Mono', 'JetBrains Mono', ui-monospace, monospace"),
    ("System sans", "system-ui, -apple-system, 'Segoe UI', sans-serif"),
    ("System serif", "ui-serif, Georgia, 'Times New Roman', serif"),
    ("System mono", "ui-monospace, 'SFMono-Regular', Menlo, monospace"),
];

/// Largest radius the sliders offer, in px.
pub const RADIUS_MAX: f64 = 32.0;

/// Presets as (`data-theme` value, label): every theme but "System".
#[must_use]
pub fn presets() -> Vec<(&'static str, &'static str)> {
    Theme::ALL.into_iter().filter(|t| *t != Theme::System).map(|t| (t.as_str(), t.label())).collect()
}

/// Machine input over `css` (normally `hydroxy_tokens::CSS`).
#[must_use]
pub fn input(css: &'static str, default_preset: &str) -> ThemeEditorInput {
    ThemeEditorInput { css, default_preset: default_preset.to_string() }
}

/// The font option label a stack matches (by first family), or the family itself.
#[must_use]
pub fn font_choice(stack: &str) -> String {
    let family = first_family(stack);
    FONT_OPTIONS
        .iter()
        .find(|(l, s)| first_family(s) == family || *l == family)
        .map_or_else(|| family.to_string(), |(l, _)| (*l).to_string())
}

/// The stack to set when `choice` is picked for `token`: the preset's own stack
/// when it names the same family, so re-picking it isn't an edit.
#[must_use]
pub fn font_event(state: &ThemeEditorState, token: &str, choice: &str) -> ThemeEditorEvent {
    let base = state.base.get(token).map_or("", String::as_str);
    let value = if font_choice(base) == choice {
        base.to_string()
    } else {
        FONT_OPTIONS.iter().find(|(l, _)| *l == choice).map_or_else(|| choice.to_string(), |(_, s)| (*s).to_string())
    };
    ThemeEditorEvent::Set { token: token.to_string(), value }
}

/// Font select options; adds the preset's family when hydroxy doesn't list it.
#[must_use]
pub fn font_options(state: &ThemeEditorState, token: &str) -> Vec<String> {
    let mut out: Vec<String> = FONT_OPTIONS.iter().map(|(l, _)| (*l).to_string()).collect();
    let current = font_choice(state.value(token));
    if !out.contains(&current) {
        out.push(current);
    }
    out
}

/// A radius token in px.
#[must_use]
pub fn radius(state: &ThemeEditorState, token: &str) -> f64 {
    px(state.value(token)).unwrap_or(0.0)
}

/// The event for a radius slider.
#[must_use]
#[allow(clippy::cast_possible_truncation, reason = "clamped to 0..=RADIUS_MAX")]
pub fn radius_event(token: &str, value: f64) -> ThemeEditorEvent {
    let v = value.clamp(0.0, RADIUS_MAX).round() as i32;
    ThemeEditorEvent::Set { token: token.to_string(), value: if v == 0 { "0".into() } else { format!("{v}px") } }
}

/// The event for the spacing slider.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "clamped to the scale range")]
pub fn space_event(value: f64) -> ThemeEditorEvent {
    let (lo, hi) = SPACE_SCALE_RANGE;
    ThemeEditorEvent::SetSpaceScale(value.clamp(f64::from(lo), f64::from(hi)).round() as u16)
}

/// The colour picker's value for a token (black when it isn't a hex colour).
#[must_use]
pub fn picker_value(state: &ThemeEditorState, token: &str) -> Rgba {
    state.color(token).unwrap_or(Rgba::rgb(0, 0, 0))
}

/// The event for a colour picked for `token`.
#[must_use]
pub fn color_event(token: &str, color: Rgba) -> ThemeEditorEvent {
    ThemeEditorEvent::Set { token: token.to_string(), value: color.hex(false) }
}

/// The worst failing check a token is the foreground of.
#[must_use]
pub fn token_issue(issues: &[ContrastIssue], token: &str) -> Option<ContrastIssue> {
    issues.iter().filter(|i| i.fg == token).min_by(|a, b| a.ratio.total_cmp(&b.ratio)).cloned()
}

/// A ratio as text: `3.2:1`.
#[must_use]
pub fn ratio_text(ratio: f64) -> String {
    format!("{:.1}:1", (ratio * 10.0).floor() / 10.0)
}

/// One issue as a sentence: `Muted text on Card is 3.2:1, needs 4.5:1`.
#[must_use]
pub fn issue_text(issue: &ContrastIssue) -> String {
    format!(
        "{} on {} is {}, needs {}",
        label(issue.fg),
        label(issue.bg),
        ratio_text(issue.ratio),
        ratio_text(issue.min)
    )
}

/// The live region's summary.
#[must_use]
pub fn summary(issues: &[ContrastIssue]) -> String {
    match issues.len() {
        0 => "All text and surface pairs meet WCAG AA contrast.".into(),
        1 => "1 pair is below WCAG AA contrast.".into(),
        n => format!("{n} pairs are below WCAG AA contrast."),
    }
}

/// Export formats.
pub const EXPORT_FORMATS: [(&str, &str); 2] = [("css", "CSS"), ("json", "JSON")];

/// The export text for a format value from [`EXPORT_FORMATS`].
#[must_use]
pub fn export(state: &ThemeEditorState, format: &str) -> String {
    if format == "json" { state.export_json() } else { state.export_css() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_resolves_every_token() {
        for (preset, _) in presets() {
            let s = ThemeEditorState::new(&input(hydroxy_tokens::CSS, preset));
            for (token, _) in COLOR_TOKENS {
                let v = s.value(token);
                assert!(v.len() == 7 && v.parse::<Rgba>().is_ok(), "{preset} {token} = {v:?}");
            }
            for (token, _) in RADIUS_TOKENS {
                assert!(px(s.value(token)).is_some(), "{preset} {token}");
            }
            for (token, _) in FONT_TOKENS {
                assert!(!s.value(token).is_empty(), "{preset} {token}");
                assert!(FONT_OPTIONS.iter().any(|(l, _)| *l == font_choice(s.value(token))), "{preset} {token}");
            }
            assert!(s.tokens().iter().any(|(n, v)| n == "space-5" && v == "16px"), "{preset}");
            assert_eq!(s.is_dark(), !matches!(preset, "light" | "neobrutalist" | "solarized-light"), "{preset}");
        }
    }

    #[test]
    fn presets_match_the_stylesheet() {
        let mecha = ThemeEditorState::new(&input(hydroxy_tokens::CSS, "mecha"));
        assert_eq!(mecha.value("accent"), "#ff7a00");
        assert_eq!(mecha.value("radius-md"), "2px");
        assert_eq!(font_choice(mecha.value("font-display")), "Chakra Petch");
        let light = ThemeEditorState::new(&input(hydroxy_tokens::CSS, "light"));
        assert_eq!(light.value("text-1"), "#0b1229");
        assert_eq!(light.value("accent"), "#644fb1");
        let nb = ThemeEditorState::new(&input(hydroxy_tokens::CSS, "neobrutalist-dark"));
        assert_eq!(nb.value("radius-lg"), "0");
        // The preview restates the type composites so font edits show.
        assert!(light.style().contains("--text-sm: 400 14px/1.5 var(--font-body);"));
    }

    #[test]
    fn controls_map_to_events() {
        let s = ThemeEditorState::new(&input(hydroxy_tokens::CSS, "light"));
        assert_eq!(
            font_event(&s, "font-body", "Hanken Grotesk"),
            ThemeEditorEvent::Set { token: "font-body".into(), value: s.value("font-body").into() }
        );
        assert_eq!(
            radius_event("radius-md", 40.0),
            ThemeEditorEvent::Set { token: "radius-md".into(), value: "32px".into() }
        );
        assert_eq!(
            radius_event("radius-md", 0.2),
            ThemeEditorEvent::Set { token: "radius-md".into(), value: "0".into() }
        );
        assert_eq!(space_event(124.6), ThemeEditorEvent::SetSpaceScale(125));
        let issue = ContrastIssue { fg: "text-3", bg: "surface-card", ratio: 3.27, min: 4.5 };
        assert_eq!(issue_text(&issue), "Muted text on Card is 3.2:1, needs 4.5:1");
        assert_eq!(summary(&[issue]), "1 pair is below WCAG AA contrast.");
    }
}
